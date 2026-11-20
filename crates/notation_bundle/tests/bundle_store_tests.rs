//! Persistence tests: bundle directory, library index and setlist file.
//!
//! Each test owns a unique directory under the system temp dir and removes it on
//! drop, so nothing leaks between tests and nothing touches the repository. No
//! child process is spawned, so there is no environment to declare.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use notation_bundle::bundle::{Library, Setlist, SetlistId, SongBundle, SongId};
use notation_bundle::bundle_store::{
    self, LIBRARY_FILE, MANIFEST_FILE, SCHEMA_VERSION, SONG_FILE, StoreError,
};
use notation_bundle::chordpro::ChordProParser;

static COUNTER: AtomicU32 = AtomicU32::new(0);

/// A unique directory that removes itself when dropped.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "kentron-bundle-{}-{unique}-{name}",
            std::process::id()
        ));
        fs::create_dir_all(&path).unwrap_or_else(|error| panic!("scratch dir: {error}"));
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn bundle(id: &str, title: &str) -> SongBundle {
    let source = format!("{{title: {title}}}\n{{key: C}}\n\n[Verse]\n[C]texto\n");
    let song = ChordProParser::parse(&source).expect("fixture should parse");
    let id = SongId::new(id).expect("id should be valid");
    SongBundle::new(id, song).expect("fixture carries a title")
}

#[test]
fn save_then_load_round_trips_the_bundle() {
    let scratch = Scratch::new("roundtrip");
    let dir = scratch.path().join("santo");
    let original = bundle("santo", "Santo");

    let manifest = bundle_store::save_bundle(&dir, &original, Some("2026-11-20T13:00:00Z"))
        .expect("save must succeed");

    assert_eq!(manifest.schema_version, SCHEMA_VERSION);
    assert_eq!(manifest.id, "santo");
    assert_eq!(manifest.updated_at.as_deref(), Some("2026-11-20T13:00:00Z"));
    assert_eq!(manifest.files, [SONG_FILE]);
    assert!(dir.join(MANIFEST_FILE).is_file());
    assert!(dir.join(SONG_FILE).is_file());

    let loaded = bundle_store::load_bundle(&dir).expect("load must succeed");
    assert_eq!(loaded, original);
    assert_eq!(loaded.title(), "Santo");
}

#[test]
fn load_reports_a_missing_manifest() {
    let scratch = Scratch::new("missing");

    let error = bundle_store::load_bundle(scratch.path()).expect_err("must fail");

    assert!(matches!(error, StoreError::MissingFile { .. }));
    assert_eq!(error.message_key(), "store.file.missing");
}

#[test]
fn load_refuses_a_newer_schema() {
    let scratch = Scratch::new("newer");
    let dir = scratch.path().join("santo");
    fs::create_dir_all(&dir).expect("dir");
    fs::write(
        dir.join(MANIFEST_FILE),
        r#"{"schema_version": 2, "id": "santo", "updated_at": null, "files": ["song.cho"]}"#,
    )
    .expect("write manifest");
    fs::write(dir.join(SONG_FILE), "{title: Santo}\n\n[C]texto\n").expect("write song");

    let error = bundle_store::load_bundle(&dir).expect_err("must fail");

    match error {
        StoreError::UnsupportedSchema { found, supported } => {
            assert_eq!(found, 2);
            assert_eq!(supported, SCHEMA_VERSION);
        }
        other => panic!("expected UnsupportedSchema, got {other:?}"),
    }
}

#[test]
fn load_refuses_a_version_with_no_migration_step() {
    let scratch = Scratch::new("older");
    let dir = scratch.path().join("santo");
    fs::create_dir_all(&dir).expect("dir");
    fs::write(
        dir.join(MANIFEST_FILE),
        r#"{"schema_version": 0, "id": "santo", "updated_at": null, "files": ["song.cho"]}"#,
    )
    .expect("write manifest");
    fs::write(dir.join(SONG_FILE), "{title: Santo}\n\n[C]texto\n").expect("write song");

    let error = bundle_store::load_bundle(&dir).expect_err("must fail");

    match error {
        StoreError::UnsupportedSchema { found, .. } => assert_eq!(found, 0),
        other => panic!("expected UnsupportedSchema, got {other:?}"),
    }
}

#[test]
fn load_refuses_a_manifest_that_does_not_declare_the_song() {
    let scratch = Scratch::new("undeclared");
    let dir = scratch.path().join("santo");
    fs::create_dir_all(&dir).expect("dir");
    fs::write(
        dir.join(MANIFEST_FILE),
        r#"{"schema_version": 1, "id": "santo", "updated_at": null, "files": []}"#,
    )
    .expect("write manifest");
    fs::write(dir.join(SONG_FILE), "{title: Santo}\n\n[C]texto\n").expect("write song");

    let error = bundle_store::load_bundle(&dir).expect_err("must fail");

    assert!(matches!(error, StoreError::UndeclaredSong { .. }));
    assert_eq!(error.message_key(), "store.manifest.undeclared_song");
}

#[test]
fn library_index_round_trips_every_bundle() {
    let scratch = Scratch::new("library");
    let root = scratch.path();
    let mut library = Library::new();
    for (id, title) in [("santo", "Santo"), ("graca", "Graça")] {
        let bundle = bundle(id, title);
        bundle_store::save_bundle(&root.join(id), &bundle, None).expect("save bundle");
        library.add(bundle).expect("add");
    }

    bundle_store::save_library(root, &library).expect("save index");
    assert!(root.join(LIBRARY_FILE).is_file());

    let loaded = bundle_store::load_library(root).expect("load library");

    assert_eq!(loaded.len(), 2);
    assert_eq!(
        loaded
            .iter()
            .map(|bundle| bundle.title().to_owned())
            .collect::<Vec<_>>(),
        ["Graça", "Santo"]
    );
}

#[test]
fn setlist_round_trips_and_refuses_a_dangling_entry() {
    let scratch = Scratch::new("setlist");
    let root = scratch.path();
    let mut library = Library::new();
    for (id, title) in [("santo", "Santo"), ("graca", "Graça")] {
        let bundle = bundle(id, title);
        bundle_store::save_bundle(&root.join(id), &bundle, None).expect("save bundle");
        library.add(bundle).expect("add");
    }

    let mut setlist = Setlist::new(SetlistId::new("domingo").expect("id"), "Domingo")
        .expect("setlist should build");
    setlist
        .push(&library, SongId::new("graca").expect("id"))
        .expect("push graca");
    setlist
        .push(&library, SongId::new("santo").expect("id"))
        .expect("push santo");

    let path = root.join("domingo.json");
    bundle_store::save_setlist(&path, &setlist).expect("save setlist");

    let loaded = bundle_store::load_setlist(&path, &library).expect("load setlist");
    assert_eq!(loaded, setlist);

    let mut without_santo = Library::new();
    without_santo
        .add(bundle("graca", "Graça"))
        .expect("add graca only");

    let error = bundle_store::load_setlist(&path, &without_santo).expect_err("must fail");
    assert_eq!(error.message_key(), "setlist.song.missing");
}

#[test]
fn export_copies_the_whole_directory() {
    let scratch = Scratch::new("export");
    let source = scratch.path().join("santo");
    let target = scratch.path().join("backup").join("santo");
    bundle_store::save_bundle(&source, &bundle("santo", "Santo"), None).expect("save");
    fs::write(source.join("score.musicxml"), "<score-partwise/>").expect("extra file");

    bundle_store::export_bundle(&source, &target).expect("export");

    assert!(target.join(MANIFEST_FILE).is_file());
    assert!(target.join(SONG_FILE).is_file());
    assert!(target.join("score.musicxml").is_file());
    let loaded = bundle_store::load_bundle(&target).expect("load copy");
    assert_eq!(loaded.title(), "Santo");
}
