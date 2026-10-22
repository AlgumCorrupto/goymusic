pub const NUM_NOTES: u8 = 128;

// standard modes
pub const SCALE_IONIAN: [bool; 12] = [true, false, true, false, true, true, false, true, false, true, false, true];
pub const SCALE_DORIAN: [bool; 12] = [true, false, true, true, false, true, false, true, false, true, true, false];
pub const SCALE_PHRYGIAN: [bool; 12] = [true, true, false, true, false, true, false, true, true, false, true, false];
pub const SCALE_LYDIAN: [bool; 12] = [true, false, true, false, true, false, true, true, false, true, false, true];
pub const SCALE_MIXOLYDIAN: [bool; 12] = [true, false, true, false, true, true, false, true, false, true, true, false];
pub const SCALE_AEOLIAN: [bool; 12] = [true, false, true, true, false, true, false, true, true, false, true, false];
pub const SCALE_LOCRIAN: [bool; 12] = [true, true, false, true, false, true, true, false, true, false, true, false];

// Pentatonic scales

// Major pentatonic: 1 2 3 5 6
pub const SCALE_PENTATONIC_MAJOR: [bool; 12] = [true, false, true, false, true, false, false, true, false, true, false, false];

// Minor pentatonic: 1 b3 4 5 b7
pub const SCALE_PENTATONIC_MINOR: [bool; 12] = [true, false, false, true, false, true, false, true, false, false, true, false];

// Egyptian (suspended pentatonic): 1 2 4 5 b7
pub const SCALE_PENTATONIC_EGYPTIAN: [bool; 12] = [true, false, true, false, false, true, false, true, false, false, true, false];

// Blues minor / man gong: 1 b3 4 b6 b7
pub const SCALE_PENTATONIC_BLUES_MINOR: [bool; 12] = [true, false, false, true, false, true, false, false, true, false, true, false];

// Blues major / ritusen: 1 2 4 5 6
pub const SCALE_PENTATONIC_BLUES_MAJOR: [bool; 12] = [true, false, true, false, false, true, false, true, false, true, false, false];

pub const SCALE_CHROMATIC: [bool; 12] = [true; 12];

// Minor blues (hexatonic): 1 b3 4 b5 5 b7
pub const SCALE_BLUES: [bool; 12] = [true, false, false, true, false, true, true, true, false, false, true, false];

// Major blues (hexatonic): 1 2 b3 3 5 6
pub const SCALE_BLUES_MAJOR: [bool; 12] = [true, false, true, true, true, false, false, true, false, true, false, false];

// Arpeggios

// Major 7: 1 3 5 7
pub const ARP_MAJ7: [bool; 12] = [true, false, false, false, true, false, false, true, false, false, false, true];

// Minor 7: 1 b3 5 b7
pub const ARP_MIN7: [bool; 12] = [true, false, false, true, false, false, false, true, false, false, true, false];

// Dominant 7: 1 3 5 b7
pub const ARP_DOM7: [bool; 12] = [true, false, false, false, true, false, false, true, false, false, true, false];

// Suspended
// Sus2: 1 2 5
pub const ARP_SUS2: [bool; 12] = [true, false, true, false, false, false, false, true, false, false, false, false];

// Sus4: 1 4 5
pub const ARP_SUS4: [bool; 12] = [true, false, false, false, false, true, false, true, false, false, false, false];

// 7sus4: 1 4 5 b7
pub const ARP_7SUS4: [bool; 12] = [true, false, false, false, false, true, false, true, false, false, true, false];


// Lydian augmented: 1 2 3 #4 #5 6 7
pub const SCALE_LYDIAN_AUGMENTED: [bool; 12] = [
    true, false, true, false, true, false,
    false, true, false, true, false, true,
];

// Whole-tone: 1 2 3 #4 #5 b7
pub const SCALE_WHOLE_TONE: [bool; 12] = [
    true, false, true, false, true, false,
    true, false, true, false, true, false,
];

// Seven-note scales

// 1. Major / Dorian family
pub const SCALE_HOLDSWORTH: [bool; 12] = [
    true, false, true, false, true, true,
    false, true, false, true, false, true,
];

// 2. Melodic minor
pub const SCALE_HOLDSWORTH_MELODIC_MINOR: [bool; 12] = [
    true, false, true, true, false, true,
    false, true, false, true, false, true,
];

// 3. Harmonic minor
pub const SCALE_HOLDSWORTH_HARMONIC_MINOR: [bool; 12] = [
    true, false, true, true, false, true,
    false, true, true, false, false, true,
];

// 4. Melodic minor #4 / Lydian minor
pub const SCALE_HOLDSWORTH_MELODIC_MINOR_SHARP_4: [bool; 12] = [
    true, false, true, true, false, false,
    true, true, false, true, false, true,
];

// Eight-note scales

// 5. Half-whole diminished
pub const SCALE_HOLDSWORTH_DOMINANT_DIMINISHED: [bool; 12] = [
    true, true, false, true, true, false,
    true, true, false, true, true, false,
];

// 6. Jazz major / bebop major
pub const SCALE_HOLDSWORTH_JAZZ_MAJOR: [bool; 12] = [
    true, false, true, false, true, true,
    false, true, true, true, false, true,
];

// 7. Jazz dominant
pub const SCALE_HOLDSWORTH_JAZZ_DOMINANT: [bool; 12] = [
    true, false, true, false, true, true,
    false, true, false, true, true, true,
];

// 8. Bebop Dorian
pub const SCALE_HOLDSWORTH_BEBOP_DORIAN: [bool; 12] = [
    true, false, true, true, false, true,
    false, true, false, true, true, true,
];

// 9. Bebop melodic minor
pub const SCALE_HOLDSWORTH_BEBOP_MELODIC_MINOR: [bool; 12] = [
    true, false, true, true, false, true,
    false, true, true, true, false, true,
];

// Nine-note scale

// 10. Tcherepnin scale / Messiaen mode 3
pub const SCALE_HOLDSWORTH_TCHEREPNIN: [bool; 12] = [
    true, false, true, true, true, false,
    true, true, true, false, true, true,
];

pub const BPM_TO_HZ: f32 = 1.0 / 60.0; // bpm * BPM_TO_HZ = beats per second
pub const HZ_TO_BPM: f32 = 60.0;       // hz  * HZ_TO_BPM = beats per minute

pub const DURATION_BEAT: f32 = 1.0;

pub const DURATION_WHOLE: f32 = DURATION_BEAT * 4.0;
pub const DURATION_HALF: f32 = DURATION_BEAT * 2.0;
pub const DURATION_QUARTER: f32 = DURATION_BEAT;

pub const DURATION_8TH_STRAIGHT: f32 = DURATION_BEAT / 2.0;
pub const DURATION_16TH_STRAIGHT: f32 = DURATION_BEAT / 4.0;
pub const DURATION_32TH_STRAIGHT: f32 = DURATION_BEAT / 8.0;

pub const DURATION_8TH_TRIPLET: f32 = DURATION_BEAT / 3.0;
pub const DURATION_16TH_TRIPLET: f32 = DURATION_BEAT / 6.0;

pub const DURATION_8TH_QUINTUPLET: f32 = DURATION_BEAT / 5.0;
pub const DURATION_16TH_QUINTUPLET: f32 = DURATION_BEAT / 10.0;
