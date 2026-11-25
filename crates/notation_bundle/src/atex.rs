//! Guitar Pro GPIF (AlphaTab/ATEX) parser.
//!
//! Parses the flat-list XML format used by Guitar Pro files (`<GPIF>`) into the
//! canonical [`Song`] model. The format stores all entities in lists referenced
//! by numeric IDs, which this module resolves into the expected hierarchy.
//!
//! ## Supported features
//!
//! - Song metadata: title, artist, key, time signature, tempo
//! - Standard guitar tuning from track properties
//! - Note events with fret/string/pitch -> chord detection
//! - Lyric text with ATEX syllable conventions (`-` joins, `+` = space)
//!
//! ## Limitations
//!
//! - Only track 0, voice 0 are processed (single-part lead sheet)
//! - Chord detection is template-based (major/minor/dim/aug/etc.)
//! - Rests and grace notes are ignored
//! - Section names are not encoded in GPIF; output uses a single unnamed section

use std::collections::HashMap;

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::chordpro::{ChordPlacement, Line, Section, Song};

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Parse a GPIF XML string into a [`Song`].
///
/// # Errors
///
/// Returns an error message when the XML is malformed or no `<GPIF>` root is
/// found.
pub fn parse(input: &str) -> Result<Song, String> {
    let mut reader = Reader::from_str(input);
    let mut buf = Vec::new();

    let mut title = String::new();
    let mut artist = String::new();
    let mut key_fifths: i32 = 0;
    let mut time_num: i32 = 4;
    let mut time_den: i32 = 4;
    let mut tempo: i32 = 120;
    let mut beat_texts: Vec<String> = Vec::new();
    let mut beat_note_pitches: Vec<Vec<i32>> = Vec::new();
    let mut master_bar_count: usize = 0;
    let mut in_gpif = false;

    #[derive(Clone, Copy)]
    enum Mode {
        Idle,
        InScore, InTitle, InArtist,
        InMasterTrack, InMasterBars, InMasterBar,
        InTracks, InTrack, InTrackStaffProps,
        InBars, InBar, InVoice, InBeat, InText,
        InNotes, InNote, InNoteProperties, InRhythms,
    }
    use Mode::*;

    let mut mode = Idle;
    // current_beat_idx tracks which Vec index we write beat text into
    let mut current_beat_idx: Option<usize> = None;
    let mut current_prop_child: Option<Vec<u8>> = None;
    let mut current_child: Option<Vec<u8>> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = e.name().as_ref().to_vec();

                if !in_gpif {
                    if name == b"GPIF" {
                        in_gpif = true;
                    }
                    continue;
                }

                match mode {
                    Idle => match name.as_slice() {
                        b"Score" => mode = InScore,
                        b"MasterTrack" => mode = InMasterTrack,
                        b"MasterBars" => mode = InMasterBars,
                        b"Tracks" => mode = InTracks,
                        b"Bars" => mode = InBars,
                        b"Rhythms" => mode = InRhythms,
                        _ => {}
                    },

                    InScore => match name.as_slice() {
                        b"Title" => mode = InTitle,
                        b"Artist" | b"Music" if artist.is_empty() => mode = InArtist,
                        _ => {}
                    },

                    InMasterTrack | InRhythms => {
                        current_child = Some(name.clone());
                    }

                    InMasterBars => {
                        if name == b"MasterBar" {
                            master_bar_count += 1;
                            mode = InMasterBar;
                        }
                    }

                    InMasterBar => {
                        current_child = Some(name.clone());
                    }

                    InTracks => {
                        if name == b"Track" { mode = InTrack; }
                    }

                    InTrack => {
                        if name == b"Properties" { mode = InTrackStaffProps; }
                    }

                    InTrackStaffProps => {}

                    InBars => {
                        if name == b"Bar" { mode = InBar; }
                    }

                    InBar => {
                        if name == b"Voice" { mode = InVoice; }
                    }

                    InVoice => {
                        if name == b"Beat" {
                            mode = InBeat;
                            let idx = beat_texts.len();
                            beat_texts.push(String::new());
                            beat_note_pitches.push(Vec::new());
                            current_beat_idx = Some(idx);
                        }
                    }

                    InBeat => match name.as_slice() {
                        b"Text" => mode = InText,
                        b"Notes" => mode = InNotes,
                        _ => {}
                    },

                    InNotes => {
                        if name == b"Note" { mode = InNote; }
                    }

                    InNote => {
                        if name == b"Properties" { mode = InNoteProperties; }
                    }

                    InNoteProperties => {
                        if name == b"Pitch" || name == b"String" || name == b"Fret" {
                            current_prop_child = Some(name.clone());
                        } else {
                            current_prop_child = None;
                        }
                    }
                    InText | InTitle | InArtist => {}
                }
            }

            Ok(Event::Empty(ref _e)) => {}

            Ok(Event::Text(ref e)) => {
                let text = e.unescape().unwrap_or_default().to_string();
                match mode {
                    InTitle => { title.push_str(&text); }
                    InArtist => { artist.push_str(&text); }
                    InMasterBar if master_bar_count == 1 => {
                        match current_child.as_deref() {
                            Some(b"KeyAccidentals") => {
                                key_fifths = text.trim().parse::<i32>().unwrap_or(0);
                            }
                            Some(b"TimeNumerator") => {
                                time_num = text.trim().parse::<i32>().unwrap_or(4);
                            }
                            Some(b"TimeDenominator") => {
                                time_den = text.trim().parse::<i32>().unwrap_or(4);
                            }
                            _ => {}
                        }
                    }
                    InMasterTrack => {
                        if current_child.as_deref() == Some(b"Value") {
                            if let Some(bpm_str) = text.split_whitespace().next() {
                                if let Ok(bpm) = bpm_str.parse::<i32>() {
                                    tempo = bpm;
                                }
                            }
                        }
                    }
                    InText => {
                        if let Some(idx) = current_beat_idx {
                            if let Some(bt) = beat_texts.get_mut(idx) {
                                bt.push_str(&text);
                            }
                        }
                    }
                    InNoteProperties if current_prop_child.as_deref() == Some(b"Pitch") => {
                        if let (Ok(p), Some(pitches)) =
                            (text.trim().parse::<i32>(), beat_note_pitches.last_mut())
                        {
                            pitches.push(p);
                        }
                    }
                    _ => {}
                }
            }

            Ok(Event::CData(ref e)) => {
                let text = String::from_utf8_lossy(e.as_ref()).to_string();
                match mode {
                    InTitle => { title.push_str(&text); }
                    InArtist => { artist.push_str(&text); }
                    InText if current_beat_idx.is_some() => {
                        let idx = current_beat_idx.unwrap();
                        if let Some(bt) = beat_texts.get_mut(idx) {
                            bt.push_str(&text);
                        }
                    }
                    InNoteProperties if current_prop_child.as_deref() == Some(b"Pitch") => {
                        if let (Ok(p), Some(pitches)) =
                            (text.trim().parse::<i32>(), beat_note_pitches.last_mut())
                        {
                            pitches.push(p);
                        }
                    }
                    _ => {}
                }
            }

            Ok(Event::End(ref e)) => {
                let name = e.name().as_ref().to_vec();

                match mode {
                    InScore if name == b"Score" => mode = Idle,
                    InTitle if name == b"Title" => mode = InScore,
                    InArtist if name == b"Artist" || name == b"Music" => mode = InScore,
                    InMasterTrack if name == b"MasterTrack" => mode = Idle,
                    InMasterBars if name == b"MasterBars" => mode = Idle,
                    InMasterBar if name == b"MasterBar" => mode = InMasterBars,
                    InTracks if name == b"Tracks" => mode = Idle,
                    InTrack if name == b"Track" => mode = InTracks,
                    InTrackStaffProps if name == b"Properties" => mode = InTrack,
                    InBars if name == b"Bars" => mode = Idle,
                    InBar if name == b"Bar" => mode = InBars,
                    InVoice if name == b"Voice" => mode = InBar,
                    InBeat if name == b"Beat" => {
                        mode = InVoice;
                        current_beat_idx = None;
                    }
                    InNotes if name == b"Notes" => mode = InBeat,
                    InNote if name == b"Note" => mode = InNotes,
                    InNoteProperties if name == b"Properties" => mode = InNote,
                    InText if name == b"Text" => mode = InBeat,
                    InRhythms if name == b"Rhythms" => mode = Idle,
                    _ => {}
                }
            }

            Ok(Event::Eof) => break,

            Err(e) => {
                return Err(format!(
                    "GPIF XML error at pos {}: {e}",
                    reader.buffer_position()
                ));
            }

            _ => {}
        }

        buf.clear();
    }
    // ── Post-processing: build the Song ──

    if !in_gpif {
        return Err("no <GPIF> root element found".to_owned());
    }

    let lyric_text = join_atex_lyrics(&beat_texts);

    let mut chords_per_beat: Vec<Option<String>> = Vec::new();
    for pitches in &beat_note_pitches {
        chords_per_beat.push(detect_chord_from_pitches(pitches));
    }

    let mut chord_placements: Vec<ChordPlacement> = Vec::new();
    if let Some(Some(first_chord)) = chords_per_beat.first() {
        chord_placements.push(ChordPlacement {
            position: 0,
            chord: first_chord.clone(),
        });
    }

    let line = Line {
        chords: chord_placements,
        text: lyric_text,
    };

    let section = Section {
        name: None,
        lines: vec![line],
    };

    let mut metadata = HashMap::new();
    if !title.is_empty() {
        metadata.insert("title".to_owned(), title);
    }
    if !artist.is_empty() {
        metadata.insert("artist".to_owned(), artist);
    }
    metadata.insert("key".to_owned(), fifths_to_key_name(key_fifths));
    metadata.insert("time".to_owned(), format!("{time_num}/{time_den}"));
    metadata.insert("tempo".to_owned(), tempo.to_string());

    Ok(Song {
        metadata,
        sections: vec![section],
    })
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Join ATEX lyric syllables into a continuous lyric string.
fn join_atex_lyrics(beat_texts: &[String]) -> String {
    let mut out = String::new();
    let mut pending_join = false; // true when previous syllable ended with `-` or `_`

    for text in beat_texts {
        // Remove truncation markers (`_`), replace spaces (`+`)
        let processed: String = text
            .chars()
            .filter(|&c| c != '_')
            .collect::<String>()
            .replace('+', " ");

        // Check if this text ends with a connector (word continuation)
        let ends_with_connector = text.ends_with('-') || text.ends_with('_');
        // Get the content without trailing connector for joining
        let content = if ends_with_connector {
            let trimmed = if text.ends_with('-') {
                processed.strip_suffix('-')
            } else {
                processed.strip_suffix('_')
            };
            trimmed.unwrap_or(&processed)
        } else {
            &processed
        };

        if pending_join {
            // No space between joined syllables
            let trimmed = content.trim_start();
            out.push_str(trimmed);
        } else {
            if !out.is_empty() {
                out.push(' ');
            }
            out.push_str(content);
        }

        pending_join = ends_with_connector;
    }

    out.trim().to_owned()
}

/// Convert fifths (KeyAccidentals) to a key name.
fn fifths_to_key_name(fifths: i32) -> String {
    const KEYS: &[&str] = &[
        "Cb", "Gb", "Db", "Ab", "Eb", "Bb", "F", "C", "G", "D", "A", "E",
        "B", "F#", "C#",
    ];
    let index = (fifths + 7) as usize;
    if index < KEYS.len() {
        KEYS[index].to_owned()
    } else {
        "C".to_owned()
    }
}

// ---------------------------------------------------------------------------
// Chord detection from MIDI pitch sets
// ---------------------------------------------------------------------------

const PITCH_CLASS_NAMES: &[&str] = &[
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

const CHORD_TEMPLATES: &[(&str, &[i32])] = &[
    ("", &[0, 4, 7]),
    ("m", &[0, 3, 7]),
    ("dim", &[0, 3, 6]),
    ("aug", &[0, 4, 8]),
    ("sus2", &[0, 2, 7]),
    ("sus4", &[0, 5, 7]),
    ("dim7", &[0, 3, 6, 9]),
    ("m7", &[0, 3, 7, 10]),
    ("7", &[0, 4, 7, 10]),
    ("maj7", &[0, 4, 7, 11]),
    ("m7b5", &[0, 3, 6, 10]),
    ("6", &[0, 4, 7, 9]),
    ("m6", &[0, 3, 7, 9]),
    ("add9", &[0, 4, 7, 2]),
    ("madd9", &[0, 3, 7, 2]),
];

fn detect_chord_from_pitches(pitches: &[i32]) -> Option<String> {
    if pitches.is_empty() {
        return None;
    }

    let mut classes: Vec<i32> = pitches
        .iter()
        .map(|p| p.rem_euclid(12))
        .collect();
    classes.sort();
    classes.dedup();

    if classes.is_empty() {
        return None;
    }

    if classes.len() == 1 {
        let root_name = PITCH_CLASS_NAMES[classes[0] as usize];
        return Some(root_name.to_owned());
    }

    let mut best_match: Option<(i32, &'static str, usize)> = None;

    for &root_pc in &classes {
        let intervals: Vec<i32> = classes
            .iter()
            .map(|&pc| (pc - root_pc).rem_euclid(12))
            .collect();

        for &(suffix, template) in CHORD_TEMPLATES {
            if !template.iter().all(|&t| intervals.contains(&t)) {
                continue;
            }

            let coverage = intervals
                .iter()
                .filter(|&&i| template.contains(&i))
                .count();

            let is_better = match best_match {
                Some((_, _, best_cov)) => coverage > best_cov,
                None => true,
            };

            if is_better {
                best_match = Some((root_pc, suffix, coverage));
            }
        }
    }

    best_match.map(|(root_pc, suffix, _)| {
        let root_name = PITCH_CLASS_NAMES[root_pc as usize];
        format!("{root_name}{suffix}")
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn gpif_body() -> &'static str {
        r#"<GPIF>
<Score><Title>Sample Song</Title><Artist>Test Artist</Artist><Words/><Music/></Score>
<MasterTrack><Tracks>0</Tracks><Automations><Automation><Type>Tempo</Type><Value>120 2</Value></Automation></Automations></MasterTrack>
<MasterBars>
<MasterBar id="0"><KeyAccidentals>0</KeyAccidentals><KeyMode>0</KeyMode><TimeNumerator>4</TimeNumerator><TimeDenominator>4</TimeDenominator></MasterBar>
<MasterBar id="1"><KeyAccidentals>0</KeyAccidentals><KeyMode>0</KeyMode><TimeNumerator>4</TimeNumerator><TimeDenominator>4</TimeDenominator></MasterBar>
</MasterBars>
<Tracks><Track id="0"><Name>Guitar</Name><Staves><Staff id="0"><Properties>
<Property name="Tuning"><Pitches>40 45 50 55 59 64</Pitches></Property>
<Property name="Frets"><Frets>-1 3 2 0 1 0</Frets></Property>
</Properties></Staff></Staves></Track></Tracks>
<Bars><Bar id="0"><Voices><Voice id="0"><Beats>
<Beat id="0"><Duration>0</Duration><Dynamic>f</Dynamic><Text>Hel-</Text><Notes>
<Note id="0"><Properties><Property name="String"><String>5</String></Property><Property name="Fret"><Fret>3</Fret></Property><Property name="Pitch"><Pitch>48</Pitch></Property></Properties></Note>
<Note id="1"><Properties><Property name="String"><String>4</String></Property><Property name="Fret"><Fret>2</Fret></Property><Property name="Pitch"><Pitch>52</Pitch></Property></Properties></Note>
<Note id="2"><Properties><Property name="String"><String>3</String></Property><Property name="Fret"><Fret>0</Fret></Property><Property name="Pitch"><Pitch>55</Pitch></Property></Properties></Note>
<Note id="3"><Properties><Property name="String"><String>2</String></Property><Property name="Fret"><Fret>1</Fret></Property><Property name="Pitch"><Pitch>60</Pitch></Property></Properties></Note>
<Note id="4"><Properties><Property name="String"><String>1</String></Property><Property name="Fret"><Fret>0</Fret></Property><Property name="Pitch"><Pitch>64</Pitch></Property></Properties></Note>
</Notes></Beat>
<Beat id="1"><Duration>0</Duration><Dynamic>f</Dynamic><Text>lo</Text><Notes>
<Note id="5"><Properties><Property name="String"><String>5</String></Property><Property name="Fret"><Fret>3</Fret></Property><Property name="Pitch"><Pitch>48</Pitch></Property></Properties></Note>
<Note id="6"><Properties><Property name="String"><String>4</String></Property><Property name="Fret"><Fret>2</Fret></Property><Property name="Pitch"><Pitch>52</Pitch></Property></Properties></Note>
<Note id="7"><Properties><Property name="String"><String>3</String></Property><Property name="Fret"><Fret>0</Fret></Property><Property name="Pitch"><Pitch>55</Pitch></Property></Properties></Note>
<Note id="8"><Properties><Property name="String"><String>2</String></Property><Property name="Fret"><Fret>1</Fret></Property><Property name="Pitch"><Pitch>60</Pitch></Property></Properties></Note>
<Note id="9"><Properties><Property name="String"><String>1</String></Property><Property name="Fret"><Fret>0</Fret></Property><Property name="Pitch"><Pitch>64</Pitch></Property></Properties></Note>
</Notes></Beat>
</Beats></Voice></Voices></Bar>
<Bar id="1"><Voices><Voice id="0"><Beats>
<Beat id="2"><Duration>0</Duration><Dynamic>f</Dynamic><Text>world</Text><Notes>
<Note id="10"><Properties><Property name="String"><String>5</String></Property><Property name="Fret"><Fret>3</Fret></Property><Property name="Pitch"><Pitch>48</Pitch></Property></Properties></Note>
<Note id="11"><Properties><Property name="String"><String>4</String></Property><Property name="Fret"><Fret>2</Fret></Property><Property name="Pitch"><Pitch>52</Pitch></Property></Properties></Note>
<Note id="12"><Properties><Property name="String"><String>3</String></Property><Property name="Fret"><Fret>0</Fret></Property><Property name="Pitch"><Pitch>55</Pitch></Property></Properties></Note>
<Note id="13"><Properties><Property name="String"><String>2</String></Property><Property name="Fret"><Fret>1</Fret></Property><Property name="Pitch"><Pitch>60</Pitch></Property></Properties></Note>
<Note id="14"><Properties><Property name="String"><String>1</String></Property><Property name="Fret"><Fret>0</Fret></Property><Property name="Pitch"><Pitch>64</Pitch></Property></Properties></Note>
</Notes></Beat>
</Beats></Voice></Voices></Bar></Bars>
<Rhythms><Rhythm id="0"><NoteValue>Quarter</NoteValue></Rhythm></Rhythms>
</GPIF>"#
    }

    #[test]
    fn atex_gpif_song() {
        let song = parse(gpif_body()).expect("GPIF should parse successfully");

        assert_eq!(song.metadata.get("title").map(String::as_str), Some("Sample Song"));
        assert_eq!(song.metadata.get("artist").map(String::as_str), Some("Test Artist"));
        assert_eq!(song.metadata.get("key").map(String::as_str), Some("C"));
        assert_eq!(song.metadata.get("time").map(String::as_str), Some("4/4"));
        assert_eq!(song.metadata.get("tempo").map(String::as_str), Some("120"));

        assert_eq!(song.sections.len(), 1, "one section");
        assert!(song.sections[0].name.is_none(), "unnamed section");
        assert_eq!(song.sections[0].lines.len(), 1, "one line");
        assert_eq!(song.sections[0].lines[0].text, "Hello world", "lyrics grouped");

        let chords = &song.sections[0].lines[0].chords;
        assert!(!chords.is_empty(), "chord detected");
        assert_eq!(chords[0].chord, "C");
        assert_eq!(chords[0].position, 0);
    }

    #[test]
    fn empty_input_returns_error() {
        let err = parse("").expect_err("empty input should fail");
        assert!(err.contains("GPIF"), "error mentions GPIF: {err}");
    }

    #[test]
    fn malformed_xml_returns_error() {
        // Unclosed attribute value causes XML parse error
        let err = parse(r#"<GPIF><Score><Title attr="unclosed>broken</GPIF>"#)
            .expect_err("malformed XML should fail");
        assert!(!err.is_empty(), "error message present");
    }

    #[test]
    fn no_gpif_root_returns_error() {
        let err = parse("<root></root>").expect_err("no GPIF root should fail");
        assert!(err.contains("GPIF"), "error mentions GPIF root: {err}");
    }

    #[test]
    fn detect_chord_from_major_triad() {
        let chord = detect_chord_from_pitches(&[48, 52, 55]);
        assert_eq!(chord, Some("C".to_owned()));
    }

    #[test]
    fn detect_chord_from_minor_triad() {
        let chord = detect_chord_from_pitches(&[45, 48, 52]);
        assert_eq!(chord, Some("Am".to_owned()));
    }

    #[test]
    fn detect_chord_from_dominant_seventh() {
        let chord = detect_chord_from_pitches(&[55, 59, 62, 65]);
        assert_eq!(chord, Some("G7".to_owned()));
    }

    #[test]
    fn detect_chord_single_note() {
        let chord = detect_chord_from_pitches(&[60]);
        assert_eq!(chord, Some("C".to_owned()));
    }

    #[test]
    fn detect_chord_empty_returns_none() {
        let chord = detect_chord_from_pitches(&[]);
        assert_eq!(chord, None);
    }

    #[test]
    fn join_atex_lyrics_with_hyphen() {
        let texts = vec!["Hel-".to_owned(), "lo".to_owned(), "world".to_owned()];
        let lyrics = join_atex_lyrics(&texts);
        assert_eq!(lyrics, "Hello world");
    }

    #[test]
    fn join_atex_lyrics_with_plus_as_space() {
        let texts = vec!["one+two".to_owned(), "three".to_owned()];
        let lyrics = join_atex_lyrics(&texts);
        assert_eq!(lyrics, "one two three");
    }

    #[test]
    fn join_atex_lyrics_with_underscore_truncated() {
        let texts = vec!["Hel_".to_owned(), "lo".to_owned()];
        let lyrics = join_atex_lyrics(&texts);
        assert_eq!(lyrics, "Hello");
    }

    #[test]
    fn join_atex_empty_input() {
        let lyrics = join_atex_lyrics(&[]);
        assert_eq!(lyrics, "");
    }

    #[test]
    fn fifths_to_key_name_various() {
        assert_eq!(fifths_to_key_name(0), "C");
        assert_eq!(fifths_to_key_name(1), "G");
        assert_eq!(fifths_to_key_name(-1), "F");
        assert_eq!(fifths_to_key_name(2), "D");
        assert_eq!(fifths_to_key_name(-2), "Bb");
        assert_eq!(fifths_to_key_name(7), "C#");
        assert_eq!(fifths_to_key_name(-7), "Cb");
    }
}
