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

pub mod baixo;
pub mod chords;
pub mod compasso;
pub mod decode;
pub mod dynamics;
pub mod fichas;
pub mod lyrics;
pub mod melody;
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

/// RMS da banda do baixo num trecho, dividido pela mediana da música.
///
/// Devolve 0 quando não há mediana (música sem grave nenhum) — e aí a abstenção
/// do detector está certa por definição.
fn energia_do_trecho(banda: &[f32], mediana: f32, fps: f32, inicio_s: f32, fim_s: f32) -> f32 {
    if mediana <= f32::EPSILON || banda.is_empty() {
        return 0.0;
    }
    let inicio = ((inicio_s * fps) as usize).min(banda.len());
    let fim = ((fim_s * fps) as usize).min(banda.len());
    if inicio >= fim {
        return 0.0;
    }
    banda[inicio..fim].iter().sum::<f32>() / (fim - inicio) as f32 / mediana
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
    // A melodia é extraída antes dos acordes: ela arbitra a terça que o croma
    // perdeu na mixagem (ADR-0014, bloco 3).
    let melodia = melody::extrair(spectro, audio.taxa);
    let mut chords = chords::detect_com_melodia(&chroma, Some(&melodia));
    let melodia_trechos: Vec<report::MelodiaSegment> = chords
        .iter()
        .map(|trecho| {
            let contagem = melodia.contagem(trecho.inicio_s, trecho.fim_s);
            let dominante = melodia.dominante(trecho.inicio_s, trecho.fim_s);
            report::MelodiaSegment {
                inicio_s: trecho.inicio_s,
                fim_s: trecho.fim_s,
                quadros_com_f0: melodia.quadros_com_f0(trecho.inicio_s, trecho.fim_s),
                dominante: dominante
                    .map(|(classe, _, _)| chords::nome(classe, chords::Qualidade::Maior)),
                fracao_dominante: dominante.map_or(0.0, |(_, vezes, total_f0)| {
                    vezes as f32 / total_f0.max(1) as f32
                }),
                contagem,
            }
        })
        .collect();
    // O tempo vem antes do baixo: o compasso usa a novidade de croma, e o baixo
    // usa o compasso só para a leitura por trecho.
    let tempo = structure::estimate_tempo(spectro, audio.taxa);
    // O compasso é medido **no espectro**, não nos trechos de acorde: a âncora é a
    // mudança de harmonia medida no croma, e assim o compasso não depende do
    // detector de acorde — sem circularidade entre os dois (ADR-0014, bloco 5).
    let compasso = compasso::analisar(spectro, audio.taxa, &tempo);
    let sections = structure::segment(spectro, &chroma, audio.taxa, opts.trecho_minimo_s);
    // O baixo é lido **fora** do croma: o croma dobrou o registro, e é o
    // registro que separa `C` de `C/E` (ADR-0014, bloco 4). Os trechos usam
    // os mesmos limites dos acordes, para a inversão ser comparável.
    let baixo = baixo::extrair(spectro, audio.taxa);
    // Energia da banda do baixo, relativa à mediana da música: é o que separa
    // "não há baixo tocando" de "há baixo e não consegui resolver". Nem toda
    // introdução tem grave, e nem sempre o baixo fala — abster ali é acerto.
    let banda_baixo = spectro.energia_banda(audio.taxa, 40.0, 250.0);
    let mut ordenada = banda_baixo.clone();
    ordenada.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mediana_banda = ordenada.get(ordenada.len() / 2).copied().unwrap_or(0.0);
    let mut baixo_trechos: Vec<report::BaixoSegment> = Vec::new();
    for trecho in &mut chords {
        if let Some((midi, confianca)) = baixo.do_trecho(trecho.inicio_s, trecho.fim_s) {
            let firme = confianca >= baixo::CONFIANCA_MINIMA;
            // O baixo decide o nome quando o conjunto tem **dois nomes
            // legítimos** — `Dm7`/`F6`, `sus4`/`sus2`. O croma não pode
            // decidir: as classes de altura são as mesmas e a correlação
            // empata. Medido no repertório: 39,5% das inversões eram isso.
            if firme {
                if let Some((t, q)) = chords::nome_pelo_baixo(trecho.tonica, trecho.qualidade, midi)
                {
                    trecho.tonica = t;
                    trecho.qualidade = q;
                    trecho.acorde = chords::nome(t, q);
                }
            }
            baixo_trechos.push(report::BaixoSegment {
                inicio_s: trecho.inicio_s,
                fim_s: trecho.fim_s,
                midi,
                nota: baixo::nome(midi),
                confianca,
                cifra: if firme {
                    chords::com_baixo(trecho.tonica, trecho.qualidade, midi)
                } else {
                    // Sem evidência estável o motor não escreve a barra:
                    // inversão inventada é pior que inversão omitida.
                    None
                },
                energia: energia_do_trecho(
                    &banda_baixo,
                    mediana_banda,
                    baixo.fps,
                    trecho.inicio_s,
                    trecho.fim_s,
                ),
                tonalidade: baixo.tonalidade_do_trecho(trecho.inicio_s, trecho.fim_s),
            });
        }
    }

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
        baixo: baixo_trechos,
        melodia: melodia_trechos,
        compasso,
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
