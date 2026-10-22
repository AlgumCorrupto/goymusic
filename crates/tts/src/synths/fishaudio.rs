use std::error::Error;
use std::io::{Cursor, Read};
use super::{Audio, ParamDesc, ParamKind, Synth, Value};

const MODELS: [&str; 4] = ["s1", "s2.1-pro", "s2-pro", "s2.1-pro-free"];


// TODO: load from an init file
/// (display name, fish.audio reference id)
const REFERENCES: &[(&str, &str)] = &[
    ("Dagoth Ur", "a3ac8f70fc1b4880812b5a48748c5f7c"),
    ("Pigeon ASMR", "424d78b883cc4a068df24ab425fd1e18"),
    ("Van Darkholme", "ac3e2eee47a1414d8e705613f7e5aca7"),
    ("Sponge Bob", "deb5d2b89e3247918d4937d9c0864cca"),
    ("Chief Keef", "28adaaaee1f44a0785b9880938b041b4"),
    ("Indian", "71e5c1d4909b4c10b0fcd3194331ad58"),
    ("Postal Dude", "bc7d31c3fddd4d06b7bc8b2962d50982"),
    ("DJ Smokey", "8a2f5be9b42f49f0871b2f62c68ff6bb"),
    ("OG Loc", "e160a42a23734ca8bd5f7b8d68be4f3f"),
    ("Peter Griffin", "b1d36a18f8d84bd59dead30474cbe3d7"),
    ("Donald J Trump", "4d5fa13d58824062a669e37665626dba"),
    ("Alex Jones", "d596396ca9e14998bd3b6a0d3f69e707"),
];

#[derive(Clone)]
pub struct FishSynth {
    api_key: String,
    reference_id: String, // the voice model id from fish.audio
    model: String,
}

impl Default for FishSynth {
    fn default() -> Self {
        Self {
            api_key: std::env::var("FISH_API_KEY").unwrap_or_default(),
            reference_id: REFERENCES[0].1.into(),
            model: "s2.1-pro-free".into(),
        }
    }
}

impl Synth for FishSynth {
    fn name(&self) -> &str { "Fish Audio" }

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
                kind: ParamKind::Choice(REFERENCES.iter().map(|(n, _)| n.to_string()).collect()),
            },
        ]
    }

    fn get(&self, key: &str) -> Option<Value> {
        match key {
            "api_key" => Some(Value::Text(self.api_key.clone())),
            "reference_id" => Some(Value::Text(self.reference_id.clone())),
            "model" => Some(Value::Choice(self.model.clone())),
            "preset" => {
                let name = REFERENCES
                    .iter()
                    .find(|(_, id)| !id.is_empty() && *id == self.reference_id)
                    .map(|(n, _)| *n)
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
                match REFERENCES.iter().find(|(n, _)| *n == name) {
                    Some((_, id)) if !id.is_empty() => self.reference_id = id.to_string(),
                    Some(_) => {} // "Custom": keep whatever is typed
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