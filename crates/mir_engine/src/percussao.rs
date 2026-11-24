//! A relação entre o **bumbo** e a **caixa** — o que separa `6/8` de `3/4`.
//!
//! A pergunta do dono (2026-11-24) expôs a falha da medida anterior: a `dobra`
//! somava bumbo, caixa, chimbau, violão e voz num fluxo espectral só, e **para
//! essa soma o acento do bumbo e o da caixa são o mesmo tipo de evento**. Não
//! são: o bumbo marca o **tempo 1**, a caixa marca o **contratempo**. Somados,
//! `6/8` (bumbo em 1, caixa em 4) e `3/4` (bumbo em 1, caixa em 2 e 3) produzem
//! o mesmo envelope — contraste **0,42 nos dois**, empate exato medido.
//!
//! Aqui os dois são medidos **separados**, e o metro sai da relação:
//!
//! - **`6/8`**: a caixa cai na **metade** do ciclo do bumbo (dois grupos de três).
//! - **`3/4`**: a caixa cai em **um terço e dois terços** (três grupos de dois).
//!
//! **Este módulo pressupõe o stem de bateria** (ADR-0017). No mix o número sai
//! contaminado, e isso está medido: a banda de 30–110 Hz carrega o baixo, e a
//! "barra do bumbo" sai **0,49–1,28 s** em vez dos 2–3 s de um compasso.

use crate::spectrum::Spectrogram;
use serde::Serialize;

/// Faixa do bumbo. O baixo vive aqui também — ver o cabeçalho.
const BUMBO_DE_HZ: f32 = 30.0;
const BUMBO_ATE_HZ: f32 = 110.0;
/// Faixa da caixa: os fios, que é o que a distingue do chimbau.
const CAIXA_DE_HZ: f32 = 1_500.0;
const CAIXA_ATE_HZ: f32 = 5_000.0;
/// Ciclo plausível do bumbo, em segundos.
const CICLO_MIN_S: f32 = 0.4;
const CICLO_MAX_S: f32 = 5.0;
/// Piso do ataque, em desvios-padrão acima da média da própria banda.
const PISO_ATAQUE: f32 = 1.5;
/// Espaçamento mínimo entre ataques da mesma peça.
const ESPACO_MIN_S: f32 = 0.08;
/// Quantas fatias do ciclo do bumbo o histograma da caixa usa.
pub const FATIAS: usize = 6;

/// A relação medida entre o bumbo e a caixa.
#[derive(Debug, Clone, Serialize, Default)]
pub struct BumboCaixa {
    /// Período do bumbo — a barra quando ele toca uma vez por compasso.
    pub periodo_bumbo_s: f32,
    /// Ataques de bumbo e de caixa encontrados.
    pub bumbos: usize,
    pub caixas: usize,
    /// Onde cada caixa cai no ciclo do bumbo, em [`FATIAS`] fatias normalizadas.
    pub caixa_no_ciclo: [f32; FATIAS],
    /// Em quantas partes a caixa dobra o ciclo do bumbo: **2** (metade, `6/8`),
    /// **3** (terços, `3/4`) ou **0** quando não há evidência.
    pub dobra: u8,
    /// Por que esta leitura.
    pub regra: String,
}

/// Novidade (fluxo positivo) da energia de uma banda, normalizada pela média.
fn novidade_da_banda(spectro: &Spectrogram, taxa: u32, de_hz: f32, ate_hz: f32) -> Vec<f32> {
    let energia = spectro.energia_banda(taxa, de_hz, ate_hz);
    let mut saida = Vec::with_capacity(energia.len());
    let mut anterior = energia.first().copied().unwrap_or(0.0);
    for &e in &energia {
        saida.push((e - anterior).max(0.0));
        anterior = e;
    }
    let media = saida.iter().sum::<f32>() / (saida.len().max(1) as f32);
    if media > f32::EPSILON {
        for x in &mut saida {
            *x /= media;
        }
    }
    saida
}

/// Período dominante por autocorrelação, com **interpolação parabólica** — a
/// mesma que o `structure::estimate_tempo` usa, e pelo mesmo motivo: sem ela o
/// período só assume valores de lag inteiro.
fn periodo_dominante(novidade: &[f32], fps: f32, min_s: f32, max_s: f32) -> f32 {
    let media = novidade.iter().sum::<f32>() / (novidade.len().max(1) as f32);
    let curva: Vec<f32> = novidade.iter().map(|v| (v - media).max(0.0)).collect();
    let min_lag = (fps * min_s).round() as usize;
    let max_lag = ((fps * max_s).round() as usize)
        .min(curva.len() / 2)
        .max(min_lag + 2);
    if curva.len() < 8 || max_lag <= min_lag + 1 {
        return 0.0;
    }
    let correlacao = |lag: usize| -> f32 {
        (0..curva.len() - lag)
            .map(|i| curva[i] * curva[i + lag])
            .sum()
    };
    let mut melhor = (min_lag, f32::MIN);
    for lag in min_lag..max_lag {
        let soma = correlacao(lag);
        if soma > melhor.1 {
            melhor = (lag, soma);
        }
    }
    // Correlação nula é ausência de periodicidade, não período no menor lag —
    // sem esta guarda o silêncio devolvia `min_lag` e o módulo inventava uma
    // dobra.
    if melhor.1 <= f32::EPSILON {
        return 0.0;
    }
    let lag = if melhor.0 > min_lag && melhor.0 + 1 < max_lag {
        let (a, b, c) = (correlacao(melhor.0 - 1), melhor.1, correlacao(melhor.0 + 1));
        let denominador = a - 2.0 * b + c;
        if denominador.abs() > f32::EPSILON {
            (melhor.0 as f32 + 0.5 * (a - c) / denominador)
                .clamp(melhor.0 as f32 - 0.5, melhor.0 as f32 + 0.5)
        } else {
            melhor.0 as f32
        }
    } else {
        melhor.0 as f32
    };
    lag / fps
}

/// Ataques de uma novidade: máximo local acima do piso, com espaçamento mínimo.
///
/// **Piso relativo à própria banda**, e é isso que faz o bumbo e a caixa serem
/// comparáveis: o piso absoluto deixaria a banda mais energética dominar.
fn ataques(novidade: &[f32], fps: f32) -> Vec<f32> {
    if novidade.len() < 3 {
        return Vec::new();
    }
    let media = novidade.iter().sum::<f32>() / novidade.len() as f32;
    let variancia = novidade
        .iter()
        .map(|v| (v - media) * (v - media))
        .sum::<f32>()
        / novidade.len() as f32;
    let piso = media + PISO_ATAQUE * variancia.sqrt();
    // Sem variância não há ataque: a novidade constante (silêncio, ou banda
    // vazia) passaria no piso e cada quadro viraria um ataque.
    if variancia <= f32::EPSILON {
        return Vec::new();
    }
    let espaco = (ESPACO_MIN_S * fps).round() as usize;
    let mut saida: Vec<f32> = Vec::new();
    let mut ultimo: Option<usize> = None;
    for i in 1..novidade.len() - 1 {
        if novidade[i] < piso || novidade[i] < novidade[i - 1] || novidade[i] < novidade[i + 1] {
            continue;
        }
        if let Some(u) = ultimo {
            if i - u < espaco {
                continue;
            }
        }
        ultimo = Some(i);
        saida.push(i as f32 / fps);
    }
    saida
}

/// Mede a relação entre o bumbo e a caixa.
///
/// Devolve `dobra = 0` sem evidência — o mesmo princípio de abstenção do resto
/// do motor: pouco ataque ou período incoerente não viram palpite.
#[must_use]
pub fn medir(spectro: &Spectrogram, taxa: u32) -> BumboCaixa {
    let fps = spectro.quadros_por_segundo(taxa).max(1.0);
    let bumbo = novidade_da_banda(spectro, taxa, BUMBO_DE_HZ, BUMBO_ATE_HZ);
    let caixa = novidade_da_banda(spectro, taxa, CAIXA_DE_HZ, CAIXA_ATE_HZ);
    let periodo = periodo_dominante(&bumbo, fps, CICLO_MIN_S, CICLO_MAX_S);
    let ataques_bumbo = ataques(&bumbo, fps);
    let ataques_caixa = ataques(&caixa, fps);
    let mut saida = BumboCaixa {
        periodo_bumbo_s: periodo,
        bumbos: ataques_bumbo.len(),
        caixas: ataques_caixa.len(),
        ..BumboCaixa::default()
    };
    if periodo <= 0.0 || ataques_bumbo.is_empty() || ataques_caixa.len() < 4 {
        saida.regra = format!(
            "sem evidência: período {periodo:.2} s, {} bumbo(s), {} caixa(s)",
            ataques_bumbo.len(),
            ataques_caixa.len()
        );
        return saida;
    }
    let mut contagem = [0usize; FATIAS];
    let mut total = 0usize;
    for &c in &ataques_caixa {
        // A caixa pertence ao ciclo do último bumbo que a precede.
        let Some(&ultimo) = ataques_bumbo.iter().rfind(|&&b| b <= c + 0.02) else {
            continue;
        };
        let fracao = (c - ultimo) / periodo;
        if !(0.0..1.0).contains(&fracao) {
            continue;
        }
        // A fatia é a **mais próxima** do centro, não o intervalo em que a fração
        // cai: com `floor` a metade exata (T/2) cai na fronteira entre duas
        // fatias e a leitura se divide ao meio — medido no sintético, 71% numa
        // fatia e 29% na seguinte, quando a verdade era o ponto exato.
        let fatia = ((fracao * FATIAS as f32).round() as usize) % FATIAS;
        contagem[fatia] += 1;
        total += 1;
    }
    if total < 4 {
        saida.regra = format!("sem evidência: só {total} caixa(s) no ciclo do bumbo");
        return saida;
    }
    for (i, fatia) in contagem.iter().enumerate() {
        saida.caixa_no_ciclo[i] = *fatia as f32 / total as f32;
    }
    // A dobra: a **metade** do ciclo contra os **terços**. É a leitura que os
    // músicos descrevem — dividir o seis por dois (`6/8`) ou por três (`3/4`).
    let metade = saida.caixa_no_ciclo[3];
    let tercos = saida.caixa_no_ciclo[2] + saida.caixa_no_ciclo[4];
    saida.dobra = if metade > tercos {
        2
    } else if tercos > metade {
        3
    } else {
        0
    };
    saida.regra = format!(
        "período do bumbo {periodo:.2} s · {} bumbo(s) e {} caixa(s) · caixa no ciclo {} · \
         metade {metade:.2} contra terços {tercos:.2} → dobra em {}",
        ataques_bumbo.len(),
        ataques_caixa.len(),
        saida
            .caixa_no_ciclo
            .iter()
            .map(|v| format!("{:.0}%", v * 100.0))
            .collect::<Vec<_>>()
            .join(" "),
        saida.dobra
    );
    saida
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spectrum::Spectrogram;
    use crate::wav::Audio;

    const TAXA: u32 = 22_050;

    /// Bateria sintética: bumbo em `bumbo_em` e caixa em `caixa_em` (1-based,
    /// dentro do compasso de `compasso` batidas), a 120 BPM. O bumbo é um seno
    /// grave curto; a caixa, ruído branco curto — as duas bandas que este módulo
    /// mede, sem nada mais no sinal.
    fn bateria(compasso: u8, bumbo_em: &[u8], caixa_em: &[u8]) -> Vec<f32> {
        let duracao_s = 24.0f32;
        let n = (duracao_s * TAXA as f32) as usize;
        let mut v = vec![0.0f32; n];
        let periodo = 0.5f32; // 120 BPM
        let mut k = 0usize;
        while (k as f32) * periodo < duracao_s {
            let pos = (k % usize::from(compasso)) as u8 + 1;
            let i0 = (k as f32 * periodo * TAXA as f32) as usize;
            if bumbo_em.contains(&pos) {
                for j in 0..(TAXA as usize / 40) {
                    if i0 + j < n {
                        let t = j as f32 / TAXA as f32;
                        v[i0 + j] +=
                            (-25.0 * t).exp() * (2.0 * std::f32::consts::PI * 60.0 * t).sin();
                    }
                }
            }
            if caixa_em.contains(&pos) {
                for j in 0..(TAXA as usize / 60) {
                    if i0 + j < n {
                        let t = j as f32 / TAXA as f32;
                        let x = (((i0 + j) as f32 * 12.9898).sin()) * 43_758.547;
                        v[i0 + j] += 0.8 * (-40.0 * t).exp() * (x - x.floor() - 0.5);
                    }
                }
            }
            k += 1;
        }
        v
    }

    fn medir_amostras(amostras: Vec<f32>) -> BumboCaixa {
        let audio = Audio {
            amostras,
            taxa: TAXA,
            canais: 1,
        };
        let spectro = Spectrogram::new(&audio, 4096, 1024);
        medir(&spectro, TAXA)
    }

    #[test]
    fn a_caixa_na_metade_e_composto_e_nos_tercos_e_simples() {
        // `6/8`: bumbo no 1, caixa no 4 de seis — a caixa cai na **metade** do
        // ciclo do bumbo. É a medida que a `dobra` não fazia: lá os dois acentos
        // eram somados e davam o mesmo envelope (0,42 nos dois, empate exato).
        let composto = medir_amostras(bateria(6, &[1], &[4]));
        assert_eq!(
            composto.dobra, 2,
            "6/8 não dobrou em dois: {}",
            composto.regra
        );
        assert!(
            (composto.periodo_bumbo_s - 3.0).abs() < 0.3,
            "a barra do bumbo devia ser 3,0 s: {}",
            composto.regra
        );

        // `3/4`: bumbo no 1, caixa no 2 e no 3 — a caixa cai em **um terço e
        // dois terços**.
        let simples = medir_amostras(bateria(3, &[1], &[2, 3]));
        assert_eq!(
            simples.dobra, 3,
            "3/4 não dobrou em três: {}",
            simples.regra
        );
        assert!(
            (simples.periodo_bumbo_s - 1.5).abs() < 0.25,
            "a barra do bumbo devia ser 1,5 s: {}",
            simples.regra
        );
    }

    #[test]
    fn silencio_nao_inventa_relacao() {
        let silencio = medir_amostras(vec![0.0; TAXA as usize * 12]);
        assert_eq!(silencio.dobra, 0, "inventou dobra no silêncio");
        assert!(
            silencio.regra.contains("sem evidência"),
            "{}",
            silencio.regra
        );
    }
}
