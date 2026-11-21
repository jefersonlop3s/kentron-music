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
        format!(
            r#"{{"schema_version": {}, "id": "santo", "updated_at": null, "files": ["song.cho"]}}"#,
            SCHEMA_VERSION + 1
        ),
    )
    .expect("write manifest");
    fs::write(dir.join(SONG_FILE), "{title: Santo}\n\n[C]texto\n").expect("write song");

    let error = bundle_store::load_bundle(&dir).expect_err("must fail");

    match error {
        StoreError::UnsupportedSchema { found, supported } => {
            assert_eq!(found, SCHEMA_VERSION + 1);
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

/// A agenda é o evento: data, tipo e execuções com **versão e tom**.
#[test]
fn agenda_guarda_evento_e_versao_da_execucao() {
    let scratch = Scratch::new("agenda");
    let mut library = Library::new();
    library.add(bundle("essencia", "Essência")).expect("add");

    let mut agenda = Setlist::new(
        SetlistId::new("culto-2026-11-22").expect("id"),
        "Culto de domingo",
    )
    .expect("agenda")
    .com_data("2026-11-22")
    .com_tipo("culto");
    agenda
        .push_execucao(
            &library,
            notation_bundle::bundle::Execucao::new(SongId::new("essencia").expect("song"))
                .com_versao("versão do álbum")
                .com_tom("E"),
        )
        .expect("push");

    assert_eq!(agenda.data(), Some("2026-11-22"));
    assert_eq!(agenda.tipo(), Some("culto"));
    // A mesma obra vai voltar em outro culto, **em outra versão** — e é isso que
    // a execução guarda, não a obra.
    let mut outro = Setlist::new(SetlistId::new("culto-2026-11-29").expect("id"), "Culto 2")
        .expect("agenda")
        .com_data("2026-11-29")
        .com_tipo("culto");
    outro
        .push_execucao(
            &library,
            notation_bundle::bundle::Execucao::new(SongId::new("essencia").expect("song"))
                .com_versao("versão acústica")
                .com_tom("D"),
        )
        .expect("push");

    let dir = scratch.path().join(bundle_store::SETLISTS_DIR);
    fs::create_dir_all(&dir).expect("dir");
    bundle_store::save_setlist(&dir.join("culto-2026-11-22.json"), &agenda).expect("save 1");
    bundle_store::save_setlist(&dir.join("culto-2026-11-29.json"), &outro).expect("save 2");

    let lidas = bundle_store::load_setlists(scratch.path(), &library).expect("load agendas");
    assert_eq!(lidas.len(), 2);
    assert_eq!(lidas[0].data(), Some("2026-11-22"));
    assert_eq!(lidas[0].entries()[0].versao(), Some("versão do álbum"));
    assert_eq!(lidas[0].entries()[0].tom(), Some("E"));
    assert_eq!(lidas[1].entries()[0].versao(), Some("versão acústica"));
    assert_eq!(lidas[1].entries()[0].tom(), Some("D"));

    // E daí sai o relatório: 2 eventos, 2 execuções, **1 obra em 2 versões**.
    let relatorio = notation_bundle::bundle::relatorio_de_uso(&lidas, &library).expect("relatório");
    assert_eq!(relatorio.eventos, 2);
    assert_eq!(relatorio.execucoes, 2);
    assert_eq!(relatorio.obras_distintas(), 1);
    assert_eq!(relatorio.obras_com_mais_de_uma_versao(), 1);
    let resumo = &relatorio.por_obra[0];
    assert_eq!(resumo.execucoes, 2);
    assert_eq!(
        resumo.versoes,
        vec![
            ("versão do álbum".to_owned(), 1),
            ("versão acústica".to_owned(), 1)
        ]
    );
    let texto = relatorio.texto();
    assert!(texto.contains("obras distintas: 1"), "{texto}");
    assert!(texto.contains("versão acústica"), "{texto}");
}

/// Agenda v1 (entrada era um id solto) migra para v2 **sem inventar versão**.
#[test]
fn agenda_v1_migra_para_v2_sem_inventar_versao() {
    let scratch = Scratch::new("agenda-v1");
    let mut library = Library::new();
    library.add(bundle("santo", "Santo")).expect("add");

    // Arquivo no formato antigo, escrito à mão — é o que existe em campo.
    let dir = scratch.path().join(bundle_store::SETLISTS_DIR);
    fs::create_dir_all(&dir).expect("dir");
    fs::write(
        dir.join("culto-antigo.json"),
        r#"{"schema_version": 1, "id": "culto-antigo", "name": "Culto antigo", "entries": ["santo"]}"#,
    )
    .expect("write v1");

    let lidas = bundle_store::load_setlists(scratch.path(), &library).expect("migrar e ler");

    assert_eq!(lidas.len(), 1);
    assert_eq!(lidas[0].name(), "Culto antigo");
    // Sem data e sem tipo no arquivo antigo: ausente é ausente.
    assert_eq!(lidas[0].data(), None);
    assert_eq!(lidas[0].tipo(), None);
    // E a versão **não** foi inventada.
    assert_eq!(lidas[0].entries()[0].versao(), None);
    assert_eq!(
        lidas[0].entries()[0]
            .versao()
            .unwrap_or(notation_bundle::bundle::VERSAO_NAO_DECLARADA),
        "(versão não declarada)"
    );

    // O relatório diz "versão não declarada" em vez de chutar um arranjo.
    let relatorio = notation_bundle::bundle::relatorio_de_uso(&lidas, &library).expect("relatório");
    assert!(
        relatorio.texto().contains("(versão não declarada)"),
        "{}",
        relatorio.texto()
    );
}

/// Sem pasta de agendas, o relatório é vazio — e não erro.
#[test]
fn sem_agendas_o_relatorio_e_vazio() {
    let scratch = Scratch::new("sem-agenda");
    let library = Library::new();
    let lidas = bundle_store::load_setlists(scratch.path(), &library).expect("sem agenda");
    assert!(lidas.is_empty());
    let relatorio = notation_bundle::bundle::relatorio_de_uso(&lidas, &library).expect("relatório");
    assert_eq!(relatorio.eventos, 0);
    assert_eq!(relatorio.execucoes, 0);
    assert_eq!(relatorio.obras_distintas(), 0);
}
