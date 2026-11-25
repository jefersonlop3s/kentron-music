//! Separação de stems via ONNX (Demucs 6-stem, htdemucs_6s).
//!
//! Carrega modelo ONNX do `StemSplitio/htdemucs-6s-onnx` (MIT), executa
//! inferência via `ort`. Aceita áudio bruto (`mix` tensor (1,2,343980)),
//! devolve 6 stems: drums, bass, other, vocals, guitar, piano.

use crate::wav::Audio;
use std::path::Path;

pub const TAXA_MODELO: u32 = 44_100;
pub const SEGMENTO_S: f32 = 7.8;
pub const N_AMOSTRAS: usize = (SEGMENTO_S * TAXA_MODELO as f32) as usize;
pub const N_CANAIS: usize = 2;
pub const N_STEMS: usize = 6;
pub const STEMS: [&str; N_STEMS] = ["drums", "bass", "other", "vocals", "guitar", "piano"];

fn criar_janela(overlap: usize) -> Vec<f32> {
    let n = N_AMOSTRAS;
    let mut w = vec![1.0f32; n];
    let fade = 1.0f32 / overlap as f32;
    for i in 0..overlap {
        w[i] = i as f32 * fade;
        w[n - 1 - i] = i as f32 * fade;
    }
    w
}

fn preparar_audio(audio: &Audio) -> Result<Vec<Vec<f32>>, String> {
    if audio.taxa != TAXA_MODELO {
        return Err(format!(
            "o modelo ONNX exige {} Hz, entrada tem {} Hz",
            TAXA_MODELO, audio.taxa
        ));
    }
    let n = audio.amostras.len() / audio.canais as usize;
    if audio.canais == 1 {
        Ok(vec![audio.amostras.clone(), audio.amostras.clone()])
    } else if audio.canais >= 2 {
        let ch0 = audio.amostras[..n].iter().copied().collect::<Vec<_>>();
        let ch1 = audio.amostras[n..(2 * n).min(audio.amostras.len())]
            .iter()
            .copied()
            .collect::<Vec<_>>();
        Ok(vec![ch0, ch1])
    } else {
        Err(format!("canais insuficientes: {}", audio.canais))
    }
}

#[must_use]
pub fn separar(audio: &Audio, model_path: &Path) -> Result<Vec<(String, Audio)>, String> {
    let canais = preparar_audio(audio)?;
    let total = canais[0].len();
    if total == 0 {
        return Err("áudio vazio".to_owned());
    }

    // Carrega o modelo ONNX
    let env = ort::init().build().map_err(|e| e.to_string())?;
    use ort::session::Session;
    let mut sess = Session::builder(&env)
        .map_err(|e| e.to_string())?
        .with_intra_threads(2)
        .map_err(|e| e.to_string())?
        .commit_from_file(model_path)
        .map_err(|e| e.to_string())?;

    let overlap = N_AMOSTRAS / 4;
    let stride = N_AMOSTRAS - overlap;
    let n_chunks = (total + stride - 1) / stride;
    let janela = criar_janela(overlap);

    let mut saidas = (0..N_STEMS)
        .map(|_| vec![0.0f32; total * N_CANAIS])
        .collect::<Vec<_>>();
    let mut pesos = vec![0.0f32; total * N_CANAIS];

    for chunk_idx in 0..n_chunks {
        let start = chunk_idx * stride;
        let end = (start + N_AMOSTRAS).min(total);
        let clen = end - start;

        let mut chunk = vec![0.0f32; N_AMOSTRAS * N_CANAIS];
        for i in 0..clen {
            chunk[i * N_CANAIS + 0] = canais[0][start + i];
            chunk[i * N_CANAIS + 1] = canais[1].get(start + i).copied().unwrap_or(0.0);
        }

        use ort::value::TensorRef;
        let entrada = TensorRef::from_array_view((
            vec![1i64, N_CANAIS as i64, N_AMOSTRAS as i64],
            chunk.as_slice(),
        ))
        .map_err(|e| e.to_string())?;

        let saida = sess.run(ort::inputs![entrada]).map_err(|e| e.to_string())?;

        let (dims, dados) = saida["stems"]
            .try_extract_tensor::<f32>()
            .map_err(|e| e.to_string())?;

        for s in 0..dims[1].min(N_STEMS as i64) as usize {
            for c in 0..dims[2].min(N_CANAIS as i64) as usize {
                let off = ((s * dims[2] as usize + c) * N_AMOSTRAS) as usize;
                for i in 0..clen {
                    let idx = (start + i) * N_CANAIS + c;
                    saidas[s][idx] += dados[off + i] * janela[i];
                    if s == 0 {
                        pesos[idx] += janela[i];
                    }
                }
            }
        }
    }

    for s in 0..N_STEMS {
        for i in 0..total * N_CANAIS {
            if pesos[i] > 1e-8f32 {
                saidas[s][i] /= pesos[i];
            }
        }
    }

    let resultado: Vec<(String, Audio)> = STEMS
        .iter()
        .enumerate()
        .map(|(s, nome)| {
            let mut meta = std::collections::HashMap::new();
            meta.insert("source".to_owned(), "stems_onnx".to_owned());
            (
                (*nome).to_owned(),
                Audio {
                    metadados: meta,
                    amostras: saidas[s].clone(),
                    taxa: TAXA_MODELO,
                    canais: N_CANAIS as u16,
                },
            )
        })
        .collect();
    Ok(resultado)
}

#[must_use]
pub fn caminho_padrao() -> std::path::PathBuf {
    std::path::Path::new("models")
        .join("weights")
        .join("htdemucs_6s")
        .join("htdemucs_6s.onnx")
        .to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn audio_sint(taxa: u32, dur: f32) -> Audio {
        let n = (taxa as f32 * dur) as usize;
        Audio {
            metadados: HashMap::new(),
            amostras: (0..n)
                .map(|i| {
                    0.3 * (2.0 * std::f32::consts::PI * 200.0 * (i as f32 / taxa as f32)).sin()
                })
                .collect::<Vec<_>>(),
            taxa,
            canais: 1,
        }
    }

    #[test]
    fn modelo_existe() {
        // O modelo ONNX é grande (130 MB) e não entra no repositório.
        // O teste avisa educadamente se não encontrar.
        let path = caminho_padrao();
        if !path.exists() {
            eprintln!("modelo ONNX não encontrado em {:?}", path.to_string_lossy());
            eprintln!("baixe com:");
            eprintln!("  curl -L -o models/weights/htdemucs_6s/htdemucs_6s.onnx \\");
            eprintln!(
                "    'https://huggingface.co/StemSplitio/htdemucs-6s-onnx/resolve/main/htdemucs_6s_fp16weights.onnx'"
            );
        }
    }

    #[test]
    fn taxa_errada_falha() {
        assert!(
            separar(
                &audio_sint(22_050, 1.0),
                std::path::Path::new("/tmp/x.onnx")
            )
            .is_err()
        );
    }

    #[test]
    fn modelo_ausente_falha() {
        assert!(
            separar(
                &audio_sint(44_100, 1.0),
                std::path::Path::new("/tmp/none.onnx")
            )
            .is_err()
        );
    }
}
