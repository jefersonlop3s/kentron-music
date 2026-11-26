//! Persistence tests: bundle directory, library index and setlist file.
//!
//! Each test owns a unique directory under the system temp dir and removes it on
//! drop, so nothing leaks between tests and nothing touches the repository. No
//! child process is spawned, so there is no environment to declare.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use notation_bundle::bundle::{Library, Setlist, SetlistId, SongBundle, SongId, Tablatura};
use notation_bundle::bundle_store::{
    self, LIBRARY_FILE, MANIFEST_FILE, MUSICXML_FILE, SCHEMA_VERSION, SONG_FILE, StoreError,
    TABLATURE_DIR, caminho_da_tablatura,
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

/// Partitura MusicXML de fixture: deliberadamente **maior que a cifra extraída**.
///
/// A segunda `<part>`, a `<figured-bass>` e a nota com duas ligaduras são coisas que
/// `musicxml::parse` **não** representa no `Song`. É por isso que o round-trip
/// compara bytes: se a gravação reescrevesse a partitura a partir do modelo, todo
/// esse conteúdo sumiria e a comparação byte a byte é quem denuncia.
const MUSICXML_SANTO: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<score-partwise version="4.0">
  <part-list>
    <score-part id="P1"><part-name>Melodia</part-name></score-part>
    <score-part id="P2"><part-name>Baixo</part-name></score-part>
  </part-list>
  <part id="P1">
    <measure number="1">
      <attributes><divisions>1</divisions><key><fifths>0</fifths></key>
        <time><beats>4</beats><beat-type>4</beat-type></time></attributes>
      <figured-bass><figure><figure-number>6</figure-number></figure></figured-bass>
      <harmony><root><root-step>C</root-step></root><kind>major</kind></harmony>
      <note>
        <pitch><step>C</step><octave>4</octave></pitch><duration>4</duration>
        <tie type="start"/><lyric><syllabic>single</syllabic><text>San</text></lyric>
      </note>
    </measure>
  </part>
  <part id="P2">
    <measure number="1">
      <note><pitch><step>C</step><octave>3</octave></pitch><duration>4</duration><tie type="start"/></note>
    </measure>
  </part>
</score-partwise>
"#;

/// Tablatura GPIF de fixture — também maior que o `Song` que o parser tira dela:
/// a segunda trilha e o `<Rhythm>` não entram no modelo (ver `atex`).
const ATEX_BASE: &str = r#"<GPIF>
<Score><Title>Santo</Title><Artist>Fixture</Artist></Score>
<MasterBars><MasterBar id="0"><KeyAccidentals>0</KeyAccidentals></MasterBar></MasterBars>
<Tracks><Track id="0"><Name>Violao</Name></Track><Track id="1"><Name>Baixo</Name></Track></Tracks>
<Bars><Bar id="0"><Voices><Voice id="0"><Beats>
<Beat id="0"><Text>San-</Text></Beat><Beat id="1"><Text>to</Text></Beat>
</Beats></Voice></Voices></Bar></Bars>
<Rhythms><Rhythm id="0"><NoteValue>Quarter</NoteValue></Rhythm></Rhythms>
</GPIF>"#;

const ATEX_SOLO: &str = r#"<GPIF>
<Score><Title>Santo</Title></Score>
<MasterBars><MasterBar id="0"><KeyAccidentals>0</KeyAccidentals></MasterBar></MasterBars>
<Bars><Bar id="0"><Voices><Voice id="0"><Beats><Beat id="0"><Text>Solo</Text></Beat></Beats></Voice></Voices></Bar></Bars>
</GPIF>"#;

/// Bundle com partitura e duas tablaturas — o caso que a R1.1 pede.
///
/// A ordem de inserção e' `solo` **depois** `base`, de proposito: invertida em
/// relacao a ordem alfabetica, ela distingue "ordem de insercao" de "ordenado".
fn bundle_com_fontes(id: &str, title: &str) -> SongBundle {
    bundle(id, title)
        .com_musicxml(MUSICXML_SANTO)
        .com_tablatura(Tablatura::nova("solo", ATEX_SOLO).expect("tab válida"))
        .expect("primeira tablatura")
        .com_tablatura(Tablatura::nova("base", ATEX_BASE).expect("tab válida"))
        .expect("segunda tablatura")
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
    let original = bundle_com_fontes("santo", "Santo");
    bundle_store::save_bundle(&source, &original, None).expect("save");

    bundle_store::export_bundle(&source, &target).expect("export");

    assert!(target.join(MANIFEST_FILE).is_file());
    assert!(target.join(SONG_FILE).is_file());
    assert!(target.join(MUSICXML_FILE).is_file());
    // A pasta de tablaturas vem junto: exportar não pode deixar as fontes para trás.
    assert!(target.join(caminho_da_tablatura("base")).is_file());
    assert!(target.join(caminho_da_tablatura("solo")).is_file());

    let loaded = bundle_store::load_bundle(&target).expect("load copy");
    assert_eq!(loaded, original);
    assert_eq!(loaded.musicxml(), Some(MUSICXML_SANTO));
    assert_eq!(loaded.tabs().len(), 2);
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

// ---------------------------------------------------------------------------
// R1.1 — partitura e tablaturas dentro do bundle
//
// A comparação do round-trip é **byte a byte**, de propósito: a fonte é a
// verdade, e o modelo parseado é menor que ela (`musicxml::parse` ignora
// `<figured-bass>`, as trilhas além da melhor e o `tie`; `atex::parse` ignora
// pausas, ornamentos e as trilhas além da primeira). Comparar modelos deixaria
// passar exatamente o defeito que se quer barrar: gravar a fonte reescrita a
// partir do modelo — todo conteúdo não representado sumiria em silêncio.
// ---------------------------------------------------------------------------

#[test]
fn partitura_e_tablaturas_sobrevivem_ao_round_trip_byte_a_byte() {
    let scratch = Scratch::new("fontes");
    let dir = scratch.path().join("santo");
    let original = bundle_com_fontes("santo", "Santo");

    let manifest = bundle_store::save_bundle(&dir, &original, Some("2026-11-26T21:00:00Z"))
        .expect("save must succeed");

    // O manifesto declara o inventário inteiro, na ordem em que o bundle o carrega:
    // `.cho`, partitura, tablaturas na ordem de inserção — que **não** é alfabética.
    assert_eq!(
        manifest.files,
        [
            SONG_FILE.to_owned(),
            MUSICXML_FILE.to_owned(),
            caminho_da_tablatura("solo"),
            caminho_da_tablatura("base"),
        ]
    );
    assert!(dir.join(MUSICXML_FILE).is_file());
    assert!(dir.join(caminho_da_tablatura("base")).is_file());
    assert!(dir.join(caminho_da_tablatura("solo")).is_file());

    let loaded = bundle_store::load_bundle(&dir).expect("load must succeed");

    // Byte a byte: o arquivo que o disco devolve é o que foi escrito.
    assert_eq!(
        fs::read(dir.join(MUSICXML_FILE)).expect("read score"),
        MUSICXML_SANTO.as_bytes()
    );
    assert_eq!(loaded.musicxml(), Some(MUSICXML_SANTO));
    assert_eq!(loaded.tabs().len(), 2);
    assert_eq!(loaded.tabs()[0].nome(), "solo");
    assert_eq!(loaded.tabs()[0].atex(), ATEX_SOLO);
    assert_eq!(loaded.tabs()[1].nome(), "base");
    assert_eq!(loaded.tabs()[1].atex(), ATEX_BASE);

    // E o modelo inteiro volta igual, o que inclui as fontes.
    assert_eq!(loaded, original);

    // O caminho cru concorda com o reconciliador.
    assert_eq!(
        bundle_store::load_musicxml_from_bundle(&dir).expect("raw score"),
        Some(MUSICXML_SANTO.to_owned())
    );
}

/// As fontes guardadas são o **verdadeiro** arquivo, não bytes quaisquer: elas
/// parseiam para o modelo que o texto de origem produz.
#[test]
fn as_fontes_guardadas_sao_arquivos_de_verdade() {
    let scratch = Scratch::new("fontes-parsam");
    let dir = scratch.path().join("santo");

    bundle_store::save_bundle(&dir, &bundle_com_fontes("santo", "Santo"), None).expect("save");
    let loaded = bundle_store::load_bundle(&dir).expect("load");

    let partitura = notation_bundle::musicxml::parse(loaded.musicxml().expect("tem partitura"))
        .expect("MusicXML guardado deve parsear");
    // A partitura não traz `<work-title>`; o que ela tem de cifra é a harmonia e a
    // letra, e é isso que prova que o arquivo guardado é a partitura de verdade.
    assert_eq!(partitura.metadata.get("key").map(String::as_str), Some("C"));
    assert_eq!(partitura.sections[0].lines[0].chords[0].chord, "C");
    assert!(
        partitura.sections[0].lines[0].text.contains("San"),
        "letra extraída: {:?}",
        partitura.sections[0].lines[0].text
    );

    let tab =
        notation_bundle::atex::parse(loaded.tabs()[0].atex()).expect("ATEX guardado deve parsear");
    assert_eq!(tab.metadata.get("title").map(String::as_str), Some("Santo"));
    assert_eq!(tab.sections[0].lines[0].text, "Solo");
    assert_eq!(
        tab,
        notation_bundle::atex::parse(ATEX_SOLO).expect("origem")
    );
}

/// Obra sem partitura nem tablatura continua com inventário mínimo: nada de arquivo
/// declarado a mais, nada de pasta `tabs/` vazia.
#[test]
fn bundle_sem_fontes_nao_declara_arquivo_a_mais() {
    let scratch = Scratch::new("sem-fontes");
    let dir = scratch.path().join("santo");

    let manifest = bundle_store::save_bundle(&dir, &bundle("santo", "Santo"), None).expect("save");

    assert_eq!(manifest.files, [SONG_FILE.to_owned()]);
    assert!(!dir.join(MUSICXML_FILE).exists());
    assert!(!dir.join(TABLATURE_DIR).exists());
    let loaded = bundle_store::load_bundle(&dir).expect("load");
    assert_eq!(loaded.musicxml(), None);
    assert!(loaded.tabs().is_empty());
}
/// Bundle escrito **antes** desta mudança carrega inteiro: manifesto v2 com só o
/// `.cho`, sem partitura e sem tablatura.
///
/// É o teste que sustenta a decisão de **não** subir `schema_version`: o formato novo
/// é superconjunto do antigo, e o arquivo antigo continua sendo um arquivo válido.
///
/// A versão vai **literal** (`2`), não a constante `SCHEMA_VERSION`: medido em
/// 2026-11-26, com a constante o teste acompanhava um bump em silêncio e passava
/// (`SCHEMA_VERSION = 3` sem passo de migração deixava este teste verde). Literal, ele
/// reprova no dia em que alguém subir a versão — que é o dia de escrever o degrau na
/// cadeia `MIGRATIONS` e de reargumentar a decisão.
#[test]
fn bundle_antigo_desta_build_carrega_sem_as_fontes() {
    assert_eq!(
        SCHEMA_VERSION, 2,
        "se a versao subiu, o degrau de migracao tem de existir na cadeia e este teste se ajusta"
    );
    let scratch = Scratch::new("antigo-v2");
    let dir = scratch.path().join("santo");
    fs::create_dir_all(&dir).expect("dir");
    fs::write(
        dir.join(MANIFEST_FILE),
        r#"{"schema_version": 2, "id": "santo", "updated_at": "2026-11-20T13:00:00Z", "files": ["song.cho"]}"#,
    )
    .expect("write manifest");
    fs::write(dir.join(SONG_FILE), "{title: Santo}\n\n[C]texto\n").expect("write song");

    let loaded = bundle_store::load_bundle(&dir).expect("bundle antigo deve carregar");

    assert_eq!(loaded.title(), "Santo");
    assert_eq!(loaded.musicxml(), None);
    assert!(loaded.tabs().is_empty());

    // E regravar o antigo não inventa arquivo: o inventário continua mínimo.
    let regravado = bundle_store::save_bundle(&dir, &loaded, None).expect("regravar");
    assert_eq!(regravado.files, [SONG_FILE.to_owned()]);
}

/// Bundle v1 (formato de antes da emenda da agenda) carrega pela cadeia de migração.
#[test]
fn bundle_v1_carrega_pela_cadeia_de_migracao() {
    let scratch = Scratch::new("antigo-v1");
    let dir = scratch.path().join("santo");
    fs::create_dir_all(&dir).expect("dir");
    fs::write(
        dir.join(MANIFEST_FILE),
        r#"{"schema_version": 1, "id": "santo", "updated_at": null, "files": ["song.cho"]}"#,
    )
    .expect("write manifest");
    fs::write(dir.join(SONG_FILE), "{title: Santo}\n\n[C]texto\n").expect("write song");

    let loaded = bundle_store::load_bundle(&dir).expect("v1 deve migrar e carregar");

    assert_eq!(loaded.title(), "Santo");
    assert_eq!(loaded.musicxml(), None);
    assert!(loaded.tabs().is_empty());
}

/// O manifesto declarar arquivo que não está em disco é erro, não leitura parcial.
#[test]
fn manifesto_que_declara_arquivo_ausente_reprova() {
    let scratch = Scratch::new("declarado-ausente");
    let dir = scratch.path().join("santo");
    fs::create_dir_all(&dir).expect("dir");
    fs::write(dir.join(SONG_FILE), "{title: Santo}\n\n[C]texto\n").expect("write song");

    let escrever_manifesto = |declarados: &[&str]| {
        let lista = declarados
            .iter()
            .map(|declarado| format!("\"{declarado}\""))
            .collect::<Vec<_>>()
            .join(", ");
        fs::write(
            dir.join(MANIFEST_FILE),
            format!(
                r#"{{"schema_version": {SCHEMA_VERSION}, "id": "santo", "updated_at": null, "files": [{lista}]}}"#
            ),
        )
        .expect("write manifest");
    };

    // A partitura ausente é o primeiro declarado que falta.
    escrever_manifesto(&[SONG_FILE, MUSICXML_FILE, "tabs/base.atex"]);
    let error = bundle_store::load_bundle(&dir).expect_err("arquivo ausente deve reprovar");
    match &error {
        StoreError::MissingFile { path } => assert!(path.ends_with(MUSICXML_FILE)),
        other => panic!("expected MissingFile, got {other:?}"),
    }
    assert_eq!(error.message_key(), "store.file.missing");

    // E, sem a partitura no inventário, quem falta é a tablatura declarada.
    escrever_manifesto(&[SONG_FILE, "tabs/base.atex"]);
    let error = bundle_store::load_bundle(&dir).expect_err("tablatura ausente deve reprovar");
    match &error {
        StoreError::MissingFile { path } => assert!(path.ends_with(caminho_da_tablatura("base"))),
        other => panic!("expected MissingFile, got {other:?}"),
    }
}

/// Partitura em disco **fora** do manifesto reprova: o inventário é o manifesto.
#[test]
fn partitura_fora_do_manifesto_reprova() {
    let scratch = Scratch::new("partitura-solta");
    let dir = scratch.path().join("santo");
    bundle_store::save_bundle(&dir, &bundle("santo", "Santo"), None).expect("save");
    fs::write(dir.join(MUSICXML_FILE), MUSICXML_SANTO).expect("score solto");

    let error = bundle_store::load_bundle(&dir).expect_err("partitura não declarada reprova");

    match &error {
        StoreError::UndeclaredFile { path } => assert!(path.ends_with(MUSICXML_FILE)),
        other => panic!("expected UndeclaredFile, got {other:?}"),
    }
    assert_eq!(error.message_key(), "store.manifest.undeclared_file");
}

/// Tablatura em disco fora do manifesto reprova — mesmo com o `.cho` em ordem.
#[test]
fn tablatura_fora_do_manifesto_reprova() {
    let scratch = Scratch::new("tab-solta");
    let dir = scratch.path().join("santo");
    bundle_store::save_bundle(&dir, &bundle("santo", "Santo"), None).expect("save");
    fs::create_dir_all(dir.join(TABLATURE_DIR)).expect("tabs");
    fs::write(dir.join(caminho_da_tablatura("base")), ATEX_BASE).expect("tab solta");

    let error = bundle_store::load_bundle(&dir).expect_err("tablatura não declarada reprova");

    match &error {
        StoreError::UndeclaredFile { path } => {
            assert!(path.ends_with(caminho_da_tablatura("base")));
        }
        other => panic!("expected UndeclaredFile, got {other:?}"),
    }
    assert_eq!(error.message_key(), "store.manifest.undeclared_file");
}

/// Arquivo declarado que esta build não sabe ler reprova, em vez de ser ignorado.
#[test]
fn manifesto_com_arquivo_que_esta_build_nao_le_reprova() {
    let scratch = Scratch::new("nao-lido");
    let dir = scratch.path().join("santo");
    fs::create_dir_all(&dir).expect("dir");
    fs::write(
        dir.join(MANIFEST_FILE),
        format!(
            r#"{{"schema_version": {SCHEMA_VERSION}, "id": "santo", "updated_at": null, "files": ["song.cho", "capa.png"]}}"#
        ),
    )
    .expect("write manifest");
    fs::write(dir.join(SONG_FILE), "{title: Santo}\n\n[C]texto\n").expect("write song");
    fs::write(dir.join("capa.png"), "PNG").expect("write extra");

    let error = bundle_store::load_bundle(&dir).expect_err("arquivo desconhecido reprova");

    match &error {
        StoreError::UnsupportedFile { declarado, .. } => assert_eq!(declarado, "capa.png"),
        other => panic!("expected UnsupportedFile, got {other:?}"),
    }
    assert_eq!(error.message_key(), "store.manifest.unsupported_file");
}

/// O caminho antigo, que recebe a partitura por parâmetro, continua gravando — e
/// agora **deixa** a partitura declarada no manifesto e devolvida pela carga.
#[test]
fn save_bundle_with_musicxml_grava_declara_e_devolve() {
    let scratch = Scratch::new("musicxml-param");
    let dir = scratch.path().join("santo");

    let manifest = bundle_store::save_bundle_with_musicxml(
        &dir,
        &bundle("santo", "Santo"),
        None,
        Some(MUSICXML_SANTO),
    )
    .expect("save");
    assert_eq!(
        manifest.files,
        [SONG_FILE.to_owned(), MUSICXML_FILE.to_owned()]
    );
    assert_eq!(
        bundle_store::load_bundle(&dir).expect("load").musicxml(),
        Some(MUSICXML_SANTO)
    );

    // O parâmetro **sobrepõe** a partitura que o bundle já carrega.
    let com_outra = bundle("santo", "Santo").com_musicxml("<score-partwise version=\"1.0\"/>");
    bundle_store::save_bundle_with_musicxml(&dir, &com_outra, None, Some(MUSICXML_SANTO))
        .expect("overrides");
    assert_eq!(
        fs::read_to_string(dir.join(MUSICXML_FILE)).expect("read score"),
        MUSICXML_SANTO
    );
}

/// O caminho de uma tablatura no manifesto é sempre `tabs/<nome>.atex`.
#[test]
fn caminho_da_tablatura_usa_barra_do_manifesto() {
    assert_eq!(caminho_da_tablatura("base"), "tabs/base.atex");
    assert_eq!(
        caminho_da_tablatura("base"),
        format!("{TABLATURE_DIR}/base.atex")
    );
}
