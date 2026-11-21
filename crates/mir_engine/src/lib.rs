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
pub mod report;
pub mod spectrum;
pub mod structure;
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
    let dynamics = dynamics::analyze(&audio.amostras, audio.taxa);
    let spectro = spectrum::Spectrogram::new(audio, opts.janela, opts.salto);
    let chroma = spectro.chroma(audio.taxa);
    let chords = chords::detect(&chroma);
    let tempo = structure::estimate_tempo(&spectro, audio.taxa);
    let sections = structure::segment(&spectro, &chroma, audio.taxa, opts.trecho_minimo_s);

    report::Report {
        audio: report::AudioInfo {
            taxa: audio.taxa,
            canais: audio.canais,
            duracao_s: audio.duracao(),
        },
        dynamics,
        tempo,
        chords,
        sections,
    }
}
