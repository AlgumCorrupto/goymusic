use std::error::Error;

use config::{Config, Configurable, toml};

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
}

pub struct ParamDesc {
    pub key: &'static str,
    pub label: &'static str,
    pub kind: ParamKind,
}

pub trait Synth: Send + Sync + Configurable {
    fn name(&self) -> &str;
    fn short_name(&self) -> &str; // "fish": the key under [tts]
    fn params(&self) -> Vec<ParamDesc>;
    fn get(&self, key: &str) -> Option<Value>;
    fn set(&mut self, key: &str, value: Value) -> Result<(), String>;

    fn synthesize(&self, text: &str) -> Result<Audio, Box<dyn Error>>;

    /// Lets Context hand a snapshot of the settings to a worker thread.
    fn clone_box(&self) -> Box<dyn Synth>;
}


fn path_of<S: Synth + ?Sized>(s: &S) -> String {
    format!("tts.{}", s.short_name())
}

/// Applies the saved `[tts.<short_name>]` values to a synth that already has its defaults.
pub fn load_params<S: Synth + ?Sized>(synth: &mut S, config: &Config) {
    let path = path_of(synth);
    let Some(toml::Value::Table(t)) = config.section(&path) else { return };

    for p in synth.params() {
        let Some(v) = t.get(p.key) else { continue };
        let val = match (&p.kind, v) {
            (ParamKind::Int { .. }, toml::Value::Integer(i)) => Value::Int(*i),
            (ParamKind::Float { .. }, toml::Value::Float(f)) => Value::Float(*f as f32),
            (ParamKind::Float { .. }, toml::Value::Integer(i)) => Value::Float(*i as f32), // `tempo = 120`
            (ParamKind::Bool, toml::Value::Boolean(b)) => Value::Bool(*b),
            (ParamKind::Choice(_), toml::Value::String(s)) => Value::Choice(s.clone()),
            (ParamKind::Text | ParamKind::Secret, toml::Value::String(s)) => Value::Text(s.clone()),
            _ => continue, // wrong type in the file: keep the default
        };
        if let Err(e) = synth.set(p.key, val) {
            eprintln!("config: [{path}].{}: {e}", p.key);
        }
    }
}

/// Writes the synth's params (and extras) to `[tts.<short_name>]`, replacing that table.
pub fn store_params<S: Synth + ?Sized>(synth: &S, config: &mut Config) {
    let mut t = toml::Table::new();

    for p in synth.params() {
        let Some(v) = synth.get(p.key) else { continue };
        let v = match v {
            Value::Int(i) => toml::Value::Integer(i),
            Value::Float(f) => toml::Value::Float(f as f64),
            Value::Bool(b) => toml::Value::Boolean(b),
            Value::Choice(s) | Value::Text(s) => toml::Value::String(s),
        };
        t.insert(p.key.to_string(), v);
    }

    let path = path_of(synth);
    if let Err(e) = config.set_value(&path, toml::Value::Table(t)) {
        eprintln!("config: {e}");
    }
}

pub fn all_synths(cfg: &Config) -> Vec<Box<dyn Synth>> {
    vec![
        Box::new(espeak::EspeakSynth::load(cfg)),
        Box::new(fishaudio::FishSynth::load(cfg))
    ]
}

pub mod espeak;
pub mod fishaudio;