use super::{Buf, beat, hour_to_root, note_freq};
use clap::ValueEnum;
use rodio::Player;
use std::fmt;

#[derive(Clone, Copy, ValueEnum)]
pub enum ClassicalPiece {
    Shrimp,
    Satie,
    FurElise,
    SwanLake,
    ClairDeLune,
}

impl fmt::Display for ClassicalPiece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClassicalPiece::Shrimp => write!(f, "shrimp"),
            ClassicalPiece::Satie => write!(f, "satie"),
            ClassicalPiece::FurElise => write!(f, "fur_elise"),
            ClassicalPiece::SwanLake => write!(f, "swan_lake"),
            ClassicalPiece::ClairDeLune => write!(f, "clair_de_lune"),
        }
    }
}

impl ClassicalPiece {
    fn notes(self) -> &'static [(i32, f32)] {
        match self {
            ClassicalPiece::Shrimp => SHRIMP,
            ClassicalPiece::Satie => SATIE,
            ClassicalPiece::FurElise => FUR_ELISE,
            ClassicalPiece::SwanLake => SWAN_LAKE,
            ClassicalPiece::ClairDeLune => CLAIR_DE_LUNE,
        }
    }

    fn default_bpm(self) -> f32 {
        match self {
            ClassicalPiece::Shrimp => 72.0,
            ClassicalPiece::Satie => 72.0,
            ClassicalPiece::FurElise => 140.0,
            ClassicalPiece::SwanLake => 100.0,
            ClassicalPiece::ClairDeLune => 54.0,
        }
    }
}

pub const CLASSICAL_BY_HOUR: [ClassicalPiece; 5] = [
    ClassicalPiece::Shrimp,
    ClassicalPiece::Satie,
    ClassicalPiece::FurElise,
    ClassicalPiece::SwanLake,
    ClassicalPiece::ClairDeLune,
];

/// Shrimp Quartet — justan oval, melody in G major (72 BPM).
const SHRIMP: &[(i32, f32)] = &[
    (-3, 1.0), // E  — quarter
    (-1, 0.5), // F# — eighth
    (0, 0.5),  // G  — eighth
    (2, 1.0),  // A  — quarter
    (1, 1.0),  // D  — quarter
    (5, 1.0),  // C  — quarter
    (0, 0.5),  // G  — eighth
    (-1, 0.5), // F# — eighth
    (4, 1.0),  // B  — quarter
    (2, 1.0),  // A  — quarter
];

/// Gymnopédie No. 1 — Erik Satie, opening melody in D major (72 BPM, 3/4).
/// Long sustained notes, dreamy and sparse.
const SATIE: &[(i32, f32)] = &[
    (4, 3.0),  // F# — dotted half
    (0, 3.0),  // D  — dotted half
    (2, 2.0),  // E  — half
    (-1, 1.0), // C# — quarter
    (0, 2.0),  // D  — half
    (-3, 1.0), // B  — quarter
    (-5, 3.0), // A  — dotted half
    (-3, 3.0), // B  — dotted half
];

/// Für Elise — Beethoven, opening motif in A minor (140 BPM eighth notes).
const FUR_ELISE: &[(i32, f32)] = &[
    (7, 1.0),  // E5  — eighth
    (6, 1.0),  // D#5 — eighth
    (7, 1.0),  // E5  — eighth
    (6, 1.0),  // D#5 — eighth
    (7, 1.0),  // E5  — eighth
    (2, 1.0),  // B4  — eighth
    (5, 1.0),  // D5  — eighth
    (3, 1.0),  // C5  — eighth
    (0, 2.0),  // A4  — quarter
];

/// Swan Lake — Tchaikovsky, main theme in B minor (100 BPM).
const SWAN_LAKE: &[(i32, f32)] = &[
    (0, 2.0),  // B  — half
    (2, 1.0),  // C# — quarter
    (3, 1.0),  // D  — quarter
    (5, 2.0),  // E  — half
    (7, 1.0),  // F# — quarter
    (5, 1.0),  // E  — quarter
    (3, 2.0),  // D  — half
    (2, 1.0),  // C# — quarter
    (0, 1.0),  // B  — quarter
    (-2, 2.0), // A  — half
];

/// Clair de Lune — Debussy, opening melody in Db major (54 BPM, 9/8).
const CLAIR_DE_LUNE: &[(i32, f32)] = &[
    (-3, 2.0), // Bb — half
    (-5, 1.0), // Ab — quarter
    (4, 2.0),  // F  — half
    (0, 1.0),  // Db — quarter
    (2, 1.0),  // Eb — quarter
    (4, 2.0),  // F  — half
    (-5, 1.0), // Ab — quarter
    (-3, 3.0), // Bb — dotted half
];

pub fn play_classical(player: &Player, vol: f32, bpm: Option<f32>, hour: u32, piece: &ClassicalPiece) {
    let root = hour_to_root(hour);
    let bpm = bpm.unwrap_or(piece.default_bpm());

    let mut buf = Buf::new();
    for &(interval, beats) in piece.notes() {
        buf.sine_lp(note_freq(root + interval), beat(beats, bpm), 2500.0, vol);
    }
    buf.play(player);
}
