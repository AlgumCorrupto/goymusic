use std::error::Error;
use std::sync::mpsc;
use std::time::Duration;

use ashpd::desktop::global_shortcuts::{GlobalShortcuts, NewShortcut};
use futures_util::StreamExt;
use ashpd::desktop::CreateSessionOptions;
use ashpd::desktop::global_shortcuts::BindShortcutsOptions;

use ashpd::AppID;
use ashpd;

use crate::{Binding, GlobalHotkeys, Hotkey, Key, Wake};

/// XDG shortcut trigger format, e.g. "CTRL+ALT+t".
fn to_xdg(hk: Hotkey) -> String {
    let mut s = String::new();
    if hk.mods.ctrl { s += "CTRL+"; }
    if hk.mods.shift { s += "SHIFT+"; }
    if hk.mods.alt { s += "ALT+"; }
    if hk.mods.meta { s += "LOGO+"; }
    match hk.key {
        Key::Char(c) => s.push(c.to_ascii_lowercase()),
        Key::F(n) => s += &format!("F{n}"),
        Key::Space => s += "space",
        Key::Enter => s += "Return",
        Key::Escape => s += "Escape",
        Key::Tab => s += "Tab",
    }
    s
}

pub struct PortalHotkeys<A> {
    rx: mpsc::Receiver<String>,   // activated shortcut ids
    actions: Vec<(&'static str, A)>,
}

impl<A: Copy + Send + 'static> PortalHotkeys<A> {
    pub fn new(bindings: &[Binding<A>], wake: Wake) -> Result<Self, Box<dyn Error>> {
        let (tx, rx) = mpsc::channel::<String>();
        let (ready_tx, ready_rx) = mpsc::channel::<Result<(), String>>();

        // (id, description, trigger) as owned data for the worker thread
        let list: Vec<(&'static str, &'static str, String)> = bindings
            .iter()
            .map(|b| (b.id, b.description, to_xdg(b.trigger)))
            .collect();

        std::thread::spawn(move || {
            let rt = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
                Ok(rt) => rt,
                Err(e) => { let _ = ready_tx.send(Err(e.to_string())); return; }
            };

            let res: Result<(), ashpd::Error> = rt.block_on(async {
                // inside rt.block_on(async { ... }), before GlobalShortcuts::new():
                let app_id: AppID = "org.goymusic.Goymusic".parse().expect("valid app id");
                if let Err(e) = ashpd::register_host_app(app_id).await {
                    eprintln!("hotkeys: could not register app id: {e}");
                }
                // Failing here means the portal (or its GlobalShortcuts interface) is missing.
                let proxy = GlobalShortcuts::new().await?;
                let session = proxy.create_session(CreateSessionOptions::default()).await?;
                let _ = ready_tx.send(Ok(()));

                // May show a KDE confirmation dialog and wait for the user.
                let new: Vec<NewShortcut> = list
                    .iter()
                    .map(|(id, desc, trig)| {
                        NewShortcut::new(*id, *desc).preferred_trigger(Some(trig.as_str()))
                    })
                    .collect();
                proxy
                    .bind_shortcuts(
                        &session,
                        &new,
                        None,
                        BindShortcutsOptions::default(),
                    )
                    .await?
                    .response()?;
                let mut activated = proxy.receive_activated().await?;
                while let Some(ev) = activated.next().await {
                    let _ = tx.send(ev.shortcut_id().to_string());
                    wake();
                }
                Ok(())
            });

            if let Err(e) = res {
                // If this happened before `ready`, `new` reports it below.
                let _ = ready_tx.send(Err(e.to_string()));
                eprintln!("hotkeys: portal error: {e}");
            }
        });

        // Wait only for "portal reachable", not for the user's confirmation.
        match ready_rx.recv_timeout(Duration::from_secs(5)) {
            Ok(Ok(())) => {}
            Ok(Err(e)) => return Err(e.into()),
            Err(_) => return Err("timed out talking to the portal".into()),
        }

        Ok(Self { rx, actions: bindings.iter().map(|b| (b.id, b.action)).collect() })
    }
}

impl<A: Copy> GlobalHotkeys<A> for PortalHotkeys<A> {
    fn backend_name(&self) -> &'static str { "xdg-desktop-portal" }

    fn poll(&mut self) -> Vec<A> {
        let mut out = vec![];
        while let Ok(id) = self.rx.try_recv() {
            if let Some((_, a)) = self.actions.iter().find(|(i, _)| *i == id) {
                out.push(*a);
            }
        }
        out
    }
}