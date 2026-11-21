//! Andamento e estrutura.
//!
//! - **Andamento**: autocorrelação da novidade espectral na faixa de 60–200 BPM,
//!   com dobra do resultado quando a estimativa cai no harmônico.
//! - **Batidas**: picos da novidade acima de um limiar móvel, com distância
//!   mínima entre eles.
//! - **Estrutura**: cortes por pico de novidade respeitando um mínimo de trecho;
//!   cada trecho ganha uma assinatura (croma médio) e trechos parecidos recebem
//!   o mesmo rótulo (A, B, C…).
//!
//! **Rótulo não é semântica.** O motor detecta repetição e contraste; chamar um
//! trecho de "refrão" é inferência por posição e recorrência, para o músico
//! confirmar — não é compreensão musical.

use serde::Serialize;

use crate::spectrum::{Chroma, Spectrogram};

/// Andamento estimado.
#[derive(Debug, Clone, Serialize)]
pub struct Tempo {
    pub bpm: f32,
    /// Instantes das batidas detectadas, em segundos.
    pub batidas_s: Vec<f32>,
}

/// Um trecho da estrutura.
#[derive(Debug, Clone, Serialize)]
pub struct Section {
    pub inicio_s: f32,
    pub fim_s: f32,
    /// Rótulo de repetição: `A`, `B`, `C`…
    pub rotulo: String,
    /// Papel **inferido** por posição e recorrência.
    pub papel: String,
    /// Energia média do trecho, relativa à média da música.
    pub energia_relativa: f32,
}

/// Picos de uma curva, com limiar móvel e distância mínima.
fn picos(curva: &[f32], fps: f32, distancia_min_s: f32, fator: f32) -> Vec<usize> {
    if curva.is_empty() {
        return Vec::new();
    }
    let media = curva.iter().sum::<f32>() / curva.len() as f32;
    let variancia =
        curva.iter().map(|v| (v - media) * (v - media)).sum::<f32>() / curva.len() as f32;
    let limiar = media + fator * variancia.sqrt();
    let distancia = ((distancia_min_s * fps) as usize).max(1);
    let mut saida: Vec<usize> = Vec::new();
    for i in 1..curva.len().saturating_sub(1) {
        if curva[i] >= limiar
            && curva[i] >= curva[i - 1]
            && curva[i] > curva[i + 1]
            && saida.last().is_none_or(|&u| i - u >= distancia)
        {
            saida.push(i);
        }
    }
    saida
}

/// Estima andamento e batidas pela novidade espectral.
#[must_use]
pub fn estimate_tempo(spectro: &Spectrogram, taxa: u32) -> Tempo {
    let novidade = spectro.novidade();
    let fps = spectro.quadros_por_segundo(taxa).max(1.0);
    if novidade.len() < 8 {
        return Tempo {
            bpm: 0.0,
            batidas_s: Vec::new(),
        };
    }
    // Remove o nível médio: autocorrelação de sinal com offset não acha período.
    let media = novidade.iter().sum::<f32>() / novidade.len() as f32;
    let curva: Vec<f32> = novidade.iter().map(|v| (v - media).max(0.0)).collect();
    let min_lag = (fps * 60.0 / 200.0) as usize;
    let max_lag = ((fps * 60.0 / 60.0) as usize)
        .min(curva.len() / 2)
        .max(min_lag + 1);
    let mut melhor = (min_lag, f32::MIN);
    for lag in min_lag..max_lag {
        let soma: f32 = (0..curva.len() - lag)
            .map(|i| curva[i] * curva[i + lag])
            .sum();
        if soma > melhor.1 {
            melhor = (lag, soma);
        }
    }
    let mut bpm = 60.0 * fps / melhor.0 as f32;
    // Dobra para a faixa cantada quando a estimativa cai no harmônico.
    while bpm < 70.0 {
        bpm *= 2.0;
    }
    while bpm > 190.0 {
        bpm /= 2.0;
    }
    let batidas_s = picos(&novidade, fps, 0.25, 0.8)
        .into_iter()
        .map(|i| i as f32 / fps)
        .collect();
    Tempo { bpm, batidas_s }
}

/// Segmenta a música e rotula por repetição.
#[must_use]
pub fn segment(
    spectro: &Spectrogram,
    chroma: &Chroma,
    taxa: u32,
    trecho_minimo_s: f32,
) -> Vec<Section> {
    let novidade = spectro.novidade();
    let fps = spectro.quadros_por_segundo(taxa).max(1.0);
    let duracao = novidade.len() as f32 / fps;
    if duracao <= 0.0 {
        return Vec::new();
    }
    // Fronteiras: picos fortes de novidade, espaçados pelo mínimo. O fator 1.5
    // corta só transição marcada; 1.0 cortava a cada acorde.
    let mut cortes: Vec<f32> = vec![0.0];
    for i in picos(&novidade, fps, trecho_minimo_s, 1.5) {
        let t = i as f32 / fps;
        if t - cortes.last().copied().unwrap_or(0.0) >= trecho_minimo_s {
            cortes.push(t);
        }
    }
    cortes.push(duracao);

    // Assinatura por trecho (croma médio) e energia relativa (novidade média).
    let energia_total = novidade.iter().sum::<f32>() / novidade.len() as f32;
    let mut assinaturas: Vec<[f32; 12]> = Vec::new();
    let mut trechos: Vec<(f32, f32, f32)> = Vec::new();
    for par in cortes.windows(2) {
        let (inicio, fim) = (par[0], par[1]);
        let inicio_q = ((inicio * fps) as usize).min(novidade.len());
        let fim_q = ((fim * fps) as usize).min(novidade.len());
        let energia = if fim_q > inicio_q {
            novidade[inicio_q..fim_q].iter().sum::<f32>() / (fim_q - inicio_q) as f32
        } else {
            0.0
        };
        assinaturas.push(chroma.media(inicio, fim));
        trechos.push((inicio, fim, energia));
    }

    // Agrupa por semelhança de croma: cosseno acima de 0,9 é o mesmo rótulo.
    let mut rotulos: Vec<usize> = Vec::new();
    let mut proximo = 0usize;
    for (i, assinatura) in assinaturas.iter().enumerate() {
        let norma = (assinatura.iter().map(|v| v * v).sum::<f32>()).sqrt();
        let mut achado = None;
        if norma > 0.0 {
            for (j, anterior) in assinaturas[..i].iter().enumerate() {
                let norma_j = (anterior.iter().map(|v| v * v).sum::<f32>()).sqrt();
                if norma_j <= 0.0 {
                    continue;
                }
                let cos: f32 =
                    (0..12).map(|k| assinatura[k] * anterior[k]).sum::<f32>() / (norma * norma_j);
                if cos > 0.9 {
                    achado = Some(rotulos[j]);
                    break;
                }
            }
        }
        match achado {
            Some(rotulo) => rotulos.push(rotulo),
            None => {
                rotulos.push(proximo);
                proximo += 1;
            }
        }
    }

    // Papel inferido: posição manda; a seção MAIS repetida é a candidata a
    // refrão; as outras que também voltam são seções repetidas; o que aparece
    // uma vez só, no meio, é candidato a interlúdio. É inferência, não semântica.
    let contagem_por_rotulo: Vec<usize> = (0..proximo)
        .map(|r| rotulos.iter().filter(|&&x| x == r).count())
        .collect();
    let mais_repetido = contagem_por_rotulo
        .iter()
        .enumerate()
        .max_by_key(|&(_, &n)| n)
        .map(|(r, &n)| (r, n))
        .unwrap_or((0, 0));

    trechos
        .iter()
        .enumerate()
        .map(|(i, &(inicio, fim, energia))| {
            let rotulo = rotulos[i];
            let repeticoes = contagem_por_rotulo[rotulo];
            let papel = if i == 0 {
                "introducao".to_owned()
            } else if i == trechos.len() - 1 {
                "final".to_owned()
            } else if rotulo == mais_repetido.0 && mais_repetido.1 >= 2 {
                "refrao_candidato".to_owned()
            } else if repeticoes >= 2 {
                "secao_repetida".to_owned()
            } else {
                "interludio_candidato".to_owned()
            };
            Section {
                inicio_s: inicio,
                fim_s: fim,
                rotulo: format!("{}", (b'A' + rotulo as u8) as char),
                papel,
                energia_relativa: if energia_total > 0.0 {
                    energia / energia_total
                } else {
                    1.0
                },
            }
        })
        .collect()
}
