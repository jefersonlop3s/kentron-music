//! A superfície que o Dart enxerga (ADR-0007).
//!
//! Aqui só há Rust puro: funções e tipos. O código de cola (`frb_generated.rs`
//! e o equivalente em Dart) é gerado por `flutter_rust_bridge_codegen` a partir
//! deste módulo — não se escreve FFI à mão.

use notation_bundle::chord_naming::{Chord, Note};
use notation_bundle::chordpro::{ChordProParser, Song};
use notation_bundle::harmony::HarmonyEngine;

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

/// Onde um acorde cai sobre a letra: a coluna e o acorde.
#[derive(Debug, Clone)]
pub struct ChordMark {
    /// Coluna da letra onde o acorde começa (0 = primeiro caractere).
    pub position: u32,
    /// O acorde em notação Common, como está escrito na cifra.
    pub chord: String,
}

/// Uma linha da cifra: a letra e os acordes sobre ela.
#[derive(Debug, Clone)]
pub struct SongLine {
    /// A letra, sem os acordes.
    pub text: String,
    /// Os acordes e suas colunas, na ordem em que aparecem.
    pub chords: Vec<ChordMark>,
}

/// Uma seção da cifra: estrofe, refrão, ponte.
#[derive(Debug, Clone)]
pub struct SongSection {
    /// O nome dado entre colchetes (`[Verse]`), quando existe.
    pub name: Option<String>,
    /// As linhas da seção.
    pub lines: Vec<SongLine>,
}

/// A cifra inteira, pronta para o Modo Palco desenhar.
#[derive(Debug, Clone)]
pub struct SongDetail {
    /// Título vindo de `{title: ...}`.
    pub title: String,
    /// Tom corrente; muda quando a cifra é transposta.
    pub key: Option<String>,
    /// Artista, quando declarado.
    pub artist: Option<String>,
    /// As seções, na ordem do arquivo.
    pub sections: Vec<SongSection>,
    /// Qual núcleo respondeu, no formato `kentron_ffi <versão>`.
    pub core: String,
}

/// Lê a cifra e devolve a estrutura completa, linha a linha.
///
/// É o que o Modo Palco precisa: o resumo de [`parse_song`] conta acordes, mas
/// não diz onde cada um cai sobre a letra.
///
/// # Errors
///
/// Propaga o que o parser devolver — o Dart recebe exceção, nunca estrutura
/// parcial.
pub fn read_song(input: String) -> Result<SongDetail, String> {
    let song = ChordProParser::parse(&input)?;
    Ok(detail(&song))
}

/// Transpõe a cifra para outro tom e devolve a estrutura completa.
///
/// O tom de origem vem do metadado `{key: ...}`. Sem ele não existe intervalo a
/// aplicar, e a função recusa em vez de adivinhar.
///
/// # Errors
///
/// Recusa cifra sem tom, tom inválido e acorde que o parser não entenda.
pub fn transpose_song(input: String, to_key: String) -> Result<SongDetail, String> {
    let song = ChordProParser::parse(&input)?;
    let from = song_key(&song)?;
    let to = key_note(&to_key)?;
    let transposed =
        HarmonyEngine::transpose(&song, from, to).map_err(|error| error.to_string())?;
    Ok(detail(&transposed))
}

/// Os doze tons que o Modo Palco oferece, na ordem em que a interface os mostra.
///
/// A grafia é convenção de tela: nas teclas pretas a interface escreve bemóis
/// (`Db`, `Eb`, `Ab`, `Bb`), que é como a cifra popular costuma escrever.
#[must_use]
pub fn key_choices() -> Vec<String> {
    [
        "C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B",
    ]
    .iter()
    .map(|key| (*key).to_owned())
    .collect()
}

fn detail(song: &Song) -> SongDetail {
    SongDetail {
        title: song.metadata.get("title").cloned().unwrap_or_default(),
        key: song.metadata.get("key").cloned(),
        artist: song.metadata.get("artist").cloned(),
        sections: song
            .sections
            .iter()
            .map(|section| SongSection {
                name: section.name.clone(),
                lines: section
                    .lines
                    .iter()
                    .map(|line| SongLine {
                        text: line.text.clone(),
                        chords: line
                            .chords
                            .iter()
                            .map(|placement| ChordMark {
                                position: u32::try_from(placement.position).unwrap_or(u32::MAX),
                                chord: placement.chord.clone(),
                            })
                            .collect(),
                    })
                    .collect(),
            })
            .collect(),
        core: format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
    }
}

fn song_key(song: &Song) -> Result<Note, String> {
    let key = song
        .metadata
        .get("key")
        .ok_or_else(|| "cifra sem tom: {key} ausente, não há intervalo a aplicar".to_owned())?;
    key_note(key)
}

fn key_note(key: &str) -> Result<Note, String> {
    key.trim()
        .parse::<Chord>()
        .map(|chord| chord.root)
        .map_err(|_| format!("tom inválido: {key}"))
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

    #[test]
    fn reads_the_structure_line_by_line() {
        let detail = read_song(
            "{title: Santo}\n{key: C}\n\n[Verse]\n[C]Santo, [G]santo\n[Am]santo\n".to_owned(),
        )
        .expect("fixture should parse");

        assert_eq!(detail.title, "Santo");
        assert_eq!(detail.key.as_deref(), Some("C"));
        assert_eq!(detail.sections.len(), 1);

        let verse = &detail.sections[0];
        assert_eq!(verse.name.as_deref(), Some("Verse"));
        assert_eq!(verse.lines.len(), 2);

        let first = &verse.lines[0];
        assert_eq!(first.text, "Santo, santo");
        let positions: Vec<(u32, &str)> = first
            .chords
            .iter()
            .map(|mark| (mark.position, mark.chord.as_str()))
            .collect();
        assert_eq!(positions, [(0, "C"), (7, "G")]);

        // A segunda linha começa com acorde na coluna zero.
        assert_eq!(verse.lines[1].text, "santo");
        assert_eq!(verse.lines[1].chords[0].position, 0);
        assert_eq!(verse.lines[1].chords[0].chord, "Am");
    }

    #[test]
    fn transposes_the_song_and_updates_the_key() {
        let detail = transpose_song(
            "{title: Santo}\n{key: C}\n\n[Verse]\n[C]Santo [G]santo\n".to_owned(),
            "D".to_owned(),
        )
        .expect("transpose should work");

        assert_eq!(detail.key.as_deref(), Some("D"));
        let chords: Vec<&str> = detail.sections[0].lines[0]
            .chords
            .iter()
            .map(|mark| mark.chord.as_str())
            .collect();
        assert_eq!(chords, ["D", "A"]);
    }

    #[test]
    fn refuses_to_transpose_without_a_key_instead_of_guessing() {
        let error = transpose_song("[C]sem tom\n".to_owned(), "D".to_owned())
            .expect_err("sem tom não há intervalo");
        assert!(error.contains("sem tom"), "{error}");
    }

    #[test]
    fn refuses_an_unknown_target_key() {
        let error = transpose_song("{key: C}\n[C]texto\n".to_owned(), "H".to_owned())
            .expect_err("H não é tom");
        assert!(error.contains("tom inválido"), "{error}");
    }

    #[test]
    fn offers_twelve_keys_with_flats_on_the_black_ones() {
        let keys = key_choices();
        assert_eq!(keys.len(), 12);
        assert_eq!(keys[0], "C");
        assert!(keys.contains(&"Bb".to_owned()));
        assert!(!keys.contains(&"A#".to_owned()), "a tela escreve bemóis");
    }
}
