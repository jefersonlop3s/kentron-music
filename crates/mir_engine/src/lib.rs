//! Motor MIR: do áudio para o relatório musical (ADR-0011).
//!
//! Este incremento é **DSP puro** — nenhum modelo, nenhum peso, nada de rede no
//! momento da análise. A cadeia é: ler WAV → dinâmica (EBU R128) → espectro e
//! croma → acordes → onsets, andamento e estrutura → relatório.
//!
//! O que ele **não** faz, por decisão registrada: letra (ASR de canto) e
//! tablatura de solo (pitch tracking + casamento de corda). Cada um exige peso
//! declarado no `models/MANIFEST.toml` e incremento próprio.
//!
//! Acurácia é **candidata**, não verdade: em mixagem real o acorde detectado
//! pode errar, e o relatório carrega a confiança por trecho para o músico
//! confirmar.

pub mod chords;
pub mod dynamics;
pub mod fichas;
pub mod lyrics;
pub mod report;
pub mod sheet;
pub mod spectrum;
pub mod structure;
pub mod tonality;
pub mod uso;
pub mod wav;

use wav::Audio;

/// Parâmetros da análise. O default é o que a CLI usa.
#[derive(Debug, Clone)]
pub struct Opts {
    /// Tamanho da janela da STFT, em amostras.
    pub janela: usize,
    /// Salto entre janelas, em amostras (75% de sobreposição no default).
    pub salto: usize,
    /// Duração mínima de um trecho na segmentação, em segundos.
    ///
    /// 15 s porque trecho de música é assim: com 4 s, o detector cortava a cada
    /// troca de acorde e devolvia 30+ "trechos" — medido no primeiro áudio real.
    pub trecho_minimo_s: f32,
}

impl Default for Opts {
    fn default() -> Self {
        Self {
            janela: 4096,
            salto: 1024,
            trecho_minimo_s: 15.0,
        }
    }
}

/// Analisa o áudio e devolve o relatório completo.
#[must_use]
pub fn analyze(audio: &Audio, opts: &Opts) -> report::Report {
    let spectro = spectrum::Spectrogram::new(audio, opts.janela, opts.salto);
    analyze_com(audio, opts, &spectro)
}

/// Analisa reaproveitando uma STFT já calculada.
///
/// O alinhamento da letra olha o mesmo espectro, e recalcular a STFT para ele
/// seria pagar duas vezes pelo mesmo trabalho num arquivo de seis minutos.
#[must_use]
pub fn analyze_com(audio: &Audio, opts: &Opts, spectro: &spectrum::Spectrogram) -> report::Report {
    let chroma = spectro.chroma(audio.taxa);
    let chords = chords::detect(&chroma);
    let tempo = structure::estimate_tempo(spectro, audio.taxa);
    let sections = structure::segment(spectro, &chroma, audio.taxa, opts.trecho_minimo_s);

    report::Report {
        audio: report::AudioInfo {
            taxa: audio.taxa,
            canais: audio.canais,
            duracao_s: audio.duracao(),
            afinacao_cents: spectro.desvio_de_afinacao(audio.taxa) * 100.0,
        },
        dynamics: dynamics::analyze(&audio.amostras, audio.taxa),
        tempo,
        tonalidade: tonality::analisar(&chroma),
        chords,
        sections,
    }
}

/// Análise + alinhamento da letra, com uma STFT só.
///
/// Sem letra, devolve o alinhamento vazio — não inventa linha nenhuma.
#[must_use]
pub fn analyze_com_letra(
    audio: &Audio,
    opts: &Opts,
    letra: Option<&lyrics::Letra>,
) -> (report::Report, Vec<lyrics::LinhaAlinhada>) {
    let spectro = spectrum::Spectrogram::new(audio, opts.janela, opts.salto);
    let relatorio = analyze_com(audio, opts, &spectro);
    let alinhadas = letra.map_or_else(Vec::new, |letra| {
        lyrics::alinhar(&relatorio, letra, &spectro, audio.taxa)
    });
    (relatorio, alinhadas)
}
