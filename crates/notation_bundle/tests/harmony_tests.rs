//! Harmony engine tests: transposition and notation rendering over a whole song.

use std::collections::HashMap;

use notation_bundle::chord_naming::{Letter, NotationSystem, Note, RenderError};
use notation_bundle::chordpro::{ChordPlacement, ChordProParser, Line, Section, Song};
use notation_bundle::harmony::{HarmonyEngine, HarmonyError};

const KEY_C: Note = Note::new(Letter::C, 0);
const KEY_D: Note = Note::new(Letter::D, 0);

fn sample() -> Song {
    ChordProParser::parse(
        "{title: Teste}\n{key: C}\n\n[Verse]\n[C]Amazing [G]grace how [Am]sweet the [F]sound\n",
    )
    .expect("sample song must parse")
}

/// Chords of the first line of the first section, as text.
fn first_line_chords(song: &Song) -> Vec<&str> {
    song.sections[0].lines[0]
        .chords
        .iter()
        .map(|placement| placement.chord.as_str())
        .collect()
}

#[test]
fn transposes_every_chord_and_the_key_metadata() {
    let out = HarmonyEngine::transpose(&sample(), KEY_C, KEY_D).expect("transpose");

    assert_eq!(out.metadata.get("key").map(String::as_str), Some("D"));
    assert_eq!(first_line_chords(&out), ["D", "A", "Bm", "G"]);
}

#[test]
fn transposition_keeps_lyrics_and_section_names() {
    let input = sample();
    let out = HarmonyEngine::transpose(&input, KEY_C, KEY_D).expect("transpose");

    assert_eq!(out.sections.len(), input.sections.len());
    assert_eq!(out.sections[0].name, input.sections[0].name);
    assert_eq!(
        out.sections[0].lines[0].text,
        input.sections[0].lines[0].text
    );
}

#[test]
fn renders_in_latin_notation() {
    let out = HarmonyEngine::render(&sample(), NotationSystem::Latin, None).expect("render");

    // Latin separates the root from a suffix that starts with a letter.
    assert_eq!(first_line_chords(&out), ["Do", "Sol", "La m", "Fa"]);
}

#[test]
fn renders_in_german_notation() {
    let out = HarmonyEngine::render(&sample(), NotationSystem::German, None).expect("render");

    assert_eq!(first_line_chords(&out), ["C", "G", "Am", "F"]);
}

#[test]
fn nashville_requires_a_key() {
    let error = HarmonyEngine::render(&sample(), NotationSystem::Nashville, None)
        .expect_err("Nashville needs a key");

    assert_eq!(error, HarmonyError::Render(RenderError::KeyRequired));
    assert_eq!(error.message_key(), "chord.render.key_required");
}

#[test]
fn transpose_and_render_nashville_uses_the_target_key() {
    let out =
        HarmonyEngine::transpose_and_render(&sample(), KEY_C, KEY_D, NotationSystem::Nashville)
            .expect("transpose and render");

    // In D major: D is 1, A is 5, Bm is 6m, G is 4.
    assert_eq!(first_line_chords(&out), ["1", "5", "6m", "4"]);
}

#[test]
fn invalid_chord_reports_a_parse_error() {
    let song = Song {
        metadata: HashMap::new(),
        sections: vec![Section {
            name: None,
            lines: vec![Line {
                chords: vec![ChordPlacement {
                    position: 0,
                    chord: "H7".into(),
                }],
                text: "texto".into(),
            }],
        }],
    };

    let error = HarmonyEngine::transpose(&song, KEY_C, KEY_D).expect_err("H7 is not a chord");
    assert!(matches!(error, HarmonyError::Parse(_)));
    assert_eq!(error.message_key(), "chord.parse.invalid_root");
}
