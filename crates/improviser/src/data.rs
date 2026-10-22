use rand::seq::IndexedRandom;

#[derive(Default)]
pub struct Improv {
    pub root: u8, // root note
    pub scale: [bool; 12], // the scale
    pub tempo: f32, // in bpm
    pub note_duration: f32,
    pub octave: u8,
    _clock: f32,
}

impl Improv {
    pub fn next_note(&self) -> u8 {
        let possible_notes = self.available_notes();
        const MAX_TRIES: u8 = 32;
        let mut tries_counter: u8 = MAX_TRIES;

        while tries_counter != 0 {
            let mut rng = rand::rng();
            let scale_offs = possible_notes.choose(&mut rng).copied().unwrap_or_default();
            let note_offs = scale_offs + self.root;
            let note = self.octave * 12 + note_offs;
            if note < 128 {
                return note;
            }
            tries_counter -= 1;
        }

        return 127;
    }

    pub fn available_notes(&self) -> Vec<u8> {
        self.scale
            .iter()
            .enumerate()
            .filter_map(|(i, &on)| on.then_some(i as u8))
            .collect()
    }

    pub fn get_real_duration(&self) -> f32 {
        self.note_duration * 60.0 / self.tempo.max(1.0) // seconds, 0.343 s at 175 bpm
    }

    pub fn update(&mut self, delta: f32) -> Option<u8> {
        self._clock += delta;
        let dur = self.get_real_duration();
    
        if self._clock >= dur {
            self._clock -= dur; // keeps the remainder, so there's no drift
            Some(self.next_note())
        } else {
            None
        }
    }
}