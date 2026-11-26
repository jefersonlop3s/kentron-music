//! Carrega stems de bateria (WAV) e produz análise comparativa de percussão.
//!
//! O módulo espera um diretório opcional contendo `drums.wav` (e futuramente
//! outros stems). Quando presente, o espectro do stem de bateria é analisado
//! separadamente e comparado com o resultado do mix completo.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;

use crate::pecas;
use crate::percussao::{self, BumboCaixa};
use crate::spectrum::Spectrogram;

/// Caminho para o executável Python do venv Demucs.
const DEMUCS_VENV: &str = "/private/tmp/demucs/venv/bin/python3";
/// Caminho para os pesos do modelo htdemucs_6s (verificado na ADR-0017).
const DEMUCS_WEIGHTS: &str = "/private/tmp/demucs/htdemucs_6s.th";

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
/// * `audio_path` — caminho do arquivo de áudio original (para gerar stems via Demucs).
///
/// Quando `drums_path` é `None`, devolve apenas o resultado do mix.
/// Se `drums.wav` não existe no diretório, tenta gerar chamando Demucs.
///
/// # Errors
///
/// Falha de leitura do WAV ou de computação do espectrograma.
pub fn analisar_com_stems(
    spectro: &Spectrogram,
    taxa: u32,
    cortes: &[f32],
    drums_path: Option<&Path>,
    audio_path: Option<&Path>,
) -> Result<ComparativoStems, String> {
    let mix = percussao::medir(spectro, taxa, cortes);

    let (com_stems, pecas_map, caminho_stems) = if let Some(caminho) = drums_path {
        let drums_wav = caminho.join("drums.wav");
        // Se o drums.wav não existe, tenta gerar via Demucs
        if !drums_wav.exists() {
            if let Some(audio) = audio_path {
                if let Err(e) = separar_stems(audio, caminho) {
                    eprintln!("⚠ aviso: não foi possível separar stems ({e}) — análise sem stems");
                    return Ok(ComparativoStems {
                        mix,
                        com_stems: None,
                        pecas: None,
                        caminho_stems: Some(caminho.to_path_buf()),
                    });
                }
            } else {
                eprintln!("⚠ aviso: stems não encontrados e sem caminho do áudio original");
                return Ok(ComparativoStems {
                    mix,
                    com_stems: None,
                    pecas: None,
                    caminho_stems: Some(caminho.to_path_buf()),
                });
            }
        }
        let audio = crate::wav::ler_arquivo(&drums_wav)?;
        let spectro_stem = Spectrogram::new(&audio, 4096, 1024);
        let resultado = percussao::medir(&spectro_stem, taxa, cortes);
        let pecas_map = pecas::identificar(&spectro_stem, taxa);
        (
            Some(resultado),
            Some(pecas_map),
            Some(caminho.to_path_buf()),
        )
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

/// Chama o Demucs (Python) para separar stems de bateria do áudio de entrada.
///
/// Usa o venv em [`DEMUCS_VENV`] e o modelo em [`DEMUCS_WEIGHTS`]. Gera
/// `output_dir/htdemucs_6s/<nome>/drums.wav` (formato padrão do Demucs).
/// A função move o resultado para `output_dir/drums.wav`.
///
/// # Errors
///
/// Devolve erro quando o venv não existe, o Python falha ou os stems
/// não são gerados.
fn separar_stems(audio_path: &Path, output_dir: &Path) -> Result<(), String> {
    let python = Path::new(DEMUCS_VENV);
    if !python.exists() {
        return Err(format!(
            "Demucs não encontrado em {} — instale o venv ou copie os stems manualmente",
            python.display()
        ));
    }
    let pesos = Path::new(DEMUCS_WEIGHTS);
    if !pesos.exists() {
        return Err(format!(
            "Pesos do modelo não encontrados em {}",
            pesos.display()
        ));
    }

    // Garante que o diretório de saída existe
    std::fs::create_dir_all(output_dir).map_err(|e| format!("criar dir de stems: {e}"))?;

    // O Demucs salva em <output_dir>/htdemucs_6s/<stem_name>/drums.wav
    let stem_name = audio_path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "output".to_owned());

    eprintln!("🧠 separando bateria com Demucs (pode levar alguns segundos)...");

    let status = Command::new(python)
        .arg("-m")
        .arg("demucs")
        .arg("--two-stems=drums")
        .arg("-o")
        .arg(output_dir)
        .arg(audio_path)
        .status()
        .map_err(|e| format!("falha ao executar Demucs: {e}"))?;

    if !status.success() {
        let stderr_path = output_dir.join("demucs_stderr.log");
        return Err(format!(
            "Demucs falhou (exit={:?}). Log em: {}",
            status.code(),
            stderr_path.display()
        ));
    }

    // Move de <output_dir>/htdemucs_6s/<stem_name>/drums.wav → <output_dir>/drums.wav
    // A estrutura de diretórios varia: algumas versões do Demucs usam
    // `htdemucs` (sem `_6s`) e outras `htdemucs_6s`. Testamos ambas.
    let candidatos = [
        output_dir
            .join("htdemucs_6s")
            .join(&stem_name)
            .join("drums.wav"),
        output_dir
            .join("htdemucs")
            .join(&stem_name)
            .join("drums.wav"),
    ];
    let mut encontrado = None;
    for cand in &candidatos {
        if cand.exists() {
            encontrado = Some(cand.clone());
            break;
        }
    }
    if let Some(origem) = encontrado {
        let destino = output_dir.join("drums.wav");
        std::fs::rename(&origem, &destino)
            .map_err(|e| format!("falha ao mover {origem:?} para {destino:?}: {e}"))?;
        // Remove diretório temporário do Demucs
        let _ = std::fs::remove_dir_all(output_dir.join("htdemucs_6s"));
        let _ = std::fs::remove_dir_all(output_dir.join("htdemucs"));
        eprintln!("✅ bateria separada em {destino:?}");
        Ok(())
    } else {
        Err(format!(
            "Demucs não gerou nenhum dos candidatos: {candidatos:?}"
        ))
    }
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
        let resultado = analisar_com_stems(&spectro, 22_050, &[], None, None)
            .expect("análise sem stems falhou");

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
