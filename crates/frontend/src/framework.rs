use eframe::egui::{self, ViewportBuilder, ViewportClass, ViewportCommand, ViewportId};
use std::{collections::BTreeMap, sync::Arc, time::{Duration, Instant}};
use context::data::Context as AppContext;
use crate::constants::{MAX_CATCH_UP, TICK};

use crate::screens::quick_tts::QuickTtsScreen;

use hotkeys::{Binding, GlobalHotkeys, connect};

pub trait Screen {
    fn render(&mut self, ui: &mut egui::Ui, window: &mut Window, frontend: &mut Frontend);
    fn get_name(&self) -> String;
}

pub struct Window {
    screens: Vec<Box<dyn Screen>>,
    pub default_name: Option<String>,
    uid: usize,
    pop_self: bool,
    close_requested: bool,
    style: WindowStyle,
}

impl Window {
    pub fn new(default_name: Option<&str>, uid: usize) -> Self {
        Self {
            screens: vec![],
            default_name: default_name.map(String::from),
            uid,
            pop_self: false,
            close_requested: false,
            style: WindowStyle::default(),
        }
    }

    pub fn uid(&self) -> usize {
        self.uid
    }

    fn render(&mut self, ui: &mut egui::Ui, frontend: &mut Frontend) {
        // Empty stack: nothing to show, so close instead of rendering a blank window.
        let Some(mut screen) = self.screens.pop() else {
            self.close_requested = true;
            return;
        };
        let depth = self.screens.len();
        self.pop_self = false;

        screen.render(ui, self, frontend);

        let pushed = self.screens.len() > depth;
        if !self.pop_self {
            self.screens.insert(depth.min(self.screens.len()), screen);
        }

        // The screen popped itself and pushed nothing: the stack is now empty.
        if self.screens.is_empty() {
            self.close_requested = true;
        }

        if pushed || self.pop_self || self.close_requested {
            frontend.ctx.request_repaint();
        }
        self.pop_self = false;
    }

    pub fn push<T: Screen + 'static>(&mut self, scr: T) {
        self.screens.push(Box::new(scr));
    }

    /// Remove the screen that is currently rendering.
    pub fn pop_self(&mut self) {
        self.pop_self = true;
    }

    /// Close this window (handled by Frontend after render returns).
    pub fn close(&mut self) {
        self.close_requested = true;
    }

    pub fn get_name(&self) -> String {
        if let Some(name) = &self.default_name {
            name.clone()
        } else if let Some(screen) = self.screens.last() {
            screen.get_name()
        } else {
            String::from("Binbows")
        }
    }
}

#[derive(Clone, Copy)]
pub enum Action { QuickTts }

#[derive(Clone)]
pub struct WindowStyle {
    pub size: [f32; 2],
    pub decorations: bool,
    pub always_on_top: bool,
}

impl Default for WindowStyle {
    fn default() -> Self {
        Self { size: [800.0, 600.0], decorations: true, always_on_top: false }
    }
}

pub struct Frontend {
    windows: BTreeMap<usize, Window>,
    wind_uid_seed: usize,
    main_window: usize,
    last_frame: Instant,
    accumulator: Duration,
    quick_tts: Option<usize>,
    hotkeys: Box<dyn GlobalHotkeys<Action>>,
    pub app: AppContext,
    pub ctx: egui::Context,
}

impl Frontend {
    pub fn new(cc: &eframe::CreationContext<'_>, app_context: AppContext) -> Self {
        let main_uid = 0;
        let mut windows = BTreeMap::new();
        windows.insert(main_uid, Window::new(Some("Tider's \"DAW\""), main_uid));
        // field: hotkeys: Box<dyn GlobalHotkeys<Action>>

        let ctx = cc.egui_ctx.clone();
        let hotkeys = connect(
            &[Binding {
                action: Action::QuickTts,
                id: "quick-tts",
                description: "Open quick TTS prompt",
                trigger: "Alt+T".parse().unwrap(), // later: read from customs.toml
            }],
            Arc::new(move || ctx.request_repaint()),
        );

        eprintln!("global hotkeys: {}", hotkeys.backend_name());
        Self {
            windows,
            main_window: main_uid,
            app: app_context,
            wind_uid_seed: 1,
            hotkeys,
            ctx: cc.egui_ctx.clone(),
            last_frame: Instant::now(),
            accumulator: Duration::ZERO,
            quick_tts: None,
        }
    }

    fn update_hotkeys(&mut self) {
        // at the top of ui():
        for action in self.hotkeys.poll() {
            match action {
                Action::QuickTts => self.toggle_quick_tts(),
            }
        }
    }

    fn tick(&mut self) {
        let now = Instant::now();
        self.accumulator += now - self.last_frame;
        self.last_frame = now;

        // After a long stall (window dragged, debugger...), drop the backlog instead of replaying it.
        if self.accumulator > MAX_CATCH_UP {
            self.accumulator = MAX_CATCH_UP;
        }

        while self.accumulator >= TICK {
            self.app.update(); // adjust to your signature
            self.accumulator -= TICK;
        }

        // Make sure we get called again in time for the next tick.
        self.ctx.request_repaint_after(TICK - self.accumulator);
    }

    pub fn insert_window(&mut self, default_name: Option<&str>) -> usize {
        let uid = self.wind_uid_seed;
        self.windows.insert(uid, Window::new(default_name, uid));
        self.wind_uid_seed += 1;
        self.ctx.request_repaint();
        uid
    }

    pub fn toggle_quick_tts(&mut self) {
        // Already open: bring it forward instead of opening a second one.
        if let Some(id) = self.quick_tts {
            if self.windows.contains_key(&id) {
                self.focus_window(id);
                return;
            }
        }

        let id = self.insert_window(None);
        if let Some(w) = self.windows.get_mut(&id) {
            w.style = WindowStyle {
                size: [480.0, 56.0],
                decorations: false,
                always_on_top: true,
            };
            w.push(QuickTtsScreen::new());
        }
        self.quick_tts = Some(id);
    }

    pub fn focus_window(&self, uid: usize) {
        self.ctx.send_viewport_cmd_to(
            ViewportId::from_hash_of(("window", uid)),
            ViewportCommand::Focus,
        );
    }

    pub fn window_mut(&mut self, uid: usize) -> Option<&mut Window> {
        self.windows.get_mut(&uid)
    }

    /// For closing OTHER windows. A window closes itself via `Window::close`.
    pub fn remove_window(&mut self, uid: usize) {
        if self.windows.remove(&uid).is_some() && uid == self.main_window {
            self.shutdown();
        }
    }

    pub fn shutdown(&mut self) {
        self.ctx
            .send_viewport_cmd_to(ViewportId::ROOT, ViewportCommand::Close);
    }
}


impl eframe::App for Frontend {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.app.shutdown();
    }
    
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.update_hotkeys();
        self.tick();
        // Clone so we don't keep `ui` borrowed while also passing `self` around (cheap, it's an Arc).
        let ctx = ui.ctx().clone();

        let ids: Vec<usize> = self.windows.keys().copied().collect();

        for uid in ids {
            // May be gone if another window removed it earlier this frame.
            let Some(mut window) = self.windows.remove(&uid) else { continue };

            if uid == self.main_window {
                // Main window draws into the root Ui we were given.
                egui::CentralPanel::default().show(ui, |ui| {
                    window.render(ui, self);
                });
            } else {
                let mut builder = ViewportBuilder::default()
                    .with_title(window.get_name())
                    .with_inner_size(window.style.size)
                    .with_decorations(window.style.decorations);
                if window.style.always_on_top {
                    builder = builder.with_always_on_top();
}

                ctx.show_viewport_immediate(
                    ViewportId::from_hash_of(("window", uid)),
                    builder,
                    |ctx, class| {
                        // The OS close button (X)
                        if ctx.input(|i| i.viewport().close_requested()) {
                            window.close();
                        }

                        match class {
                            ViewportClass::EmbeddedWindow => {
                                let mut open = true;
                                egui::Window::new(window.get_name())
                                    .id(egui::Id::unique(("window", uid)))
                                    .open(&mut open)
                                    .show(ctx, |ui| window.render(ui, self));
                                if !open {
                                    window.close();
                                }
                            }
                            _ => {
                                egui::CentralPanel::default().show(ctx, |ui| {
                                    window.render(ui, self);
                                });
                            }
                        }
                    },
                );
            }

            if window.close_requested {
                if uid == self.main_window {
                    self.shutdown();
                }
                // `window` dropped; its viewport disappears next frame
            } else {
                self.windows.insert(uid, window);
            }
        }
    }
}