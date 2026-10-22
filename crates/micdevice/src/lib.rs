pub trait MicDevice {
    fn play(&self, samples: &[f32], sample_rate: u32) -> Result<(), Box<dyn std::error::Error>>;
    fn stop(&self);
}

pub mod dummy;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
pub use linux::VirtualMic as VirtualMic;
// other platforms not supported
#[cfg(not(target_os = "linux"))]
pub use dummy::DummyMic as VirtualMic;