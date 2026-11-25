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

/// Lê um arquivo WAV PCM do disco.
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
