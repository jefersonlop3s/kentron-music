//! Compasso e downbeat: onde fica o tempo 1.
//!
//! O motor tinha **batida, não barra**. `structure::estimate_tempo` devolve BPM e
//! uma lista de **ataques**, e ataque não é tempo: a lista segue a música, não a
//! grade. Sem barra não existe "tempo 1", e sem tempo 1 não existe antecipação —
//! "o baixo chegou antes" só é afirmação se houver um antes. Foi por isso que os
//! 50% de baixo fora do acorde (metade deles movimento) ficaram sem nome.
//!
//! Duas evidências decidem, e as duas são medidas aqui:
//!
//! 1. **Acento** — o tempo 1 costuma ser o mais forte. O acento de cada batida é
//!    a novidade espectral somada numa janela em volta dela.
//! 2. **Troca de acorde** — acorde troca preferencialmente no tempo 1. É a
//!    evidência mais forte disponível e já existe: os limites dos trechos.
//!
//! **Fail-loud:** quando a margem entre o melhor compasso e o segundo é pequena,
//! o motor devolve **indeterminado com a razão** — não escolhe 4/4 por ser comum.
//! É a mesma regra do centro tonal (`crate::tonality`).

use serde::Serialize;

use crate::chords::ChordSegment;
use crate::spectrum::Spectrogram;
use crate::structure::Tempo;

/// Compassos testados. Binário, ternário e quaternário cobrem o repertório.
const COMPASSOS: [u8; 3] = [2, 3, 4];
/// Janela em volta da batida onde o acento é somado.
const JANELA_ACENTO_S: f32 = 0.05;
/// Distância máxima entre troca de acorde e batida para contar como alinhada.
const TOLERANCIA_TROCA_S: f32 = 0.07;
/// Margem abaixo da qual a leitura é **frágil** — não é limiar de abstenção.
///
/// O histograma de margem decai monotonicamente, **sem vale** onde cortar
/// (medido em 263 músicas, 2026-11-22). Então este número não separa "sei" de
/// "não sei": só marca quando as alternativas empatam, e quem decide o que fazer
/// com isso é quem consome.
const MARGEM_MINIMA: f32 = 0.06;

/// O compasso medido.
#[derive(Debug, Clone, Serialize)]
pub struct Compasso {
    /// Batidas por compasso. **0 = indeterminado** — o motor não escolhe.
    pub batidas_por_compasso: u8,
    /// Tempo 1 do primeiro compasso **com música** (depois do primeiro ataque).
    pub inicio_s: f32,
    /// Período da batida, em segundos.
    pub periodo_s: f32,
    /// Acento médio no tempo 1 menos o acento médio nos outros tempos.
    pub confianca: f32,
    /// Distância para o segundo colocado. Baixa = decisão frágil.
    pub margem: f32,
    /// Tempo em que a música **entra**, de 1 a `batidas_por_compasso`; 0 quando
    /// indeterminado.
    ///
    /// Não é sempre 1: o primeiro som pode cair no **tempo 3 ou 4 de um compasso
    /// fantasma** — a banda entra no tempo 1 do compasso seguinte e o tempo 1 do
    /// primeiro nunca foi tocado. Contar a barra a partir da primeira nota erra
    /// a contagem por dois ou três tempos, e o músico percebe na hora.
    pub entrada_no_tempo: u8,
    /// Por que este compasso — ou por que não houve decisão.
    pub regra: String,
}

/// Soma a novidade numa janela em volta de um instante.
fn acento(novidade: &[f32], fps: f32, t: f32) -> f32 {
    let meia = (JANELA_ACENTO_S * fps).ceil() as usize;
    let centro = (t * fps).round() as isize;
    let inicio = (centro - meia as isize).max(0) as usize;
    let fim = ((centro + meia as isize) as usize).min(novidade.len());
    if inicio >= fim {
        return 0.0;
    }
    novidade[inicio..fim].iter().sum()
}

/// Encontra a fase da grade que melhor casa com os ataques, e devolve a grade.
///
/// A grade é **regular** (`fase + k * período`), ao contrário da lista de ataques
/// do `estimate_tempo`: o que se procura aqui é o pulso, não o evento.
fn grade(periodo: f32, novidade: &[f32], fps: f32) -> (f32, Vec<f32>) {
    let duracao = novidade.len() as f32 / fps;
    let mut melhor = (0.0f32, f32::MIN);
    let mut fase = 0.0;
    while fase < periodo {
        let mut soma = 0.0;
        let mut t = fase;
        while t < duracao {
            soma += acento(novidade, fps, t);
            t += periodo;
        }
        if soma > melhor.1 {
            melhor = (fase, soma);
        }
        fase += 0.01;
    }
    let mut batidas = Vec::new();
    let mut t = melhor.0;
    while t < duracao {
        batidas.push(t);
        t += periodo;
    }
    (melhor.0, batidas)
}

/// Faixa plausível de batida, em BPM. Fora dela a "batida" é subdivisão ou
/// compasso inteiro, não pulso.
const BPM_MIN: f32 = 50.0;
const BPM_MAX: f32 = 200.0;

/// Um compasso candidato: (batidas por compasso, fase do tempo 1, escore,
/// acento relativo).
type Candidato = (u8, u8, f32, f32);

/// Resultado da avaliação de **uma** oitava de tempo.
struct Avaliacao {
    periodo: f32,
    fase: f32,
    batidas: Vec<f32>,
    indices: Vec<usize>,
    combos: Vec<Candidato>,
}

/// Avalia uma oitava de tempo: grade, âncoras de acorde e todos os compassos.
///
/// Devolve `None` quando a oitava não sustenta grade ou não tem âncora suficiente
/// — e aí ela simplesmente não concorre, em vez de concorrer mal.
fn avaliar(periodo: f32, novidade: &[f32], fps: f32, chords: &[ChordSegment]) -> Option<Avaliacao> {
    let (fase, batidas) = grade(periodo, novidade, fps);
    if batidas.len() < 8 {
        return None;
    }
    // A troca de acorde é a âncora: acorde troca no tempo 1. Só conta a troca que
    // cai **perto** de uma batida — troca fora da grade não ancora nada.
    let indices: Vec<usize> = chords
        .iter()
        .skip(1)
        .filter_map(|c| {
            let k = ((c.inicio_s - fase) / periodo).round();
            if k < 0.0 {
                return None;
            }
            let i = k as usize;
            if i >= batidas.len() || (batidas[i] - c.inicio_s).abs() > TOLERANCIA_TROCA_S {
                return None;
            }
            Some(i)
        })
        .collect();
    if indices.len() < 8 {
        return None;
    }
    let acentos: Vec<f32> = batidas.iter().map(|&t| acento(novidade, fps, t)).collect();
    let media: f32 = acentos.iter().sum::<f32>() / acentos.len() as f32;
    if media <= f32::EPSILON {
        return None;
    }
    let trocas_total = chords.len().saturating_sub(1);
    let mut combos = Vec::new();
    for m in COMPASSOS {
        for p in 0..m {
            let (mut no_um, mut fora) = (0.0f32, 0.0f32);
            let (mut n_um, mut n_fora) = (0usize, 0usize);
            for (i, &a) in acentos.iter().enumerate() {
                if (i as u8) % m == p {
                    no_um += a;
                    n_um += 1;
                } else {
                    fora += a;
                    n_fora += 1;
                }
            }
            if n_um == 0 || n_fora == 0 {
                continue;
            }
            let acento_rel = no_um / n_um as f32 / media - fora / n_fora as f32 / media;
            // A fração é sobre **todas** as trocas de acorde, não só as que caem
            // na grade: um candidato que ignora a maior parte da evidência
            // harmônica não pode empatar com o que explica toda ela. Sem isso, a
            // valsa em 3/4 empata com a leitura em meio tempo (bug medido).
            let alinhadas = indices
                .iter()
                .filter(|&&i| (i % usize::from(m)) == usize::from(p))
                .count();
            let troca_rel = alinhadas as f32 / trocas_total as f32 - 1.0 / f32::from(m);
            combos.push((m, p, acento_rel + troca_rel, acento_rel));
        }
    }
    if combos.is_empty() {
        return None;
    }
    Some(Avaliacao {
        periodo,
        fase,
        batidas,
        indices,
        combos,
    })
}

/// Mede o compasso e o tempo 1.
///
/// Devolve indeterminado (sem inventar) quando não há batida, quando a margem é
/// pequena ou quando não há troca de acorde para ancorar.
#[must_use]
pub fn analisar(
    spectro: &Spectrogram,
    taxa: u32,
    tempo: &Tempo,
    chords: &[ChordSegment],
) -> Compasso {
    let indeterminado = |regra: String, periodo: f32| Compasso {
        batidas_por_compasso: 0,
        inicio_s: 0.0,
        periodo_s: periodo,
        confianca: 0.0,
        margem: 0.0,
        entrada_no_tempo: 0,
        regra,
    };
    if tempo.bpm <= 0.0 || tempo.batidas_s.is_empty() {
        return indeterminado("indeterminado: sem batida estimada".into(), 0.0);
    }
    let novidade = spectro.novidade();
    let fps = spectro.quadros_por_segundo(taxa).max(1.0);
    if novidade.len() < 8 {
        return indeterminado("indeterminado: áudio curto demais para grade".into(), 0.0);
    }
    // A **oitava do tempo** é convenção, não verdade: 4/4 a 78 BPM é 2/4 a 156, e
    // o `estimate_tempo` já dobrou o BPM para a faixa 70–190. Medido no
    // repertório: 31% das músicas divergem da planilha numa oitava. Em vez de
    // escolher uma, o motor testa as três e deixa a evidência decidir.
    let mut avaliados: Vec<Avaliacao> = Vec::new();
    for fator in [1.0f32, 2.0, 0.5] {
        let bpm = tempo.bpm * fator;
        if (BPM_MIN..=BPM_MAX).contains(&bpm) {
            if let Some(a) = avaliar(60.0 / bpm, &novidade, fps, chords) {
                avaliados.push(a);
            }
        }
    }
    let mut melhor: Option<(&Avaliacao, Candidato)> = None;
    for a in &avaliados {
        if let Some(c) = a
            .combos
            .iter()
            .max_by(|x, y| x.2.partial_cmp(&y.2).unwrap_or(std::cmp::Ordering::Equal))
        {
            if melhor.is_none_or(|(_, m)| c.2 > m.2) {
                melhor = Some((a, *c));
            }
        }
    }
    let vencedor = match melhor {
        Some(v) => v,
        None => {
            return indeterminado(
                "indeterminado: nenhuma oitava de tempo sustenta grade e âncora".into(),
                60.0 / tempo.bpm,
            );
        }
    };
    let (vencedor, (melhor_m, melhor_p, escore, acento_rel)) = vencedor;
    // A margem é contra o melhor de **outro** compasso ou de **outra** oitava:
    // escolher entre fases do mesmo compasso não decide a barra, decide onde ela
    // começa.
    let mut rival = f32::NEG_INFINITY;
    let mut rival_m = 0u8;
    for a in &avaliados {
        for &(m, _, e, _) in &a.combos {
            let outra_oitava = (a.periodo - vencedor.periodo).abs() > 1e-6;
            if (m != melhor_m || outra_oitava) && e > rival {
                rival = e;
                rival_m = m;
            }
        }
    }
    let margem = if rival.is_finite() {
        escore - rival
    } else {
        // Sem rival não há margem a medir: só uma leitura caberia na grade.
        1.0
    };
    // O limiar de margem **não existe nos dados**: a distribuição decai
    // monotonicamente, sem vale onde cortar (medido em 263 músicas). Por isso o
    // motor **não abstém por margem** — devolve a melhor leitura com a margem e
    // diz quando ela é frágil, como faz o centro tonal. Abster fica reservado ao
    // que é estrutural: sem batida, sem âncora de acorde, sem grade.
    let fragil = margem < MARGEM_MINIMA;
    let alinhadas = vencedor
        .indices
        .iter()
        .filter(|&&i| (i % usize::from(melhor_m)) == usize::from(melhor_p))
        .count();
    // O tempo 1 do primeiro compasso **com música**. E onde a música entra: o
    // primeiro som pode cair no tempo 3 ou 4 de um compasso **fantasma**, cujo
    // tempo 1 ninguém tocou — a banda entra no tempo 1 do compasso seguinte.
    let (fase, periodo) = (vencedor.fase, vencedor.periodo);
    let primeiro_ataque = tempo.batidas_s.first().copied().unwrap_or(fase);
    let k = ((primeiro_ataque - fase) / periodo).floor();
    let pos = (k as isize - isize::from(melhor_p)).rem_euclid(isize::from(melhor_m));
    let entrada = (pos + 1) as u8;
    let tempo1 = fase + (k - pos as f32) * periodo;
    let inicio = if tempo1 >= 0.0 {
        tempo1
    } else {
        // O compasso fantasma começa **antes do áudio**: a introdução foi
        // cortada. O tempo 1 que existe é o do compasso seguinte.
        vencedor
            .batidas
            .iter()
            .enumerate()
            .filter(|(i, _)| (i % usize::from(melhor_m)) == usize::from(melhor_p))
            .map(|(_, &b)| b)
            .find(|&b| b >= primeiro_ataque)
            .unwrap_or(fase)
    };
    Compasso {
        batidas_por_compasso: melhor_m,
        inicio_s: inicio,
        periodo_s: periodo,
        confianca: acento_rel,
        margem,
        entrada_no_tempo: entrada,
        regra: format!(
            "{melhor_m} batidas por compasso: acento do tempo 1 {acento_rel:+.3} acima da média, \
             {:.0}% das trocas de acorde alinhadas, margem {margem:.3} contra {rival_m} batidas{}; \
             a música entra no tempo {entrada}{}",
            100.0 * alinhadas as f32 / vencedor.indices.len() as f32,
            if fragil {
                " — leitura frágil, as alternativas empatam"
            } else {
                ""
            },
            if entrada == 1 {
                ""
            } else {
                " — compasso fantasma, o tempo 1 do primeiro não foi tocado"
            }
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chords::Qualidade;
    use crate::wav::Audio;

    const TAXA: u32 = 44_100;

    /// Cliques: forte no tempo 1, fraco nos outros. Ruído determinístico curto,
    /// que é o que a novidade espectral enxerga como ataque.
    fn cliques(bpm: f32, compasso: u8, duracao_s: f32) -> Vec<f32> {
        let n = (duracao_s * TAXA as f32) as usize;
        let mut v = vec![0.0f32; n];
        let periodo = 60.0 / bpm;
        let mut k = 0usize;
        while (k as f32) * periodo < duracao_s {
            let amp = if k % usize::from(compasso) == 0 {
                1.0
            } else {
                0.25
            };
            let i0 = (k as f32 * periodo * TAXA as f32) as usize;
            for j in 0..(TAXA as usize / 250) {
                if i0 + j < n {
                    let x = ((i0 + j) as f32 * 12.9898).sin() * 43_758.547;
                    v[i0 + j] += amp * (x - x.floor() - 0.5);
                }
            }
            k += 1;
        }
        v
    }

    /// Trechos de acorde trocando a cada `passo_s` — as trocas são a âncora.
    fn trechos(inicio_s: f32, fim_s: f32, passo_s: f32) -> Vec<ChordSegment> {
        let mut v = Vec::new();
        let mut t = inicio_s;
        while t < fim_s {
            v.push(ChordSegment {
                inicio_s: t,
                fim_s: t + passo_s,
                acorde: "C".into(),
                confianca: 0.5,
                tonica: 0,
                qualidade: Qualidade::Maior,
            });
            t += passo_s;
        }
        v
    }

    fn grade_de(bpm: f32, duracao_s: f32) -> Tempo {
        let periodo = 60.0 / bpm;
        let mut batidas = Vec::new();
        let mut t = 0.0;
        while t < duracao_s {
            batidas.push(t);
            t += periodo;
        }
        Tempo {
            bpm,
            batidas_s: batidas,
        }
    }

    /// Ataques que só começam em `de_s` — o que `estimate_tempo` devolveria numa
    /// gravação cuja introdução começa depois.
    fn ataques_de(bpm: f32, de_s: f32, duracao_s: f32) -> Tempo {
        let periodo = 60.0 / bpm;
        let mut batidas = Vec::new();
        let mut t = de_s;
        while t < duracao_s {
            batidas.push(t);
            t += periodo;
        }
        Tempo {
            bpm,
            batidas_s: batidas,
        }
    }

    fn medir(amostras: Vec<f32>, tempo: &Tempo, chords: &[ChordSegment]) -> Compasso {
        let audio = Audio {
            amostras,
            taxa: TAXA,
            canais: 1,
        };
        let spectro = Spectrogram::new(&audio, 4096, 1024);
        analisar(&spectro, TAXA, tempo, chords)
    }

    #[test]
    fn acha_o_compasso_quaternario() {
        // 120 BPM, acento a cada 4 batidas (compasso de 2 s), troca em cada compasso.
        let audio = cliques(120.0, 4, 40.0);
        let compasso = medir(audio, &grade_de(120.0, 40.0), &trechos(0.0, 40.0, 2.0));
        assert_eq!(
            compasso.batidas_por_compasso, 4,
            "regra: {}",
            compasso.regra
        );
    }

    #[test]
    fn acha_a_valsa_ternaria() {
        // 150 BPM, acento a cada 3 batidas (compasso de 1,2 s).
        let audio = cliques(150.0, 3, 30.0);
        let compasso = medir(audio, &grade_de(150.0, 30.0), &trechos(0.0, 30.0, 1.2));
        assert_eq!(
            compasso.batidas_por_compasso, 3,
            "regra: {}",
            compasso.regra
        );
    }

    #[test]
    fn ve_o_compasso_fantasma_e_conta_a_entrada() {
        // 4/4 a 120 BPM: compasso de 2 s, batida de 0,5 s. O primeiro som só
        // aparece em 1,0 s — o **tempo 3** do primeiro compasso. O tempo 1 e o
        // tempo 2 desse compasso ninguém tocou: é o compasso fantasma. Contar a
        // barra a partir da primeira nota erraria por dois tempos.
        let mut audio = cliques(120.0, 4, 40.0);
        let corte = (1.0 * TAXA as f32) as usize;
        for a in audio.iter_mut().take(corte) {
            *a = 0.0;
        }
        let compasso = medir(
            audio,
            &ataques_de(120.0, 1.0, 40.0),
            &trechos(0.0, 40.0, 2.0),
        );
        assert_eq!(
            compasso.batidas_por_compasso, 4,
            "regra: {}",
            compasso.regra
        );
        assert_eq!(
            compasso.entrada_no_tempo, 3,
            "esperava entrada no tempo 3, regra: {}",
            compasso.regra
        );
        assert!(
            compasso.regra.contains("fantasma"),
            "a regra tem de nomear o compasso fantasma: {}",
            compasso.regra
        );
    }

    #[test]
    fn sem_troca_de_acorde_nao_escolhe_compasso() {
        // Sem âncora harmônica o motor se abstém: acento sozinho confunde 2 e 4.
        let audio = cliques(120.0, 4, 30.0);
        let compasso = medir(audio, &grade_de(120.0, 30.0), &[]);
        assert_eq!(compasso.batidas_por_compasso, 0);
        assert!(
            compasso.regra.contains("indeterminado"),
            "regra: {}",
            compasso.regra
        );
    }

    #[test]
    fn silencio_nao_inventa_compasso() {
        let compasso = medir(
            vec![0.0; TAXA as usize * 20],
            &grade_de(120.0, 20.0),
            &trechos(0.0, 20.0, 2.0),
        );
        assert_eq!(compasso.batidas_por_compasso, 0);
    }
}
