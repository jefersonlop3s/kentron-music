//! Fixture dourada da cifra com letra, e os negativos do alinhamento.
//!
//! O arquivo `.cho` gerado aqui é o **mesmo** que o teste de Flutter desenha
//! (`apps/kentron_app/test/cifra_com_letra_test.dart` lê este caminho): uma fonte
//! só, e os dois lados quebram juntos se o formato mudar.

use mir_engine::lyrics;
use mir_engine::sheet;
use mir_engine::{Opts, analyze_com_letra};
use notation_bundle::chordpro::write;

mod common;
use common::{cantar_quatro_frases, ruido};

/// Caminho da fixture, única para os dois lados (Rust e Flutter).
fn caminho_da_fixture() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cifra-com-letra.cho")
}

#[test]
fn fixture_dourada_da_cifra_com_letra() {
    let audio = cantar_quatro_frases();
    let letra = lyrics::do_corpus("amazing").expect("corpus de domínio público");
    let (relatorio, alinhadas) = analyze_com_letra(&audio, &Opts::default(), Some(&letra));
    let musica = sheet::montar(
        &relatorio,
        &alinhadas,
        "Amazing Grace",
        &letra.origem.descricao(),
    );
    let texto = write(&musica);

    let caminho = caminho_da_fixture();
    if std::env::var("MIR_ATUALIZAR_FIXTURE").is_ok() {
        std::fs::create_dir_all(caminho.parent().expect("diretório")).expect("criar diretório");
        std::fs::write(&caminho, &texto).expect("escrever fixture");
        return;
    }
    let esperado = std::fs::read_to_string(&caminho).unwrap_or_else(|erro| {
        panic!(
            "fixture ausente em {caminho:?} ({erro}). Gere com MIR_ATUALIZAR_FIXTURE=1 cargo test -p mir_engine fixture"
        )
    });
    assert_eq!(
        texto, esperado,
        "a cifra mudou de forma: confira se o app ainda desenha, e só então rode com MIR_ATUALIZAR_FIXTURE=1"
    );
}

#[test]
fn sem_voz_no_audio_a_confianca_cai_e_o_motor_nao_inventa() {
    let audio = ruido(6);
    let letra = lyrics::do_corpus("rock of ages").expect("corpus");
    let (_, alinhadas) = analyze_com_letra(&audio, &Opts::default(), Some(&letra));
    assert_eq!(
        alinhadas.len(),
        letra.linhas.len(),
        "sem frase vocal, a letra ainda precisa aparecer — só que com confiança baixa"
    );
    for linha in &alinhadas {
        assert!(
            linha.confianca <= 0.35,
            "confiança alta sem voz no áudio: {} em {:?}",
            linha.confianca,
            linha.texto
        );
    }
}

#[test]
fn sem_letra_nao_ha_linha_nenhuma() {
    let audio = cantar_quatro_frases();
    let (_, alinhadas) = analyze_com_letra(&audio, &Opts::default(), None);
    assert!(alinhadas.is_empty(), "sem letra, nada de linha");
}
