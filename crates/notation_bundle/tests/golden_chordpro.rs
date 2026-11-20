//! Golden tests for ChordPro parsing.
use notation_bundle::chordpro::ChordProParser;
use std::fs;
use std::path::PathBuf;

fn load_file(name: &str) -> String {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/golden_chordpro");
    path.push(format!("{}.cho", name));
    fs::read_to_string(path).expect("Failed to read golden file")
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

    println!("Parsed Multi-Section Song: {:#?}", song);
    assert_eq!(song.metadata.get("title").unwrap(), "Multi-Section Song");
    assert_eq!(song.sections.len(), 4);
    assert_eq!(song.sections[0].name.as_ref().unwrap(), "Verse 1");
    assert_eq!(song.sections[1].name.as_ref().unwrap(), "Chorus");
    assert_eq!(song.sections[2].name.as_ref().unwrap(), "Verse 2");
    assert_eq!(song.sections[3].name.as_ref().unwrap(), "Outro");
}
