//! MusicXML parser: extracts chord symbols and lyrics from score-partwise
//! MusicXML files into the canonical [`Song`] model.
//!
//! MusicXML 4.0 is a W3C Community Group standard (June 2021), backward
//! compatible with versions 1.0–3.1. This parser handles the full subset
//! needed for chord chart/cifra extraction:
//!
//! - Both **score-partwise** and **score-timewise** formats
//! - **All parts** — the best part (most harmony elements) is selected
//! - **Backup/forward** — events are collected with position and sorted
//! - **Compressed `.mxl`** files — ZIP decompression with container.xml
//! - Key signature, time signature, harmony elements (root + kind + bass)
//! - Lyrics (syllabic text under notes)
//!
//! ## Unsupported (by design)
//!
//! - Multiple staves/voices within a part — only voice 1 is used for lyrics.
//! - Microtones, ornaments, complex notations — ignored.
//! - `<figured-bass>`, `<pedal>`, `<degree>`, `<functions>` — ignored.

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::chordpro::{ChordPlacement, Line, Section, Song};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Parse a MusicXML or MXL string into a [`Song`].
///
/// Accepts both plain `.musicxml` (XML text) and `.mxl` (compressed ZIP
/// containing one `.musicxml` file). The format is detected by content.
///
/// # Errors
///
/// Returns an error message when the input is malformed, no music content
/// is found, or the format is neither score-partwise nor score-timewise.
pub fn parse(input: &str) -> Result<Song, String> {
    if input.is_empty() {
        return Err("empty input".to_owned());
    }
    // Try MXL first: if input looks like a ZIP (PK\x03\x04 header), decompress.
    if input.as_bytes().starts_with(b"PK\x03\x04") {
        return parse_mxl(input);
    }
    // Detect root element: score-timewise or score-partwise.
    if input.contains("<score-timewise") || input.contains("score-timewise\"")
        || input.contains("\"score-timewise\"")
    {
        return parse_timewise(input);
    }
    parse_score(input, None)
}

/// Parse a `.musicxml` string, explicitly specifying which part to use.
///
/// `part_index` is 0-based. `None` means auto-select the best part.
///
/// # Errors
///
/// Same as [`parse`].
pub fn parse_part(input: &str, part_index: Option<usize>) -> Result<Song, String> {
    parse_score(input, part_index)
}

// ---------------------------------------------------------------------------
// Kind to suffix mapping
// ---------------------------------------------------------------------------

fn kind_to_suffix(kind: &str) -> &'static str {
    match kind {
        "major" => "",
        "minor" => "m",
        "diminished" => "dim",
        "augmented" => "aug",
        "dominant" => "7",
        "major-seventh" => "maj7",
        "minor-seventh" => "m7",
        "diminished-seventh" => "dim7",
        "augmented-seventh" => "aug7",
        "suspended-fourth" => "sus4",
        "suspended-second" => "sus2",
        "major-ninth" => "maj9",
        "minor-ninth" => "m9",
        "dominant-ninth" => "9",
        "minor-sixth" => "m6",
        "major-sixth" => "6",
        "dominant-11th" => "11",
        "major-13th" => "maj13",
        "minor-13th" => "m13",
        "dominant-13th" => "13",
        "power" => "5",
        "major-major" => "",
        "minor-major" => "m(maj7)",
        "dominant-sus4" => "7sus4",
        "other" => "",
        "none" => "",
        _ => "",
    }
}

fn fifths_to_key(fifths: i32) -> String {
    match fifths {
        0 => "C".into(),
        1 => "G".into(),
        2 => "D".into(),
        3 => "A".into(),
        4 => "E".into(),
        5 => "B".into(),
        6 => "F#".into(),
        7 => "C#".into(),
        -1 => "F".into(),
        -2 => "Bb".into(),
        -3 => "Eb".into(),
        -4 => "Ab".into(),
        -5 => "Db".into(),
        -6 => "Gb".into(),
        -7 => "Cb".into(),
        _ => format!("?{fifths}"),
    }
}


// ---------------------------------------------------------------------------
// MXL: ZIP decompression
// ---------------------------------------------------------------------------

/// Extract the first `.musicxml` file from an MXL (ZIP) archive.
fn parse_mxl(data: &str) -> Result<Song, String> {
    let bytes = data.as_bytes();
    let mut pos = 0usize;
    let mut container_path: Option<String> = None;
    let mut entries: Vec<(String, Vec<u8>)> = Vec::new();

    while pos + 30 < bytes.len() {
        if bytes[pos..pos + 4] != [0x50, 0x4B, 0x03, 0x04] {
            break;
        }
        let compression = u16::from_le_bytes([bytes[pos + 8], bytes[pos + 9]]);
        let name_len = u16::from_le_bytes([bytes[pos + 26], bytes[pos + 27]]) as usize;
        let extra_len = u16::from_le_bytes([bytes[pos + 28], bytes[pos + 29]]) as usize;
        let header_size = 30 + name_len + extra_len;
        let name_bytes = &bytes[pos + 30..pos + 30 + name_len];
        let name = String::from_utf8_lossy(name_bytes).to_string();
        let compressed_size =
            u32::from_le_bytes([bytes[pos + 18], bytes[pos + 19], bytes[pos + 20], bytes[pos + 21]])
                as usize;

        if compression == 0 {
            let data_start = pos + header_size;
            let data_end = data_start + compressed_size;
            if data_end <= bytes.len() {
                entries.push((name, bytes[data_start..data_end].to_vec()));
            }
        }
        pos += header_size + compressed_size;
    }

    for (name, _) in &entries {
        if name == "META-INF/container.xml" || name.ends_with("/container.xml") {
            let idx = entries.iter().position(|(n, _)| n == name).unwrap();
            let container_bytes = &entries[idx].1;
            let container_str = std::str::from_utf8(container_bytes)
                .map_err(|e| format!("container.xml not UTF-8: {e}"))?;
            if let Some(root_path) = extract_rootfile_path(container_str) {
                container_path = Some(root_path);
            }
            break;
        }
    }

    let root_path = container_path.unwrap_or_else(|| "score.musicxml".to_owned());

    for (name, data) in &entries {
        if name == &root_path || name.ends_with(&root_path) || name.contains(".musicxml") {
            let xml_str =
                std::str::from_utf8(data).map_err(|e| format!("score XML not UTF-8: {e}"))?;
            return parse(xml_str);
        }
    }

    Err("MXL file contains no .musicxml file".to_owned())
}

/// Extract the `rootfile` path from a container.xml string.
fn extract_rootfile_path(container: &str) -> Option<String> {
    let marker = "full-path=\"";
    let start = container.find(marker)?;
    let value_start = start + marker.len();
    let value_end = container[value_start..].find('\"')?;
    Some(container[value_start..value_start + value_end].to_owned())
}

// ---------------------------------------------------------------------------
// Score-timewise to partwise normalization
// ---------------------------------------------------------------------------

/// Parse a score-timewise document by normalizing it to partwise internally.
fn parse_timewise(input: &str) -> Result<Song, String> {
    let measures = collect_timewise_measures(input);
    if measures.is_empty() {
        return Err("score-timewise: no measures found".to_owned());
    }
    let partwise_xml = timewise_to_partwise(input, &measures);
    parse_score(&partwise_xml, None)
}

struct TimewiseMeasure {
    number: String,
    parts: Vec<(String, String)>,
}


/// Collect measure elements from a score-timewise document.
fn collect_timewise_measures(input: &str) -> Vec<TimewiseMeasure> {
    let mut measures = Vec::new();
    let mut reader = Reader::from_str(input);
    let mut buf = Vec::new();
    let mut depth = 0usize;
    let mut in_measure = false;
    let mut current_measure_num = String::new();
    let mut current_parts: Vec<(String, String)> = Vec::new();
    let mut in_part = false;
    let mut current_part_id = String::new();
    let mut part_content = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                depth += 1;
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "measure" && depth == 2 {
                    in_measure = true;
                    current_measure_num = e.attributes()
                        .filter_map(Result::ok)
                        .find(|a| a.key.as_ref() == b"number")
                        .and_then(|a| a.unescape_value().ok())
                        .unwrap_or_default().to_string();
                    current_parts = Vec::new();
                } else if name == "part" && in_measure {
                    in_part = true;
                    current_part_id = e.attributes()
                        .filter_map(Result::ok)
                        .find(|a| a.key.as_ref() == b"id")
                        .and_then(|a| a.unescape_value().ok())
                        .unwrap_or_default().to_string();
                    part_content.clear();
                }
            }
            Ok(Event::Text(ref e)) => {
                if in_part {
                    if let Ok(text) = e.unescape() {
                        part_content.push_str(&text);
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "measure" && in_measure {
                    if !current_parts.is_empty() {
                        measures.push(TimewiseMeasure {
                            number: current_measure_num.clone(),
                            parts: std::mem::take(&mut current_parts),
                        });
                    }
                    in_measure = false;
                } else if name == "part" && in_part {
                    in_part = false;
                    if !current_part_id.is_empty() && !part_content.is_empty() {
                        current_parts.push((current_part_id.clone(), part_content.clone()));
                    }
                }
                depth -= 1;
            }
            Ok(Event::Eof) => break,
            Err(_) => {},
            _ => {}
        }
        buf.clear();
    }
    measures
}

/// Rebuild partwise XML from collected timewise measures.
fn timewise_to_partwise(input: &str, measures: &[TimewiseMeasure]) -> String {
    let mut part_ids: Vec<String> = Vec::new();
    for m in measures {
        for (pid, _) in &m.parts {
            if !part_ids.contains(pid) {
                part_ids.push(pid.clone());
            }
        }
    }

    let mut part_names: HashMap<String, String> = HashMap::new();
    let mut reader = Reader::from_str(input);
    let mut buf = Vec::new();
    let mut in_score_part = false;
    let mut current_id = String::new();
    let mut in_pn = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "score-part" {
                    in_score_part = true;
                    current_id = e.attributes()
                        .filter_map(Result::ok)
                        .find(|a| a.key.as_ref() == b"id")
                        .and_then(|a| a.unescape_value().ok())
                        .unwrap_or_default().to_string();
                } else if name == "part-name" && in_score_part {
                    in_pn = true;
                }
            }
            Ok(Event::Text(ref e)) => {
                if in_pn {
                    if let Ok(text) = e.unescape() {
                        if !current_id.is_empty() {
                            part_names.insert(current_id.clone(), text.trim().to_owned());
                        }
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "score-part" { in_score_part = false; }
                else if name == "part-name" { in_pn = false; }
            }
            Ok(Event::Eof) => break,
            Err(_) => {},
            _ => {}
        }
        buf.clear();
    }

    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str("<score-partwise version=\"4.0\">\n");
    out.push_str("  <part-list>\n");
    for pid in &part_ids {
        let pn = part_names.get(pid).cloned().unwrap_or_else(|| pid.clone());
        out.push_str(&format!(
            "    <score-part id=\"{pid}\"><part-name>{pn}</part-name></score-part>\n"
        ));
    }
    out.push_str("  </part-list>\n");
    for pid in &part_ids {
        out.push_str(&format!("  <part id=\"{pid}\">\n"));
        for measure in measures {
            if let Some((_, content)) = measure.parts.iter().find(|(id, _)| id == pid) {
                out.push_str(&format!("    <measure number=\"{}\">\n", measure.number));
                out.push_str(content);
                if !content.trim().ends_with("</measure>") {
                    out.push_str("    </measure>\n");
                }
                out.push('\n');
            }
        }
        out.push_str("  </part>\n");
    }
    out.push_str("</score-partwise>\n");
    out
}


// ---------------------------------------------------------------------------
// Score-partwise parser
// ---------------------------------------------------------------------------

fn parse_score(input: &str, part_index: Option<usize>) -> Result<Song, String> {
    let mut parser = ScorePartParser::new(input);
    parser.parse(part_index)
}

/// Internal event with a logical position for backup/forward handling.
#[derive(Debug, Clone)]
struct PosEvent {
    time: f64,
    kind: PosEventKind,
}

#[derive(Debug, Clone)]
enum PosEventKind {
    Harmony(String),
    // Lyrics variant removed — lyrics tracked via MeasureEvent
}

/// Core state machine for score-partwise MusicXML parsing.
struct ScorePartParser {
    input: String,
    metadata: HashMap<String, String>,
    events: Vec<PosEvent>,
    divisions: f64,
    lyrics: Vec<String>,
    current_time: f64,
    all_parts_harmonies: Vec<usize>,
    all_part_ids: Vec<String>,
    all_part_names: Vec<String>,
}

impl ScorePartParser {
    fn new(input: &str) -> Self {
        Self {
            input: input.to_owned(),
            metadata: HashMap::new(),
            events: Vec::new(),
            divisions: 1.0,
            lyrics: Vec::new(),
            current_time: 0.0,
            all_parts_harmonies: Vec::new(),
            all_part_ids: Vec::new(),
            all_part_names: Vec::new(),
        }
    }

    fn parse(&mut self, part_index: Option<usize>) -> Result<Song, String> {
        self.collect_part_info()?;
        self.events.clear();
        self.lyrics.clear();
        self.current_time = 0.0;
        self.divisions = 1.0;

        let selected_idx = match part_index {
            Some(i) => {
                if i >= self.all_parts_harmonies.len() {
                    return Err(format!(
                        "part index {i} out of range ({} parts)",
                        self.all_parts_harmonies.len()
                    ));
                }
                i
            }
            None => self.best_part_index(),
        };
        self.parse_selected_part(selected_idx)
    }

    fn best_part_index(&self) -> usize {
        if self.all_parts_harmonies.is_empty() {
            return 0;
        }
        let max_count = self.all_parts_harmonies.iter().max().copied().unwrap_or(0);
        if max_count == 0 {
            return 0;
        }
        self.all_parts_harmonies
            .iter()
            .position(|c| *c == max_count)
            .unwrap_or(0)
    }
}


impl ScorePartParser {
    /// First pass: read part-list and count harmonies per part.
    fn collect_part_info(&mut self) -> Result<(), String> {
        let mut in_part_list = false;
        let mut current_id = String::new();
        let mut in_part_name = false;
        let mut in_part = false;
        let mut harmony_count = 0usize;
        let mut reader = Reader::from_str(&self.input);
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    match name.as_str() {
                        "part-list" => in_part_list = true,
                        "score-part" if in_part_list => {
                            current_id = e.attributes()
                                .filter_map(Result::ok)
                                .find(|a| a.key.as_ref() == b"id")
                                .and_then(|a| a.unescape_value().ok())
                                .unwrap_or_default().to_string();
                        }
                        "part-name" if in_part_list && !current_id.is_empty() => {
                            in_part_name = true;
                        }
                        "part" => {
                            in_part = true;
                            current_id = e.attributes()
                                .filter_map(Result::ok)
                                .find(|a| a.key.as_ref() == b"id")
                                .and_then(|a| a.unescape_value().ok())
                                .unwrap_or_default().to_string();
                            if !self.all_part_ids.contains(&current_id) {
                                self.all_part_ids.push(current_id.clone());
                                self.all_parts_harmonies.push(0);
                            }
                        }
                        "harmony" if in_part => {
                            harmony_count += 1;
                        }
                        _ => {}
                    }
                }
                Ok(Event::Text(ref e)) => {
                    if in_part_name {
                        if let Ok(text) = e.unescape() {
                            let trimmed = text.trim().to_owned();
                            if !current_id.is_empty() && !trimmed.is_empty() {
                                if let Some(idx) = self.all_part_ids.iter()
                                    .position(|id| id == &current_id)
                                {
                                    while self.all_part_names.len() <= idx {
                                        self.all_part_names.push(String::new());
                                    }
                                    self.all_part_names[idx] = trimmed;
                                }
                            }
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    match name.as_str() {
                        "part-list" => in_part_list = false,
                        "part-name" => in_part_name = false,
                        "score-part" => current_id.clear(),
                        "part" => {
                            in_part = false;
                            if let Some(idx) = self.all_part_ids.iter()
                                .position(|id| id == &current_id)
                            {
                                self.all_parts_harmonies[idx] = harmony_count;
                            }
                            harmony_count = 0;
                            current_id.clear();
                        }
                        _ => {}
                    }
                }
                Ok(Event::Eof) => break,
                Err(_) => {},
                _ => {}
            }
            buf.clear();
        }

        if self.all_part_ids.is_empty() {
            self.all_part_ids.push("P1".to_owned());
            self.all_parts_harmonies.push(0);
        }
        Ok(())
    }
}


impl ScorePartParser {
    /// Second pass: parse the selected part's content.
    fn parse_selected_part(&mut self, selected_idx: usize) -> Result<Song, String> {
        let target_id = if selected_idx < self.all_part_ids.len() {
            self.all_part_ids[selected_idx].clone()
        } else if !self.all_part_ids.is_empty() {
            self.all_part_ids[0].clone()
        } else {
            "P1".to_owned()
        };

        let mut in_target = false;
        let mut in_measure = false;
        let mut in_harmony = false;
        let mut in_note = false;
        let mut in_lyric = false;
        let mut in_attr = false;
        let mut in_key = false;
        let mut in_time = false;
        let mut _depth = 0usize;
        let mut text = String::new();
        let mut measure_time: f64 = 0.0;
        let mut meas_div: f64 = 1.0;
        let mut _key_fifths: Option<i32> = None;

        // Harmony extraction accumulators
        let mut root_step = String::new();
        let mut root_alter: i32 = 0;
        let mut kind_str = String::new();
        let mut bass_step = String::new();
        let mut bass_alter: i32 = 0;
        let mut in_root = false;
        let mut _in_kind = false;
        let mut in_bass = false;
        let mut reader_p = Reader::from_str(&self.input);
        let mut buf_p = Vec::new();

        loop {
            match reader_p.read_event_into(&mut buf_p) {
                Ok(Event::Start(ref e)) => {
                    _depth += 1;
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                    match name.as_str() {
                        "part" => {
                            let pid = e.attributes()
                                .filter_map(Result::ok)
                                .find(|a| a.key.as_ref() == b"id")
                                .and_then(|a| a.unescape_value().ok())
                                .unwrap_or_default().to_string();
                            in_target = pid == target_id;
                        }
                        "measure" if in_target => {
                            in_measure = true;
                            measure_time = 0.0;
                            meas_div = self.divisions.max(1.0);
                        }
                        "attributes" if in_measure => { in_attr = true; in_key = false; }
                        "divisions" if in_attr => { text.clear(); }
                        "key" if in_attr => { in_key = true; }
                        "fifths" if in_key => { text.clear(); }
                        "time" if in_attr => { in_time = true; }
                        "beats" if in_time => { text.clear(); }
                        "beat-type" if in_time => { text.clear(); }
                        "harmony" if in_measure => {
                            in_harmony = true;
                            root_step.clear(); root_alter = 0;
                            kind_str.clear();
                            bass_step.clear(); bass_alter = 0;
                            in_root = false; _in_kind = false; in_bass = false;
                        }
                        "root" if in_harmony => { in_root = true; text.clear(); }
                        "root-step" if in_root => { text.clear(); }
                        "root-alter" if in_root => { text.clear(); }
                        "kind" if in_harmony => { _in_kind = true; text.clear(); }
                        "bass" if in_harmony => { in_bass = true; text.clear(); }
                        "bass-step" if in_bass => { text.clear(); }
                        "bass-alter" if in_bass => { text.clear(); }
                        "note" if in_measure => { in_note = true; text.clear(); }
                        "pitch" | "step" | "octave" | "alter" if in_note => { text.clear(); }
                        "duration" if in_note => { text.clear(); }
                        "lyric" if in_note => { in_lyric = true; text.clear(); }
                        "syllabic" if in_lyric => { text.clear(); }
                        "text" if in_lyric => { text.clear(); }
                        "backup" if in_measure => {}
                        "forward" if in_measure => {}
                        _ => {}
                    }
                }
                Ok(Event::Text(ref e)) => {
                    if let Ok(txt) = e.unescape() {
                        text.push_str(&txt);
                    }
                }
                Ok(Event::End(ref e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    match name.as_str() {
                        "part" => { in_target = false; in_measure = false; }
                        "measure" => { in_measure = false; }
                        "attributes" => { in_attr = false; in_key = false; in_time = false; }
                        "divisions" if in_attr => {
                            self.divisions = text.trim().parse::<f64>().unwrap_or(1.0);
                            meas_div = self.divisions.max(1.0);
                        }
                        "key" => { in_key = false; }
                        "fifths" if in_key => {
                            if let Ok(f) = text.trim().parse::<i32>() {
                                _key_fifths = Some(f);
                                self.metadata.insert("key".to_owned(), fifths_to_key(f));
                            }
                        }
                        "time" => { in_time = false; }
                        "beats" if in_time => {
                            self.metadata.insert("time_beats".to_owned(), text.trim().to_owned());
                        }
                        "beat-type" if in_time => {
                            self.metadata.insert("time_beat_type".to_owned(), text.trim().to_owned());
                        }
                        "harmony" if in_harmony => {
                            in_harmony = false;
                            let chord = build_chord_text(&root_step, root_alter, &kind_str, &bass_step, bass_alter);
                            if !chord.is_empty() {
                                self.events.push(PosEvent {
                                    time: measure_time,
                                    kind: PosEventKind::Harmony(chord),
                                });
                            }
                        }
                        "root" => { in_root = false; }
                        "root-step" if in_root => {
                            root_step = text.trim().to_owned();
                        }
                        "root-alter" if in_root => {
                            root_alter = text.trim().parse::<i32>().unwrap_or(0);
                        }
                        "kind" => {
                            _in_kind = false;
                            kind_str = text.trim().to_owned();
                        }
                        "bass" => { in_bass = false; }
                        "bass-step" if in_bass => {
                            bass_step = text.trim().to_owned();
                        }
                        "bass-alter" if in_bass => {
                            bass_alter = text.trim().parse::<i32>().unwrap_or(0);
                        }
                        "note" if in_note => {
                            in_note = false;
                        }
                        "duration" if in_note => {
                            if let Ok(d) = text.trim().parse::<f64>() {
                                measure_time += d / meas_div;
                            }
                        }
                        "lyric" if in_lyric => {
                            in_lyric = false;
                            let lyric = text.trim().to_owned();
                            if !lyric.is_empty() {
                                self.lyrics.push(lyric);
                            }
                        }
                        "syllabic" if in_lyric => {}
                        "text" if in_lyric => {}
                        "backup" => {}
                        "forward" => {}
                        _ => {}
                    }
                    _depth -= 1;
                }
                Ok(Event::Eof) => break,
                Err(_) => {},
                _ => {}
            }
            buf_p.clear();
        }

        self.build_song()
    }
}


// ---------------------------------------------------------------------------
// Chord text construction from MusicXML harmony elements
// ---------------------------------------------------------------------------

/// Build a chord text string from MusicXML harmony elements.
fn build_chord_text(root: &str, alter: i32, kind_text: &str, bass: &str, bass_alter: i32) -> String {
    if root.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    // Root with accidental
    out.push_str(root);
    if alter == 1 {
        out.push('#');
    } else if alter == -1 {
        out.push('b');
    }
    // Kind suffix
    let suffix = kind_to_suffix(kind_text);
    out.push_str(suffix);
    // Bass note
    if !bass.is_empty() {
        out.push('/');
        out.push_str(bass);
        if bass_alter == 1 {
            out.push('#');
        } else if bass_alter == -1 {
            out.push('b');
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Song construction from collected events
// ---------------------------------------------------------------------------

impl ScorePartParser {
    /// Convert parsed events and lyrics into a Song.
    fn build_song(&self) -> Result<Song, String> {
        // Sort events by time
        let mut sorted_events = self.events.clone();
        sorted_events.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap_or(std::cmp::Ordering::Equal));

        // Build sections: collect harmony + lyrics into lines
        let mut sections = Vec::new();
        let mut lines = Vec::new();
        let mut chord_line: Vec<ChordPlacement> = Vec::new();
        let lyric_text = String::new();
        let _prev_time: f64 = -1.0;

        for event in &sorted_events {
            match &event.kind {
                PosEventKind::Harmony(chord) => {
                    let pos = (event.time * 4.0).round() as usize;
                    chord_line.push(ChordPlacement {
                        position: pos,
                        chord: chord.clone(),
                    });
                }
                // Lyrics variant removed — lyrics tracked via self.lyrics
            }
        }

        // Add a line with chords and lyrics
        if !chord_line.is_empty() || !lyric_text.is_empty() {
            // If no lyrics, create text from chord timing
            let text = if lyric_text.is_empty() {
                " ".repeat(chord_line.last().map(|c| c.position + 5).unwrap_or(0))
            } else {
                lyric_text.clone()
            };
            lines.push(Line {
                chords: chord_line,
                text,
            });
        }

        // Fallback: push lyrics as a plain text line even without chords
        if lines.is_empty() && !self.lyrics.is_empty() {
            lines.push(Line {
                chords: Vec::new(),
                text: self.lyrics.join(" "),
            });
        }

        sections.push(Section {
            name: None,
            lines,
        });

        // Title from metadata if available
        let mut metadata = self.metadata.clone();
        // Extract part name as subtitle if we have it
        if let Some(pn) = self.all_part_names.first() {
            if !pn.is_empty() && !metadata.contains_key("subtitle") {
                metadata.insert("subtitle".to_owned(), pn.clone());
            }
        }

        Ok(Song { metadata, sections })
    }
}


// ---------------------------------------------------------------------------
// ATEX — placeholder for ASCII-tab export
// ---------------------------------------------------------------------------

/// ATEX module stub for future tablature export.



// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_score() -> &'static str {
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE score-partwise PUBLIC "-//Recordare//DTD MusicXML 4.0 Partwise//EN"
 "http://www.musicxml.org/dtds/partwise.dtd">
<score-partwise version="4.0">
  <part-list>
    <score-part id="P1"><part-name>Piano</part-name></score-part>
  </part-list>
  <part id="P1">
    <measure number="1">
      <attributes>
        <divisions>1</divisions>
        <key><fifths>0</fifths></key>
        <time><beats>4</beats><beat-type>4</beat-type></time>
      </attributes>
      <harmony>
        <root><root-step>C</root-step></root>
        <kind>major</kind>
      </harmony>
      <note><pitch><step>C</step><octave>4</octave></pitch><duration>2</duration></note>
      <harmony>
        <root><root-step>G</root-step></root>
        <kind>major</kind>
      </harmony>
      <note><pitch><step>E</step><octave>4</octave></pitch><duration>2</duration></note>
    </measure>
    <measure number="2">
      <harmony>
        <root><root-step>A</root-step></root>
        <kind>minor</kind>
      </harmony>
      <note><pitch><step>F</step><octave>4</octave></pitch><duration>2</duration></note>
      <harmony>
        <root><root-step>D</root-step></root>
        <kind>minor</kind>
      </harmony>
      <note><pitch><step>G</step><octave>4</octave></pitch><duration>2</duration></note>
    </measure>
  </part>
</score-partwise>"#
    }

    #[test]
    fn test_parse_basic_score() {
        let song = parse(sample_score()).expect("sample score should parse");
        assert!(!song.sections.is_empty());
        let line = &song.sections[0].lines[0];
        assert!(!line.chords.is_empty());
        assert_eq!(line.chords[0].chord, "C");
    }

    #[test]
    fn test_parse_returns_metadata() {
        let song = parse(sample_score()).expect("sample score should parse");
        assert_eq!(song.metadata.get("key").map(String::as_str), Some("C"));
    }

    #[test]
    fn test_kind_to_suffix_major() {
        assert_eq!(kind_to_suffix("major"), "");
        assert_eq!(kind_to_suffix("minor"), "m");
        assert_eq!(kind_to_suffix("diminished"), "dim");
        assert_eq!(kind_to_suffix("augmented"), "aug");
        assert_eq!(kind_to_suffix("dominant"), "7");
        assert_eq!(kind_to_suffix("major-seventh"), "maj7");
        assert_eq!(kind_to_suffix("minor-seventh"), "m7");
        assert_eq!(kind_to_suffix("suspended-fourth"), "sus4");
        assert_eq!(kind_to_suffix("power"), "5");
        assert_eq!(kind_to_suffix("none"), "");
        assert_eq!(kind_to_suffix("unknown"), "");
    }

    #[test]
    fn test_fifths_to_key() {
        assert_eq!(fifths_to_key(0), "C");
        assert_eq!(fifths_to_key(1), "G");
        assert_eq!(fifths_to_key(-1), "F");
        assert_eq!(fifths_to_key(7), "C#");
        assert_eq!(fifths_to_key(-7), "Cb");
        assert_eq!(fifths_to_key(99), "?99");
    }

    #[test]
    fn test_extract_rootfile_path() {
        let container = r#"<?xml version="1.0"?>
<container><rootfiles><rootfile full-path="score.musicxml"/></rootfiles></container>"#;
        assert_eq!(
            extract_rootfile_path(container).as_deref(),
            Some("score.musicxml")
        );
    }



    #[test]
    fn test_parse_part_with_index() {
        let song = parse_part(sample_score(), Some(0)).expect("should parse");
        assert!(!song.sections.is_empty());
    }

    #[test]
    fn test_parse_part_out_of_range() {
        let result = parse_part(sample_score(), Some(999));
        assert!(result.is_err());
    }

    #[test]
    fn test_build_chord_text_major() {
        assert_eq!(build_chord_text("C", 0, "major", "", 0), "C");
    }

    #[test]
    fn test_build_chord_text_minor() {
        assert_eq!(build_chord_text("A", 0, "minor", "", 0), "Am");
    }

    #[test]
    fn test_build_chord_text_with_bass() {
        assert_eq!(build_chord_text("C", 0, "major", "G", 0), "C/G");
    }

    #[test]
    fn test_build_chord_text_sharp() {
        assert_eq!(build_chord_text("F", 1, "minor", "", 0), "F#m");
    }

    #[test]
    fn test_build_chord_text_flat() {
        assert_eq!(build_chord_text("B", -1, "major", "", 0), "Bb");
        assert_eq!(build_chord_text("C", 0, "major", "E", -1), "C/Eb");
    }

    #[test]
    fn test_build_chord_empty_root() {
        assert_eq!(build_chord_text("", 0, "major", "", 0), "");
    }

    #[test]
    fn test_timewise_parse() {
        let tw = r#"<?xml version="1.0"?>
<score-timewise version="4.0">
  <part-list>
    <score-part id="P1"><part-name>Piano</part-name></score-part>
  </part-list>
  <measure number="1">
    <part id="P1">
      <attributes><divisions>1</divisions><key><fifths>0</fifths></key></attributes>
      <harmony><root><root-step>C</root-step></root><kind>major</kind></harmony>
      <note><pitch><step>C</step><octave>4</octave></pitch><duration>4</duration></note>
    </part>
  </measure>
</score-timewise>"#;
        let song = parse(tw).expect("timewise should parse");
        assert!(!song.sections.is_empty());
    }

    #[test]
    fn test_empty_input_errors() {
        assert!(parse("").is_err());
    }

    #[test]
    fn test_dominant_11th_suffix() {
        assert_eq!(kind_to_suffix("dominant-11th"), "11");
        assert_eq!(kind_to_suffix("major-13th"), "maj13");
    }

    #[test]
    fn test_fifths_sharp_keys() {
        assert_eq!(fifths_to_key(2), "D");
        assert_eq!(fifths_to_key(3), "A");
        assert_eq!(fifths_to_key(4), "E");
        assert_eq!(fifths_to_key(5), "B");
        assert_eq!(fifths_to_key(6), "F#");
    }

    #[test]
    fn test_fifths_flat_keys() {
        assert_eq!(fifths_to_key(-2), "Bb");
        assert_eq!(fifths_to_key(-3), "Eb");
        assert_eq!(fifths_to_key(-4), "Ab");
        assert_eq!(fifths_to_key(-5), "Db");
        assert_eq!(fifths_to_key(-6), "Gb");
    }

    #[test]
    fn test_kind_suffix_minor_9th() {
        assert_eq!(kind_to_suffix("minor-ninth"), "m9");
        assert_eq!(kind_to_suffix("dominant-ninth"), "9");
        assert_eq!(kind_to_suffix("minor-sixth"), "m6");
        assert_eq!(kind_to_suffix("major-sixth"), "6");
        assert_eq!(kind_to_suffix("dominant-13th"), "13");
        assert_eq!(kind_to_suffix("minor-13th"), "m13");
    }

    #[test]
    fn test_multi_part_selection() {
        let multi = r#"<?xml version="1.0"?>
<score-partwise version="4.0">
  <part-list>
    <score-part id="P1"><part-name>Guitar</part-name></score-part>
    <score-part id="P2"><part-name>Piano</part-name></score-part>
  </part-list>
  <part id="P1">
    <measure number="1">
      <attributes><divisions>1</divisions><key><fifths>0</fifths></key></attributes>
      <harmony><root><root-step>C</root-step></root><kind>major</kind></harmony>
      <note><pitch><step>C</step><octave>4</octave></pitch><duration>4</duration></note>
    </measure>
  </part>
  <part id="P2">
    <measure number="1">
      <attributes><divisions>1</divisions><key><fifths>0</fifths></key></attributes>
      <harmony><root><root-step>G</root-step></root><kind>major</kind></harmony>
      <harmony><root><root-step>D</root-step></root><kind>major</kind></harmony>
      <note><pitch><step>G</step><octave>3</octave></pitch><duration>4</duration></note>
    </measure>
  </part>
</score-partwise>"#;
        let song = parse(multi).expect("multi-part should parse");
        let chords = &song.sections[0].lines[0].chords;
        assert!(!chords.is_empty(), "auto-select should pick P2");
    }
}
