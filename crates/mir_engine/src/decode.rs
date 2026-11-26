//! Decodificação de arquivo de áudio (ADR-0016).
//!
//! WAV PCM fica com o leitor próprio (`wav`), que não tem dependência e serve o
//! caminho em memória; todo o resto vai pelo **Symphonia** (Rust puro,
//! MPL-2.0), que lê mp3/mp2, AAC-LC, ALAC, FLAC, Vorbis, Opus, ADPCM e WavPack
//! nos contêineres isomp4, mkv, ogg, riff e caf.
//!
//! O dispatch é por **conteúdo**, não por extensão: extensão mente, e um `.wav`
//! renomeado não pode virar erro de formato.
//!
//! O produto **não** embarca `ffmpeg` (ADR-0016). Formato fora da matriz vira
//! erro explícito com o motivo; converter é decisão do usuário, não do produto.

use std::collections::HashMap;
use std::io::Cursor;
use std::path::Path;

use symphonia::core::audio::{AudioBuffer, Signal};
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::default::{get_codecs, get_probe};

use crate::wav::{self, Audio};

/// Abre um arquivo de áudio: WAV PCM pelo leitor próprio, o resto pelo Symphonia.
///
/// # Errors
///
/// Arquivo ilegível, formato fora da matriz e decodificação quebrada devolvem
/// mensagem legível dizendo o que falhou.
pub fn abrir(caminho: &Path) -> Result<Audio, String> {
    let bytes = std::fs::read(caminho).map_err(|erro| format!("{caminho:?}: {erro}"))?;
    if eh_wav(&bytes) {
        return wav::ler(&bytes);
    }
    decodificar(&bytes, caminho.extension().and_then(|ext| ext.to_str()))
}

/// Um arquivo é WAV quando o cabeçalho diz isso — não quando o nome diz.
fn eh_wav(bytes: &[u8]) -> bool {
    bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WAVE"
}

/// Decodifica um buffer que não é WAV PCM, com o Symphonia.
///
/// A extensão entra só como **dica** para o sondador; quem decide é o conteúdo.
/// Áudio multicanal é misturado para mono, como o leitor de WAV faz — o resto do
/// motor assume mono.
///
/// # Errors
///
/// Formato não reconhecido, faixa sem taxa, códec não suportado e fluxo que não
/// produz amostra nenhuma devolvem erro com o motivo.
pub fn decodificar(bytes: &[u8], extensao: Option<&str>) -> Result<Audio, String> {
    let fonte = MediaSourceStream::new(Box::new(Cursor::new(bytes.to_vec())), Default::default());
    let mut dica = Hint::new();
    if let Some(ext) = extensao {
        dica.with_extension(ext);
    }
    let mut sondado = get_probe()
        .format(
            &dica,
            fonte,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|erro| format!("formato não reconhecido: {erro}"))?;
    // Extrai metadados das tags do arquivo (ID3/MP4).
    use symphonia::core::meta::Value as MetaValue;
    let mut metadados = HashMap::new();
    if let Some(meta_rev) = sondado.format.metadata().current() {
        for tag in meta_rev.tags() {
            if let MetaValue::String(ref v) = tag.value {
                if !v.is_empty() {
                    metadados.insert(tag.key.to_owned(), v.clone());
                }
            }
        }
    }

    let mut formato = sondado.format;
    let faixa = formato
        .default_track()
        .ok_or_else(|| "o arquivo não tem faixa de áudio".to_owned())?;
    let id = faixa.id;
    let parametros = faixa.codec_params.clone();
    let taxa = parametros
        .sample_rate
        .ok_or_else(|| "faixa sem taxa de amostragem".to_owned())?;
    let canais = parametros.channels.map_or(1, |c| c.count()).max(1);
    let mut decodificador = get_codecs()
        .make(&parametros, &DecoderOptions::default())
        .map_err(|erro| format!("códec não suportado: {erro}"))?;

    let mut amostras: Vec<f32> = Vec::new();
    loop {
        let pacote = match formato.next_packet() {
            Ok(pacote) => pacote,
            // Fim do fluxo é como o Symphonia reporta o término; qualquer outro
            // erro de I/O é erro de verdade e sobe.
            Err(SymphoniaError::IoError(erro))
                if erro.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break;
            }
            Err(erro) => return Err(format!("leitura do fluxo: {erro}")),
        };
        if pacote.track_id() != id {
            continue;
        }
        match decodificador.decode(&pacote) {
            Ok(decodificado) => {
                let especificacao = *decodificado.spec();
                let mut buffer =
                    AudioBuffer::<f32>::new(decodificado.capacity() as u64, especificacao);
                decodificado.convert(&mut buffer);
                let quantos = especificacao.channels.count().max(1);
                for quadro in 0..buffer.frames() {
                    let soma: f32 = (0..quantos).map(|canal| buffer.chan(canal)[quadro]).sum();
                    amostras.push(soma / quantos as f32);
                }
            }
            // Pacote corrompido no meio não derruba a análise inteira: pula.
            Err(SymphoniaError::DecodeError(_)) => continue,
            Err(erro) => return Err(format!("decodificação: {erro}")),
        }
    }
    if amostras.is_empty() {
        return Err("o arquivo não produziu amostra nenhuma".to_owned());
    }
    Ok(Audio {
        metadados,
        taxa,
        canais: u16::try_from(canais).unwrap_or(u16::MAX),
        amostras,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// WAV PCM 16 bits mono com uma senoide — o mínimo para o Symphonia ter o
    /// que decodificar. Gerado aqui: áudio de terceiro não entra em fixture.
    fn wav_de_senoide(amostras: usize) -> Vec<u8> {
        let taxa: u32 = 44_100;
        let dados: Vec<u8> = (0..amostras)
            .flat_map(|i| {
                let t = i as f32 / taxa as f32;
                let valor = 0.5 * (2.0 * std::f32::consts::PI * 440.0 * t).sin();
                ((valor * 32_768.0).round() as i16).to_le_bytes()
            })
            .collect();
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&(36 + dados.len() as u32).to_le_bytes());
        bytes.extend_from_slice(b"WAVEfmt ");
        bytes.extend_from_slice(&16u32.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes()); // PCM
        bytes.extend_from_slice(&1u16.to_le_bytes()); // mono
        bytes.extend_from_slice(&taxa.to_le_bytes());
        bytes.extend_from_slice(&(taxa * 2).to_le_bytes());
        bytes.extend_from_slice(&2u16.to_le_bytes());
        bytes.extend_from_slice(&16u16.to_le_bytes());
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&(dados.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&dados);
        bytes
    }

    #[test]
    fn symphonia_percorre_o_mesmo_caminho_do_leitor_proprio() {
        // O dispatch manda WAV para `wav::ler`; aqui o WAV vai **de propósito**
        // pelo Symphonia, para provar o encanamento — conversão para mono, taxa
        // e contagem de amostras — sem colocar arquivo comprimido em fixture.
        let bytes = wav_de_senoide(4_410);
        let proprio = wav::ler(&bytes).expect("leitor próprio");
        let symphonia = decodificar(&bytes, Some("wav")).expect("symphonia");
        assert_eq!(symphonia.taxa, proprio.taxa);
        assert_eq!(symphonia.canais, proprio.canais);
        assert_eq!(symphonia.amostras.len(), proprio.amostras.len());
        let maior_diferenca = proprio
            .amostras
            .iter()
            .zip(&symphonia.amostras)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f32, f32::max);
        assert!(
            maior_diferenca < 1e-3,
            "diferença máxima: {maior_diferenca}"
        );
    }

    #[test]
    fn buffer_que_nao_e_audio_falha_alto() {
        let erro = decodificar(&[0u8; 256], None).expect_err("lixo não é áudio");
        assert!(erro.contains("formato não reconhecido"), "{erro}");
    }

    #[test]
    fn wav_e_reconhecido_pelo_conteudo_e_nao_pelo_nome() {
        assert!(eh_wav(&wav_de_senoide(64)));
        assert!(!eh_wav(b"nao sou wav"));
    }
}
