//! ChordPro parser and model.
//!
//! This module implements the parsing of .cho files into a canonical
//! song representation.

use std::collections::HashMap;

/// A representation of a song in ChordPro format.
///
/// The model carries no serialization attributes on purpose: the persisted
/// content is the `.cho` text itself (ADR-0006), written by [`write`], and the
/// JSON files hold only identity. See `crate::bundle_store`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Song {
    pub metadata: HashMap<String, String>,
    pub sections: Vec<Section>,
}

/// A section of a song (e.g., Verse, Chorus, Bridge).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    pub name: Option<String>,
    pub lines: Vec<Line>,
}

/// A single line of a song, containing lyrics and chords.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    pub chords: Vec<ChordPlacement>,
    pub text: String,
}

/// The position and value of a chord within a line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChordPlacement {
    pub position: usize,
    pub chord: String,
}

pub struct ChordProParser;

impl ChordProParser {
    pub fn parse(input: &str) -> Result<Song, String> {
        let mut metadata = HashMap::new();
        let mut sections = Vec::new();
        let mut current_section = Section {
            name: None,
            lines: Vec::new(),
        };

        for line in input.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if trimmed.starts_with("{") && trimmed.ends_with("}") {
                let content = &trimmed[1..trimmed.len() - 1];
                if let Some((key, value)) = content.split_once(":") {
                    metadata.insert(key.trim().to_string(), value.trim().to_string());
                }
            } else if trimmed.starts_with('#') || trimmed.starts_with("[comment:") {
                continue;
            } else if trimmed.starts_with('[')
                && trimmed.ends_with(']')
                && Self::is_single_block(trimmed)
                && !Self::is_chord(trimmed)
            {
                if !current_section.lines.is_empty() || current_section.name.is_some() {
                    sections.push(current_section);
                }

                current_section = Section {
                    name: Some(trimmed[1..trimmed.len() - 1].to_string()),
                    lines: Vec::new(),
                };
            } else {
                let line_data = Self::parse_line(line);
                current_section.lines.push(line_data);
            }
        }

        if !current_section.lines.is_empty() || current_section.name.is_some() {
            sections.push(current_section);
        }

        Ok(Song { metadata, sections })
    }

    fn is_single_block(s: &str) -> bool {
        let mut bracket_count = 0;
        let mut text_found = false;

        for c in s.chars() {
            if c == '[' {
                if bracket_count != 0 {
                    return false;
                }
                bracket_count += 1;
            } else if c == ']' {
                bracket_count -= 1;
                if bracket_count < 0 {
                    return false;
                }
            } else if bracket_count == 0 {
                text_found = true;
            }
        }
        bracket_count == 0 && !text_found
    }

    fn is_chord(s: &str) -> bool {
        if s.len() > 12 {
            return false;
        }
        let content = &s[1..s.len() - 1];
        if content.is_empty() {
            return false;
        }
        if content.contains(' ') {
            return false;
        }

        let first_char = content.chars().next().unwrap();
        if !"AGCEGBDFagcegbdf".contains(first_char) && first_char != '/' {
            return false;
        }

        let allowed = "AGCEGBDFagcegbdf#b♭♯m majdimaugsu/791113+-";
        content.chars().all(|c| allowed.contains(c))
    }

    fn parse_line(input: &str) -> Line {
        let mut chords = Vec::new();
        let mut text = String::new();
        let mut current_pos = 0;
        let mut chars = input.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '[' {
                let mut chord_val = String::new();
                while let Some(&next) = chars.peek() {
                    if next == ']' {
                        chars.next();
                        break;
                    }
                    chord_val.push(chars.next().unwrap());
                }
                chords.push(ChordPlacement {
                    position: current_pos,
                    chord: chord_val,
                });
            } else {
                text.push(c);
                current_pos += 1;
            }
        }

        Line { chords, text }
    }
}

/// Serializes a song back to ChordPro text.
///
/// Metadata keys are written in sorted order so the output is deterministic —
/// `metadata` is a `HashMap` and iteration order is not stable. Sections are
/// separated by a blank line. `write` followed by [`ChordProParser::parse`]
/// returns an equal model: comments and blank lines are not part of the model,
/// so they are not part of the round-trip.
#[must_use]
pub fn write(song: &Song) -> String {
    let mut out = String::new();

    let mut keys: Vec<&String> = song.metadata.keys().collect();
    keys.sort();
    for key in keys {
        if let Some(value) = song.metadata.get(key) {
            out.push('{');
            out.push_str(key);
            out.push_str(": ");
            out.push_str(value);
            out.push_str("}\n");
        }
    }

    for section in &song.sections {
        out.push('\n');
        if let Some(name) = &section.name {
            out.push('[');
            out.push_str(name);
            out.push_str("]\n");
        }
        for line in &section.lines {
            out.push_str(&write_line(line));
            out.push('\n');
        }
    }

    out
}

/// Writes one line, reinserting each chord before the character it was
/// attached to. `ChordPlacement::position` counts characters of the lyric, as
/// the parser counted them.
fn write_line(line: &Line) -> String {
    let mut out = String::new();
    let mut chords: Vec<&ChordPlacement> = line.chords.iter().collect();
    chords.sort_by_key(|placement| placement.position);

    let mut next = 0;
    for (position, character) in line.text.chars().enumerate() {
        while next < chords.len() && chords[next].position <= position {
            push_chord(&mut out, &chords[next].chord);
            next += 1;
        }
        out.push(character);
    }
    while next < chords.len() {
        push_chord(&mut out, &chords[next].chord);
        next += 1;
    }

    out
}

fn push_chord(out: &mut String, chord: &str) {
    out.push('[');
    out.push_str(chord);
    out.push(']');
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parsing() {
        let input = "{title: Test Song}\n{artist: Artist Name}\n\n[C] Hello [G] world";
        let song = ChordProParser::parse(input).unwrap();

        assert_eq!(song.metadata.get("title").unwrap(), "Test Song");
        assert_eq!(song.metadata.get("artist").unwrap(), "Artist Name");
        assert_eq!(song.sections[0].lines[0].text, " Hello  world");
        assert_eq!(song.sections[0].lines[0].chords[0].chord, "C");
        assert_eq!(song.sections[0].lines[0].chords[0].position, 0);
        assert_eq!(song.sections[0].lines[0].chords[1].chord, "G");
        assert_eq!(song.sections[0].lines[0].chords[1].position, 7);
    }

    #[test]
    fn write_round_trips_through_the_parser() {
        let input = "{title: Santo}\n{key: C}\n\n[Verse]\n[C]Santo, [G]santo, [Am]santo\n[F]Deus do [C]universo\n\n[Chorus]\n[Dm]Toda a [G]terra te a[C]dora\n";
        let song = ChordProParser::parse(input).expect("fixture should parse");

        let written = write(&song);
        let reparsed = ChordProParser::parse(&written).expect("written text should parse");

        assert_eq!(reparsed, song, "round-trip must preserve the model");
    }

    #[test]
    fn o_compasso_escolhido_sobrevive_ao_round_trip() {
        // O motor sabe a barra mas não sempre o nome; quando não sabe, o músico
        // escolhe e registra `{time: ...}` na cifra. **A escolha tem de
        // sobreviver** ao parse e à escrita — senão o app pergunta de novo toda
        // vez, e a feature vira incômodo.
        let song = ChordProParser::parse("{title: T}\n{key: C}\n{time: 6/8}\n\n[C]x\n")
            .expect("fixture should parse");
        assert_eq!(song.metadata.get("time").map(String::as_str), Some("6/8"));

        let escrito = write(&song);
        let relido = ChordProParser::parse(&escrito).expect("a própria saída tem de reparsear");
        assert_eq!(
            relido.metadata.get("time").map(String::as_str),
            Some("6/8"),
            "a escolha do compasso se perdeu: {escrito:?}"
        );
    }

    #[test]
    fn write_is_deterministic_and_sorts_metadata() {
        let song = ChordProParser::parse("{title: T}\n{artist: A}\n{key: C}\n\n[C]x\n")
            .expect("fixture should parse");

        let first = write(&song);
        let second = write(&song);

        assert_eq!(first, second, "same model, same bytes");
        let artist = first.find("{artist:").expect("artist line");
        let key = first.find("{key:").expect("key line");
        let title = first.find("{title:").expect("title line");
        assert!(
            artist < key && key < title,
            "metadata must be sorted: {first:?}"
        );
    }

    #[test]
    fn write_keeps_chords_attached_to_their_character() {
        let song = ChordProParser::parse("[C]a[G]b\n").expect("fixture should parse");
        assert_eq!(write(&song), "\n[C]a[G]b\n");
    }

    #[test]
    fn write_puts_chords_past_the_end_after_the_lyric() {
        let song = Song {
            metadata: HashMap::new(),
            sections: vec![Section {
                name: None,
                lines: vec![Line {
                    chords: vec![ChordPlacement {
                        position: 99,
                        chord: "C".to_owned(),
                    }],
                    text: "abc".to_owned(),
                }],
            }],
        };

        assert_eq!(write(&song), "\nabc[C]\n");
    }
}
