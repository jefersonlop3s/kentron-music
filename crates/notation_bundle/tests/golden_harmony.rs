//! Golden tests for transposition and notation rendering.
//!
//! If a value changes here, the notation rules in the master plan changed too.

use std::fs;
use std::path::PathBuf;

use notation_bundle::chord_naming::{Letter, NotationSystem, Note};
use notation_bundle::chordpro::{ChordProParser, Song};
use notation_bundle::harmony::HarmonyEngine;

const KEY_C: Note = Note::new(Letter::C, 0);
const KEY_D: Note = Note::new(Letter::D, 0);

fn golden(name: &str) -> String {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/golden_harmony");
    path.push(name);
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn load_song() -> Song {
    ChordProParser::parse(&golden("worship.cho")).expect("golden song must parse")
}

/// One line per song line, chords separated by spaces.
fn flatten(song: &Song) -> String {
    song.sections
        .iter()
        .flat_map(|section| section.lines.iter())
        .map(|line| {
            line.chords
                .iter()
                .map(|placement| placement.chord.as_str())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn transposes_the_golden_song_up_two_semitones() {
    let transposed = HarmonyEngine::transpose(&load_song(), KEY_C, KEY_D).expect("transpose");

    assert_eq!(
        transposed.metadata.get("key").map(String::as_str),
        Some("D")
    );
    assert_eq!(
        flatten(&transposed),
        golden("worship_up_two_semitones.txt").trim_end()
    );
}

#[test]
fn renders_the_transposed_song_in_latin() {
    let rendered =
        HarmonyEngine::transpose_and_render(&load_song(), KEY_C, KEY_D, NotationSystem::Latin)
            .expect("transpose and render");

    assert_eq!(
        flatten(&rendered),
        golden("worship_up_two_semitones_latin.txt").trim_end()
    );
}
