//! Benchmark do alinhamento de linha com verdade **conhecida**.
//!
//! Material sintético, sem terceiros: cada caso sabe onde cada linha começa, e o
//! teste mede o erro do motor contra isso. É o instrumento que permite iterar no
//! alinhamento sem gabarito humano — o de uma música real existe (40 linhas
//! marcadas à mão), mas é **um** e vive fora do repositório.
//!
//! O valor está em rodar os **quatro desenhos** e olhar o erro de cada um:
//! blocos separados, introdução instrumental, **canto sem pausa** (o caso que o
//! detector lê como frase única) e **repetição**. Um conjunto que só tem o caso
//! fácil mede a imaginação de quem o montou.

use mir_engine::wav::Audio;
use mir_engine::{Opts, analyze_com_letra};

mod common;
use common::{
    INTRO_S, cantar_quatro_frases, canto_com_repeticao, canto_continuo_desigual, canto_sem_pausa,
    intro_quieta_e_canto,
};

struct Caso {
    nome: &'static str,
    audio: Audio,
    inicios: Vec<f32>,
}

/// Erro (mediana e máximo, em segundos) do início de cada linha alinhada contra
/// o início **conhecido** da mesma linha.
fn erro(alinhadas: &[mir_engine::lyrics::LinhaAlinhada], inicios: &[f32]) -> Option<(f32, f32)> {
    let n = alinhadas.len().min(inicios.len());
    if n == 0 {
        return None;
    }
    let mut erros: Vec<f32> = (0..n)
        .map(|i| (alinhadas[i].inicio_s - inicios[i]).abs())
        .collect();
    erros.sort_by(f32::total_cmp);
    Some((erros[n / 2], erros[n - 1]))
}

fn mede(caso: &Caso) -> (usize, Option<(f32, f32)>) {
    let letra = mir_engine::lyrics::do_usuario(
        &"linha de teste cantada\n".repeat(caso.inicios.len()),
        None,
    );
    let opts = Opts {
        trecho_minimo_s: 4.0,
        ..Opts::default()
    };
    let (_, alinhadas) = analyze_com_letra(&caso.audio, &opts, Some(&letra), None, None);
    let e = erro(&alinhadas, &caso.inicios);
    (alinhadas.len(), e)
}

fn casos() -> Vec<Caso> {
    // Blocos de 4,8 s com 0,8 s de pausa antes de cada um, menos o primeiro.
    let blocos: Vec<f32> = (0..4).map(|i| i as f32 * 5.6).collect();
    // Introdução instrumental de INTRO_S + 0,8 s, depois blocos de 4,8 s.
    let com_intro: Vec<f32> = (0..3).map(|i| INTRO_S + 0.8 + i as f32 * 4.8).collect();
    let (audio_sem_pausa, inicios_sem_pausa) = canto_sem_pausa(4);
    let (audio_repeticao, inicios_repeticao) = canto_com_repeticao();
    let (audio_desigual, inicios_desigual) = canto_continuo_desigual();
    vec![
        Caso {
            nome: "blocos_separados",
            audio: cantar_quatro_frases(),
            inicios: blocos,
        },
        Caso {
            nome: "com_introducao",
            audio: intro_quieta_e_canto(),
            inicios: com_intro,
        },
        Caso {
            nome: "sem_pausa",
            audio: audio_sem_pausa,
            inicios: inicios_sem_pausa,
        },
        Caso {
            nome: "com_repeticao",
            audio: audio_repeticao,
            inicios: inicios_repeticao,
        },
        Caso {
            nome: "duracao_desigual",
            audio: audio_desigual,
            inicios: inicios_desigual,
        },
    ]
}

/// Mede o erro do alinhamento nos quatro desenhos e imprime o placar.
///
/// O placar sai **por caso** (a skill `medicao-sem-autoengano`: agregado esconde
/// caso ruim) e o teste exige que nenhum caso fique acima do teto — e que o
/// número de linhas alinhadas seja o esperado.
#[test]
fn alinhamento_sintetico_por_caso() {
    let mut piores: Vec<(&str, f32)> = Vec::new();
    for caso in casos() {
        let (n, e) = mede(&caso);
        let (med, mx) = e.unwrap_or((f32::NAN, f32::NAN));
        eprintln!(
            "caso={:<17} linhas={} (esperado {}) mediana={med:.2}s max={mx:.2}s",
            caso.nome,
            n,
            caso.inicios.len()
        );
        assert_eq!(
            n,
            caso.inicios.len(),
            "{}: linhas alinhadas diferentes do esperado",
            caso.nome
        );
        if std::env::var("MIR_DEBUG_BENCH").is_ok() {
            let opts = Opts {
                trecho_minimo_s: 4.0,
                ..Opts::default()
            };
            let spectro = mir_engine::spectrum::Spectrogram::new(&caso.audio, opts.janela, opts.salto);
            let frases = mir_engine::lyrics::frases_vocais(&spectro, caso.audio.taxa);
            let onsets = mir_engine::onset::detectar_banda(&spectro, caso.audio.taxa, 200.0, 3500.0);
            let letra =
                mir_engine::lyrics::do_usuario(&"linha de teste cantada\n".repeat(n), None);
            let (_, al) = analyze_com_letra(&caso.audio, &opts, Some(&letra), None, None);
            eprintln!("### {}", caso.nome);
            eprintln!("  verdade  {:?}", caso.inicios);
            eprintln!(
                "  motor    {:?}",
                al.iter().map(|l| l.inicio_s).collect::<Vec<_>>()
            );
            eprintln!("  frases   {frases:?}");
            eprintln!(
                "  ataques  {:?}",
                onsets.iter().map(|&(t, _)| (t * 100.0).round() / 100.0).collect::<Vec<_>>()
            );
        }
        piores.push((caso.nome, med));
    }
    for (nome, med) in &piores {
        assert!(
            *med <= 2.0,
            "caso {nome}: mediana {med:.2}s acima do teto de 2,0 s"
        );
    }
}
