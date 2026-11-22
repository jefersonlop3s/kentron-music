//! Terça pela melodia (ADR-0014, bloco 3).
//!
//! A prova usa **power chord** — fundamental e quinta, sem terça. Se o motor
//! disser `C` ou `Cm` ali, foi a melodia que decidiu; o croma não tinha terça
//! para decidir. E o negativo: sem evidência na melodia, o motor **não inventa**.
//!
//! A janela de ganho da voz foi **medida**, não escolhida: acima dela o croma
//! decide sozinho (e o teste passaria sem provar nada); abaixo, a própria
//! melodia perde a voz. A tabela está em `o_arbitro_tem_janela_medida`.

mod common;

use mir_engine::melody;
use mir_engine::spectrum::Spectrogram;
use mir_engine::{Opts, analyze};

fn espetro_de(audio: &mir_engine::wav::Audio) -> Spectrogram {
    Spectrogram::new(audio, 4096, 1024)
}

fn acordes(audio: &mir_engine::wav::Audio) -> Vec<String> {
    let opts = Opts {
        trecho_minimo_s: 4.0,
        ..Opts::default()
    };
    analyze(audio, &opts)
        .chords
        .iter()
        .map(|trecho| trecho.acorde.clone())
        .collect()
}

/// Acordes com e sem o árbitro, sobre o **mesmo** croma — é o delta que importa.
fn sem_e_com_melodia(audio: &mir_engine::wav::Audio) -> (Vec<String>, Vec<String>) {
    let espetro = espetro_de(audio);
    let chroma = espetro.chroma(audio.taxa);
    let sem = mir_engine::chords::detect(&chroma)
        .iter()
        .map(|trecho| trecho.acorde.clone())
        .collect();
    let melodia = melody::extrair(&espetro, audio.taxa);
    let com = mir_engine::chords::detect_com_melodia(&chroma, Some(&melodia))
        .iter()
        .map(|trecho| trecho.acorde.clone())
        .collect();
    (sem, com)
}

fn classe_dominante(audio: &mir_engine::wav::Audio) -> Option<usize> {
    let espetro = espetro_de(audio);
    let melodia = melody::extrair(&espetro, audio.taxa);
    melodia
        .dominante(0.0, audio.duracao())
        .map(|(classe, _, _)| classe)
}

#[test]
fn a_voz_na_terca_maior_e_lida_como_mi() {
    // Mi4 = classe 4.
    assert_eq!(classe_dominante(&common::power_chord_com_voz(64)), Some(4));
}

#[test]
fn a_voz_na_terca_menor_e_lida_como_mi_bemol() {
    // Mi bemol 4 = classe 3.
    assert_eq!(classe_dominante(&common::power_chord_com_voz(63)), Some(3));
}

#[test]
fn terca_maior_na_voz_promove_o_power_chord() {
    let lista = acordes(&common::power_chord_com_voz(64));
    assert!(
        lista.iter().any(|acorde| acorde == "C"),
        "esperava C (a melodia canta a terça maior): {lista:?}"
    );
    assert!(
        !lista.iter().any(|acorde| acorde == "C5"),
        "sobrou potência onde a melodia decidiu: {lista:?}"
    );
}

#[test]
fn terca_menor_na_voz_promove_para_menor() {
    let lista = acordes(&common::power_chord_com_voz(63));
    assert!(
        lista.iter().any(|acorde| acorde == "Cm"),
        "esperava Cm (a melodia canta a terça menor): {lista:?}"
    );
}

#[test]
fn voz_na_fundamental_nao_inventa_terca() {
    // A melodia canta Dó4 — a fundamental. Sem terça no acorde e sem terça na
    // voz, o motor não tem de onde tirar a qualidade.
    let lista = acordes(&common::power_chord_com_voz(60));
    assert!(
        !lista.iter().any(|acorde| acorde == "Cm"),
        "inventou menor sem terça menor em lugar nenhum: {lista:?}"
    );
}

/// A prova que faltava: o croma **sozinho** tem de errar aqui.
///
/// Se ele já dissesse `C` sem a melodia, os testes acima passariam sem provar
/// nada — a fixture não isolaria o árbitro. Este teste exige o delta.
#[test]
fn o_arbitro_e_quem_decide_e_nao_o_croma() {
    let (sem, com) = sem_e_com_melodia(&common::power_chord_com_voz(64));
    assert!(
        sem.iter().any(|acorde| acorde == "C5"),
        "o croma já decidiu sozinho — a fixture não isola o árbitro. sem melodia: {sem:?}"
    );
    assert!(
        com.iter().any(|acorde| acorde == "C"),
        "a melodia não promoveu. com melodia: {com:?}"
    );
}

/// O limite do método, medido em 2026-11-22 com a voz em três ganhos.
///
/// | ganho da voz | sem melodia | com melodia |
/// |---|---|---|
/// | 0,30 | `C` | `C` — o croma decide sozinho; não prova nada |
/// | 0,16 | `C5` | `C` — **é aqui que o árbitro é a diferença** |
/// | 0,09 | `C5` | `C5` — a melodia perde a voz e o motor **abstém** |
///
/// O caso de 0,09 é a resposta honesta: sem evidência suficiente, o motor não
/// inventa qualidade. É a prova negativa do árbitro.
#[test]
fn o_arbitro_tem_janela_medida() {
    let (sem, com) = sem_e_com_melodia(&common::power_chord_com_voz_em(64, 0.16));
    assert!(sem.iter().any(|acorde| acorde == "C5"), "sem: {sem:?}");
    assert!(com.iter().any(|acorde| acorde == "C"), "com: {com:?}");

    let (sem, com) = sem_e_com_melodia(&common::power_chord_com_voz_em(64, 0.09));
    assert!(sem.iter().any(|acorde| acorde == "C5"), "sem: {sem:?}");
    assert!(
        com.iter().any(|acorde| acorde == "C5"),
        "promoveu sem evidência suficiente na melodia: {com:?}"
    );
}
