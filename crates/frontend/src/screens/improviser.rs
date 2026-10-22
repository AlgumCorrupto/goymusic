use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;

use serde::Deserialize;

use improviser::constants::*;

use eframe::egui;

use crate::framework::{Screen, Window, Frontend};

#[derive(Default)]
pub struct Selections {
    pub notes: HashMap<String, u8>,
    pub scales: HashMap<String, [bool; 12]>,
    pub note_durations: HashMap<String, f32>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct CustomSelections {
    scales: HashMap<String, [bool; 12]>,
    note_durations: HashMap<String, f32>,
}

impl Selections {
    pub fn new() -> Self {
        let mut sel = Selections { scales: HashMap::new(), note_durations: HashMap::new(), notes: HashMap::new() };

        sel.load_builtin();
        if let Err(error) = sel.load_toml(Path::new("./customs.toml")) {
            eprintln!("Failed to load customs.toml: {error}");
        }

        return sel;
    }

    fn load_builtin(&mut self) {
        // scales
        self.scales.insert(String::from("Ionian"), SCALE_IONIAN);
        self.scales.insert(String::from("Dorian"), SCALE_DORIAN);
        self.scales.insert(String::from("Mixolydian"), SCALE_MIXOLYDIAN);
        self.scales.insert(String::from("Phrygian"), SCALE_PHRYGIAN);
        self.scales.insert(String::from("Lydian"), SCALE_LYDIAN);
        self.scales.insert(String::from("Aeolian"), SCALE_AEOLIAN);
        self.scales.insert(String::from("Locrian"), SCALE_LOCRIAN);
        self.scales.insert(String::from("Chromatic"), SCALE_CHROMATIC);
        self.scales.insert(String::from("Major Pentatonic"), SCALE_PENTATONIC_MAJOR);
        self.scales.insert(String::from("Minor Pentatonic"), SCALE_PENTATONIC_MINOR);
        self.scales.insert(String::from("Egyptian"), SCALE_PENTATONIC_EGYPTIAN);
        self.scales.insert(String::from("Blues Minor Pentatonic"), SCALE_PENTATONIC_BLUES_MINOR);
        self.scales.insert(String::from("Blues Major Pentatonic"), SCALE_PENTATONIC_BLUES_MAJOR);
        self.scales.insert(String::from("Blues"), SCALE_BLUES);
        self.scales.insert(String::from("Major Blues"), SCALE_BLUES_MAJOR);
        self.scales.insert(String::from("Arp: Maj7"), ARP_MAJ7);
        self.scales.insert(String::from("Arp: Min7"), ARP_MIN7);
        self.scales.insert(String::from("Arp: Dom7"), ARP_DOM7);
        self.scales.insert(String::from("Arp: Sus2"), ARP_SUS2);
        self.scales.insert(String::from("Arp: Sus4"), ARP_SUS4);
        self.scales.insert(String::from("Arp: 7sus4"), ARP_7SUS4);

        // note durations
        self.note_durations.insert(String::from("Whole"), DURATION_WHOLE);
        self.note_durations.insert(String::from("Half"), DURATION_HALF);
        self.note_durations.insert(String::from("Quarter"), DURATION_QUARTER);
        self.note_durations.insert(String::from("Straight 8th"), DURATION_8TH_STRAIGHT);
        self.note_durations.insert(String::from("Straight 16th"), DURATION_16TH_STRAIGHT);
        self.note_durations.insert(String::from("Straight 32th"), DURATION_32TH_STRAIGHT);
        self.note_durations.insert(String::from("Triplet 8th"), DURATION_8TH_TRIPLET);
        self.note_durations.insert(String::from("Triplet 16th"), DURATION_16TH_TRIPLET);

        // note names
        self.notes.insert(String::from("C"), 0);
        self.notes.insert(String::from("C#"), 1);
        self.notes.insert(String::from("D"), 2);
        self.notes.insert(String::from("D#"), 3);
        self.notes.insert(String::from("E"), 4);
        self.notes.insert(String::from("F"), 5);
        self.notes.insert(String::from("F#"), 6);
        self.notes.insert(String::from("G"), 7);
        self.notes.insert(String::from("G#"), 8);
        self.notes.insert(String::from("A"), 9);
        self.notes.insert(String::from("A#"), 10);
        self.notes.insert(String::from("B"), 11);
    }

    fn load_toml(&mut self, path: &Path) -> Result<(), Box<dyn Error>> {
        let contents = fs::read_to_string(path)?;
        let custom: CustomSelections = toml::from_str(&contents)?;

        self.scales.extend(custom.scales);
        self.note_durations.extend(custom.note_durations);

        Ok(())
    }

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
    pub fn new() -> Self {
        Self { selections: Selections::new() }
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
                        .range(20.0..=300.0)
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