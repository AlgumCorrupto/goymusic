use config::{config_section};

use std::collections::HashMap;

#[config_section("improviser.selections")]
#[derive(Default)]
pub struct CustomSelections {
    pub scales: HashMap<String, [bool; 12]>,
    pub note_durations: HashMap<String, f32>,
}