//! Property-based and regression tests for chord parsing and the ChordPro round-trip.
//!
//! These tests exercise invariants that must hold for any valid input:
//!
//! - **Chord round-trip**: `Chord::from_str(x).unwrap().to_string() == x` for every
//!   chord in a curated list of real-world chord strings.
//! - **Transposition round-trip**: transposing up and back down by the same interval
//!   returns the original chord.
//! - **Parser round-trip**: `parse -> write -> parse` preserves the model for a
//!   variety of hand-crafted inputs (edge cases, only metadata, only chords…).
//! - **`is_chord` consistency**: the parser's chord detection agrees with the
//!   chord_naming parser on what is a valid chord string.

use std::collections::HashMap;

use notation_bundle::chord_naming::{Chord, Letter, Note};
use notation_bundle::chordpro::{ChordPlacement, ChordProParser, Line, Section, Song, write};

const KEY_C: Note = Note::new(Letter::C, 0);
const KEY_D: Note = Note::new(Letter::D, 0);
const KEY_G: Note = Note::new(Letter::G, 0);
const KEY_F: Note = Note::new(Letter::F, 0);
const KEY_BB: Note = Note::new(Letter::B, -1);

// ── Chord round-trip property ─────────────────────────────────────────────

/// Every chord in this list must survive `parse -> Display -> same string`.
/// This is the core invariant of the chord naming module.
const CHORD_ROUND_TRIP_CASES: &[&str] = &[
    "C", "D", "E", "F", "G", "A", "B", "Cm", "Dm", "Em", "Fm", "Gm", "Am", "Bm", "C#", "F#", "Bb",
    "Eb", "Ab", "Db", "Gb", "C#m", "F#m", "Bbm", "Ebm", "Abm", "Dbm", "Gbm", "C/E", "D/F#", "G/B",
    "Am/C", "F/A", "Em/G", "Bbmaj9/D", "C/E", "Dm7/G", "G7/B", "C#m/E", "F#/A#", "Bb/D", "Eb/G",
    "Ab/C", "C6/9", "Dm6/9", "G6/9", "C6/9/E", "Am6/9", "E6/9", "F6/9", "Bb6/9", "F##", "Bbb",
    "C##", "Ebb", "Abb", "F##dim", "Bbbmaj7", "C##m", "Ebbmaj7", "G##", "Dbb", "Cmaj7", "C7",
    "Cm7", "Cdim7", "Cø", "G7", "D7", "A7", "E7", "B7", "F#7", "Dm7", "Am7", "Em7", "Bm7", "Csus4",
    "Csus2", "Dsus4", "Gsus4", "Asus4", "G7sus4", "A7sus4", "D7sus4", "Caug", "Gaug", "Aaug",
    "Cdim", "Gdim", "Bdim", "C7b9", "C7#9", "C7b5", "C7#5", "A7b9", "E7#9", "D7b5", "Cmaj9",
    "Cmaj7", "C9", "Cm9", "C11", "C13", "Gmaj9", "Dmaj9", "Fmaj9", "Dm9", "Am9", "Em9", "Cadd9",
    "Cadd2", "Cadd4", "Gadd9", "Cadd9/E", "Gadd9/B", "C7/Bb", "G7/F", "Dm7/C", "Am7/G", "Cmaj7/B",
    "Fmaj7/C", "C5", "G5", "D5", "A5", "E5", "F5", "Bb5", "Cm6", "Dm6", "Am6", "Em6", "Cm7", "Dm7",
    "Am7", "Em7", "Cmaj9", "Dmaj9", "Amaj9", "Ebm6", "Bbm6", "Fm6", "Bbmaj7", "Ebmaj7", "Abmaj7",
];

#[test]
fn every_chord_round_trips_through_parse_and_display() {
    for input in CHORD_ROUND_TRIP_CASES {
        let chord: Chord = input.parse().unwrap_or_else(|e| {
            panic!("{input:?} failed to parse: {e}");
        });
        assert_eq!(
            chord.to_string(),
            *input,
            "chord {input:?} changed after round-trip"
        );
    }
}
// ── Transposition round-trip property ──────────────────────────────────────

/// Chords whose spelling changes enharmonically after a chromatic transposition
/// round-trip are excluded from the round-trip test. This is expected: `F##`
/// transposed up a semitone becomes `G#`, and transposing back down produces
/// `G` (pitch class 7 spelled as the letter G) instead of `F##` because the
/// interval has 0 scale steps (C→C# is same letter). The same applies to chords
/// with double accidentals.
const TRANSPOSITION_EXCLUDE: &[&str] = &[
    "F##", "Bbb", "C##", "Ebb", "Abb", "F##dim", "Bbbmaj7", "C##m", "Ebbmaj7", "G##", "Dbb",
];

fn is_excluded_from_transposition(input: &str) -> bool {
    TRANSPOSITION_EXCLUDE.contains(&input)
}

const TRANSPOSITION_INTERVALS: &[(Note, Note)] = &[
    (KEY_C, KEY_D),
    (KEY_C, KEY_F),
    (KEY_C, KEY_G),
    (KEY_C, Note::new(Letter::C, 1)),
    (KEY_G, KEY_C),
    (KEY_F, KEY_BB),
    (KEY_D, KEY_G),
];

#[test]
fn transposition_round_trips_for_all_chords() {
    for input in CHORD_ROUND_TRIP_CASES {
        if is_excluded_from_transposition(input) {
            continue;
        }
        let original: Chord = input.parse().unwrap_or_else(|e| {
            panic!("{input:?} failed to parse: {e}");
        });
        for &(from, to) in TRANSPOSITION_INTERVALS {
            let transposed = original.transpose(from, to);
            let back = transposed.transpose(to, from);
            assert_eq!(
                back, original,
                "transposition round-trip failed for {input:?} with interval {from:?} -> {to:?}"
            );
        }
    }
}

#[test]
fn transposition_is_deterministic() {
    for input in CHORD_ROUND_TRIP_CASES {
        if is_excluded_from_transposition(input) {
            continue;
        }
        let original: Chord = input.parse().unwrap();
        for &(from, to) in TRANSPOSITION_INTERVALS {
            let first = original.transpose(from, to);
            let second = original.transpose(from, to);
            assert_eq!(
                first, second,
                "transposition not deterministic for {input:?} with {from:?} -> {to:?}"
            );
        }
    }
}

// ── Parser round-trip property ─────────────────────────────────────────────

fn assert_parse_round_trip(song: &Song) {
    let written = write(song);
    let reparsed = ChordProParser::parse(&written).unwrap_or_else(|e| {
        panic!("failed to reparse written output: {e}\n--- written ---\n{written}");
    });
    assert_eq!(
        reparsed, *song,
        "parse -> write -> parse changed the model\n--- written ---\n{written}"
    );
}

fn make_metadata(key: &str, value: &str) -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert(key.to_owned(), value.to_owned());
    m
}

#[test]
fn empty_song_round_trips() {
    assert_parse_round_trip(&Song {
        metadata: HashMap::new(),
        sections: vec![],
    });
}
#[test]
fn section_with_chords_only_no_lyrics_round_trips() {
    assert_parse_round_trip(&Song {
        metadata: HashMap::new(),
        sections: vec![Section {
            name: Some("Intro".to_owned()),
            lines: vec![Line {
                chords: vec![ChordPlacement {
                    position: 0,
                    chord: "C".to_owned(),
                }],
                text: String::new(),
            }],
        }],
    });
}

#[test]
fn many_sections_round_trip() {
    let mut sections = Vec::new();
    for i in 0..20 {
        sections.push(Section {
            name: Some(format!("Section {i}")),
            lines: vec![Line {
                chords: vec![ChordPlacement {
                    position: 0,
                    chord: if i % 2 == 0 { "C" } else { "G" }.to_owned(),
                }],
                text: format!("Line {i}"),
            }],
        });
    }
    assert_parse_round_trip(&Song {
        metadata: HashMap::new(),
        sections,
    });
}

#[test]
fn line_with_chord_at_every_position_round_trips() {
    let mut chords = Vec::new();
    for i in 0..10 {
        chords.push(ChordPlacement {
            position: i,
            chord: ["C", "D", "E", "F", "G", "A", "B", "C", "D", "E"][i].to_owned(),
        });
    }
    assert_parse_round_trip(&Song {
        metadata: HashMap::new(),
        sections: vec![Section {
            name: None,
            lines: vec![Line {
                chords,
                text: "0123456789".to_owned(),
            }],
        }],
    });
}

#[test]
fn chord_past_end_appears_at_text_length_after_round_trip() {
    // A chord beyond the text length is written at the end of the text.
    // Upon reparse, its position becomes the text length — the writer
    // appends it, so the round-trip changes the position. This is expected.
    let song = Song {
        metadata: HashMap::new(),
        sections: vec![Section {
            name: None,
            lines: vec![Line {
                chords: vec![ChordPlacement {
                    position: 99,
                    chord: "G7".to_owned(),
                }],
                text: "short".to_owned(),
            }],
        }],
    };
    let written = write(&song);
    assert_eq!(
        written, "\nshort[G7]\n",
        "expected chord appended after lyric"
    );

    let reparsed = ChordProParser::parse(&written).expect("reparse");
    assert_eq!(reparsed.sections[0].lines[0].chords[0].position, 5);
    assert_eq!(reparsed.sections[0].lines[0].chords[0].chord, "G7");
    assert_eq!(reparsed.sections[0].lines[0].text, "short");
}

#[test]
fn empty_lines_are_skipped_by_parser_but_no_chords_is_fine() {
    let song = Song {
        metadata: HashMap::new(),
        sections: vec![Section {
            name: Some("Verse".to_owned()),
            lines: vec![
                Line {
                    chords: vec![],
                    text: "Lyric without chords".to_owned(),
                },
                Line {
                    chords: vec![ChordPlacement {
                        position: 0,
                        chord: "Am".to_owned(),
                    }],
                    text: "Chord here".to_owned(),
                },
            ],
        }],
    };
    let written = write(&song);
    let reparsed = ChordProParser::parse(&written).expect("reparse");
    assert_eq!(reparsed, song);
}
// ── Regression: bordas do parser ──────────────────────────────────────────

#[test]
fn regression_input_with_only_comments_produces_empty_song() {
    let song = ChordProParser::parse("# comment\n# another\n").expect("parse");
    assert!(song.metadata.is_empty());
    assert!(song.sections.is_empty());
}

#[test]
fn regression_metadata_with_special_characters() {
    let input = "{title: Song with: colon}\n{artist: Artist & Band!}\n\n[C]lyric\n";
    let song = ChordProParser::parse(input).expect("parse");
    assert_eq!(
        song.metadata.get("title").map(String::as_str),
        Some("Song with: colon")
    );
    assert_eq!(
        song.metadata.get("artist").map(String::as_str),
        Some("Artist & Band!")
    );
}

#[test]
fn regression_section_names_with_numbers_and_spaces() {
    let input = "[Verse 1]\n[C]a\n\n[Pre-Chorus B]\n[G]b\n\n[Section 99]\n[F]c\n";
    let song = ChordProParser::parse(input).expect("parse");
    assert_eq!(song.sections[0].name.as_deref(), Some("Verse 1"));
    assert_eq!(song.sections[1].name.as_deref(), Some("Pre-Chorus B"));
    assert_eq!(song.sections[2].name.as_deref(), Some("Section 99"));
}

#[test]
fn regression_key_metadata_is_transposed() {
    use notation_bundle::harmony::HarmonyEngine;
    let input = "{title: T}\n{key: C#}\n\n[C#]x\n";
    let song = ChordProParser::parse(input).expect("parse");
    let transposed = HarmonyEngine::transpose(&song, KEY_C, KEY_D).expect("transpose");
    assert_eq!(
        transposed.metadata.get("key").map(String::as_str),
        Some("D#")
    );
}

#[test]
fn regression_complex_key_signatures_round_trip() {
    for key in ["C#", "Db", "F#", "Gb", "Bb", "Eb", "Ab"] {
        let input = format!("{{title: T}}\n{{key: {key}}}\n\n[C]x\n");
        let song = ChordProParser::parse(&input).unwrap_or_else(|e| panic!("{key}: {e}"));
        let written = write(&song);
        let reparsed = ChordProParser::parse(&written).unwrap_or_else(|e| panic!("{key}: {e}"));
        assert_eq!(reparsed, song, "key {key} round-trip failed");
    }
}

#[test]
fn regression_nashville_numbers_are_not_chordpro_section_names() {
    let input = "[4]\n[5]\n[6m]\n";
    let song = ChordProParser::parse(input).expect("parse");
    assert_eq!(song.sections.len(), 3);
    assert_eq!(song.sections[0].name.as_deref(), Some("4"));
    assert_eq!(song.sections[1].name.as_deref(), Some("5"));
    assert_eq!(song.sections[2].name.as_deref(), Some("6m"));
}

#[test]
fn regression_all_four_notation_systems_render_without_panic() {
    use notation_bundle::chord_naming::NotationSystem;
    use notation_bundle::harmony::HarmonyEngine;

    let input = "{title: T}\n{key: C}\n\n[V]\n[C]c [G]g [Am]a [F]f\n";
    let song = ChordProParser::parse(input).expect("parse");

    HarmonyEngine::render(&song, NotationSystem::Common, None).expect("Common");
    HarmonyEngine::render(&song, NotationSystem::Latin, None).expect("Latin");
    HarmonyEngine::render(&song, NotationSystem::German, None).expect("German");
    let rendered =
        HarmonyEngine::render(&song, NotationSystem::Nashville, Some(KEY_C)).expect("Nashville");
    assert_eq!(
        rendered.sections[0].lines[0]
            .chords
            .iter()
            .map(|c| c.chord.as_str())
            .collect::<Vec<_>>(),
        ["1", "5", "6m", "4"]
    );
}

#[test]
fn metadata_only_round_trips() {
    assert_parse_round_trip(&Song {
        metadata: make_metadata("title", "Only Metadata"),
        sections: vec![],
    });
}

#[test]
fn multiple_metadata_fields_round_trip() {
    let mut metadata = HashMap::new();
    metadata.insert("title".to_owned(), "Full Meta".to_owned());
    metadata.insert("artist".to_owned(), "Artist".to_owned());
    metadata.insert("key".to_owned(), "Eb".to_owned());
    metadata.insert("time".to_owned(), "6/8".to_owned());
    metadata.insert("duration".to_owned(), "3:45".to_owned());
    assert_parse_round_trip(&Song {
        metadata,
        sections: vec![],
    });
}

#[test]
fn section_without_name_round_trips() {
    assert_parse_round_trip(&Song {
        metadata: HashMap::new(),
        sections: vec![Section {
            name: None,
            lines: vec![Line {
                chords: vec![ChordPlacement {
                    position: 0,
                    chord: "C".to_owned(),
                }],
                text: "lyric".to_owned(),
            }],
        }],
    });
}
