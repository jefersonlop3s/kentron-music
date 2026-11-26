//! Leitor de WAV PCM — sem dependência externa.
//!
//! O `ffmpeg` converte qualquer formato para WAV antes da análise:
//!
//! ```text
//! ffmpeg -i entrada.m4a -ac 1 -ar 44100 -c:a pcm_s16le saida.wav
//! ```
//!
//! Então este leitor cobre só PCM: 8/16/24/32 bits inteiros e 32 bits float,
//! mono ou multicanal (mistura para mono somando e dividindo). Qualquer outra
//! coisa — WAV comprimido, cabeçalho torto, formato desconhecido — devolve erro
//! explícito, em vez de número silenciosamente errado.

use std::collections::HashMap;
use std::path::Path;

/// Áudio mono em ponto flutuante, com a taxa de amostragem.
#[derive(Debug, Clone)]
pub struct Audio {
    pub taxa: u32,
    pub canais: u16,
    /// Amostras mono, em `[-1.0, 1.0]`.
    pub amostras: Vec<f32>,
    /// Metadados extraídos do arquivo (tags ID3/MP4), quando disponíveis.
    /// Chaves comuns: "title", "artist", "album", "date".
    pub metadados: HashMap<String, String>,
}

impl Audio {
    /// Duração em segundos.
    #[must_use]
    pub fn duracao(&self) -> f32 {
        if self.taxa == 0 {
            0.0
        } else {
            self.amostras.len() as f32 / self.taxa as f32
        }
    }
}

/// Escreve um arquivo WAV PCM (16 bits, canais do Audio, taxa do Audio).
///
/// O `Audio` carrega as amostras intercaladas (L,R,L,R,...) quando `canais > 1`.
/// Usa formato PCM 16 bits — qualidade suficiente para stems de bateria e
/// aceito por toda ferramenta de áudio.
///
/// # Errors
///
/// Falha de escrita no disco.
pub fn escrever(audio: &Audio, caminho: &Path) -> Result<(), String> {
    let n_amostras = audio.amostras.len();
    let taxa = audio.taxa;
    let canais = audio.canais;
    // Os dois campos de tamanho do RIFF são **u32**. Escritos como `usize`
    // (u64 nesta plataforma) eles ocupavam 8 bytes onde o formato exige 4: o
    // "WAVE" ia para o byte 12, o cabeçalho saía com 52 bytes e **nenhum**
    // leitor aceitava o arquivo — nem o `ler` deste módulo, nem o ffprobe
    // (medido em 2026-11-26: "riff form is not wave"/"Invalid data found").
    // O `try_from` mantém o limite do formato explícito: acima de 4 GB o RIFF
    // não representa o arquivo, e isso é erro — não truncamento silencioso.
    let dados_len = u32::try_from(n_amostras.saturating_mul(2))
        .map_err(|_| format!("{n_amostras} amostras não cabem num RIFF (u32)"))?;
    let file_size = dados_len
        .checked_add(36)
        .ok_or_else(|| format!("{dados_len} bytes de dados não cabem num RIFF (u32)"))?;
    let mut buf = Vec::with_capacity(file_size as usize + 8);
    buf.extend_from_slice(b"RIFF");
    buf.extend_from_slice(&file_size.to_le_bytes());
    buf.extend_from_slice(b"WAVEfmt ");
    buf.extend_from_slice(&16u32.to_le_bytes()); // chunk size
    buf.extend_from_slice(&1u16.to_le_bytes()); // PCM
    buf.extend_from_slice(&canais.to_le_bytes());
    buf.extend_from_slice(&taxa.to_le_bytes());
    let byte_rate = taxa * canais as u32 * 2u32;
    buf.extend_from_slice(&byte_rate.to_le_bytes());
    let block_align = canais * 2u16;
    buf.extend_from_slice(&block_align.to_le_bytes());
    buf.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
    buf.extend_from_slice(b"data");
    buf.extend_from_slice(&dados_len.to_le_bytes());
    for s in &audio.amostras {
        let amostra = (s.clamp(-1.0, 1.0) * 32_768.0) as i16;
        buf.extend_from_slice(&amostra.to_le_bytes());
    }
    std::fs::write(caminho, &buf).map_err(|erro| format!("{caminho:?}: {erro}"))?;
    Ok(())
}
///
/// # Errors
///
/// Falha de leitura e qualquer formato fora de PCM devolvem mensagem legível.
pub fn ler_arquivo(caminho: &Path) -> Result<Audio, String> {
    let bytes = std::fs::read(caminho).map_err(|erro| format!("{caminho:?}: {erro}"))?;
    ler(&bytes)
}

/// Lê WAV PCM de um buffer.
///
/// # Errors
///
/// Cabeçalho inválido, formato não-PCM ou amostra truncada devolvem erro.
pub fn ler(bytes: &[u8]) -> Result<Audio, String> {
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err("não é RIFF/WAVE".to_owned());
    }
    let mut pos = 12;
    let mut fmt: Option<(u16, u16, u32, u16)> = None; // formato, canais, taxa, bits
    let mut dados: Option<&[u8]> = None;
    while pos + 8 <= bytes.len() {
        let id = &bytes[pos..pos + 4];
        let tamanho = u32::from_le_bytes(bytes[pos + 4..pos + 8].try_into().unwrap()) as usize;
        let inicio = pos + 8;
        let fim = (inicio + tamanho).min(bytes.len());
        let corpo = &bytes[inicio..fim];
        match id {
            b"fmt " if corpo.len() >= 16 => {
                let formato = u16::from_le_bytes(corpo[0..2].try_into().unwrap());
                let canais = u16::from_le_bytes(corpo[2..4].try_into().unwrap());
                let taxa = u32::from_le_bytes(corpo[4..8].try_into().unwrap());
                let bits = u16::from_le_bytes(corpo[14..16].try_into().unwrap());
                // WAVE_FORMAT_EXTENSIBLE (0xFFFE): o formato real vem no GUID,
                // nos dois primeiros bytes depois dos 16.
                let efetivo = if formato == 0xFFFE && corpo.len() >= 26 {
                    u16::from_le_bytes(corpo[24..26].try_into().unwrap())
                } else {
                    formato
                };
                fmt = Some((efetivo, canais, taxa, bits));
            }
            b"data" => dados = Some(corpo),
            _ => {}
        }
        pos = inicio + tamanho + (tamanho % 2);
    }
    let (formato, canais, taxa, bits) = fmt.ok_or_else(|| "bloco fmt ausente".to_owned())?;
    let dados = dados.ok_or_else(|| "bloco data ausente".to_owned())?;
    if canais == 0 || taxa == 0 {
        return Err("fmt com canais ou taxa zerados".to_owned());
    }
    let mono = decodificar(dados, formato, canais, bits)?;
    Ok(Audio {
        metadados: HashMap::new(),
        taxa,
        canais,
        amostras: mono,
    })
}

/// Decodifica PCM e mistura para mono.
fn decodificar(dados: &[u8], formato: u16, canais: u16, bits: u16) -> Result<Vec<f32>, String> {
    let bytes_por_amostra = match (formato, bits) {
        (1, 8) => 1,
        (1, 16) => 2,
        (1, 24) => 3,
        (1, 32) => 4,
        (3, 32) => 4,
        (f, b) => return Err(format!("PCM não suportado: formato {f}, {b} bits")),
    };
    let passo = bytes_por_amostra * usize::from(canais);
    if passo == 0 {
        return Err("passo zero".to_owned());
    }
    let quadros = dados.len() / passo;
    let mut mono = Vec::with_capacity(quadros);
    for q in 0..quadros {
        let base = q * passo;
        let mut soma = 0.0f32;
        for c in 0..usize::from(canais) {
            let i = base + c * bytes_por_amostra;
            let a = &dados[i..i + bytes_por_amostra];
            soma += match (formato, bits) {
                (1, 8) => (f32::from(a[0]) - 128.0) / 128.0,
                (1, 16) => f32::from(i16::from_le_bytes(a.try_into().unwrap())) / 32_768.0,
                (1, 24) => {
                    let bruto = i32::from_le_bytes([a[0], a[1], a[2], 0]);
                    // 24 bits vêm em complemento de dois: estende o sinal.
                    let valor = (bruto << 8) >> 8;
                    valor as f32 / 8_388_608.0
                }
                (1, 32) => i32::from_le_bytes(a.try_into().unwrap()) as f32 / 2_147_483_648.0,
                (3, 32) => f32::from_le_bytes(a.try_into().unwrap()),
                _ => 0.0,
            };
        }
        mono.push(soma / f32::from(canais));
    }
    Ok(mono)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn audio_de_teste(canais: u16, quadros: usize) -> Audio {
        Audio {
            metadados: HashMap::new(),
            taxa: 44_100,
            canais,
            amostras: (0..quadros * usize::from(canais))
                .map(|i| {
                    let t = i as f32 / 44_100.0;
                    (2.0 * std::f32::consts::PI * 440.0 * t).sin() * 0.5
                })
                .collect(),
        }
    }

    /// O gate que faltava: `escrever` não tinha **nenhum** teste e **um** único
    /// chamador (a escrita dos stems ONNX), que nunca havia rodado. O arquivo
    /// saía com o "WAVE" no byte 12 e o `ler` deste próprio módulo o recusava —
    /// e a falha era engolida por `.ok()` no chamador.
    #[test]
    fn escrever_produz_riff_valido_e_o_leitor_le_de_volta() {
        let original = audio_de_teste(2, 500);
        let caminho = std::env::temp_dir().join("kentron-wav-ida-e-volta.wav");
        escrever(&original, &caminho).expect("escrever");
        let bytes = std::fs::read(&caminho).expect("ler os bytes escritos");

        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..12], b"WAVE", "o \"WAVE\" não está no byte 8");
        assert_eq!(
            bytes.len(),
            44 + original.amostras.len() * 2,
            "44 bytes de cabeçalho + 2 por amostra"
        );
        assert_eq!(
            u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize,
            bytes.len() - 8,
            "o tamanho declarado no RIFF tem de fechar com o arquivo"
        );

        let lido = ler(&bytes).expect("ler o que o módulo escreveu");
        assert_eq!(lido.taxa, original.taxa);
        assert_eq!(lido.canais, original.canais);
        // `ler` mistura para mono: um quadro = média dos dois canais.
        assert_eq!(lido.amostras.len(), original.amostras.len() / 2);
        for (i, obtido) in lido.amostras.iter().enumerate() {
            let esperado = (original.amostras[i * 2] + original.amostras[i * 2 + 1]) / 2.0;
            assert!(
                (obtido - esperado).abs() < 1.0 / 32_768.0,
                "quadro {i}: {obtido} != {esperado}"
            );
        }
    }

    #[test]
    fn escrever_e_ler_em_mono() {
        let original = audio_de_teste(1, 100);
        let caminho = std::env::temp_dir().join("kentron-wav-mono.wav");
        escrever(&original, &caminho).expect("escrever");
        let lido = ler(&std::fs::read(&caminho).expect("bytes")).expect("ler");
        assert_eq!(lido.canais, 1);
        assert_eq!(lido.amostras.len(), 100);
    }
}
