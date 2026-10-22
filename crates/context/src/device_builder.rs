use mididevice::{MidiOut};

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub fn make_midi() -> Result<MidiOut, Box<dyn std::error::Error>> {
    MidiOut::new("goymusic", "goymusic port")
}

#[cfg(target_os = "windows")]
pub fn make_midi() -> Result<MidiOut, Box<dyn std::error::Error>> {
    //MidiOut::new("goymusic", "loopMIDI")
    MidiOut::new()
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
pub fn make_midi() -> Result<MidiOut, Box<dyn std::error::Error>> {
    MidiOut::new()
}

use micdevice::{VirtualMic};

#[cfg(target_os = "linux")]
pub fn make_mic() -> Result<VirtualMic, Box<dyn std::error::Error>> {
    VirtualMic::new("goymusic_mic")
}

#[cfg(not(target_os = "linux"))]
pub fn make_mic() -> Result<PlatformMic, Box<dyn std::error::Error>> {
    VirtualMic::new()
}