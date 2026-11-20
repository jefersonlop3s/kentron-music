//! ChordPro parser and model.
//!
//! This module implements the parsing of .cho files into a canonical
//! song representation.

use std::collections::HashMap;

/// A representation of a song in ChordPro format.
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
            } else if trimmed.starts_with("#") {
                continue;
            } else if trimmed.starts_with("[comment:") {
                continue;
            } else if trimmed.starts_with("[") && trimmed.ends_with("]") && Self::is_single_block(trimmed) && !Self::is_chord(trimmed) {
                if !current_section.lines.is_empty() {
                    sections.push(current_section);
                } else if current_section.name.is_some() {
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
        let mut chars = s.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '[' {
                if bracket_count != 0 { return false; }
                bracket_count += 1;
            } else if c == ']' {
                bracket_count -= 1;
                if bracket_count < 0 { return false; }
            } else if bracket_count == 0 {
                text_found = true;
            }
        }
        bracket_count == 0 && !text_found
    }

    fn is_chord(s: &str) -> bool {
        if s.len() > 12 { return false; }
        let content = &s[1..s.len() - 1];
        if content.is_empty() { return false; }
        if content.contains(' ') { return false; }
        
        let first_char = content.chars().next().unwrap();
        if ! "AGCEGBDFagcegbdf".contains(first_char) && first_char != '/' {
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
}
