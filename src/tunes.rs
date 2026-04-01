use clap::ValueEnum;
use rodio::Player;
use rodio::buffer::SamplesBuffer;
use std::f32::consts::PI;
use std::fmt;
use std::num::NonZero;

const SAMPLE_RATE: u32 = 48000;
const SR: f32 = SAMPLE_RATE as f32;

#[derive(Clone, ValueEnum)]
pub enum Tune {
    Beep,
    Clock,
    Chords,
    Scale,
    Zelda,
}

#[derive(Clone, ValueEnum)]
pub enum BeepPattern {
    Success,
    Failure,
}

impl fmt::Display for BeepPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BeepPattern::Success => write!(f, "success"),
            BeepPattern::Failure => write!(f, "failure"),
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
pub enum ZeldaSong {
    Lullaby,
    Epona,
    Saria,
    Sun,
    Time,
    Storms,
    Minuet,
    Bolero,
    Serenade,
    Nocturne,
    Requiem,
    Prelude,
}

impl fmt::Display for ZeldaSong {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ZeldaSong::Lullaby => write!(f, "lullaby"),
            ZeldaSong::Epona => write!(f, "epona"),
            ZeldaSong::Saria => write!(f, "saria"),
            ZeldaSong::Sun => write!(f, "sun"),
            ZeldaSong::Time => write!(f, "time"),
            ZeldaSong::Storms => write!(f, "storms"),
            ZeldaSong::Minuet => write!(f, "minuet"),
            ZeldaSong::Bolero => write!(f, "bolero"),
            ZeldaSong::Serenade => write!(f, "serenade"),
            ZeldaSong::Nocturne => write!(f, "nocturne"),
            ZeldaSong::Requiem => write!(f, "requiem"),
            ZeldaSong::Prelude => write!(f, "prelude"),
        }
    }
}

pub const ZELDA_BY_HOUR: [ZeldaSong; 12] = [
    ZeldaSong::Time,     // 00, 12
    ZeldaSong::Nocturne, // 01, 13
    ZeldaSong::Requiem,  // 02, 14
    ZeldaSong::Lullaby,  // 03, 15
    ZeldaSong::Serenade, // 04, 16
    ZeldaSong::Minuet,   // 05, 17
    ZeldaSong::Prelude,  // 06, 18
    ZeldaSong::Sun,      // 07, 19
    ZeldaSong::Epona,    // 08, 20
    ZeldaSong::Saria,    // 09, 21
    ZeldaSong::Bolero,   // 10, 22
    ZeldaSong::Storms,   // 11, 23
];

/// Pre-renders all samples into a single continuous buffer (like ffmpeg does).
/// Tracks phase across notes so frequency changes don't click.
struct Buf {
    samples: Vec<f32>,
    phase: f32,
    lp_state: f32,
}

impl Buf {
    fn new() -> Self {
        Buf {
            samples: Vec::new(),
            phase: 0.0,
            lp_state: 0.0,
        }
    }

    fn sine_lp(&mut self, freq: f32, secs: f32, cutoff: f32, vol: f32) {
        let rc = 1.0 / (2.0 * PI * cutoff);
        let dt = 1.0 / SR;
        let alpha = dt / (rc + dt);
        let n = (SR * secs) as usize;
        let phase_inc = 2.0 * PI * freq / SR;
        for _ in 0..n {
            let raw = vol * self.phase.sin();
            self.lp_state += alpha * (raw - self.lp_state);
            self.samples.push(self.lp_state);
            self.phase += phase_inc;
        }
        self.phase %= 2.0 * PI;
    }

    fn silence(&mut self, secs: f32) {
        let n = (SR * secs) as usize;
        // Drain lp_state toward zero naturally during silence
        let rc = 1.0 / (2.0 * PI * 200.0); // gentle drain
        let dt = 1.0 / SR;
        let alpha = dt / (rc + dt);
        for _ in 0..n {
            self.lp_state *= 1.0 - alpha;
            self.samples.push(self.lp_state);
        }
        self.phase = 0.0;
    }

    fn play(mut self, player: &Player) {
        // Fade last 10ms to zero
        let fade_samples = (SR * 0.01) as usize;
        let len = self.samples.len();
        let start = len.saturating_sub(fade_samples);
        for i in start..len {
            let t = (len - i) as f32 / fade_samples as f32;
            self.samples[i] *= t;
        }
        // Pad 50ms of silence so the audio system doesn't pop on stop
        self.samples
            .extend(std::iter::repeat_n(0.0, (SR * 0.05) as usize));
        let buf = SamplesBuffer::new(
            NonZero::new(1).unwrap(),
            NonZero::new(SAMPLE_RATE).unwrap(),
            self.samples,
        );
        player.append(buf);
    }
}

fn note_freq(semitones_from_a4: i32) -> f32 {
    440.0 * 2.0_f32.powf(semitones_from_a4 as f32 / 12.0)
}

fn hour_to_root(hour: u32) -> i32 {
    (hour % 12) as i32 - 9
}

/// The OG beep. Ascending tones for success, descending for failure.
pub fn play_beep(player: &Player, vol: f32, pattern: &BeepPattern) {
    let tones: &[(f32, f32)] = match pattern {
        BeepPattern::Success => &[
            (440.0, 0.3),
            (460.0, 0.3),
            (480.0, 0.3),
            (500.0, 0.3),
            (520.0, 0.7),
        ],
        BeepPattern::Failure => &[
            (520.0, 0.3),
            (500.0, 0.3),
            (480.0, 0.3),
            (460.0, 0.3),
            (440.0, 0.7),
        ],
    };

    let mut buf = Buf::new();
    for &(freq, dur) in tones {
        buf.sine_lp(freq, dur, 2000.0, vol);
    }
    buf.play(player);
}

/// Westminster-style clock: melodic chime intro then deep bongs for the hour.
pub fn play_clock(player: &Player, vol: f32, hour: u32) {
    let e5 = note_freq(7);
    let c5 = note_freq(3);
    let d5 = note_freq(5);
    let g4 = note_freq(-2);

    let chime: &[(f32, f32)] = &[
        (e5, 0.25),
        (c5, 0.25),
        (d5, 0.25),
        (g4, 0.5),
        (g4, 0.25),
        (d5, 0.25),
        (e5, 0.25),
        (c5, 0.5),
    ];

    let mut buf = Buf::new();
    for (i, &(freq, dur)) in chime.iter().enumerate() {
        buf.sine_lp(freq, dur, 3000.0, vol * 0.8);
        if i == 3 {
            buf.silence(0.4);
        }
    }

    buf.silence(0.5);

    let bong = note_freq(-12);
    let count = match hour % 12 {
        0 => 12,
        h => h,
    };
    for i in 0..count {
        buf.sine_lp(bong, 0.6, 1000.0, vol);
        if i < count - 1 {
            buf.silence(0.5);
        }
    }
    buf.play(player);
}

/// Chord style: unique chord per hour.
/// Hours 0-11 get major chords, 12-23 get minor chords.
pub fn play_chords(player: &Player, vol: f32, hour: u32) {
    let root = hour_to_root(hour);
    let third = if hour >= 12 { 3 } else { 4 };
    let f1 = note_freq(root);
    let f2 = note_freq(root + third);
    let f3 = note_freq(root + 7);
    let amp = vol / 1.7;

    let mut buf = Buf::new();
    let n = (SR * 0.8) as usize;
    let rc = 1.0 / (2.0 * PI * 3000.0);
    let dt = 1.0 / SR;
    let alpha = dt / (rc + dt);
    let mut lp = 0.0_f32;
    let mut p1 = 0.0_f32;
    let mut p2 = 0.0_f32;
    let mut p3 = 0.0_f32;
    let inc1 = 2.0 * PI * f1 / SR;
    let inc2 = 2.0 * PI * f2 / SR;
    let inc3 = 2.0 * PI * f3 / SR;
    for _ in 0..n {
        let raw = amp * (p1.sin() + p2.sin() + p3.sin()) / 3.0;
        lp += alpha * (raw - lp);
        buf.samples.push(lp);
        p1 += inc1;
        p2 += inc2;
        p3 += inc3;
    }
    buf.play(player);
}

/// Scale style: 5-note ascending run.
/// Hours 0-11 use major intervals, 12-23 use minor intervals.
pub fn play_scale(player: &Player, vol: f32, hour: u32) {
    let root = hour_to_root(hour);
    let intervals: &[i32] = if hour >= 12 {
        &[0, 2, 3, 5, 7]
    } else {
        &[0, 2, 4, 5, 7]
    };

    let mut buf = Buf::new();
    for (i, &interval) in intervals.iter().enumerate() {
        let dur = if i == intervals.len() - 1 { 0.4 } else { 0.2 };
        buf.sine_lp(note_freq(root + interval), dur, 3000.0, vol);
    }
    buf.play(player);
}

/// Ocarina note pitches as semitones from A4.
#[derive(Clone, Copy)]
enum Pitch {
    D4 = -7,
    F4 = -4,
    A4 = 0,
    B4 = 2,
    D5 = 5,
}

/// Musical note durations at ~110 BPM.
#[derive(Clone, Copy)]
enum Len {
    Eighth,
    Quarter,
    DotQuarter,
    Half,
    DotHalf,
}

impl Len {
    const fn secs(self) -> f32 {
        match self {
            Len::Eighth => 0.27,
            Len::Quarter => 0.54,
            Len::DotQuarter => 0.81,
            Len::Half => 1.08,
            Len::DotHalf => 1.54,
        }
    }
}

use Len::*;
use Pitch::*;

impl ZeldaSong {
    fn notes(&self) -> &'static [(Pitch, Len)] {
        match self {
            // 3/4 waltz: half, quarter, dotted half
            ZeldaSong::Lullaby => &[
                (B4, Half),
                (D5, Quarter),
                (A4, DotHalf),
                (B4, Half),
                (D5, Quarter),
                (A4, DotHalf),
            ],
            // 3/4: half, half, half+quarter (held)
            ZeldaSong::Epona => &[
                (D5, Half),
                (B4, Half),
                (A4, DotHalf),
                (D5, Half),
                (B4, Half),
                (A4, DotHalf),
            ],
            // 4/4 bouncy: eighth, eighth, quarter+eighth
            ZeldaSong::Saria => &[
                (F4, Eighth),
                (A4, Eighth),
                (B4, DotQuarter),
                (F4, Eighth),
                (A4, Eighth),
                (B4, DotQuarter),
            ],
            // 4/4 march: quarter, quarter, half
            ZeldaSong::Sun => &[
                (A4, Quarter),
                (F4, Quarter),
                (D5, Half),
                (A4, Quarter),
                (F4, Quarter),
                (D5, Half),
            ],
            // 3/4 stately: half, quarter, dotted half
            ZeldaSong::Time => &[
                (A4, Half),
                (D4, Quarter),
                (F4, DotHalf),
                (A4, Half),
                (D4, Quarter),
                (F4, DotHalf),
            ],
            // 6/8 driving: eighth, eighth, dotted quarter
            ZeldaSong::Storms => &[
                (D4, Eighth),
                (F4, Eighth),
                (D5, DotQuarter),
                (D4, Eighth),
                (F4, Eighth),
                (D5, DotQuarter),
            ],
            // quarters with held final
            ZeldaSong::Minuet => &[
                (D4, Quarter),
                (D5, Quarter),
                (B4, Quarter),
                (A4, Quarter),
                (B4, Quarter),
                (A4, Half),
            ],
            // even quarters, held final
            ZeldaSong::Bolero => &[
                (F4, Quarter),
                (D4, Quarter),
                (F4, Quarter),
                (D4, Quarter),
                (A4, Quarter),
                (F4, Quarter),
                (A4, Quarter),
                (F4, Half),
            ],
            // flowing halves, pickup quarter, held final
            ZeldaSong::Serenade => &[
                (D4, Half),
                (F4, Half),
                (A4, Half),
                (A4, Quarter),
                (B4, DotHalf),
            ],
            // alternating half/quarter, held final
            ZeldaSong::Nocturne => &[
                (B4, Half),
                (A4, Quarter),
                (A4, Quarter),
                (D4, Half),
                (B4, Quarter),
                (A4, Quarter),
                (F4, Half),
            ],
            // three quarters then descending halves
            ZeldaSong::Requiem => &[
                (D4, Quarter),
                (F4, Quarter),
                (D4, Quarter),
                (A4, Half),
                (F4, Half),
                (D4, Half),
            ],
            // quick quarters, held final
            ZeldaSong::Prelude => &[
                (D5, Quarter),
                (A4, Quarter),
                (D5, Quarter),
                (A4, Quarter),
                (B4, Quarter),
                (D5, Half),
            ],
        }
    }
}

/// Ocarina of Time songs.
pub fn play_zelda(player: &Player, vol: f32, song: &ZeldaSong) {
    let notes = song.notes();

    let mut buf = Buf::new();
    for &(pitch, len) in notes {
        buf.sine_lp(note_freq(pitch as i32), len.secs(), 2000.0, vol);
    }
    buf.play(player);
}

/// Wee woo wee woo (for --help).
pub fn play_wee_woo(player: &Player, vol: f32) {
    let mut buf = Buf::new();
    for _ in 0..2 {
        buf.sine_lp(880.0, 0.15, 2000.0, vol);
        buf.sine_lp(660.0, 0.15, 2000.0, vol);
    }
    buf.play(player);
}
