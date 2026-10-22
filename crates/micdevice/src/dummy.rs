use std::error::Error;

use crate::MicDevice;

/// Fallback for platforms with no backend: accepts audio and discards it.
pub struct DummyMic;

impl DummyMic {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        eprintln!("micdevice: no virtual mic backend on this platform, audio will be discarded");
        Ok(Self)
    }
}

impl MicDevice for DummyMic {
    fn play(&self, _samples: &[f32], _sample_rate: u32) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn stop(&self) { }
}