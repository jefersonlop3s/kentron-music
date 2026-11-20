//! Bundle integration tests: the in-memory model composing with the parser and
//! the harmony engine.
//!
//! The unit tests in `bundle.rs` cover the model's own invariants; these prove
//! that a bundle carries a real `.cho` fixture and that its song still goes
//! through transposition without losing bibliographic identity.

use std::fs;
use std::path::PathBuf;

use notation_bundle::bundle::{BundleError, Library, Setlist, SetlistId, SongBundle, SongId};
use notation_bundle::chord_naming::{Letter, Note};
use notation_bundle::chordpro::ChordProParser;
use notation_bundle::harmony::HarmonyEngine;

const KEY_C: Note = Note::new(Letter::C, 0);
const KEY_D: Note = Note::new(Letter::D, 0);

fn golden(name: &str) -> String {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/golden_harmony");
    path.push(name);
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn golden_bundle(id: &str) -> SongBundle {
    let song = ChordProParser::parse(&golden("worship.cho")).expect("golden song must parse");
    let id = SongId::new(id).expect("id should be valid");
    SongBundle::new(id, song).expect("golden song carries a title")
}

#[test]
fn bundle_from_golden_fixture_keeps_identity_after_transposition() {
    let bundle = golden_bundle("santo");
    assert_eq!(bundle.title(), "Santo");
    assert_eq!(bundle.key(), Some("C"));

    let transposed = HarmonyEngine::transpose(bundle.song(), KEY_C, KEY_D).expect("transpose");

    assert_eq!(
        transposed.metadata.get("title").map(String::as_str),
        Some("Santo")
    );
    assert_eq!(
        transposed.metadata.get("key").map(String::as_str),
        Some("D")
    );
    assert_eq!(bundle.id().as_str(), "santo");
}

#[test]
fn library_and_setlist_resolve_in_setlist_order() {
    let mut library = Library::new();
    library.add(golden_bundle("santo")).expect("add golden");

    let outro = ChordProParser::parse("{title: Outro}\n{key: G}\n\n[Verse]\n[G]texto\n")
        .expect("fixture should parse");
    let outro_id = SongId::new("outro").expect("id should be valid");
    library
        .add(SongBundle::new(outro_id, outro).expect("title comes from metadata"))
        .expect("add outro");

    let mut setlist = Setlist::new(SetlistId::new("domingo").expect("id"), "Domingo")
        .expect("setlist should build");
    setlist
        .push(&library, SongId::new("outro").expect("id"))
        .expect("push outro");
    setlist
        .push(&library, SongId::new("santo").expect("id"))
        .expect("push santo");

    let titles = setlist
        .songs(&library)
        .expect("setlist resolves")
        .iter()
        .map(|bundle| bundle.title().to_owned())
        .collect::<Vec<_>>();
    assert_eq!(titles, ["Outro", "Santo"]);
}

#[test]
fn removing_a_song_in_use_fails_loudly_on_resolution() {
    let mut library = Library::new();
    library.add(golden_bundle("santo")).expect("add");

    let mut setlist = Setlist::new(SetlistId::new("domingo").expect("id"), "Domingo")
        .expect("setlist should build");
    let id = SongId::new("santo").expect("id should be valid");
    setlist.push(&library, id.clone()).expect("push");

    library.remove(&id);

    let error = setlist
        .songs(&library)
        .expect_err("dangling entry must fail");
    assert_eq!(error.message_key(), "setlist.song.missing");
    assert!(matches!(error, BundleError::MissingSong(_)));
}
