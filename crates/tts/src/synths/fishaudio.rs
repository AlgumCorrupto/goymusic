use std::collections::BTreeMap;
use std::error::Error;
use std::io::{Cursor, Read};

use config::Configurable;

use super::{Audio, ParamDesc, ParamKind, Synth, Value, load_params, store_params};

const MODELS: [&str; 4] = ["s1", "s2.1-pro", "s2-pro", "s2.1-pro-free"];


type FishSynthVoices = BTreeMap<String, String>;

#[derive(Clone)]
pub struct FishSynth {
    api_key: String,
    reference_id: String, // the voice model id from fish.audio
    model: String,
    reference_list: FishSynthVoices
}

impl Default for FishSynth {
    fn default() -> Self {
        let mut reference_list: BTreeMap<String, String> = FishSynthVoices::new();
        reference_list.insert("Dagoth Ur".into(), "a3ac8f70fc1b4880812b5a48748c5f7c".into());
        Self {
            api_key: std::env::var("FISH_API_KEY").unwrap_or_default(),
            reference_id: reference_list.get(&String::from("Dagoth Ur")).unwrap_or(&String::default()).clone(),
            model: "s2.1-pro-free".into(),
            reference_list
        }
    }
}

impl Configurable for FishSynth {
    fn load(config: &config::Config) -> Self
    where
        Self: Sized {  
        let mut s = Self::default();

        if let Some(list) = config.get::<FishSynthVoices>("tts.fish.references") {
            s.reference_list = list;
        }
        if let Some(voice) = config.get::<String>("tts.fish.voice") {
            match s.reference_list.get(&voice) {
                Some(id) => s.reference_id = id.clone(),
                None => eprintln!("config: [tts.fish].voice: unknown voice name '{voice}'"),
            }
        }

        load_params(&mut s, config);
        s
    }

    fn store(&self, config: &mut config::Config) {
        store_params(self, config);
    }
}

impl Synth for FishSynth {
    fn name(&self) -> &str { "Fish Audio" }
    fn short_name(&self) -> &str { "fish" }

    fn params(&self) -> Vec<ParamDesc> {
        vec![
            ParamDesc { key: "api_key", label: "API key", kind: ParamKind::Secret },
            ParamDesc { key: "reference_id", label: "Voice model ID", kind: ParamKind::Text },
            ParamDesc {
                key: "model",
                label: "Model",
                kind: ParamKind::Choice(MODELS.iter().map(|s| s.to_string()).collect()),
            },
            ParamDesc {
                key: "preset",
                label: "Voice preset",
                kind: ParamKind::Choice(
                    self.reference_list.keys().cloned().collect(),
                ),
            },
        ]
    }

    fn get(&self, key: &str) -> Option<Value> {
        match key {
            "api_key" => Some(Value::Text(self.api_key.clone())),
            "reference_id" => Some(Value::Text(self.reference_id.clone())),
            "model" => Some(Value::Choice(self.model.clone())),
            "preset" => {
                let name = self
                    .reference_list
                    .iter()
                    .find(|(_, id)| !id.is_empty() && **id == self.reference_id)
                    .map(|(name, _)| name.as_str())
                    .unwrap_or("Custom");

                Some(Value::Choice(name.to_string()))
            },
            _ => None,
        }
    }

    fn set(&mut self, key: &str, value: Value) -> Result<(), String> {
        match (key, value) {
            ("api_key", Value::Text(v)) => self.api_key = v.trim().to_string(),
            ("reference_id", Value::Text(v)) => self.reference_id = v.trim().to_string(),
            ("model", Value::Choice(v)) => {
                if MODELS.contains(&v.as_str()) {
                    self.model = v;
                } else {
                    return Err(format!("unknown model: {v}"));
                }
            },
            ("preset", Value::Choice(name)) => {
                match self.reference_list.get(&name) {
                    Some(id) if !id.is_empty() => {
                        self.reference_id = id.clone();
                    }
                    Some(_) => {} // Empty ID: keep the custom reference ID.
                    None => return Err(format!("unknown preset: {name}")),
                }
            },
            (k, _) => return Err(format!("unknown param or wrong type: {k}")),
        }
        Ok(())
    }

    fn synthesize(&self, text: &str) -> Result<Audio, Box<dyn Error>> {
        if self.api_key.is_empty() {
            return Err("Fish Audio API key is not set".into());
        }

        let mut body = serde_json::json!({ "text": text, "format": "wav" });
        if !self.reference_id.is_empty() {
            body["reference_id"] = self.reference_id.clone().into();
        }

        let resp = ureq::post("https://api.fish.audio/v1/tts")
            .set("Authorization", &format!("Bearer {}", self.api_key))
            .set("model", &self.model)
            .send_json(body)
            .map_err(|e| match e {
                ureq::Error::Status(code, r) => {
                    format!("Fish Audio HTTP {code}: {}", r.into_string().unwrap_or_default())
                }
                e => e.to_string(),
            })?;

        let mut bytes = Vec::new();
        resp.into_reader().read_to_end(&mut bytes)?;

        let mut reader = hound::WavReader::new(Cursor::new(bytes))?;
        let spec = reader.spec();
        let channels = spec.channels as usize;

        let interleaved: Vec<f32> = match spec.sample_format {
            hound::SampleFormat::Float => reader.samples::<f32>().map_while(Result::ok).collect(),
            hound::SampleFormat::Int => {
                let max = (1i64 << (spec.bits_per_sample - 1)) as f32;
                reader
                    .samples::<i32>()
                    .map_while(Result::ok)
                    .map(|v| v as f32 / max)
                    .collect()
            }
        };

        if interleaved.is_empty() {
            return Err("Fish Audio returned no audio data".into());
        }

        // downmix to mono, since Audio is mono
        let samples = interleaved
            .chunks(channels)
            .map(|f| f.iter().sum::<f32>() / channels as f32)
            .collect();

        Ok(Audio { samples, sample_rate: spec.sample_rate })
    }

    fn clone_box(&self) -> Box<dyn Synth> { Box::new(self.clone()) }
}