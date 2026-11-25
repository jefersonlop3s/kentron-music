//! Carrega stems de bateria (WAV) e produz análise comparativa de percussão.
//!
//! O módulo espera um diretório opcional contendo `drums.wav` (e futuramente
//! outros stems). Quando presente, o espectro do stem de bateria é analisado
//! separadamente e comparado com o resultado do mix completo.

use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::pecas;
use crate::percussao::{self, BumboCaixa};
use crate::spectrum::Spectrogram;

/// Resultado comparativo entre a percussão do mix completo e do stem de bateria
/// isolado.
#[derive(Debug, Clone, Serialize)]
pub struct ComparativoStems {
    /// Análise de percussão no mix completo (comportamento atual).
    pub mix: BumboCaixa,
    /// Análise de percussão no stem de bateria apenas, quando disponível.
    pub com_stems: Option<BumboCaixa>,
    /// Mapa de peças individuais identificadas no stem de bateria.
    pub pecas: Option<pecas::MapaPecas>,
    /// O diretório de stems usado, se houver.
    pub caminho_stems: Option<PathBuf>,
}

/// Carrega stems de bateria de `drums_path` e produz a análise comparativa.
///
/// * `spectro` — espectrograma do mix completo (já calculado).
/// * `taxa` — taxa de amostragem do áudio original.
/// * `cortes` — fronteiras harmônicas (escala da barra).
/// * `drums_path` — diretório opcional contendo `drums.wav`.
///
/// Quando `drums_path` é `None`, devolve apenas o resultado do mix.
///
/// # Errors
///
/// Falha de leitura do WAV ou de computação do espectrograma.
pub fn analisar_com_stems(
    spectro: &Spectrogram,
    taxa: u32,
    cortes: &[f32],
    drums_path: Option<&Path>,
) -> Result<ComparativoStems, String> {
    let mix = percussao::medir(spectro, taxa, cortes);

    let (com_stems, pecas_map, caminho_stems) = if let Some(caminho) = drums_path {
        let drums_wav = caminho.join("drums.wav");
        let audio = crate::wav::ler_arquivo(&drums_wav)?;
        let spectro_stem = Spectrogram::new(&audio, 4096, 1024);
        let resultado = percussao::medir(&spectro_stem, taxa, cortes);
        let pecas_map = pecas::identificar(&spectro_stem, taxa);
        (Some(resultado), Some(pecas_map), Some(caminho.to_path_buf()))
    } else {
        (None, None, None)
    };

    Ok(ComparativoStems {
        mix,
        com_stems,
        pecas: pecas_map,
        caminho_stems,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    use crate::wav::Audio;

    /// Cria um áudio sintético simples (senoide) para teste.
    fn audio_sintetico(taxa: u32, duracao_s: f32) -> Audio {
        let n = (taxa as f32 * duracao_s) as usize;
        let amostras: Vec<f32> = (0..n)
            .map(|i| {
                let t = i as f32 / taxa as f32;
                0.3 * (2.0 * std::f32::consts::PI * 200.0 * t).sin()
            })
            .collect();
        Audio {
            metadados: HashMap::new(),
            amostras,
            taxa,
            canais: 1,
        }
    }

    #[test]
    fn sem_caminho_devolve_so_o_mix() {
        let audio = audio_sintetico(22_050, 4.0);
        let spectro = Spectrogram::new(&audio, 4096, 1024);
        let resultado =
            analisar_com_stems(&spectro, 22_050, &[], None).expect("análise sem stems falhou");

        assert!(
            resultado.com_stems.is_none(),
            "com_stems devia ser None sem caminho"
        );
        assert!(
            resultado.caminho_stems.is_none(),
            "caminho_stems devia ser None sem caminho"
        );
        // O mix deve ter sido computado (não é Default)
        assert!(
            resultado.mix.bumbos > 0 || resultado.mix.regra.contains("sem evidência"),
            "mix não foi computado"
        );
    }
}