use std::error::Error;

use crate::MidiDevice;

/// Fallback: accepts MIDI and discards it.
pub struct DummyMidi;

impl DummyMidi {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        eprintln!("mididevice: no MIDI backend available, notes will be discarded");
        Ok(Self)
    }
}

impl MidiDevice for DummyMidi {
    fn send(&mut self, _bytes: &[u8]) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}