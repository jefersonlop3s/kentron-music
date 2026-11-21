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

/// Uma execução da agenda: a obra **e a versão em que ela foi tocada naquele
/// evento**.
///
/// A versão é campo de primeira classe (ADR-0006, emenda): a mesma obra volta em
/// outro culto e **nem sempre na mesma versão** — outro arranjo, outra gravação,
/// outro tom. Sem isso, duas execuções diferentes viram a mesma linha no
/// relatório, e o relatório mente.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Execucao {
    song: SongId,
    versao: Option<String>,
    tom: Option<String>,
}

impl Execucao {
    /// Execução sem versão declarada — o caso da agenda preenchida só com a obra.
    #[must_use]
    pub fn new(song: SongId) -> Self {
        Self {
            song,
            versao: None,
            tom: None,
        }
    }

    /// Declara a versão. Texto vazio é o mesmo que não declarar.
    #[must_use]
    pub fn com_versao(mut self, versao: impl Into<String>) -> Self {
        let versao = versao.into();
        let versao = versao.trim();
        if !versao.is_empty() {
            self.versao = Some(versao.to_owned());
        }
        self
    }

    /// Declara o tom em que a obra foi tocada naquele evento.
    #[must_use]
    pub fn com_tom(mut self, tom: impl Into<String>) -> Self {
        let tom = tom.into();
        let tom = tom.trim();
        if !tom.is_empty() {
            self.tom = Some(tom.to_owned());
        }
        self
    }

    /// A obra executada.
    #[must_use]
    pub fn song(&self) -> &SongId {
        &self.song
    }

    /// Versão declarada, quando existe.
    #[must_use]
    pub fn versao(&self) -> Option<&str> {
        self.versao.as_deref()
    }

    /// Tom declarado, quando existe.
    #[must_use]
    pub fn tom(&self) -> Option<&str> {
        self.tom.as_deref()
    }
}

/// A **agenda de um evento**: a playlist do culto, do ensaio ou do evento, com
/// data e tipo.
///
/// Antes era só uma lista de músicas; a emenda da ADR-0006 a transformou no
/// registro do que a equipe já mantém — agenda por evento, uma playlist por
/// evento. O relatório de uso é subproduto disto, não hábito novo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Setlist {
    id: SetlistId,
    name: String,
    data: Option<String>,
    tipo: Option<String>,
    entries: Vec<Execucao>,
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
            data: None,
            tipo: None,
            entries: Vec::new(),
        })
    }

    /// Declares the event date (`AAAA-MM-DD`).
    #[must_use]
    pub fn com_data(mut self, data: impl Into<String>) -> Self {
        let data = data.into();
        let data = data.trim();
        if !data.is_empty() {
            self.data = Some(data.to_owned());
        }
        self
    }

    /// Declares the event kind: `culto`, `ensaio`, `evento`.
    #[must_use]
    pub fn com_tipo(mut self, tipo: impl Into<String>) -> Self {
        let tipo = tipo.into();
        let tipo = tipo.trim();
        if !tipo.is_empty() {
            self.tipo = Some(tipo.to_owned());
        }
        self
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

    /// Event date, when declared.
    #[must_use]
    pub fn data(&self) -> Option<&str> {
        self.data.as_deref()
    }

    /// Event kind, when declared.
    #[must_use]
    pub fn tipo(&self) -> Option<&str> {
        self.tipo.as_deref()
    }

    /// Executions in setlist order.
    #[must_use]
    pub fn entries(&self) -> &[Execucao] {
        &self.entries
    }

    /// Appends a song without a declared version.
    ///
    /// # Errors
    ///
    /// Refuses duplicates and songs the library does not hold.
    pub fn push(&mut self, library: &Library, id: SongId) -> Result<(), BundleError> {
        self.push_execucao(library, Execucao::new(id))
    }

    /// Appends an execution (obra + versão + tom).
    ///
    /// # Errors
    ///
    /// Refuses a song already in this event and songs the library does not hold.
    pub fn push_execucao(
        &mut self,
        library: &Library,
        execucao: Execucao,
    ) -> Result<(), BundleError> {
        if library.get(execucao.song()).is_none() {
            return Err(BundleError::MissingSong(execucao.song().clone()));
        }
        if self
            .entries
            .iter()
            .any(|existente| existente.song() == execucao.song())
        {
            return Err(BundleError::DuplicateSetlistEntry(execucao.song().clone()));
        }
        self.entries.push(execucao);
        Ok(())
    }

    /// Removes the first occurrence of a song, returning whether it was present.
    pub fn remove(&mut self, id: &SongId) -> bool {
        match self.entries.iter().position(|entry| entry.song() == id) {
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
            .map(|entry| {
                library
                    .get(entry.song())
                    .ok_or_else(|| BundleError::MissingSong(entry.song().clone()))
            })
            .collect()
    }
}

/// Uso de uma obra no período: quantas vezes, e **em que versões**.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumoObra {
    pub song: SongId,
    pub titulo: String,
    /// Total de execuções no período.
    pub execucoes: usize,
    /// Versões usadas, na ordem da primeira aparição, com a contagem.
    pub versoes: Vec<(String, usize)>,
}

/// Versão de quem não declarou versão. Texto fixo de propósito: o relatório diz
/// "não declarada" em vez de inventar um rótulo.
pub const VERSAO_NAO_DECLARADA: &str = "(versão não declarada)";

/// Relatório de uso: **subproduto da agenda** (ADR-0006, emenda), não hábito novo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelatorioUso {
    pub eventos: usize,
    pub execucoes: usize,
    pub por_obra: Vec<ResumoObra>,
}

impl RelatorioUso {
    /// Quantas obras distintas foram tocadas no período.
    #[must_use]
    pub fn obras_distintas(&self) -> usize {
        self.por_obra.len()
    }

    /// Quantas obras saíram em mais de uma versão no período.
    #[must_use]
    pub fn obras_com_mais_de_uma_versao(&self) -> usize {
        self.por_obra
            .iter()
            .filter(|resumo| resumo.versoes.len() > 1)
            .count()
    }

    /// Relatório legível, ordenado por uso (mais tocada primeiro) e depois por
    /// título — ordem estável, para o texto poder ser comparado em teste.
    #[must_use]
    pub fn texto(&self) -> String {
        let mut texto = format!(
            "eventos: {} · execuções: {} · obras distintas: {}\n\n",
            self.eventos,
            self.execucoes,
            self.obras_distintas()
        );
        let mut ordenado = self.por_obra.clone();
        ordenado.sort_by(|a, b| {
            b.execucoes
                .cmp(&a.execucoes)
                .then_with(|| a.titulo.cmp(&b.titulo))
        });
        for resumo in &ordenado {
            texto.push_str(&format!(
                "{:>3}×  {}  [{}]\n",
                resumo.execucoes, resumo.titulo, resumo.song
            ));
            for (versao, vezes) in &resumo.versoes {
                texto.push_str(&format!("        {vezes}× {versao}\n"));
            }
        }
        texto
    }
}

/// Monta o relatório de uso a partir das agendas.
///
/// # Errors
///
/// Falha alto quando uma agenda aponta para obra que a biblioteca não tem: um
/// relatório com menos entradas do que a agenda teria é pior que erro.
pub fn relatorio_de_uso(
    setlists: &[Setlist],
    library: &Library,
) -> Result<RelatorioUso, BundleError> {
    let mut por_obra: Vec<ResumoObra> = Vec::new();
    let mut execucoes = 0usize;
    for setlist in setlists {
        for entrada in setlist.entries() {
            let bundle = library
                .get(entrada.song())
                .ok_or_else(|| BundleError::MissingSong(entrada.song().clone()))?;
            execucoes += 1;
            let versao = entrada.versao().unwrap_or(VERSAO_NAO_DECLARADA).to_owned();
            match por_obra
                .iter_mut()
                .find(|resumo| resumo.song == *entrada.song())
            {
                Some(resumo) => {
                    resumo.execucoes += 1;
                    match resumo
                        .versoes
                        .iter_mut()
                        .find(|(existente, _)| *existente == versao)
                    {
                        Some((_, vezes)) => *vezes += 1,
                        None => resumo.versoes.push((versao, 1)),
                    }
                }
                None => por_obra.push(ResumoObra {
                    song: entrada.song().clone(),
                    titulo: bundle.title().to_owned(),
                    execucoes: 1,
                    versoes: vec![(versao, 1)],
                }),
            }
        }
    }
    Ok(RelatorioUso {
        eventos: setlists.len(),
        execucoes,
        por_obra,
    })
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
