use std::error::Error;

use midir::os::unix::VirtualOutput;
use midir::{MidiOutput, MidiOutputConnection};

use crate::MidiDevice;

pub struct VirtualMidi {
    connection: MidiOutputConnection,
}

impl VirtualMidi {
    pub fn new(client_name: &str, port_name: &str) -> Result<Self, Box<dyn Error>> {
        let midi_out = MidiOutput::new(client_name)?;
        let connection = midi_out
            .create_virtual(port_name)
            .map_err(|e| e.to_string())?;
        Ok(Self { connection })
    }
}

impl MidiDevice for VirtualMidi {
    fn send(&mut self, bytes: &[u8]) -> Result<(), Box<dyn Error>> {
        self.connection.send(bytes)?;
        Ok(())
    }
}