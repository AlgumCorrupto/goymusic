use std::collections::HashMap;

use config::Configurable;
use context::config::config_section;

use eframe::egui;

use crate::framework::{Screen, Window, Frontend};

#[derive(Default)]
#[config_section("improviser.selections")]
pub struct Selections {
    pub notes: HashMap<String, u8>,
    pub scales: HashMap<String, [bool; 12]>,
    pub note_durations: HashMap<String, f32>,
    pub min_bpm: f32,
    pub max_bpm: f32,
}

impl Selections {
    /// C, C#, D ... B
    pub fn sorted_notes(&self) -> Vec<(&String, &u8)> {
        let mut v: Vec<_> = self.notes.iter().collect();
        v.sort_by_key(|(_, val)| **val);
        v
    }

    /// Alphabetical
    pub fn sorted_scales(&self) -> Vec<(&String, &[bool; 12])> {
        let mut v: Vec<_> = self.scales.iter().collect();
        v.sort_by(|a, b| a.0.cmp(b.0));
        v
    }

    /// Longest to shortest (assuming a bigger value = a longer note)
    pub fn sorted_durations(&self) -> Vec<(&String, &f32)> {
        let mut v: Vec<_> = self.note_durations.iter().collect();
        v.sort_by(|a, b| b.1.total_cmp(a.1));
        v
    }
}

/// Generic dropdown: shows the name matching the current value ("Custom" if none matches).
fn combo<T: PartialEq + Copy>(
    ui: &mut eframe::egui::Ui,
    id: &str,
    current: &mut T,
    items: &[(&String, &T)],
) {
    let selected = items
        .iter()
        .find(|(_, v)| **v == *current)
        .map(|(name, _)| name.as_str())
        .unwrap_or("Not selected");

    eframe::egui::ComboBox::from_id_salt(id)
        .selected_text(selected)
        .show_ui(ui, |ui| {
            for (name, value) in items {
                ui.selectable_value(current, **value, name.as_str());
            }
        });
}

pub struct ImproviserScreen {
    selections: Selections
}

impl ImproviserScreen {
    pub fn new(frontend: &Frontend) -> Self {
        Self { selections: Selections::load(frontend.app.config()) }
    }
}

impl Screen for ImproviserScreen {
    fn render(&mut self, ui: &mut eframe::egui::Ui, _window: &mut Window, frontend: &mut Frontend) {
        let app = &mut frontend.app;

        ui.heading("Mr. Holdsworth");
        ui.separator();

        if ui
            .button(if app.improv_playing { "Pause" } else { "Play" })
            .clicked()
        {
            app.improv_playing = !app.improv_playing;
        }

        ui.add_space(8.0);

        let improv = app.improv_mut();
        let notes = self.selections.sorted_notes();
        let scales = self.selections.sorted_scales();
        let durations = self.selections.sorted_durations();

        egui::Grid::new("improv_params")
            .num_columns(2)
            .spacing([12.0, 8.0])
            .show(ui, |ui| {
                ui.label("Root note");
                combo(ui, "root", &mut improv.root, &notes);
                ui.end_row();

                ui.label("Scale");
                combo(ui, "scale", &mut improv.scale, &scales);
                ui.end_row();

                ui.label("Note duration");
                combo(ui, "duration", &mut improv.note_duration, &durations);
                ui.end_row();

                ui.label("Octave");
                ui.add(egui::Slider::new(&mut improv.octave, 0..=8));
                ui.end_row();

                ui.label("Tempo");
                ui.add(
                    egui::DragValue::new(&mut improv.tempo)
                        .range(self.selections.min_bpm..=self.selections.max_bpm)
                        .speed(0.5)
                        .suffix(" bpm"),
                );
                ui.end_row();
            });
    }

    fn get_name(&self) -> String {
        "Mr. Holdsworth".into()
    }
}