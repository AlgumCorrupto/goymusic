use std::error::Error;
use std::io::Write;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::MicDevice;

pub struct VirtualMic {
    sink: String,
    module_ids: Vec<u32>,
    current: Arc<Mutex<Option<Child>>>, // the pacat process that is playing right now
}

fn load_module(args: &[String]) -> Result<u32, Box<dyn Error>> {
    let out = Command::new("pactl").arg("load-module").args(args).output()?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).into_owned().into());
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().parse()?)
}

impl VirtualMic {
    pub fn new(name: &str) -> Result<Self, Box<dyn Error>> {
        Self::unload_stale(name);
        
        let sink = format!("{name}_sink");

        let sink_id = load_module(&[
            "module-null-sink".into(),
            format!("sink_name={sink}"),
            format!("sink_properties=device.description={name}_sink"),
        ])?;
        let src_id = load_module(&[
            "module-remap-source".into(),
            format!("master={sink}.monitor"),
            format!("source_name={name}"),
            format!("source_properties=device.description={name}"),
        ])?;

        Ok(Self { sink, module_ids: vec![sink_id, src_id], current: Arc::default() })
    }
    fn unload_stale(name: &str) {
        let Ok(out) = Command::new("pactl").args(["list", "short", "modules"]).output() else { return };
        let sink_arg = format!("sink_name={name}_sink");
        let src_arg = format!("source_name={name}");

        let mut ids: Vec<String> = String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter_map(|line| {
                let mut c = line.split('\t');
                let (id, module, args) = (c.next()?, c.next()?, c.next()?);
                let ours = (module == "module-null-sink" && args.contains(&sink_arg))
                    || (module == "module-remap-source" && args.contains(&src_arg));
                ours.then(|| id.to_string())
            })
            .collect();

        ids.reverse(); // the source depends on the sink, so unload it first
        for id in ids {
            let _ = Command::new("pactl").args(["unload-module", &id]).status();
        }
    }
}

impl MicDevice for VirtualMic {
    fn play(&self, samples: &[f32], sample_rate: u32) -> Result<(), Box<dyn Error>> {
        self.stop(); // cut off the previous clip so clips never overlap

        let bytes: Vec<u8> = samples
            .iter()
            .flat_map(|s| ((s.clamp(-1.0, 1.0) * 32767.0) as i16).to_le_bytes())
            .collect();

        let mut child = Command::new("pacat")
            .args([
                "--playback",
                &format!("--device={}", self.sink),
                &format!("--rate={sample_rate}"),
                "--format=s16le",
                "--channels=1",
            ])
            .stdin(Stdio::piped())
            .spawn()?;

        let mut stdin = child.stdin.take().ok_or("no stdin")?;
        let pid = child.id();
        *self.current.lock().unwrap() = Some(child);

        let slot = self.current.clone();
        std::thread::spawn(move || {
            // If stop() kills pacat, this fails with a broken pipe, which is fine.
            let _ = stdin.write_all(&bytes);
            drop(stdin); // EOF: pacat plays out what it has and exits

            // Reap the process so it doesn't become a zombie. Stop waiting if
            // stop() already took it, or a newer clip replaced it.
            loop {
                {
                    let mut guard = slot.lock().unwrap();
                    match guard.as_mut() {
                        Some(c) if c.id() == pid => {
                            if !matches!(c.try_wait(), Ok(None)) {
                                *guard = None;
                                break;
                            }
                        }
                        _ => break,
                    }
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        });
        Ok(())
    }

    fn stop(&self) {
        if let Some(mut child) = self.current.lock().unwrap().take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

impl Drop for VirtualMic {
    fn drop(&mut self) {
        for id in self.module_ids.iter().rev() {
            let _ = Command::new("pactl").args(["unload-module", &id.to_string()]).status();
        }
    }
}