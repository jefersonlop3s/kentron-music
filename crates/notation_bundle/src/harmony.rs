//! Song transposition and notation rendering.
//!
//! [`HarmonyEngine`] applies a key change to every chord of a [`Song`] and writes the
//! result in one of the four notation systems. Chords are stored as text in Common
//! notation, so the engine parses, transforms and re-serializes them.

use std::collections::HashMap;
use std::fmt;

use crate::chord_naming::{Chord, NotationSystem, Note, ParseError, RenderError};
use crate::chordpro::{ChordPlacement, Line, Section, Song};

/// Why a song could not be transposed or rendered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HarmonyError {
    /// A chord in the song is not valid Common notation.
    Parse(ParseError),
    /// A chord cannot be written in the requested system.
    Render(RenderError),
}

impl HarmonyError {
    /// Stable key the UI layer maps to a localized message.
    #[must_use]
    pub const fn message_key(&self) -> &'static str {
        match self {
            Self::Parse(error) => error.message_key(),
            Self::Render(error) => error.message_key(),
        }
    }
}

impl From<ParseError> for HarmonyError {
    fn from(error: ParseError) -> Self {
        Self::Parse(error)
    }
}

impl From<RenderError> for HarmonyError {
    fn from(error: RenderError) -> Self {
        Self::Render(error)
    }
}

impl fmt::Display for HarmonyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(error) => write!(f, "invalid chord: {error}"),
            Self::Render(error) => write!(f, "cannot render chord: {error}"),
        }
    }
}

impl std::error::Error for HarmonyError {}

/// Key-change and notation operations over a whole song.
pub struct HarmonyEngine;

impl HarmonyEngine {
    /// Transposes every chord by the interval from `from` to `to`.
    ///
    /// The `key` metadata entry, when present and parseable, is transposed as well.
    ///
    /// # Errors
    ///
    /// Returns [`HarmonyError::Parse`] when a chord in the song is not valid.
    pub fn transpose(song: &Song, from: Note, to: Note) -> Result<Song, HarmonyError> {
        let mut out = map_chords(song, |chord| {
            Ok(chord.parse::<Chord>()?.transpose(from, to).to_string())
        })?;
        out.metadata = transpose_metadata(&song.metadata, from, to);
        Ok(out)
    }

    /// Writes every chord in `system`.
    ///
    /// `key` is the tonic used by [`NotationSystem::Nashville`]; the other systems
    /// ignore it.
    ///
    /// # Errors
    ///
    /// Returns [`HarmonyError::Parse`] for an invalid chord and [`HarmonyError::Render`]
    /// when the system needs a key and none is given.
    pub fn render(
        song: &Song,
        system: NotationSystem,
        key: Option<Note>,
    ) -> Result<Song, HarmonyError> {
        map_chords(song, |chord| {
            Ok(chord.parse::<Chord>()?.render(system, key)?)
        })
    }

    /// Transposes and then renders, using `to` as the Nashville tonic.
    ///
    /// # Errors
    ///
    /// See [`Self::transpose`] and [`Self::render`].
    pub fn transpose_and_render(
        song: &Song,
        from: Note,
        to: Note,
        system: NotationSystem,
    ) -> Result<Song, HarmonyError> {
        let transposed = Self::transpose(song, from, to)?;
        Self::render(&transposed, system, Some(to))
    }
}

fn map_chords(
    song: &Song,
    transform: impl Fn(&str) -> Result<String, HarmonyError>,
) -> Result<Song, HarmonyError> {
    let mut sections = Vec::with_capacity(song.sections.len());
    for section in &song.sections {
        let mut lines = Vec::with_capacity(section.lines.len());
        for line in &section.lines {
            let mut chords = Vec::with_capacity(line.chords.len());
            for placement in &line.chords {
                chords.push(ChordPlacement {
                    position: placement.position,
                    chord: transform(&placement.chord)?,
                });
            }
            lines.push(Line {
                chords,
                text: line.text.clone(),
            });
        }
        sections.push(Section {
            name: section.name.clone(),
            lines,
        });
    }
    Ok(Song {
        metadata: song.metadata.clone(),
        sections,
    })
}

fn transpose_metadata(
    metadata: &HashMap<String, String>,
    from: Note,
    to: Note,
) -> HashMap<String, String> {
    metadata
        .iter()
        .map(|(key, value)| {
            let transposed = if key.eq_ignore_ascii_case("key") {
                value
                    .parse::<Chord>()
                    .ok()
                    .map(|chord| chord.transpose(from, to).to_string())
            } else {
                None
            };
            (key.clone(), transposed.unwrap_or_else(|| value.clone()))
        })
        .collect()
}
