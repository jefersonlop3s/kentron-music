//! A superfície que o Dart enxerga (ADR-0007).
//!
//! Aqui só há Rust puro: funções e tipos. O código de cola (`frb_generated.rs`
//! e o equivalente em Dart) é gerado por `flutter_rust_bridge_codegen` a partir
//! deste módulo — não se escreve FFI à mão.

use notation_bundle::chordpro::ChordProParser;

/// O resumo de uma cifra: o que a tela precisa para listar e abrir a música.
#[derive(Debug, Clone)]
pub struct SongSummary {
    /// Título vindo de `{title: ...}`; vazio quando a cifra não declara um.
    pub title: String,
    /// Tom original (`{key: ...}`), quando declarado.
    pub key: Option<String>,
    /// Artista (`{artist: ...}`), quando declarado.
    pub artist: Option<String>,
    /// Quantidade de seções (estrofes, refrões, pontes).
    pub sections: u32,
    /// Acordes distintos, na ordem em que aparecem na cifra.
    pub chords: Vec<String>,
    /// Qual núcleo respondeu, no formato `notation_bundle <versão>`: a tela usa
    /// isto para provar que a resposta atravessou a ponte.
    pub core: String,
}

/// Lê uma cifra ChordPro e devolve o resumo que a interface usa.
///
/// # Errors
///
/// Recusa texto vazio (cifra vazia não é cifra) e propaga o que o parser
/// devolver — o Dart recebe exceção, nunca um resultado parcial.
pub fn parse_song(input: String) -> Result<SongSummary, String> {
    if input.trim().is_empty() {
        return Err("cifra vazia: nada para ler".to_owned());
    }

    let song = ChordProParser::parse(&input)?;

    let mut chords: Vec<String> = Vec::new();
    for section in &song.sections {
        for line in &section.lines {
            for placement in &line.chords {
                if !chords.contains(&placement.chord) {
                    chords.push(placement.chord.clone());
                }
            }
        }
    }

    Ok(SongSummary {
        title: song.metadata.get("title").cloned().unwrap_or_default(),
        key: song.metadata.get("key").cloned(),
        artist: song.metadata.get("artist").cloned(),
        sections: song.sections.len() as u32,
        chords,
        core: format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summarizes_a_chordpro_song() {
        let summary = parse_song(
            "{title: Santo}\n{key: C}\n{artist: Autor}\n\n[Verse]\n[C]Santo, [G]santo\n[Am]santo\n"
                .to_owned(),
        )
        .expect("fixture should parse");

        assert_eq!(summary.title, "Santo");
        assert_eq!(summary.key.as_deref(), Some("C"));
        assert_eq!(summary.artist.as_deref(), Some("Autor"));
        assert_eq!(summary.sections, 1);
        assert_eq!(summary.chords, ["C", "G", "Am"]);
        assert!(summary.core.starts_with("kentron_ffi "), "{}", summary.core);
    }

    #[test]
    fn reports_absence_of_metadata_without_inventing_it() {
        let summary = parse_song("[C]so letra\n".to_owned()).expect("fixture should parse");

        assert_eq!(summary.title, "");
        assert_eq!(summary.key, None);
        assert_eq!(summary.artist, None);
    }

    #[test]
    fn empty_input_is_refused_instead_of_returning_an_empty_summary() {
        // O parser, hoje, não falha para texto não-vazio — o `Result` da ponte
        // é um seam, e a única recusa viva é a cifra vazia. Prove no negativo.
        let error = parse_song("   \n".to_owned()).expect_err("empty input must fail");
        assert!(error.contains("vazia"), "{error}");

        // E texto que não é cifra passa como letra sem acorde, sem inventar erro.
        let summary = parse_song("apenas uma linha\n".to_owned()).expect("text should parse");
        assert_eq!(summary.title, "");
        assert!(summary.chords.is_empty());
    }
}
