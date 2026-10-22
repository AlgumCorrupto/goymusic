use std::error::Error;
use wmidi::{Channel, ControlFunction, MidiMessage, Note, U7};

pub trait MidiDevice {
    /// Backends implement only this: push raw MIDI bytes out.
    fn send(&mut self, bytes: &[u8]) -> Result<(), Box<dyn Error>>;

    fn note_on(&mut self, channel: u8, note: u8, velocity: u8) -> Result<(), Box<dyn Error>> {
        let c = Channel::from_index(channel).unwrap_or(Channel::Ch1);
        let msg = MidiMessage::NoteOn(c, Note::from_u8_lossy(note), U7::from_u8_lossy(velocity));
        send_message(self, msg)
    }

    fn note_off(&mut self, channel: u8, note: u8, velocity: u8) -> Result<(), Box<dyn Error>> {
        let c = Channel::from_index(channel).unwrap_or(Channel::Ch1);
        let msg = MidiMessage::NoteOff(c, Note::from_u8_lossy(note), U7::from_u8_lossy(velocity));
        send_message(self, msg)
    }

    /// CC 123: silences every held note on the channel (use on pause / exit).
    fn all_notes_off(&mut self, channel: u8) -> Result<(), Box<dyn Error>> {
        let c = Channel::from_index(channel).unwrap_or(Channel::Ch1);
        let msg = MidiMessage::ControlChange(c, ControlFunction::ALL_NOTES_OFF, U7::from_u8_lossy(0));
        send_message(self, msg)
    }
}

fn send_message<S: MidiDevice + ?Sized>(sink: &mut S, msg: MidiMessage) -> Result<(), Box<dyn Error>> {
    let mut bytes = [0u8; 3];
    let len = msg.copy_to_slice(&mut bytes)?;
    sink.send(&bytes[..len])
}

pub mod dummy;

#[cfg(any(target_os = "linux", target_os = "macos"))]
mod unix;

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub use unix::VirtualMidi as MidiOut;
#[cfg(target_os = "windows")]
pub use dummy::DummyMidi as Device;