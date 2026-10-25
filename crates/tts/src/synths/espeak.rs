use std::error::Error;
use std::process::Command;
use config::Configurable;

use crate::synths::{load_params, store_params};

use super::{Audio, ParamDesc, ParamKind, Synth, Value};

#[derive(Clone)]
pub struct EspeakSynth {
    voice: String,
    speed: u32,
    pitch: u32,
    voices: Vec<String>, // queried once at construction
}

impl Default for EspeakSynth {
    fn default() -> Self {
        let mut voices = list_voices();
        if voices.is_empty() {
            voices.push("en".into());
        }
        Self { voice: "en".into(), speed: 120, pitch: 50, voices }
    }
}

/// `espeak-ng --voices` prints a table: Pty Language Age/Gender VoiceName File ...
/// Column 1 (Language) is accepted by `-v`. Check the output on your machine.
fn list_voices() -> Vec<String> {
    let Ok(out) = Command::new("espeak-ng").arg("--voices").output() else {
        return vec![];
    };
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .skip(1) // header
        .filter_map(|l| l.split_whitespace().nth(1).map(String::from))
        .collect()
}

impl Configurable for EspeakSynth {
    fn load(config: &config::Config) -> Self
    where
        Self: Sized {
        let mut s = Self::default();
        load_params(&mut s, config);
        s
    }

    fn store(&self, config: &mut config::Config) {
        store_params(self, config);
    }
}

impl Synth for EspeakSynth {
    fn name(&self) -> &str { "eSpeak NG" }
    fn short_name(&self) -> &str { "espeak" }

    fn params(&self) -> Vec<ParamDesc> {
        vec![
            ParamDesc { key: "voice", label: "Voice", kind: ParamKind::Choice(self.voices.clone()) },
            ParamDesc { key: "speed", label: "Speed (wpm)", kind: ParamKind::Int { min: 80, max: 450 } },
            ParamDesc { key: "pitch", label: "Pitch", kind: ParamKind::Int { min: 0, max: 99 } },
        ]
    }

    fn get(&self, key: &str) -> Option<Value> {
        match key {
            "voice" => Some(Value::Choice(self.voice.clone())),
            "speed" => Some(Value::Int(self.speed as i64)),
            "pitch" => Some(Value::Int(self.pitch as i64)),
            _ => None,
        }
    }

    fn set(&mut self, key: &str, value: Value) -> Result<(), String> {
        match (key, value) {
            ("voice", Value::Choice(v)) => self.voice = v,
            ("speed", Value::Int(v)) => self.speed = v.clamp(80, 450) as u32,
            ("pitch", Value::Int(v)) => self.pitch = v.clamp(0, 99) as u32,
            (k, _) => return Err(format!("unknown param or wrong type: {k}")),
        }
        Ok(())
    }

    fn synthesize(&self, text: &str) -> Result<Audio, Box<dyn Error>> {
        let out = Command::new("espeak-ng")
            .args(["-v", &self.voice])
            .args(["-s", &self.speed.to_string()])
            .args(["-p", &self.pitch.to_string()])
            .arg("--stdout")
            .arg(text)
            .output()?;

        if !out.status.success() {
            return Err(String::from_utf8_lossy(&out.stderr).into_owned().into());
        }

        // 44-byte WAV header, then 16-bit mono PCM. I parse it by hand because
        // espeak writes a placeholder length when streaming to a pipe.
        let wav = out.stdout;
        if wav.len() < 44 {
            return Err("short wav".into());
        }
        let sample_rate = u32::from_le_bytes([wav[24], wav[25], wav[26], wav[27]]);
        let samples = wav[44..]
            .chunks_exact(2)
            .map(|b| i16::from_le_bytes([b[0], b[1]]) as f32 / 32768.0)
            .collect();

        Ok(Audio { samples, sample_rate })
    }

    fn clone_box(&self) -> Box<dyn Synth> { Box::new(self.clone()) }
}