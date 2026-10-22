use crate::framework::{Screen, Window, Frontend};
use eframe::egui;

pub struct QuickTtsScreen {
    text: String,
    first_frame: bool,
}

impl QuickTtsScreen {
    pub fn new() -> Self { Self { text: String::new(), first_frame: true } }
}

impl Screen for QuickTtsScreen {
    fn render(&mut self, ui: &mut egui::Ui, window: &mut Window, frontend: &mut Frontend) {
        let resp = ui.add(
            egui::TextEdit::singleline(&mut self.text)
                .hint_text("Say something... (Enter to speak, Esc to cancel)")
                .desired_width(f32::INFINITY),
        );
        if self.first_frame {
            resp.request_focus();
            self.first_frame = false;
        }

        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            window.close();
        } else if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            if !self.text.trim().is_empty() {
                frontend.app.speak(self.text.clone());
            }
            window.close();
        }
    }

    fn get_name(&self) -> String { "Quick TTS".into() }
}