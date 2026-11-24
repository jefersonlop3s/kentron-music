//! Golden tests for ChordPro parsing and round-trip preservation.
//!
//! Each fixture proves that the parser correctly reads a `.cho` file and that
//! `parse -> write -> parse` preserves the model exactly. This is the gate
//! that proves advanced chord types, double accidentals, inverted chords,
//! `6/9` suffixes, and multilingual lyrics survive storage and retrieval.

use notation_bundle::chordpro::{ChordProParser, write};
use std::fs;
use std::path::PathBuf;

fn load_file(name: &str) -> String {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/golden_chordpro");
    path.push(format!("{}.cho", name));
    fs::read_to_string(path).expect("Failed to read golden file")
}

/// Parses a `.cho` fixture, writes it back, re-parses, and asserts the two
/// models are equal. Proves the round-trip preserves all content.
fn assert_round_trip(name: &str) {
    let input = load_file(name);
    let song = ChordProParser::parse(&input).unwrap_or_else(|e| {
        panic!("{name}: failed to parse: {e}");
    });

    let written = write(&song);
    let reparsed = ChordProParser::parse(&written).unwrap_or_else(|e| {
        panic!("{name}: failed to reparse written output: {e}\n--- written ---\n{written}");
    });

    assert_eq!(
        reparsed, song,
        "{name}: round-trip (parse → write → parse) changed the model"
    );
}

#[test]
fn test_basic_song_golden() {
    let input = load_file("basic_song");
    let song = ChordProParser::parse(&input).expect("Parse failed");

    assert_eq!(song.metadata.get("title").unwrap(), "Golden Song");
    assert_eq!(song.metadata.get("artist").unwrap(), "Golden Artist");
    assert_eq!(song.sections.len(), 2);
    assert_eq!(song.sections[0].lines[0].text, " Let's start  here");
    assert_eq!(song.sections[1].lines[0].text, " This is the  chorus");
}

#[test]
fn test_multi_section_golden() {
    let input = load_file("multi_section");
    let song = ChordProParser::parse(&input).expect("Parse failed");

    assert_eq!(song.metadata.get("title").unwrap(), "Multi-Section Song");
    assert_eq!(song.sections.len(), 4);
    assert_eq!(song.sections[0].name.as_ref().unwrap(), "Verse 1");
    assert_eq!(song.sections[1].name.as_ref().unwrap(), "Chorus");
    assert_eq!(song.sections[2].name.as_ref().unwrap(), "Verse 2");
    assert_eq!(song.sections[3].name.as_ref().unwrap(), "Outro");
}

// ── Fixtures de parser: acordes invertidos, 6/9, acidentes duplos ──────────

#[test]
fn advanced_chords_round_trip() {
    assert_round_trip("advanced_chords");
}

#[test]
fn advanced_chords_inverted_are_preserved() {
    let song = ChordProParser::parse(&load_file("advanced_chords")).expect("parse");
    let verse = &song.sections[0];
    assert_eq!(
        verse.lines[0]
            .chords
            .iter()
            .map(|c| c.chord.as_str())
            .collect::<Vec<_>>(),
        ["D/F#", "G/B", "Am/C"],
        "inverted chords in Verse line 1"
    );
    assert_eq!(
        verse.lines[1]
            .chords
            .iter()
            .map(|c| c.chord.as_str())
            .collect::<Vec<_>>(),
        ["Bbmaj9/D", "C/E"],
        "inverted chords with flats in Verse line 2"
    );
}

#[test]
fn advanced_chords_six_nine_is_preserved() {
    let song = ChordProParser::parse(&load_file("advanced_chords")).expect("parse");
    let chorus = &song.sections[1];
    assert_eq!(
        chorus.lines[0]
            .chords
            .iter()
            .map(|c| c.chord.as_str())
            .collect::<Vec<_>>(),
        ["C6/9", "Dm6/9"],
        "6/9 chords in Chorus line 1"
    );
    assert_eq!(
        chorus.lines[1]
            .chords
            .iter()
            .map(|c| c.chord.as_str())
            .collect::<Vec<_>>(),
        ["G6/9", "C6/9/E"],
        "6/9 chords in Chorus line 2"
    );
}

#[test]
fn advanced_chords_double_accidentals_are_preserved() {
    let song = ChordProParser::parse(&load_file("advanced_chords")).expect("parse");
    let bridge = &song.sections[2];
    assert_eq!(
        bridge.lines[0]
            .chords
            .iter()
            .map(|c| c.chord.as_str())
            .collect::<Vec<_>>(),
        ["F##dim", "Bbbmaj7"],
        "double accidentals in Bridge line 1"
    );
    assert_eq!(
        bridge.lines[1]
            .chords
            .iter()
            .map(|c| c.chord.as_str())
            .collect::<Vec<_>>(),
        ["C##m", "Ebb"],
        "double accidentals in Bridge line 2"
    );
}

// ── Letras multilíngues ────────────────────────────────────────────────────

#[test]
fn multilingual_lyrics_round_trip() {
    assert_round_trip("multilingual");
}

#[test]
fn multilingual_lyrics_preserve_accents_and_non_latin() {
    let song = ChordProParser::parse(&load_file("multilingual")).expect("parse");

    assert_eq!(song.sections[0].lines[0].text, "Coração alegre canta");
    assert_eq!(song.sections[0].lines[1].text, "Louvor ao grande Rei");

    assert_eq!(song.sections[1].lines[0].text, "Alegría del Señor es");
    assert_eq!(song.sections[1].lines[1].text, "Mi fortaleza");

    assert_eq!(
        song.sections[2].lines[0].text,
        "Danket dem Herrn, denn er ist freundlich"
    );
    assert_eq!(song.sections[2].lines[1].text, "Seine Güte währet ewiglich");

    assert_eq!(song.sections[3].lines[0].text, "日本語の 歌詞も ここに");
    assert_eq!(song.sections[3].lines[1].text, "テスト 中です");
}

// ── Casos de borda ─────────────────────────────────────────────────────────

#[test]
fn edge_cases_round_trip() {
    assert_round_trip("edge_cases");
}

#[test]
fn edge_cases_suspended_and_altered_chords() {
    let song = ChordProParser::parse(&load_file("edge_cases")).expect("parse");
    let verse = &song.sections[1];
    assert_eq!(
        verse.lines[0]
            .chords
            .iter()
            .map(|c| c.chord.as_str())
            .collect::<Vec<_>>(),
        ["Am", "Bdim", "Caug"],
        "basic, diminished and augmented"
    );
    assert_eq!(
        verse.lines[1]
            .chords
            .iter()
            .map(|c| c.chord.as_str())
            .collect::<Vec<_>>(),
        ["Dsus4", "Esus2", "Fmaj7"],
        "suspended and major seventh"
    );
    assert_eq!(
        verse.lines[2]
            .chords
            .iter()
            .map(|c| c.chord.as_str())
            .collect::<Vec<_>>(),
        ["G7sus4", "A7b9"],
        "altered chords"
    );
}

#[test]
fn edge_cases_preserves_metadata_like_time_and_duration() {
    let song = ChordProParser::parse(&load_file("edge_cases")).expect("parse");
    assert_eq!(song.metadata.get("time").map(String::as_str), Some("4/4"));
    assert_eq!(
        song.metadata.get("duration").map(String::as_str),
        Some("0:00")
    );
}
