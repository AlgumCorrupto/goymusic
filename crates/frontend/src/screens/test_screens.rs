use eframe::egui;

// Adjust this path to wherever Screen / Window / Frontend live in your crate.
use crate::framework::{Frontend, Screen, Window};

/// Entry point: push this onto the main window.
pub struct HomeScreen;

impl Screen for HomeScreen {
    fn render(&mut self, ui: &mut egui::Ui, window: &mut Window, frontend: &mut Frontend) {
        ui.heading(format!("Home (window {})", window.uid()));
        ui.separator();

        if ui.button("Push counter").clicked() {
            window.push(CounterScreen::default());
        }
        if ui.button("Push nested (depth 1)").clicked() {
            window.push(NestedScreen { depth: 1 });
        }
        if ui.button("Replace me with counter").clicked() {
            window.push(CounterScreen::default());
            window.pop_self(); // new screen stays, this one is dropped
        }
        if ui.button("Window manager").clicked() {
            window.push(WindowManagerScreen::default());
        }
        if ui.button("Open new window with counter").clicked() {
            let id = frontend.insert_window(Some("Counter window"));
            if let Some(w) = frontend.window_mut(id) {
                w.push(CounterScreen::default());
            }
        }

        ui.separator();
        if ui.button("Close this window").clicked() {
            window.close(); // on the main window this shuts the app down
        }
    }

    fn get_name(&self) -> String {
        "Home".into()
    }
}

/// Has its own state, to verify it survives while another screen is on top of it.
#[derive(Default)]
pub struct CounterScreen {
    count: i32,
    text: String,
}

impl Screen for CounterScreen {
    fn render(&mut self, ui: &mut egui::Ui, window: &mut Window, _frontend: &mut Frontend) {
        ui.heading("Counter");
        ui.horizontal(|ui| {
            if ui.button("-").clicked() {
                self.count -= 1;
            }
            ui.label(self.count.to_string());
            if ui.button("+").clicked() {
                self.count += 1;
            }
        });
        ui.text_edit_singleline(&mut self.text);

        ui.separator();
        if ui.button("Push nested on top").clicked() {
            window.push(NestedScreen { depth: 1 });
        }
        if ui.button("Back").clicked() {
            window.pop_self();
        }
    }

    fn get_name(&self) -> String {
        format!("Counter ({})", self.count)
    }
}

/// Stack depth test: keep pushing, then pop back one at a time.
pub struct NestedScreen {
    depth: usize,
}

impl Screen for NestedScreen {
    fn render(&mut self, ui: &mut egui::Ui, window: &mut Window, _frontend: &mut Frontend) {
        ui.heading(format!("Nested, depth {}", self.depth));

        if ui.button("Go deeper").clicked() {
            window.push(NestedScreen { depth: self.depth + 1 });
        }
        if ui.button("Back").clicked() {
            window.pop_self();
        }
    }

    fn get_name(&self) -> String {
        format!("Nested {}", self.depth)
    }
}

/// Opens windows and then manipulates them from here, using `window_mut` / `remove_window`.
#[derive(Default)]
pub struct WindowManagerScreen {
    opened: Vec<usize>,
}

impl Screen for WindowManagerScreen {
    fn render(&mut self, ui: &mut egui::Ui, window: &mut Window, frontend: &mut Frontend) {
        ui.heading("Window manager");

        // Forget windows that were closed some other way (OS close button, their own Quit...).
        // `window_mut` is None for gone windows. It is never our own id, since we don't store it.
        self.opened.retain(|&id| frontend.window_mut(id).is_some());

        if ui.button("Open window").clicked() {
            let id = frontend.insert_window(Some("Managed"));
            if let Some(w) = frontend.window_mut(id) {
                w.push(NestedScreen { depth: 1 });
            }
            self.opened.push(id);
        }

        ui.separator();
        ui.label(format!("{} managed window(s)", self.opened.len()));

        // Collect actions first, then apply them, so we don't mutate `opened` while iterating.
        let mut to_close = None;
        let mut to_push = None;
        for &id in &self.opened {
            ui.horizontal(|ui| {
                ui.label(format!("#{id}"));
                if ui.button("Push counter").clicked() {
                    to_push = Some(id);
                }
                if ui.button("Close").clicked() {
                    to_close = Some(id);
                }
            });
        }
        if let Some(id) = to_push {
            if let Some(w) = frontend.window_mut(id) {
                w.push(CounterScreen::default());
            }
        }
        if let Some(id) = to_close {
            frontend.remove_window(id);
        }

        ui.separator();
        if ui.button("Close all managed").clicked() {
            for id in self.opened.drain(..) {
                frontend.remove_window(id);
            }
        }
        if ui.button("Back").clicked() {
            window.pop_self();
        }
    }

    fn get_name(&self) -> String {
        "Window manager".into()
    }
}