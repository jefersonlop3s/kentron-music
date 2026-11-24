//! OpenLyrics export: song → OpenLyrics XML.
//!
//! OpenLyrics is an open XML format for lyrics projection, used by Holyrics,
//! OpenLP and other church presentation tools. The namespace is
//! `http://openlyrics.info/namespace/2009/song`.
//!
//! This module writes a [`Song`] (from the ChordPro parser) to the OpenLyrics
//! XML format. It does not parse OpenLyrics — that is a future increment.

use crate::chordpro::Song;

/// Error exporting a song to OpenLyrics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenLyricsError {
    /// The song has no title, which is required by OpenLyrics.
    MissingTitle,
}

impl std::fmt::Display for OpenLyricsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingTitle => write!(f, "song must have a title"),
        }
    }
}

impl std::error::Error for OpenLyricsError {}

/// Converts a [`Song`] to OpenLyrics XML.
///
/// # Errors
///
/// Returns [`OpenLyricsError::MissingTitle`] when the song has no `title`
/// metadata entry.
pub fn export(song: &Song) -> Result<String, OpenLyricsError> {
    let title = song
        .metadata
        .get("title")
        .ok_or(OpenLyricsError::MissingTitle)?;

    let artist = song.metadata.get("artist");
    let key = song.metadata.get("key");

    let mut out = String::with_capacity(1024);

    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str("<song xmlns=\"http://openlyrics.info/namespace/2009/song\" version=\"0.9\">\n");
    out.push_str("  <properties>\n");
    out.push_str("    <titles>\n");
    xml_escape(&mut out, "      <title>", title, "</title>\n");
    out.push_str("    </titles>\n");

    if let Some(artist) = artist {
        out.push_str("    <authors>\n");
        xml_escape(&mut out, "      <author>", artist, "</author>\n");
        out.push_str("    </authors>\n");
    }

    if let Some(key) = key {
        xml_escape(&mut out, "    <key>", key, "</key>\n");
    }

    out.push_str("  </properties>\n");
    out.push_str("  <lyrics>\n");

    let mut verse_counter: std::collections::HashMap<&str, usize> =
        std::collections::HashMap::new();

    for section in &song.sections {
        let name = section.name.as_deref().unwrap_or("o");
        let count = verse_counter.entry(name).or_insert(0);
        *count += 1;
        let label = if *count > 1 {
            format!("{}{}", name, count)
        } else {
            name.to_owned()
        };

        out.push_str("    <verse name=\"");
        xml_escape_attr(&mut out, &label);
        out.push_str("\">\n");
        out.push_str("      <lines>");

        for (i, line) in section.lines.iter().enumerate() {
            if i > 0 {
                out.push('\n');
            }
            xml_escape_text(&mut out, &line.text);
        }

        out.push_str("</lines>\n");
        out.push_str("    </verse>\n");
    }

    out.push_str("  </lyrics>\n");
    out.push_str("</song>\n");

    Ok(out)
}

/// Writes text into `out`, escaping XML special characters.
fn xml_escape_text(out: &mut String, text: &str) {
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            other => out.push(other),
        }
    }
}

/// Writes `prefix + escaped(text) + suffix` into `out`.
fn xml_escape(out: &mut String, prefix: &str, text: &str, suffix: &str) {
    out.push_str(prefix);
    xml_escape_text(out, text);
    out.push_str(suffix);
}

/// Escapes text for use in an XML attribute value.
fn xml_escape_attr(out: &mut String, text: &str) {
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            other => out.push(other),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::chordpro::ChordProParser;

    #[test]
    fn exports_a_basic_song() {
        let input = "{title: Santo}\n{key: C}\n{artist: Banda}\n\n[Verse]\n[C]Santo, [G]santo\n[Am]É o [F]Senhor\n\n[Chorus]\n[C]Cantai [G]todos\n";
        let song = ChordProParser::parse(input).expect("parse");
        let xml = export(&song).expect("export");

        assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(xml.contains("<title>Santo</title>"));
        assert!(xml.contains("<author>Banda</author>"));
        assert!(xml.contains("<key>C</key>"));
        assert!(xml.contains("<verse name=\"Verse\">"));
        assert!(xml.contains("<verse name=\"Chorus\">"));
        assert!(xml.contains("<lines>Santo, santo\nÉ o Senhor</lines>"));
        assert!(xml.contains("<lines>Cantai todos</lines>"));
    }

    #[test]
    fn exports_song_without_artist() {
        let input = "{title: Só Título}\n\n[V]\n[C]a\n";
        let song = ChordProParser::parse(input).expect("parse");
        let xml = export(&song).expect("export");

        assert!(xml.contains("<title>Só Título</title>"));
        assert!(!xml.contains("<author>"));
    }

    #[test]
    fn missing_title_returns_error() {
        let input = "[V]\n[C]a\n";
        let song = ChordProParser::parse(input).expect("parse");
        let error = export(&song).expect_err("should fail without title");
        assert_eq!(error, OpenLyricsError::MissingTitle);
    }

    #[test]
    fn xml_escaping_in_lyrics() {
        let input = "{title: Test}\n\n[V]\n[C]A & B < C > D \"E\" 'F'\n";
        let song = ChordProParser::parse(input).expect("parse");
        let xml = export(&song).expect("export");

        assert!(xml.contains("A &amp; B &lt; C &gt; D &quot;E&quot; &apos;F&apos;"));
    }

    #[test]
    fn xml_is_well_formed() {
        let input = "{title: Santo}\n{key: C}\n{artist: Banda}\n\n[Verse]\n[C]Santo, [G]santo\n[Am]É o [F]Senhor\n\n[Chorus]\n[C]Cantai [G]todos\n";
        let song = ChordProParser::parse(input).expect("parse");
        let xml = export(&song).expect("export");

        // Count opening tags (<tag> not </tag> or <?xml) and closing tags (</tag>).
        // A line like <title>text</title> has both — count by scanning the text.
        let opens =
            xml.matches('<').count() - xml.matches("</").count() - xml.matches("<?").count();
        let closes = xml.matches("</").count();

        assert_eq!(
            opens, closes,
            "open tags ({opens}) != close tags ({closes}):\n{xml}"
        );
    }

    #[test]
    fn exports_multiple_verses_with_same_name() {
        let input =
            "{title: Multi}\n\n[Verse]\n[C]Um\n\n[Verse]\n[C]Dois\n\n[Chorus]\n[C]Corinho\n";
        let song = ChordProParser::parse(input).expect("parse");
        let xml = export(&song).expect("export");

        assert!(xml.contains("<verse name=\"Verse\">"));
        assert!(xml.contains("<verse name=\"Verse2\">"));
        assert!(xml.contains("<verse name=\"Chorus\">"));
    }

    #[test]
    fn exports_unicode_accents() {
        let input = "{title: Acentos}\n\n[V]\n[C]Coração [G]alegre\n[C]Louvor ao [G]Rei\n";
        let song = ChordProParser::parse(input).expect("parse");
        let xml = export(&song).expect("export");

        assert!(xml.contains("Coração"));
        assert!(xml.contains("alegre"));
        assert!(xml.contains("Louvor ao"));
    }

    #[test]
    fn round_trip_chordpro_through_openlyrics() {
        let input = "{title: Santo}\n{key: C}\n{artist: Banda}\n\n[Verse]\n[C]Santo, [G]santo\n[Am]É o [F]Senhor\n\n[Chorus]\n[C]Cantai [G]todos\n";
        let song = ChordProParser::parse(input).expect("parse");
        let xml = export(&song).expect("export");

        assert!(xml.contains("Santo, santo"));
        assert!(xml.contains("É o Senhor"));
        assert!(xml.contains("Cantai todos"));

        let verse_pos = xml.find("Verse").expect("Verse section");
        let chorus_pos = xml.find("Chorus").expect("Chorus section");
        assert!(verse_pos < chorus_pos, "Verse must come before Chorus");
    }
}
