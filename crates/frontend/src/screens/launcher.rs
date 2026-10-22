use eframe::egui;

use crate::framework::{Frontend, Screen, Window};
use super::improviser::ImproviserScreen;
use super::tts::TtsScreen; // adjust the paths

/// Tracks at most one window opened from a launcher button.
#[derive(Default)]
struct WindowSlot {
    id: Option<usize>,
}

impl WindowSlot {
    /// Draws the open / focus / close buttons for this slot.
    /// `make` builds the first screen, and only runs when the window is actually opened.
    fn ui<S: Screen + 'static>(
        &mut self,
        ui: &mut egui::Ui,
        frontend: &mut Frontend,
        label: &str,
        make: impl FnOnce() -> S,
    ) {
        // Forget the window if it was closed some other way (OS X button, etc.).
        if let Some(id) = self.id {
            if frontend.window_mut(id).is_none() {
                self.id = None;
            }
        }

        match self.id {
            None => {
                if ui.button(format!("Open {label}")).clicked() {
                    // None as the name: the title follows the top screen's get_name().
                    let id = frontend.insert_window(None);
                    if let Some(w) = frontend.window_mut(id) {
                        w.push(make());
                    }
                    self.id = Some(id);
                }
            }
            Some(id) => {
                ui.horizontal(|ui| {
                    ui.label(label);
                    if ui.button("Focus").clicked() {
                        frontend.focus_window(id);
                    }
                    if ui.button("Close").clicked() {
                        frontend.remove_window(id);
                        self.id = None;
                    }
                });
            }
        }
    }
}

#[derive(Default)]
pub struct LauncherScreen {
    improviser: WindowSlot,
    tts_config: WindowSlot,
}

impl Screen for LauncherScreen {
    fn render(&mut self, ui: &mut egui::Ui, window: &mut Window, frontend: &mut Frontend) {
        ui.heading("Launcher");
        ui.separator();

        self.improviser.ui(ui, frontend, "Improviser", ImproviserScreen::new);
        self.tts_config.ui(ui, frontend, "TTS", TtsScreen::new);

        ui.separator();
        if ui.button("Quit").clicked() {
            window.close(); // on the main window this shuts the app down
        }
    }

    fn get_name(&self) -> String {
        "Launcher".into()
    }
}