use improviser::data::Improv;
use mididevice::{MidiDevice, dummy::DummyMidi};
use micdevice::{MicDevice, dummy::DummyMic};
use tts::synths::{Synth, espeak::EspeakSynth, Audio};
use std::time::Instant;
use std::sync::mpsc;
use crate::device_builder::{make_midi, make_mic};
use std::sync::mpsc::TryRecvError;

pub struct Context {
    improv: Improv,
    midiout: Box<dyn MidiDevice>,
    micout: Box<dyn MicDevice>,
    clock: Instant,
    pub improv_playing: bool,
    tts: Box<dyn Synth>,
    tts_rx: Option<mpsc::Receiver<Result<Audio, String>>>,
}

impl Default for Context {
    fn default() -> Self {
        let midiout: Box<dyn MidiDevice> = match make_midi() {
            Ok(m) => Box::new(m),
            Err(e) => {
                eprintln!("MIDI unavailable: {e}");
                Box::new(DummyMidi::new().unwrap())
            }
        };

        let micout: Box<dyn MicDevice> = match make_mic() {
            Ok(m) => Box::new(m),
            Err(e) => {
                eprintln!("virtual mic unavailable: {e}");
                Box::new(DummyMic::new().unwrap())
            }
        };

        Self {
            midiout,
            micout,
            improv: Improv::default(),
            improv_playing: false,
            tts: Box::new(EspeakSynth::default()),
            tts_rx: None,
            clock: Instant::now(),
        }
    }
}

impl Context {
    // Context::new takes the initial synth (the frontend picks it)
    pub fn synth(&self) -> &dyn Synth { self.tts.as_ref() }
    pub fn synth_mut(&mut self) -> &mut dyn Synth { self.tts.as_mut() }

    pub fn set_synth(&mut self, synth: Box<dyn Synth>) {
        self.tts = synth;
    }

    pub fn speak(&mut self, text: String) {
        let synth = self.tts.clone_box(); // settings snapshot for the worker thread
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(synth.synthesize(&text).map_err(|e| e.to_string()));
        });
        self.tts_rx = Some(rx);
    }

    pub fn stop_speaking(&mut self) {
        self.micout.stop();
        self.tts_rx = None; // ignore any synthesis still in flight
    }

    pub fn improv_mut(&mut self) -> &mut Improv {
        &mut self.improv
    }

    pub fn update_improviser(&mut self, dt: f32) {
        // improviser
        if !self.improv_playing {
            return;
        }
        
        match self.improv.update(dt) {
            Some(note) => {
                self.midiout.all_notes_off(1).ok();
                self.midiout.note_on(1, note, 127).ok();
            },
            _ => {}
        }
    }

    fn poll_tts(&mut self) {
        let Some(rx) = &self.tts_rx else { return };

        match rx.try_recv() {
            Ok(Ok(audio)) => {
                if let Err(e) = self.micout.play(&audio.samples, audio.sample_rate) {
                    eprintln!("mic playback failed: {e}");
                }
                self.tts_rx = None;
            }
            Ok(Err(e)) => {
                eprintln!("TTS failed: {e}");
                self.tts_rx = None;
            }
            Err(TryRecvError::Empty) => {} // still synthesizing
            Err(TryRecvError::Disconnected) => self.tts_rx = None, // worker died
        }
    }

    pub fn update(&mut self) {
        let dt = self.clock.elapsed().as_secs_f32();
        self.clock = Instant::now();

        self.update_improviser(dt);
        self.poll_tts();
    }
}