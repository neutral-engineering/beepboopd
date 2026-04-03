mod beep;
mod chords;
mod classical;
mod clock;
mod countries;
mod jazz;
mod scale;
mod wee_woo;
mod zelda;

pub use beep::{BeepPattern, play_beep};
pub use chords::play_chords;
pub use classical::{CLASSICAL_BY_HOUR, ClassicalPiece, play_classical};
pub use clock::play_clock;
pub use countries::{Anthem, COUNTRIES_BY_HOUR, play_countries};
pub use jazz::play_jazz;
pub use scale::play_scale;
pub use wee_woo::play_wee_woo;
pub use zelda::{ZELDA_BY_HOUR, ZeldaSong, play_zelda};

use clap::ValueEnum;
use rodio::Player;
use rodio::buffer::SamplesBuffer;
use std::f32::consts::PI;
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
    Jazz,
    Classical,
    Countries,
}

/// Convert beats at a given BPM to seconds.
fn beat(beats: f32, bpm: f32) -> f32 {
    beats * 60.0 / bpm
}

/// Frequency for a note given as semitones from A4 (440 Hz).
fn note_freq(semitones_from_a4: i32) -> f32 {
    440.0 * 2.0_f32.powf(semitones_from_a4 as f32 / 12.0)
}

/// Map hour (0-23) to root note as semitones from A4.
fn hour_to_root(hour: u32) -> i32 {
    (hour % 12) as i32 - 9
}

/// Pre-renders all samples into a single continuous buffer (like ffmpeg does).
/// Tracks phase across notes so frequency changes don't click.
pub(crate) struct Buf {
    samples: Vec<f32>,
    phase: f32,
    lp_state: f32,
}

impl Buf {
    pub(crate) fn new() -> Self {
        Buf {
            samples: Vec::new(),
            phase: 0.0,
            lp_state: 0.0,
        }
    }

    pub(crate) fn sine_lp(&mut self, freq: f32, secs: f32, cutoff: f32, vol: f32) {
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

    pub(crate) fn silence(&mut self, secs: f32) {
        let n = (SR * secs) as usize;
        let rc = 1.0 / (2.0 * PI * 200.0);
        let dt = 1.0 / SR;
        let alpha = dt / (rc + dt);
        for _ in 0..n {
            self.lp_state *= 1.0 - alpha;
            self.samples.push(self.lp_state);
        }
        self.phase = 0.0;
    }

    pub(crate) fn play(mut self, player: &Player) {
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
