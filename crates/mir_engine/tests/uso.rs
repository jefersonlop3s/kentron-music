//! Prova do relatório de uso: a agenda (ADR-0006) classificada pela ficha
//! (ADR-0012).
//!
//! O repertório de teste é montado em diretório próprio sob o temp, com
//! biblioteca e agendas de verdade — o caminho é o mesmo que a CLI usa. Nenhum
//! material de terceiros entra: o hino é do corpus de domínio público.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use mir_engine::uso;
use notation_bundle::bundle::{Execucao, Library, Setlist, SetlistId, SongBundle, SongId};
use notation_bundle::bundle_store;
use notation_bundle::chordpro::ChordProParser;

static COUNTER: AtomicU32 = AtomicU32::new(0);

struct Scratch(PathBuf);

impl Scratch {
    fn new(nome: &str) -> Self {
        let unico = COUNTER.fetch_add(1, Ordering::Relaxed);
        let caminho =
            std::env::temp_dir().join(format!("kentron-uso-{}-{unico}-{nome}", std::process::id()));
        fs::create_dir_all(&caminho).unwrap_or_else(|erro| panic!("diretório: {erro}"));
        Self(caminho)
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

fn bundle(id: &str, titulo: &str) -> SongBundle {
    let fonte = format!("{{title: {titulo}}}\n{{key: C}}\n\n[Verse]\n[C]texto\n");
    let song = ChordProParser::parse(&fonte).expect("fixture analisa");
    SongBundle::new(SongId::new(id).expect("id"), song).expect("bundle")
}

/// Monta um bundle de repertório: um hino de domínio público (corpus) e uma obra
/// contemporânea **sem ficha**, tocada em dois cultos com versões diferentes.
fn montar_repertorio(dir: &Path) -> Library {
    let mut library = Library::new();
    for (id, titulo) in [
        ("amazing-grace", "Amazing Grace"),
        ("essencia", "Essência da Adoração"),
    ] {
        let song_bundle = bundle(id, titulo);
        bundle_store::save_bundle(&dir.join(id), &song_bundle, None).expect("save bundle");
        library.add(song_bundle).expect("add");
    }
    bundle_store::save_library(dir, &library).expect("save library");

    let agenda_dir = dir.join(bundle_store::SETLISTS_DIR);
    fs::create_dir_all(&agenda_dir).expect("dir agendas");
    let mut primeiro = Setlist::new(SetlistId::new("culto-1").expect("id"), "Culto 1")
        .expect("agenda")
        .com_data("2026-11-15")
        .com_tipo("culto");
    primeiro
        .push_execucao(
            &library,
            Execucao::new(SongId::new("amazing-grace").expect("id")),
        )
        .expect("push");
    primeiro
        .push_execucao(
            &library,
            Execucao::new(SongId::new("essencia").expect("id"))
                .com_versao("álbum")
                .com_tom("E"),
        )
        .expect("push");
    bundle_store::save_setlist(&agenda_dir.join("culto-1.json"), &primeiro).expect("save");

    let mut segundo = Setlist::new(SetlistId::new("culto-2").expect("id"), "Culto 2")
        .expect("agenda")
        .com_data("2026-11-22")
        .com_tipo("culto");
    segundo
        .push_execucao(
            &library,
            Execucao::new(SongId::new("essencia").expect("id"))
                .com_versao("acústica")
                .com_tom("D"),
        )
        .expect("push");
    bundle_store::save_setlist(&agenda_dir.join("culto-2.json"), &segundo).expect("save");
    library
}

#[test]
fn relatorio_separa_dominio_publico_de_obra_sem_ficha() {
    let scratch = Scratch::new("classificacao");
    montar_repertorio(scratch.path());

    let relatorio = uso::do_diretorio(scratch.path(), 2026).expect("relatório");

    assert_eq!(relatorio.uso.eventos, 2);
    assert_eq!(relatorio.uso.execucoes, 3);
    assert_eq!(relatorio.uso.obras_distintas(), 2);
    // A obra que repete saiu em duas versões — é o que o relatório precisa dizer.
    assert_eq!(relatorio.uso.obras_com_mais_de_uma_versao(), 1);
    // Uma obra com ficha (corpus) e uma sem: é a lista de trabalho.
    assert_eq!(relatorio.publicas(), 1);
    assert_eq!(relatorio.sem_ficha(), 1);

    let essencia = relatorio
        .obras
        .iter()
        .find(|obra| obra.titulo.contains("Essência"))
        .expect("obra do repertório");
    assert!(essencia.veredito.is_none(), "sem ficha não afirma nada");
    assert_eq!(essencia.execucoes, 2);
    assert_eq!(essencia.versoes.len(), 2);

    let texto = relatorio.texto();
    assert!(texto.contains("SEM FICHA"), "{texto}");
    // O veredito sai com a **regra** que o produziu, e sem repetir o rótulo.
    assert!(
        texto.contains("[domínio público (vida de John Newton (1807) + 70 anos)]"),
        "{texto}"
    );
    assert!(
        !texto.contains("domínio público — domínio público"),
        "{texto}"
    );
    assert!(texto.contains("versão"), "{texto}");
}

#[test]
fn diretorio_sem_biblioteca_falha_alto_em_vez_de_mentir() {
    let scratch = Scratch::new("vazio");
    let erro = uso::do_diretorio(scratch.path(), 2026).expect_err("sem biblioteca é erro");
    assert!(erro.contains("biblioteca"), "{erro}");
}
