use tts::synths::{all_synths, Synth};
use eframe::egui;
use crate::framework::*;

use tts::synths::{ParamKind, Value};

fn params_ui(ui: &mut egui::Ui, synth: &mut dyn Synth) {
    egui::Grid::new("synth_params")
        .num_columns(2)
        .spacing([12.0, 8.0])
        .show(ui, |ui| {
            for desc in synth.params() {
                let Some(mut value) = synth.get(desc.key) else { continue };
                ui.label(desc.label);

                let changed = match (&desc.kind, &mut value) {
                    (ParamKind::Int { min, max }, Value::Int(v)) => {
                        ui.add(egui::Slider::new(v, *min..=*max)).changed()
                    }
                    (ParamKind::Float { min, max }, Value::Float(v)) => {
                        ui.add(egui::Slider::new(v, *min..=*max)).changed()
                    }
                    (ParamKind::Bool, Value::Bool(v)) => ui.checkbox(v, "").changed(),
                    (ParamKind::Text, Value::Text(v)) => ui.text_edit_singleline(v).changed(),
                    (ParamKind::Choice(options), Value::Choice(cur)) => {
                        let mut changed = false;
                        egui::ComboBox::from_id_salt(desc.key)
                            .selected_text(cur.as_str())
                            .show_ui(ui, |ui| {
                                for o in options {
                                    changed |= ui.selectable_value(cur, o.clone(), o).changed();
                                }
                            });
                        changed
                    },
                    (ParamKind::Secret, Value::Text(v)) => ui
                        .add(egui::TextEdit::singleline(v).password(true))
                        .changed(),
                    _ => false, // kind/value mismatch: a backend bug, ignore it
                };
                ui.end_row();

                if changed {
                    let _ = synth.set(desc.key, value);
                }
            }
        });
}

pub struct TtsScreen {
    catalog: Vec<Box<dyn Synth>>,
    selected: usize,
    text: String,
}

impl TtsScreen {
    pub fn new() -> Self {
        Self { catalog: all_synths(), selected: 0, text: String::new() }
    }
}

impl Screen for TtsScreen {
    fn render(&mut self, ui: &mut egui::Ui, _window: &mut Window, frontend: &mut Frontend) {
        ui.heading("Text to speech");

        let app = &mut frontend.app;

        // Backend picker: choosing one installs a fresh clone into Context.
        let current = app.synth().name().to_string();

        
        egui::ComboBox::from_id_salt("backend")
            .selected_text(&current)
            .show_ui(ui, |ui| {
                for (i, s) in self.catalog.iter().enumerate() {
                    if ui.selectable_label(s.name() == current, s.name()).clicked()
                        && s.name() != current
                    {
                        self.selected = i;
                        app.set_synth(s.clone_box());
                    }
                }
            });

        ui.separator();
        if ui.button("Stop").clicked() {
            app.stop_speaking();
        }
        
        params_ui(ui, app.synth_mut()); // edits the live synth in Context directly

        ui.separator();

        // Consume the keys before the TextEdit sees them.
        let (send, newline) = ui.input_mut(|i| {
            // Check Ctrl+Enter first: consume_key matches by modifiers, but be explicit.
            let newline = i.consume_key(egui::Modifiers::COMMAND, egui::Key::Enter);
            let send = i.consume_key(egui::Modifiers::NONE, egui::Key::Enter);
            (send, newline)
        });

        let output = egui::TextEdit::multiline(&mut self.text).show(ui);

        if newline {
            // Insert "\n" at the cursor, since egui ignores Ctrl+Enter itself.
            if let Some(range) = output.cursor_range {
                let idx = range.primary.index;
                let byte = self.text.char_indices().nth(idx.into()).map(|(b, _)| b).unwrap_or(self.text.len());
                self.text.insert(byte, '\n');
            
                // Move the cursor after the inserted newline.
                let mut state = output.state;
                state.cursor.set_char_range(Some(egui::text::CCursorRange::one(
                    egui::text::CCursor::new(idx + 1),
                )));
                state.store(ui.ctx(), output.response.id);
            }
        }

        if (send && output.response.has_focus()) || ui.button("Speak").clicked() {
            if !self.text.trim().is_empty() {
                app.speak(self.text.clone());
            }
        }
    }

    fn get_name(&self) -> String { "TTS".into() }
}