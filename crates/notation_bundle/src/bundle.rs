//! Song bundle model: one song package, the local library and setlists.
//!
//! This module is the **in-memory model only**. Persistence — file layout, schema
//! version and migrations — is decided by ADR-0006, which is still proposed, so
//! nothing here reads or writes files. The invariant that survives either way is
//! referential integrity: a setlist never points at a song the library does not
//! hold, and a refusal is always explicit (never a silent skip).
//!
//! Title, artist and key are read from the ChordPro metadata, so the `.cho`
//! content stays the single source of truth for bibliographic data. Rust code
//! never returns user-facing prose: errors expose a stable `message_key()` that
//! the Flutter layer resolves through ARB files. The `Display` text is English
//! and meant for logs only.

use std::collections::BTreeMap;
use std::fmt;

use crate::chordpro::Song;

/// Stable identifier of a song inside the local library.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SongId(String);

impl SongId {
    /// Wraps an identifier, rejecting empty or whitespace-only values.
    pub fn new(value: impl Into<String>) -> Result<Self, BundleError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(BundleError::EmptyId);
        }
        Ok(Self(value))
    }

    /// Borrows the identifier as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SongId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Stable identifier of a setlist inside the local library.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SetlistId(String);

impl SetlistId {
    /// Wraps an identifier, rejecting empty or whitespace-only values.
    pub fn new(value: impl Into<String>) -> Result<Self, BundleError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(BundleError::EmptyId);
        }
        Ok(Self(value))
    }

    /// Borrows the identifier as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SetlistId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Why a bundle, library or setlist operation was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BundleError {
    /// An identifier is empty or whitespace-only.
    EmptyId,
    /// The song has no `title` in its ChordPro metadata.
    MissingTitle,
    /// The setlist name is empty or whitespace-only.
    EmptySetlistName,
    /// The library already holds a song with this identifier.
    DuplicateSong(SongId),
    /// The setlist already holds this song.
    DuplicateSetlistEntry(SongId),
    /// The setlist references a song the library does not hold.
    MissingSong(SongId),
}

impl BundleError {
    /// Stable key the UI layer maps to a localized message.
    #[must_use]
    pub const fn message_key(&self) -> &'static str {
        match self {
            Self::EmptyId => "bundle.id.empty",
            Self::MissingTitle => "bundle.title.missing",
            Self::EmptySetlistName => "setlist.name.empty",
            Self::DuplicateSong(_) => "bundle.song.duplicate",
            Self::DuplicateSetlistEntry(_) => "setlist.entry.duplicate",
            Self::MissingSong(_) => "setlist.song.missing",
        }
    }
}

impl fmt::Display for BundleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyId => write!(f, "identifier must not be empty"),
            Self::MissingTitle => write!(f, "song has no title in its metadata"),
            Self::EmptySetlistName => write!(f, "setlist name must not be empty"),
            Self::DuplicateSong(id) => write!(f, "song id already in library: {id}"),
            Self::DuplicateSetlistEntry(id) => write!(f, "song already in setlist: {id}"),
            Self::MissingSong(id) => write!(f, "song not in library: {id}"),
        }
    }
}

impl std::error::Error for BundleError {}

/// One song package: identity, tags and the canonical [`Song`].
///
/// Title, artist and key are read from the song metadata, so they are never
/// duplicated in the bundle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SongBundle {
    id: SongId,
    song: Song,
    tags: Vec<String>,
}

impl SongBundle {
    /// Builds a bundle, requiring a non-empty `title` in the song metadata.
    pub fn new(id: SongId, song: Song) -> Result<Self, BundleError> {
        match song.metadata.get("title") {
            Some(title) if !title.trim().is_empty() => Ok(Self {
                id,
                song,
                tags: Vec::new(),
            }),
            _ => Err(BundleError::MissingTitle),
        }
    }

    /// Borrows the bundle identifier.
    #[must_use]
    pub fn id(&self) -> &SongId {
        &self.id
    }

    /// Borrows the canonical song.
    #[must_use]
    pub fn song(&self) -> &Song {
        &self.song
    }

    /// Song title, taken from the metadata.
    #[must_use]
    pub fn title(&self) -> &str {
        self.song.metadata.get("title").map_or("", String::as_str)
    }

    /// Song artist, when the metadata declares one.
    #[must_use]
    pub fn artist(&self) -> Option<&str> {
        self.song.metadata.get("artist").map(String::as_str)
    }

    /// Original key, when the metadata declares one.
    #[must_use]
    pub fn key(&self) -> Option<&str> {
        self.song.metadata.get("key").map(String::as_str)
    }

    /// Tags attached to the bundle, in insertion order.
    #[must_use]
    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    /// Attaches a tag, ignoring empty values and exact duplicates.
    pub fn add_tag(&mut self, tag: impl Into<String>) {
        let tag = tag.into();
        let trimmed = tag.trim();
        if trimmed.is_empty() {
            return;
        }
        if !self.tags.iter().any(|existing| existing == trimmed) {
            self.tags.push(trimmed.to_owned());
        }
    }
}

/// The local song library: unique identifiers, one canonical song each.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Library {
    songs: BTreeMap<SongId, SongBundle>,
}

impl Library {
    /// Creates an empty library.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a bundle, refusing a duplicate identifier.
    pub fn add(&mut self, bundle: SongBundle) -> Result<(), BundleError> {
        let id = bundle.id().clone();
        if self.songs.contains_key(&id) {
            return Err(BundleError::DuplicateSong(id));
        }
        self.songs.insert(id, bundle);
        Ok(())
    }

    /// Looks a song up by identifier.
    #[must_use]
    pub fn get(&self, id: &SongId) -> Option<&SongBundle> {
        self.songs.get(id)
    }

    /// Removes a song, returning it when it was present.
    ///
    /// This does not inspect setlists: a caller that owns setlists must refuse
    /// the removal while any setlist still references the song, otherwise
    /// [`Setlist::songs`] starts failing loudly.
    pub fn remove(&mut self, id: &SongId) -> Option<SongBundle> {
        self.songs.remove(id)
    }

    /// Number of songs in the library.
    #[must_use]
    pub fn len(&self) -> usize {
        self.songs.len()
    }

    /// Whether the library holds no songs.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.songs.is_empty()
    }

    /// Iterates over the songs in identifier order.
    pub fn iter(&self) -> impl Iterator<Item = &SongBundle> {
        self.songs.values()
    }

    /// Songs whose title contains `needle`, case-insensitively.
    #[must_use]
    pub fn find_by_title(&self, needle: &str) -> Vec<&SongBundle> {
        let needle = needle.trim().to_lowercase();
        self.songs
            .values()
            .filter(|bundle| bundle.title().to_lowercase().contains(&needle))
            .collect()
    }
}

/// An ordered selection of songs from the library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Setlist {
    id: SetlistId,
    name: String,
    entries: Vec<SongId>,
}

impl Setlist {
    /// Creates an empty setlist, requiring a non-empty name.
    pub fn new(id: SetlistId, name: impl Into<String>) -> Result<Self, BundleError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(BundleError::EmptySetlistName);
        }
        Ok(Self {
            id,
            name: name.trim().to_owned(),
            entries: Vec::new(),
        })
    }

    /// Borrows the setlist identifier.
    #[must_use]
    pub fn id(&self) -> &SetlistId {
        &self.id
    }

    /// Setlist name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Song identifiers in setlist order.
    #[must_use]
    pub fn entries(&self) -> &[SongId] {
        &self.entries
    }

    /// Appends a song, refusing duplicates and songs the library does not hold.
    pub fn push(&mut self, library: &Library, id: SongId) -> Result<(), BundleError> {
        if library.get(&id).is_none() {
            return Err(BundleError::MissingSong(id));
        }
        if self.entries.contains(&id) {
            return Err(BundleError::DuplicateSetlistEntry(id));
        }
        self.entries.push(id);
        Ok(())
    }

    /// Removes the first occurrence of a song, returning whether it was present.
    pub fn remove(&mut self, id: &SongId) -> bool {
        match self.entries.iter().position(|entry| entry == id) {
            Some(index) => {
                self.entries.remove(index);
                true
            }
            None => false,
        }
    }

    /// Resolves the entries into songs, in setlist order.
    ///
    /// Fails loudly when an entry points at a song the library no longer holds,
    /// instead of silently returning a shorter list.
    pub fn songs<'a>(&'a self, library: &'a Library) -> Result<Vec<&'a SongBundle>, BundleError> {
        self.entries
            .iter()
            .map(|id| {
                library
                    .get(id)
                    .ok_or_else(|| BundleError::MissingSong(id.clone()))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chordpro::ChordProParser;

    fn song(title: &str) -> Song {
        ChordProParser::parse(&format!("{{title: {title}}}\n{{key: C}}\n\n[C]texto\n"))
            .unwrap_or_else(|error| panic!("fixture should parse: {error}"))
    }

    fn bundle(id: &str, title: &str) -> SongBundle {
        let id = SongId::new(id).unwrap_or_else(|error| panic!("id should be valid: {error}"));
        SongBundle::new(id, song(title))
            .unwrap_or_else(|error| panic!("bundle should build: {error}"))
    }

    #[test]
    fn rejects_empty_identifiers() {
        assert_eq!(SongId::new("  "), Err(BundleError::EmptyId));
        assert_eq!(SetlistId::new(""), Err(BundleError::EmptyId));
    }

    #[test]
    fn bundle_reads_bibliographic_data_from_metadata() {
        let bundle = bundle("santo", "Santo");
        assert_eq!(bundle.title(), "Santo");
        assert_eq!(bundle.key(), Some("C"));
        assert_eq!(bundle.artist(), None);
        assert!(bundle.tags().is_empty());
    }

    #[test]
    fn bundle_requires_a_title() {
        let song = ChordProParser::parse("{key: C}\n\n[C]texto\n").expect("fixture should parse");
        let id = SongId::new("sem-titulo").expect("id should be valid");
        assert_eq!(SongBundle::new(id, song), Err(BundleError::MissingTitle));
    }

    #[test]
    fn tags_ignore_empty_values_and_duplicates() {
        let mut bundle = bundle("santo", "Santo");
        bundle.add_tag("  adoração  ");
        bundle.add_tag("adoração");
        bundle.add_tag("");
        assert_eq!(bundle.tags(), ["adoração"]);
    }

    #[test]
    fn library_refuses_duplicate_identifiers() {
        let mut library = Library::new();
        library.add(bundle("santo", "Santo")).expect("first add");
        let error = library
            .add(bundle("santo", "Outro"))
            .expect_err("duplicate must fail");
        assert_eq!(error.message_key(), "bundle.song.duplicate");
        assert_eq!(library.len(), 1);
    }

    #[test]
    fn library_finds_by_title_case_insensitively() {
        let mut library = Library::new();
        library.add(bundle("santo", "Santo")).expect("add");
        library.add(bundle("graca", "Graça")).expect("add");

        let found = library.find_by_title("SANTO");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].title(), "Santo");
    }

    #[test]
    fn setlist_refuses_songs_outside_the_library() {
        let library = Library::new();
        let mut setlist = Setlist::new(SetlistId::new("domingo").expect("id"), "Domingo")
            .expect("setlist should build");
        let id = SongId::new("santo").expect("id should be valid");

        let error = setlist.push(&library, id).expect_err("missing song");
        assert_eq!(error.message_key(), "setlist.song.missing");
        assert!(setlist.entries().is_empty());
    }

    #[test]
    fn setlist_refuses_duplicate_entries() {
        let mut library = Library::new();
        library.add(bundle("santo", "Santo")).expect("add");
        let mut setlist = Setlist::new(SetlistId::new("domingo").expect("id"), "Domingo")
            .expect("setlist should build");
        let id = SongId::new("santo").expect("id should be valid");

        setlist.push(&library, id.clone()).expect("first push");
        let error = setlist.push(&library, id).expect_err("duplicate");
        assert_eq!(error.message_key(), "setlist.entry.duplicate");
        assert_eq!(setlist.entries().len(), 1);
    }

    #[test]
    fn setlist_resolves_entries_in_order() {
        let mut library = Library::new();
        library.add(bundle("santo", "Santo")).expect("add");
        library.add(bundle("graca", "Graça")).expect("add");
        let mut setlist = Setlist::new(SetlistId::new("domingo").expect("id"), "Domingo")
            .expect("setlist should build");

        setlist
            .push(&library, SongId::new("graca").expect("id"))
            .expect("push");
        setlist
            .push(&library, SongId::new("santo").expect("id"))
            .expect("push");

        let songs = setlist.songs(&library).expect("setlist should resolve");
        assert_eq!(
            songs.iter().map(|song| song.title()).collect::<Vec<_>>(),
            ["Graça", "Santo"]
        );
    }

    #[test]
    fn setlist_fails_loudly_when_a_song_disappears() {
        let mut library = Library::new();
        library.add(bundle("santo", "Santo")).expect("add");
        let mut setlist = Setlist::new(SetlistId::new("domingo").expect("id"), "Domingo")
            .expect("setlist should build");
        let id = SongId::new("santo").expect("id should be valid");
        setlist.push(&library, id.clone()).expect("push");

        library.remove(&id);

        let error = setlist
            .songs(&library)
            .expect_err("dangling entry must fail");
        assert_eq!(error.message_key(), "setlist.song.missing");
    }

    #[test]
    fn setlist_remove_reports_whether_it_removed_anything() {
        let mut library = Library::new();
        library.add(bundle("santo", "Santo")).expect("add");
        let mut setlist = Setlist::new(SetlistId::new("domingo").expect("id"), "Domingo")
            .expect("setlist should build");
        let id = SongId::new("santo").expect("id should be valid");
        setlist.push(&library, id.clone()).expect("push");

        assert!(setlist.remove(&id));
        assert!(!setlist.remove(&id));
        assert!(setlist.entries().is_empty());
    }
}
