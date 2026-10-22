use std::error::Error;

pub struct Audio {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
}

pub enum ParamKind {
    Int { min: i64, max: i64 },
    Float { min: f32, max: f32 },
    Bool,
    Choice(Vec<String>),
    Text, // e.g. a server URL for an HTTP backend
    Secret, // stored as Value::Text, but drawn masked
}

#[derive(Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f32),
    Bool(bool),
    Choice(String),
    Text(String),
    Secret,
}

pub struct ParamDesc {
    pub key: &'static str,
    pub label: &'static str,
    pub kind: ParamKind,
}

pub trait Synth: Send + Sync {
    fn name(&self) -> &str;

    /// Called every frame, so ranges and choices may depend on the current state.
    fn params(&self) -> Vec<ParamDesc>;
    fn get(&self, key: &str) -> Option<Value>;
    fn set(&mut self, key: &str, value: Value) -> Result<(), String>;

    fn synthesize(&self, text: &str) -> Result<Audio, Box<dyn Error>>;

    /// Lets Context hand a snapshot of the settings to a worker thread.
    fn clone_box(&self) -> Box<dyn Synth>;
}

pub fn all_synths() -> Vec<Box<dyn Synth>> {
    vec![
        Box::new(espeak::EspeakSynth::default()),
        Box::new(fishaudio::FishSynth::default())
    ]
    // later: Box::new(PiperSynth::new()), Box::new(HttpSynth::new()), ...
}

pub mod espeak;
pub mod fishaudio;