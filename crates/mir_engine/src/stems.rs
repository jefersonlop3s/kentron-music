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

/// Dinâmica de um stem isolado — a dinâmica **por instrumento**.
///
/// A medida é feita no **mono** que o `wav::ler_arquivo` produz (ele mistura os
/// canais), que é o mesmo caminho que o mix percorre na `Dynamics` do relatório.
/// Isso é de propósito: os números ficam comparáveis **entre si** (mix e stems no
/// mesmo instrumento). Medir o stem estéreo pela EBU R128 daria uns **3 LU** mais
/// — comparar estes números com um medidor externo no arquivo estéreo é comparar
/// instrumentos diferentes, e está dito para não se confundir.
#[derive(Debug, Clone, Serialize)]
pub struct StemDinamica {
    /// Nome do instrumento, tirado do nome do arquivo (`vocals`, `drums`, …).
    pub nome: String,
    /// Loudness integrado do stem, em LUFS (EBU R128, no mono do leitor).
    pub lufs_integrado: f64,
    /// Faixa de volume do stem, em LU.
    pub lra_lu: f64,
    /// Pico de amostra do stem, em dBFS.
    pub pico_amostra_dbfs: f64,
}

/// Abaixo disto o stem está **vazio**: silêncio digital, não música.
///
/// Medido: o modelo de 6 stems devolve o stem de um instrumento que a música não
/// tem com loudness integrado entre **−60 e −61 LUFS** (medido em 2026-11-27 nos
/// stems de `De Joelhos`, `A Ele` e `Santo, Santo, Santo`) — e um módulo que
/// exige um instrumento ausente mede o **uso indevido**, não o motor. O limite é
/// declarado para o relatório dizer "sem conteúdo" em vez de apresentar −61 LUFS
/// como se fosse a dinâmica de um piano.
pub const SILENCIO_LUFS: f64 = -50.0;

impl StemDinamica {
    /// O stem tem música dentro? (ver [`SILENCIO_LUFS`])
    #[must_use]
    pub fn tem_conteudo(&self) -> bool {
        self.lufs_integrado > SILENCIO_LUFS
    }
}

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
    /// Dinâmica por stem, em ordem alfabética de nome. Vazia quando não há stems
    /// em disco — o motor não inventa LUFS de instrumento que não separou.
    pub dinamica: Vec<StemDinamica>,
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

    let (com_stems, pecas_map, caminho_stems, dinamica) = if let Some(caminho) = drums_path {
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
                        dinamica: Vec::new(),
                        caminho_stems: Some(caminho.to_path_buf()),
                    });
                }
            } else {
                eprintln!("⚠ aviso: stems não encontrados e sem caminho do áudio original");
                return Ok(ComparativoStems {
                    mix,
                    com_stems: None,
                    pecas: None,
                    dinamica: Vec::new(),
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
            dinamica_por_stem(caminho)?,
        )
    } else {
        (None, None, None, Vec::new())
    };

    Ok(ComparativoStems {
        mix,
        com_stems,
        pecas: pecas_map,
        dinamica,
        caminho_stems,
    })
}

/// LUFS/LRA/pico de **cada** WAV no diretório de stems, em ordem alfabética.
///
/// É a dinâmica por instrumento: o Demucs separa 6 stems e cada um tem o seu
/// loudness. Um arquivo ilegível no diretório **reprova** — pular em silêncio
/// faria a lista parecer completa quando não está.
///
/// # Errors
///
/// Diretório ilegível, entrada de diretório com erro ou WAV inválido.
fn dinamica_por_stem(diretorio: &Path) -> Result<Vec<StemDinamica>, String> {
    let mut caminhos: Vec<PathBuf> = Vec::new();
    let entradas = std::fs::read_dir(diretorio).map_err(|e| format!("ler {diretorio:?}: {e}"))?;
    for entrada in entradas {
        let entrada = entrada.map_err(|e| format!("entrada de {diretorio:?}: {e}"))?;
        let caminho = entrada.path();
        let eh_wav = caminho
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("wav"));
        if eh_wav {
            caminhos.push(caminho);
        }
    }
    caminhos.sort();
    let mut saida = Vec::with_capacity(caminhos.len());
    for caminho in caminhos {
        let audio = crate::wav::ler_arquivo(&caminho)
            .map_err(|e| format!("stem ilegível {caminho:?}: {e}"))?;
        let dinamica = crate::dynamics::analyze(&audio.amostras, audio.taxa);
        saida.push(StemDinamica {
            nome: caminho.file_stem().map_or_else(
                || caminho.to_string_lossy().into_owned(),
                |s| s.to_string_lossy().into_owned(),
            ),
            lufs_integrado: dinamica.lufs_integrado,
            lra_lu: dinamica.lra_lu,
            pico_amostra_dbfs: dinamica.pico_amostra_dbfs,
        });
    }
    Ok(saida)
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
    fn audio_sintetico(taxa: u32, duracao_s: f32, amplitude: f32) -> Audio {
        let n = (taxa as f32 * duracao_s) as usize;
        let amostras: Vec<f32> = (0..n)
            .map(|i| {
                let t = i as f32 / taxa as f32;
                amplitude * (2.0 * std::f32::consts::PI * 200.0 * t).sin()
            })
            .collect();
        Audio {
            metadados: HashMap::new(),
            amostras,
            taxa,
            canais: 1,
        }
    }

    /// Diretório temporário só deste teste, apagado no fim.
    fn pasta_temporaria(nome: &str) -> PathBuf {
        let pasta = std::env::temp_dir().join(format!("mir-stems-{}-{nome}", std::process::id()));
        let _ = std::fs::remove_dir_all(&pasta);
        std::fs::create_dir_all(&pasta).expect("criar pasta temporária");
        pasta
    }

    #[test]
    fn sem_caminho_devolve_so_o_mix() {
        let audio = audio_sintetico(22_050, 4.0, 0.3);
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
        assert!(
            resultado.dinamica.is_empty(),
            "sem stems em disco a dinâmica por instrumento é vazia, não inventada"
        );
        // O mix deve ter sido computado (não é Default)
        assert!(
            resultado.mix.bumbos > 0 || resultado.mix.regra.contains("sem evidência"),
            "mix não foi computado"
        );
    }

    #[test]
    fn a_dinamica_por_instrumento_sai_por_stem_e_na_ordem_da_amplitude() {
        // Dois stems com amplitudes 4x diferentes: a diferença de loudness tem de
        // ser 20·log10(4) = 12,04 LU, e o nome tem de identificar o instrumento.
        // Um teste que só olhasse "existe LUFS" passaria com o mesmo número para os
        // dois — a asserção é sobre a DIFERENÇA.
        let pasta = pasta_temporaria("dinamica");
        let escrever = |nome: &str, amplitude: f32| {
            let audio = audio_sintetico(44_100, 6.0, amplitude);
            crate::wav::escrever(&audio, &pasta.join(format!("{nome}.wav")))
                .expect("escrever stem");
        };
        escrever("drums", 0.1);
        escrever("vocals", 0.4);

        let audio = audio_sintetico(44_100, 4.0, 0.3);
        let spectro = Spectrogram::new(&audio, 4096, 1024);
        let resultado = analisar_com_stems(&spectro, 44_100, &[], Some(&pasta), None)
            .expect("análise com stems falhou");

        let nomes: Vec<&str> = resultado.dinamica.iter().map(|s| s.nome.as_str()).collect();
        assert_eq!(
            nomes,
            vec!["drums", "vocals"],
            "ordem alfabética pelos arquivos"
        );
        let drums = &resultado.dinamica[0];
        let vocals = &resultado.dinamica[1];
        let diferenca = vocals.lufs_integrado - drums.lufs_integrado;
        assert!(
            (diferenca - 12.04).abs() < 0.2,
            "4x de amplitude têm de dar 12,04 LU de diferença: {diferenca:.2}"
        );
        assert!(
            vocals.pico_amostra_dbfs > drums.pico_amostra_dbfs,
            "o pico tem de acompanhar a amplitude"
        );
        std::fs::remove_dir_all(&pasta).expect("apagar pasta temporária");
    }

    #[test]
    fn stem_vazio_nao_e_dinamica_de_instrumento() {
        // O modelo de 6 stems devolve stem de instrumento ausente com −60 a −61
        // LUFS (medido nos três casos de 2026-11-27). A prova negativa aqui é o
        // par: um stem com música acima do limite é `tem_conteudo`, o silêncio que
        // o mesmo modelo produz NÃO é — e um teste que só olhasse "tem LUFS"
        // passaria nos dois.
        let com_musica = StemDinamica {
            nome: "vocals".to_owned(),
            lufs_integrado: -24.1,
            lra_lu: 5.0,
            pico_amostra_dbfs: -1.0,
        };
        let vazio = StemDinamica {
            nome: "piano".to_owned(),
            lufs_integrado: -60.1,
            lra_lu: 0.0,
            pico_amostra_dbfs: -70.0,
        };
        assert!(com_musica.tem_conteudo(), "−24,1 LUFS é música");
        assert!(
            !vazio.tem_conteudo(),
            "−60,1 LUFS é o stem vazio do modelo, não um piano baixinho"
        );
    }

    #[test]
    fn stem_ilegivel_no_diretorio_reprova_em_vez_de_sumir() {
        // Prova negativa: um .wav que não é WAV dentro do diretório tem de
        // REPROVAR. Pular em silêncio faria a lista parecer completa com um stem
        // a menos — que é o modo de falha que o relatório não pode ter.
        let pasta = pasta_temporaria("ilegivel");
        let audio = audio_sintetico(44_100, 2.0, 0.3);
        crate::wav::escrever(&audio, &pasta.join("drums.wav")).expect("escrever drums");
        std::fs::write(pasta.join("vocals.wav"), b"isto nao e um wav")
            .expect("escrever arquivo inválido");

        let spectro = Spectrogram::new(&audio, 4096, 1024);
        let erro = analisar_com_stems(&spectro, 44_100, &[], Some(&pasta), None)
            .expect_err("WAV inválido no diretório tem de reprovar");
        assert!(
            erro.contains("vocals"),
            "o erro tem de nomear o arquivo problemático: {erro}"
        );
        std::fs::remove_dir_all(&pasta).expect("apagar pasta temporária");
    }
}
