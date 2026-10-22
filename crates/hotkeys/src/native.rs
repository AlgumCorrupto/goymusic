use std::collections::HashMap;
use std::error::Error;
use std::sync::mpsc;

use global_hotkey::hotkey::{Code, HotKey, Modifiers as GhMods};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};

use crate::{Binding, GlobalHotkeys, Hotkey, Key, Wake};

const LETTERS: [Code; 26] = [
    Code::KeyA, Code::KeyB, Code::KeyC, Code::KeyD, Code::KeyE, Code::KeyF, Code::KeyG,
    Code::KeyH, Code::KeyI, Code::KeyJ, Code::KeyK, Code::KeyL, Code::KeyM, Code::KeyN,
    Code::KeyO, Code::KeyP, Code::KeyQ, Code::KeyR, Code::KeyS, Code::KeyT, Code::KeyU,
    Code::KeyV, Code::KeyW, Code::KeyX, Code::KeyY, Code::KeyZ,
];
const DIGITS: [Code; 10] = [
    Code::Digit0, Code::Digit1, Code::Digit2, Code::Digit3, Code::Digit4,
    Code::Digit5, Code::Digit6, Code::Digit7, Code::Digit8, Code::Digit9,
];
const FKEYS: [Code; 12] = [
    Code::F1, Code::F2, Code::F3, Code::F4, Code::F5, Code::F6,
    Code::F7, Code::F8, Code::F9, Code::F10, Code::F11, Code::F12,
];

fn to_code(key: Key) -> Option<Code> {
    Some(match key {
        Key::Char(c) if c.is_ascii_lowercase() => LETTERS[(c as u8 - b'a') as usize],
        Key::Char(c) if c.is_ascii_digit() => DIGITS[(c as u8 - b'0') as usize],
        Key::F(n) if (1..=12).contains(&n) => FKEYS[n as usize - 1],
        Key::Space => Code::Space,
        Key::Enter => Code::Enter,
        Key::Escape => Code::Escape,
        Key::Tab => Code::Tab,
        _ => return None,
    })
}

fn to_hotkey(hk: Hotkey) -> Option<HotKey> {
    let mut mods = GhMods::empty();
    if hk.mods.ctrl { mods |= GhMods::CONTROL; }
    if hk.mods.shift { mods |= GhMods::SHIFT; }
    if hk.mods.alt { mods |= GhMods::ALT; }
    if hk.mods.meta { mods |= GhMods::SUPER; }
    Some(HotKey::new(Some(mods), to_code(hk.key)?))
}

pub struct NativeHotkeys<A> {
    manager: GlobalHotKeyManager,
    registered: Vec<HotKey>,
    actions: HashMap<u32, A>, // hotkey id -> action
    rx: mpsc::Receiver<GlobalHotKeyEvent>,
}

impl<A: Copy> NativeHotkeys<A> {
    pub fn new(bindings: &[Binding<A>], wake: Wake) -> Result<Self, Box<dyn Error>> {
        let manager = GlobalHotKeyManager::new()?;

        // Own channel + handler, so we don't share the crate's global receiver.
        let (tx, rx) = mpsc::channel();
        GlobalHotKeyEvent::set_event_handler(Some(move |ev: GlobalHotKeyEvent| {
            let _ = tx.send(ev);
            wake();
        }));

        let mut registered = Vec::new();
        let mut actions = HashMap::new();
        for b in bindings {
            let Some(hk) = to_hotkey(b.trigger) else {
                eprintln!("hotkeys: '{}' can't be mapped to a native key", b.trigger);
                continue;
            };
            match manager.register(hk) {
                Ok(()) => {
                    actions.insert(hk.id(), b.action);
                    registered.push(hk);
                }
                // Usually: another app already owns the combo.
                Err(e) => eprintln!("hotkeys: could not register {}: {e}", b.trigger),
            }
        }

        if registered.is_empty() && !bindings.is_empty() {
            return Err("no hotkey could be registered".into());
        }
        Ok(Self { manager, registered, actions, rx })
    }
}

impl<A: Copy> GlobalHotkeys<A> for NativeHotkeys<A> {
    fn backend_name(&self) -> &'static str { "global-hotkey" }

    fn poll(&mut self) -> Vec<A> {
        let mut out = vec![];
        while let Ok(ev) = self.rx.try_recv() {
            if ev.state == HotKeyState::Pressed {
                if let Some(a) = self.actions.get(&ev.id) {
                    out.push(*a);
                }
            }
        }
        out
    }
}

impl<A> Drop for NativeHotkeys<A> {
    fn drop(&mut self) {
        for hk in &self.registered {
            let _ = self.manager.unregister(*hk);
        }
    }
}