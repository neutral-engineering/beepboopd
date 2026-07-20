use super::{Buf, beat, note_freq};
use clap::ValueEnum;
use rodio::Player;
use std::fmt;

#[derive(Clone, Copy, ValueEnum)]
pub enum Anthem {
    Usa,
    Uk,
    France,
    Germany,
    Japan,
    Brazil,
    Australia,
    Canada,
    Mexico,
    Italy,
    SouthKorea,
    India,
}

impl fmt::Display for Anthem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Anthem::Usa => write!(f, "usa"),
            Anthem::Uk => write!(f, "uk"),
            Anthem::France => write!(f, "france"),
            Anthem::Germany => write!(f, "germany"),
            Anthem::Japan => write!(f, "japan"),
            Anthem::Brazil => write!(f, "brazil"),
            Anthem::Australia => write!(f, "australia"),
            Anthem::Canada => write!(f, "canada"),
            Anthem::Mexico => write!(f, "mexico"),
            Anthem::Italy => write!(f, "italy"),
            Anthem::SouthKorea => write!(f, "south_korea"),
            Anthem::India => write!(f, "india"),
        }
    }
}

pub const COUNTRIES_BY_HOUR: [Anthem; 12] = [
    Anthem::Usa,        // 00, 12
    Anthem::Uk,         // 01, 13
    Anthem::France,     // 02, 14
    Anthem::Germany,    // 03, 15
    Anthem::Japan,      // 04, 16
    Anthem::Brazil,     // 05, 17
    Anthem::Australia,  // 06, 18
    Anthem::Canada,     // 07, 19
    Anthem::Mexico,     // 08, 20
    Anthem::Italy,      // 09, 21
    Anthem::SouthKorea, // 10, 22
    Anthem::India,      // 11, 23
];

impl Anthem {
    /// Opening melody as (semitones from A4, beats).
    fn notes(self) -> &'static [(i32, f32)] {
        match self {
            // "Oh say can you see, by the dawn's early light"
            // Key of Bb, 3/4.
            Anthem::Usa => &[
                (-4, 1.5),  // F4  — dotted quarter (Oh)
                (-7, 0.5),  // D4  — eighth (say)
                (-10, 1.0), // Bb3 — quarter (can you)
                (-7, 1.0),  // D4  — quarter (see)
                (-4, 1.0),  // F4  — quarter (by the)
                (1, 2.0),   // Bb4 — half (dawn's)
                (0, 1.5),   // A4  — dotted quarter (ear-)
                (-2, 0.5),  // G4  — eighth (-ly)
                (0, 1.0),   // A4  — quarter (light)
                (1, 2.0),   // Bb4 — half
            ],
            // "God save our gracious King" — from MIDI
            // Key of G, 3/4.
            Anthem::Uk => &[
                (-2, 1.0),  // G4
                (-2, 1.0),  // G4
                (0, 1.0),   // A4
                (-3, 1.5),  // F#4
                (-2, 0.5),  // G4
                (0, 1.0),   // A4
                (2, 1.0),   // B4
                (2, 1.0),   // B4
                (3, 1.0),   // C5
                (2, 1.5),   // B4
            ],
            // "Allons enfants de la patrie"
            // Key of G, 4/4.
            Anthem::France => &[
                (-2, 0.5),  // G4  (Al-)
                (-2, 0.5),  // G4  (-lons)
                (-2, 1.0),  // G4  (en-)
                (2, 1.0),   // B4  (-fants)
                (2, 1.0),   // B4  (de)
                (2, 1.0),   // B4  (la)
                (5, 2.0),   // D5  (pa-)
                (2, 1.0),   // B4  (-tri-)
                (-2, 1.0),  // G4  (-e)
            ],
            // "Einigkeit und Recht und Freiheit" — from MIDI
            // Key of Eb, 4/4 (Haydn melody).
            Anthem::Germany => &[
                (6, 1.5),   // Eb5
                (8, 0.5),   // F5
                (10, 1.0),  // G5
                (8, 1.0),   // F5
                (11, 1.0),  // Ab5
                (10, 1.0),  // G5
                (8, 0.5),   // F5
                (5, 0.5),   // D5
                (6, 1.0),   // Eb5
                (6, 1.0),   // Eb5
            ],
            // "Kimigayo wa" — from MIDI
            // Key of D, 4/4. Pentatonic, solemn.
            Anthem::Japan => &[
                (-7, 1.0),  // D4
                (-9, 1.0),  // C4
                (-7, 1.0),  // D4
                (-5, 1.0),  // E4
                (-2, 1.0),  // G4
                (-5, 1.0),  // E4
                (-7, 1.5),  // D4
                (-5, 1.0),  // E4
                (-2, 1.0),  // G4
                (0, 1.0),   // A4
            ],
            // "Ouviram do Ipiranga as margens plácidas"
            // Key of Eb, 4/4.
            Anthem::Brazil => &[
                (-6, 0.5),  // Eb4
                (-6, 0.5),  // Eb4
                (-6, 1.0),  // Eb4
                (-1, 1.0),  // Ab4
                (-1, 1.0),  // Ab4
                (-1, 1.0),  // Ab4
                (1, 2.0),   // Bb4
                (-1, 1.0),  // Ab4
                (-2, 1.0),  // G4
            ],
            // "Australians all let us rejoice" — from MIDI (transposed -12)
            // Key of C, 4/4.
            Anthem::Australia => &[
                (-2, 1.0),  // G4
                (3, 1.5),   // C5
                (-2, 0.5),  // G4
                (-5, 1.0),  // E4
                (-2, 1.0),  // G4
                (3, 1.5),   // C5
                (3, 0.5),   // C5
                (3, 1.0),   // C5
                (7, 1.0),   // E5
                (5, 1.0),   // D5
            ],
            // "O Canada, our home and native land" — from MIDI
            // Key of Eb, 4/4.
            Anthem::Canada => &[
                (-2, 2.0),  // G4  (O)
                (1, 1.5),   // Bb4 (Ca-)
                (1, 0.5),   // Bb4 (-na-)
                (-6, 3.0),  // Eb4 (-da)
                (-4, 1.0),  // F4
                (-2, 1.0),  // G4
                (-1, 1.0),  // Ab4
                (-1, 1.0),  // Ab4
                (1, 1.0),   // Bb4
                (3, 1.0),   // C5
            ],
            // "Mexicanos, al grito de guerra" — from MIDI
            // Key of C, 4/4.
            Anthem::Mexico => &[
                (3, 1.0),   // C5  (Me-)
                (7, 0.5),   // E5  (-xi-)
                (10, 1.0),  // G5  (-ca-)
                (10, 0.5),  // G5  (-nos)
                (10, 1.0),  // G5  (al)
                (10, 1.0),  // G5
                (12, 0.5),  // A5
                (14, 0.5),  // B5
                (15, 2.0),  // C6  (gue-)
            ],
            // "Fratelli d'Italia" — from MIDI
            // Key of Bb, 4/4.
            Anthem::Italy => &[
                (-4, 0.5),  // F4  (Fra-)
                (-4, 0.5),  // F4  (-tel-)
                (-2, 0.5),  // G4  (-li)
                (-4, 2.0),  // F4  (d'I-)
                (5, 1.0),   // D5  (-ta-)
                (5, 1.0),   // D5  (-lia)
                (6, 0.5),   // Eb5
                (5, 2.0),   // D5
                (5, 1.0),   // D5
                (8, 0.5),   // F5
            ],
            // "Donghaemulgwa baekdusani" — from MIDI
            // Key of A, 4/4.
            Anthem::SouthKorea => &[
                (-5, 1.0),  // E4
                (0, 1.5),   // A4
                (-5, 1.0),  // E4
                (-1, 0.5),  // G#4
                (-3, 1.0),  // F#4
                (0, 1.0),   // A4
                (-5, 1.0),  // E4
                (-8, 1.0),  // C#4
                (-5, 1.0),  // E4
                (0, 1.0),   // A4
            ],
            // "Jana gana mana adhinayaka" — from MIDI
            // Key of Ab, 4/4.
            Anthem::India => &[
                (-6, 0.5),  // Eb4 (Ja-)
                (-4, 0.5),  // F4  (-na)
                (-2, 0.5),  // G4  (ga-)
                (-2, 0.5),  // G4  (-na)
                (-2, 0.5),  // G4  (ma-)
                (-2, 0.5),  // G4  (-na)
                (-2, 0.5),  // G4  (a-)
                (-2, 0.5),  // G4  (-dhi-)
                (-2, 1.0),  // G4  (-na-)
                (-2, 0.5),  // G4  (-ya-)
            ],
        }
    }

    fn default_bpm(self) -> f32 {
        match self {
            Anthem::Usa => 90.0,
            Anthem::Uk => 88.0,
            Anthem::France => 120.0,
            Anthem::Germany => 77.0,
            Anthem::Japan => 78.0,
            Anthem::Brazil => 100.0,
            Anthem::Australia => 86.0,
            Anthem::Canada => 100.0,
            Anthem::Mexico => 78.0,
            Anthem::Italy => 89.0,
            Anthem::SouthKorea => 65.0,
            Anthem::India => 90.0,
        }
    }
}

pub fn play_countries(player: &Player, vol: f32, bpm: Option<f32>, anthem: &Anthem) {
    let bpm = bpm.unwrap_or(anthem.default_bpm());

    let mut buf = Buf::new();
    for &(semitone, beats) in anthem.notes() {
        buf.sine_lp(note_freq(semitone), beat(beats, bpm), 2500.0, vol);
    }
    buf.play(player);
}
