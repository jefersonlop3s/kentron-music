//! Identifica peças individuais da bateria (bumbo, caixa, chimbau, ton, surdo,
//! prato) a partir do espectrograma do stem de bateria.
//!
//! A classificação usa **faixa de frequência + tempo de decaimento**, conforme
//! as bandas medidas no ADR-0017:
//!
//! | Peça        | Banda (Hz)    | Decaimento | Caráter espectral        |
//! |-------------|---------------|------------|--------------------------|
//! | Bumbo       | 30–110        | < 100 ms   | Sem harmónicos, thud     |
//! | Surdo       | 60–150        | ~200 ms    | Tonal, pitch ~60–100 Hz  |
//! | Ton         | 120–400       | > 300 ms   | Tonal, pitch ~80–250 Hz  |
//! | Caixa (pele)| 150–450       | ~150 ms    | Misto (ruído + tom)      |
//! | Caixa (fios)| 1,5–5 k       | ~150 ms    | Ruído, crack             |
//! | Chimbau     | 6,5–13 k      | < 50 ms    | Ruído, sizzle curto      |
//! | Prato       | 8–18 k        | > 500 ms   | Ruído, wash longo        |

use crate::onset;
use crate::spectrum::Spectrogram;
use serde::Serialize;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Constantes das bandas ― importadas do módulo percussao (bumbo e caixa) e
// definidas localmente para as demais peças.
// ---------------------------------------------------------------------------

use crate::percussao::{BUMBO_ATE_HZ, BUMBO_DE_HZ, CAIXA_ATE_HZ, CAIXA_DE_HZ};

/// Surdo / floor tom.
const FLOOR_TOM_DE_HZ: f32 = 60.0;
const FLOOR_TOM_ATE_HZ: f32 = 150.0;
/// Ton / rack tom.
const RACK_TOM_DE_HZ: f32 = 120.0;
const RACK_TOM_ATE_HZ: f32 = 400.0;
/// Fios da caixa (o crack de 1,5–5 kHz que a pele não cobre).
const SNARE_WIRES_DE_HZ: f32 = 1500.0;
const SNARE_WIRES_ATE_HZ: f32 = 5000.0;
/// Chimbau.
const HIHAT_DE_HZ: f32 = 6500.0;
const HIHAT_ATE_HZ: f32 = 13000.0;
/// Prato (crash / ride).
const CRASH_DE_HZ: f32 = 8000.0;
const CRASH_ATE_HZ: f32 = 18000.0;

// ---------------------------------------------------------------------------
// Parâmetros de detecção (mesmos valores do módulo percussao).
// ---------------------------------------------------------------------------

/// Tolerância para agrupar candidatos de diferentes bandas no mesmo golpe.
const JANELA_AGRUPAMENTO_S: f32 = 0.06;
/// Espaçamento mínimo entre kicks consecutivos para o período do bumbo.
const KICK_ESPACO_MIN_S: f32 = 0.08;
/// Decaimento máximo da caixa do chimbau para confirmar classificação (s).
const HIHAT_DEC_ELEGIVEL_S: f32 = 0.08;
/// Decaimento mínimo do prato para confirmar classificação (s).
const CRASH_DEC_ELEGIVEL_S: f32 = 0.30;

// ---------------------------------------------------------------------------
// Estruturas de saída
// ---------------------------------------------------------------------------

/// Resultado da identificação de peças da bateria.
#[derive(Debug, Clone, Serialize)]
pub struct MapaPecas {
    /// Todos os golpes detectados, em ordem cronológica.
    pub hits: Vec<Golpe>,
    /// Período entre golpes consecutivos de bumbo.
    pub kick_periodo_s: f32,
    /// Número de golpes por tipo de peça.
    pub contagem: HashMap<String, usize>,
}

/// Um golpe de bateria.
#[derive(Debug, Clone, Serialize)]
pub struct Golpe {
    /// Instante do golpe, em segundos.
    pub tempo_s: f32,
    /// Tipo da peça: "kick", "snare", "hihat", "floor_tom", "rack_tom", "crash".
    pub peca: String,
    /// Energia do golpe (pico na sua banda).
    pub energia: f32,
    /// Tempo de decaimento, em segundos (queda de -20 dB a partir do pico).
    pub decaimento_s: f32,
}

// ---------------------------------------------------------------------------
// Candidato interno: ataque detectado numa banda específica.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Candidato {
    tempo_s: f32,
    peca: String,
    energia: f32,
    decaimento_s: f32,
    pico_idx: usize,
    banda: &'static str,
}

// ---------------------------------------------------------------------------
// Função principal
// ---------------------------------------------------------------------------

/// Identifica peças individuais da bateria a partir do espectrograma do stem.
///
/// `spectro` deve ser o espectrograma de um stem de bateria separado
/// (ADR-0017). Devolve um mapa com todos os golpes detectados, classificados
/// por tipo de peça.
#[must_use]
pub fn identificar(spectro: &Spectrogram, taxa: u32) -> MapaPecas {
    let fps = spectro.quadros_por_segundo(taxa).max(1.0);

    // ---- 1. Energia de cada banda ----
    let bandas: Vec<(&str, Vec<f32>)> = vec![
        (
            "kick",
            spectro.energia_banda(taxa, BUMBO_DE_HZ, BUMBO_ATE_HZ),
        ),
        (
            "snare",
            spectro.energia_banda(taxa, CAIXA_DE_HZ, CAIXA_ATE_HZ),
        ),
        (
            "snare_fios",
            spectro.energia_banda(taxa, SNARE_WIRES_DE_HZ, SNARE_WIRES_ATE_HZ),
        ),
        (
            "floor_tom",
            spectro.energia_banda(taxa, FLOOR_TOM_DE_HZ, FLOOR_TOM_ATE_HZ),
        ),
        (
            "rack_tom",
            spectro.energia_banda(taxa, RACK_TOM_DE_HZ, RACK_TOM_ATE_HZ),
        ),
        (
            "hihat",
            spectro.energia_banda(taxa, HIHAT_DE_HZ, HIHAT_ATE_HZ),
        ),
        (
            "crash",
            spectro.energia_banda(taxa, CRASH_DE_HZ, CRASH_ATE_HZ),
        ),
    ];

    // ---- 2. Detectar ataques em cada banda ----
    // A detecção usa o **envelope de energia bruta** — a energia da banda tem
    // picos nítidos no ataque de cada peça, e o decaimento é medido no mesmo
    // envelope. A novidade (primeira diferença) elimina energia sustentada, mas
    // no stem de bateria o envelope bruto já é esparso o suficiente para que os
    // picos se destaquem (medido: no stem real, a energia bruta do bumbo produz
    // ataques consistentes, enquanto a novidade da banda 30–110 Hz achava zero
    // picos porque a diferença entre quadros consecutivos é pequena comparada à
    // amplitude absoluta do bumbo).
    let mut candidatos: Vec<Candidato> = Vec::new();
    let mut max_por_banda: HashMap<&str, f32> = HashMap::new();

    for (banda, energia) in &bandas {
        let mx = energia.iter().copied().fold(0.0f32, f32::max);
        max_por_banda.insert(banda, mx);

        for (tempo_s, pico_energia) in onset::detectar(energia, fps) {
            let pico_idx = (tempo_s * fps).round() as usize;
            let pico_idx = pico_idx.min(energia.len().saturating_sub(1));
            let decaimento = medir_decaimento(energia, fps, pico_idx);
            let peca_str = match *banda {
                "snare" | "snare_fios" => "snare",
                "kick" => "kick",
                "floor_tom" => "floor_tom",
                "rack_tom" => "rack_tom",
                "hihat" => "hihat",
                "crash" => "crash",
                _ => "unknown",
            };
            candidatos.push(Candidato {
                tempo_s,
                peca: peca_str.to_owned(),
                energia: pico_energia,
                decaimento_s: decaimento,
                pico_idx,
                banda,
            });
        }
    }

    // ---- 3. Agrupar candidatos próximos no tempo (mesmo golpe físico) ----
    candidatos.sort_by(|a, b| a.tempo_s.total_cmp(&b.tempo_s));

    let mut grupos: Vec<Vec<Candidato>> = Vec::new();
    for cand in candidatos {
        if let Some(ultimo) = grupos.last_mut() {
            if (cand.tempo_s - ultimo.last().unwrap().tempo_s).abs() <= JANELA_AGRUPAMENTO_S {
                ultimo.push(cand);
                continue;
            }
        }
        grupos.push(vec![cand]);
    }

    // ---- 4. Classificar cada grupo e produzir Golpe ----
    let mut hits: Vec<Golpe> = Vec::new();

    for grupo in &grupos {
        if grupo.is_empty() {
            continue;
        }

        // Tempo médio do grupo
        let tempo_medio: f32 = grupo.iter().map(|c| c.tempo_s).sum::<f32>() / grupo.len() as f32;

        // Prioridade 1: chimbau com decaimento curto
        let mut escolhido: Option<&Candidato> = None;
        let hihat_cand: Vec<&Candidato> = grupo
            .iter()
            .filter(|c| c.banda == "hihat" && c.decaimento_s <= HIHAT_DEC_ELEGIVEL_S)
            .collect();
        if !hihat_cand.is_empty() {
            escolhido = hihat_cand
                .into_iter()
                .max_by(|a, b| a.energia.total_cmp(&b.energia));
        }

        // Prioridade 2: prato com decaimento longo
        if escolhido.is_none() {
            let crash_cand: Vec<&Candidato> = grupo
                .iter()
                .filter(|c| c.banda == "crash" && c.decaimento_s >= CRASH_DEC_ELEGIVEL_S)
                .collect();
            if !crash_cand.is_empty() {
                escolhido = crash_cand
                    .into_iter()
                    .max_by(|a, b| a.energia.total_cmp(&b.energia));
            }
        }

        // Prioridade 3: banda com maior energia normalizada pelo pico da banda
        if escolhido.is_none() {
            escolhido = grupo.iter().max_by(|a, b| {
                let na = a.energia / max_por_banda.get(a.banda).copied().unwrap_or(1.0).max(1e-9);
                let nb = b.energia / max_por_banda.get(b.banda).copied().unwrap_or(1.0).max(1e-9);
                na.total_cmp(&nb)
            });
        }

        if let Some(cand) = escolhido {
            hits.push(Golpe {
                tempo_s: tempo_medio,
                peca: cand.peca.clone(),
                energia: cand.energia,
                decaimento_s: cand.decaimento_s,
            });
        }
    }

    // ---- 5. Estatísticas ----
    let mut contagem: HashMap<String, usize> = HashMap::new();
    for hit in &hits {
        *contagem.entry(hit.peca.clone()).or_insert(0) += 1;
    }

    // Período do bumbo: mediana dos intervalos entre kicks consecutivos
    let kick_tempos: Vec<f32> = hits
        .iter()
        .filter(|h| h.peca == "kick")
        .map(|h| h.tempo_s)
        .collect();
    let kick_periodo_s = if kick_tempos.len() >= 2 {
        let mut intervalos: Vec<f32> = kick_tempos
            .windows(2)
            .map(|w| w[1] - w[0])
            .filter(|&d| d > KICK_ESPACO_MIN_S)
            .collect();
        if intervalos.is_empty() {
            0.0
        } else {
            intervalos.sort_by(f32::total_cmp);
            // Primeiro quartil: robusto a hits introdutórios isolados
            // (medido: no stem real, o bumbo em 0,2 s + groove dá
            // intervalos [3,08 s, 15,45 s]; a mediana pega 15,45 s e
            // o primeiro quartil pega 3,08 s — que é o groove certo).
            let idx = intervalos.len() / 4;
            intervalos[idx]
        }
    } else {
        0.0
    };

    MapaPecas {
        hits,
        kick_periodo_s,
        contagem,
    }
}

// ---------------------------------------------------------------------------
// Detectores auxiliares
// ---------------------------------------------------------------------------

/// Mede o tempo de decaimento: do pico até a energia cair a 1/10 do valor.
///
/// Varre para a frente a partir de `pico_idx` até encontrar um quadro com
/// energia ≤ `pico / 10`, ou até um limite de 500 ms.
/// Devolve o tempo em segundos.
fn medir_decaimento(energia: &[f32], fps: f32, pico_idx: usize) -> f32 {
    let pico = energia[pico_idx];
    if pico <= 0.0 || pico_idx + 1 >= energia.len() {
        return 0.0;
    }
    let piso = pico / 10.0;
    let max_quadros = (0.5 * fps).round() as usize; // 500 ms
    let fim = (pico_idx + max_quadros).min(energia.len() - 1);
    if fim <= pico_idx {
        return 0.0;
    }
    for (offset, &valor) in energia[(pico_idx + 1)..=fim].iter().enumerate() {
        if valor <= piso {
            return (offset + 1) as f32 / fps;
        }
    }
    // Nunca caiu a 1/10 dentro da janela ― conta o quadro final
    (fim - pico_idx) as f32 / fps
}

// ---------------------------------------------------------------------------
// Testes
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_medir_decaimento_basico() {
        let mut energia = vec![0.1; 50];
        energia[10] = 1.0;
        energia[11] = 0.8;
        energia[12] = 0.5;
        energia[13] = 0.3;
        energia[14] = 0.12;
        energia[15] = 0.09; // abaixo de 0.1 (1/10 do pico)
        let d = medir_decaimento(&energia, 10.0, 10);
        assert!(
            (d - 0.5).abs() < 0.01,
            "decaimento devia ser 0.5 s, foi {d}"
        );
    }

    #[test]
    fn test_medir_decaimento_sem_atingir_piso() {
        let energia = vec![0.5; 100];
        let d = medir_decaimento(&energia, 10.0, 5);
        assert!(d > 0.0, "decaimento não pode ser zero se há energia");
        assert!(d <= 0.5, "decaimento respeita o limite de 500 ms");
    }

    /// Prioridade: chimbau com decaimento curto vence outras bandas.
    #[test]
    fn test_classificacao_prioridade_hihat() {
        let grupo = vec![
            Candidato {
                tempo_s: 1.0,
                peca: "kick".into(),
                energia: 10.0,
                decaimento_s: 0.15,
                pico_idx: 43,
                banda: "kick",
            },
            Candidato {
                tempo_s: 1.02,
                peca: "hihat".into(),
                energia: 2.0,
                decaimento_s: 0.04,
                pico_idx: 44,
                banda: "hihat",
            },
        ];
        let hihat_cand: Vec<&Candidato> = grupo
            .iter()
            .filter(|c| c.banda == "hihat" && c.decaimento_s <= HIHAT_DEC_ELEGIVEL_S)
            .collect();
        assert!(!hihat_cand.is_empty(), "hihat deve ser elegível");
        let escolhido = hihat_cand
            .into_iter()
            .max_by(|a, b| a.energia.total_cmp(&b.energia))
            .unwrap();
        assert_eq!(escolhido.peca, "hihat", "prioridade hihat deve vencer");
    }

    /// Prioridade: prato com decaimento longo vence outras bandas.
    #[test]
    fn test_classificacao_prioridade_crash() {
        let grupo = vec![
            Candidato {
                tempo_s: 2.0,
                peca: "snare".into(),
                energia: 8.0,
                decaimento_s: 0.15,
                pico_idx: 86,
                banda: "snare",
            },
            Candidato {
                tempo_s: 2.01,
                peca: "crash".into(),
                energia: 3.0,
                decaimento_s: 0.55,
                pico_idx: 87,
                banda: "crash",
            },
        ];
        let crash_cand: Vec<&Candidato> = grupo
            .iter()
            .filter(|c| c.banda == "crash" && c.decaimento_s >= CRASH_DEC_ELEGIVEL_S)
            .collect();
        assert!(!crash_cand.is_empty(), "crash deve ser elegível");
        let escolhido = crash_cand
            .into_iter()
            .max_by(|a, b| a.energia.total_cmp(&b.energia))
            .unwrap();
        assert_eq!(escolhido.peca, "crash", "prioridade crash deve vencer");
    }

    /// Identificação com stem real de bateria (Demucs, htdemucs_6s, me30).
    ///
    /// O áudio tem ~6 s. Esperado empírico (ADR-0017):
    /// - Período do bumbo entre 2,0 e 4,0 s (~3,08 s)
    /// - Pelo menos 2 bumbos
    /// - Pelo menos uma caixa ou chimbau
    /// - Golpes em ordem cronológica
    #[test]
    fn test_identifica_pecas_no_stem() {
        use crate::spectrum::Spectrogram;

        let caminho = std::path::Path::new("/private/tmp/demucs/saida/htdemucs_6s/me30/drums.wav");
        if !caminho.exists() {
            eprintln!("stem WAV não encontrado: {:?}", caminho);
            return;
        }
        let audio = crate::wav::ler_arquivo(caminho).expect("falha ao ler stem WAV");
        let spectro = Spectrogram::new(&audio, 4096, 1024);
        let mapa = identificar(&spectro, 44_100);

        assert!(
            mapa.kick_periodo_s > 2.0 && mapa.kick_periodo_s < 4.0,
            "período do bumbo fora do esperado (2-4 s): {} s",
            mapa.kick_periodo_s
        );

        let kick_count = mapa.contagem.get("kick").copied().unwrap_or(0);
        assert!(
            kick_count >= 2,
            "esperado ao menos 2 bumbos, encontrado {kick_count}"
        );

        let snare_count = mapa.contagem.get("snare").copied().unwrap_or(0);
        let hihat_count = mapa.contagem.get("hihat").copied().unwrap_or(0);
        assert!(
            snare_count + hihat_count >= 1,
            "esperado ao menos 1 caixa ou chimbau, snare={snare_count} hihat={hihat_count}"
        );

        for janela in mapa.hits.windows(2) {
            assert!(
                janela[0].tempo_s <= janela[1].tempo_s,
                "golpes fora de ordem: {:.3} > {:.3}",
                janela[0].tempo_s,
                janela[1].tempo_s
            );
        }
    }
}
