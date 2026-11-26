//! Song bundle persistence: manifest, ChordPro source, MusicXML score and
//! tablatures (ADR-0006).
//!
//! Layout of a bundle directory:
//!
//! ```text
//! <song-id>/
//!   meta.json        manifest: schema_version, id, updated_at, files
//!   song.cho         ChordPro source — single source of truth for content
//!   score.musicxml   optional
//!   tabs/*.atex      optional
//! ```
//!
//! **The manifest is the inventory.** It declares every content file the
//! directory holds, and loading cross-checks the two directions: a declared file
//! that is absent, a declared file this build cannot read, and a `score.musicxml`
//! or `tabs/*.atex` on disk that the manifest does not declare all fail loudly —
//! never a silent skip.
//!
//! Every JSON file carries `schema_version`. A version this build does not know
//! **fails loudly** ([`StoreError::UnsupportedSchema`]) instead of being read
//! partially, and the migration chain walks one step per version when a step
//! exists.
//!
//! The model in [`crate::bundle`] stays free of serialization attributes: the
//! wire structs here are private, and every value re-enters the model through
//! its validating constructor.

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::bundle::{
    BundleError, Execucao, Library, Setlist, SetlistId, SongBundle, SongId, Tablatura,
};
use crate::chordpro::{self, ChordProParser};

/// Schema version written by this build.
pub const SCHEMA_VERSION: u32 = 2;
/// Manifest file name inside a bundle directory.
pub const MANIFEST_FILE: &str = "meta.json";
/// ChordPro source file name inside a bundle directory.
pub const SONG_FILE: &str = "song.cho";
/// Optional MusicXML source file name inside a bundle directory (R1.1).
pub const MUSICXML_FILE: &str = "score.musicxml";
/// Directory holding the tablatures inside a bundle directory (ADR-0006).
pub const TABLATURE_DIR: &str = "tabs";
/// Extension of a tablature file inside [`TABLATURE_DIR`].
pub const TABLATURE_EXT: &str = "atex";
/// Library index file name, next to the bundle directories.
pub const LIBRARY_FILE: &str = "library.json";
/// Directory holding the agendas (one file per event), next to the library.
pub const SETLISTS_DIR: &str = "setlists";

/// Why a bundle, library or setlist could not be read or written.
#[derive(Debug)]
pub enum StoreError {
    /// The file system refused the operation.
    Io { path: PathBuf, source: io::Error },
    /// A file exists but is not valid JSON for its schema.
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
    /// A required file is absent.
    MissingFile { path: PathBuf },
    /// The ChordPro source is not parseable.
    ChordPro { path: PathBuf, message: String },
    /// The manifest does not declare the ChordPro source it must carry.
    UndeclaredSong { path: PathBuf },
    /// The manifest declares a content file this build cannot read.
    UnsupportedFile { path: PathBuf, declarado: String },
    /// The directory holds a content file the manifest does not declare.
    UndeclaredFile { path: PathBuf },
    /// `schema_version` is absent or not an unsigned integer.
    InvalidSchemaVersion { path: PathBuf },
    /// `schema_version` is not one this build can read or migrate.
    UnsupportedSchema { found: u32, supported: u32 },
    /// The model refused the value read from disk.
    Bundle(BundleError),
}

impl StoreError {
    /// Stable key the UI layer maps to a localized message.
    #[must_use]
    pub const fn message_key(&self) -> &'static str {
        match self {
            Self::Io { .. } => "store.io",
            Self::Json { .. } => "store.json",
            Self::MissingFile { .. } => "store.file.missing",
            Self::ChordPro { .. } => "store.chordpro",
            Self::UndeclaredSong { .. } => "store.manifest.undeclared_song",
            Self::UnsupportedFile { .. } => "store.manifest.unsupported_file",
            Self::UndeclaredFile { .. } => "store.manifest.undeclared_file",
            Self::InvalidSchemaVersion { .. } => "store.schema.invalid",
            Self::UnsupportedSchema { .. } => "store.schema.unsupported",
            Self::Bundle(error) => error.message_key(),
        }
    }
}

impl From<BundleError> for StoreError {
    fn from(error: BundleError) -> Self {
        Self::Bundle(error)
    }
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => write!(f, "{}: {source}", path.display()),
            Self::Json { path, source } => write!(f, "{}: {source}", path.display()),
            Self::MissingFile { path } => write!(f, "missing file: {}", path.display()),
            Self::ChordPro { path, message } => {
                write!(f, "{}: {message}", path.display())
            }
            Self::UndeclaredSong { path } => {
                write!(
                    f,
                    "{}: manifest does not declare {SONG_FILE}",
                    path.display()
                )
            }
            Self::UnsupportedFile { path, declarado } => write!(
                f,
                "{}: manifest declares {declarado}, which this build does not read",
                path.display()
            ),
            Self::UndeclaredFile { path } => write!(
                f,
                "{}: content file is not declared in {MANIFEST_FILE}",
                path.display()
            ),
            Self::InvalidSchemaVersion { path } => write!(
                f,
                "{}: schema_version is absent or not an unsigned integer",
                path.display()
            ),
            Self::UnsupportedSchema { found, supported } => write!(
                f,
                "schema_version {found} is not supported (this build reads {supported})"
            ),
            Self::Bundle(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for StoreError {}

/// The manifest of a bundle directory, as stored in [`MANIFEST_FILE`].
///
/// `id` is a plain string on the wire; the validated [`SongId`] is built from it
/// on load. `files` is the declared inventory of the directory and must list the
/// ChordPro source, so a hand-edited manifest that forgot it fails loudly.
///
/// The inventory lists, in this order, what the directory holds: [`SONG_FILE`]
/// always, [`MUSICXML_FILE`] when the song has a score, and
/// `tabs/<nome>.atex` for each tablature — the tablature order is the insertion
/// order of [`crate::bundle::SongBundle::tabs`], so saving and loading give back
/// the same list. Paths are `/`-separated and relative to the bundle directory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundleManifest {
    /// Schema version of this file.
    pub schema_version: u32,
    /// Song identifier, validated into [`SongId`] on load.
    pub id: String,
    /// Free-form timestamp set by the caller; the core owns no clock.
    pub updated_at: Option<String>,
    /// Files the directory declares, relative to it.
    pub files: Vec<String>,
}

/// Library index: the identifiers the library holds, in identifier order.
#[derive(Debug, Serialize, Deserialize)]
struct LibraryFile {
    schema_version: u32,
    songs: Vec<String>,
}

/// Agenda file: identity, event (date, kind) and the ordered executions.
///
/// `entries` era lista de ids na v1; a v2 carrega a **execução** (obra + versão +
/// tom), porque a mesma obra volta em outro evento e nem sempre na mesma versão
/// (ADR-0006, emenda).
#[derive(Debug, Serialize, Deserialize)]
struct SetlistFile {
    schema_version: u32,
    id: String,
    name: String,
    #[serde(default)]
    data: Option<String>,
    #[serde(default)]
    tipo: Option<String>,
    entries: Vec<ExecucaoFile>,
}

/// Uma execução no arquivo da agenda.
#[derive(Debug, Serialize, Deserialize)]
struct ExecucaoFile {
    song: String,
    #[serde(default)]
    versao: Option<String>,
    #[serde(default)]
    tom: Option<String>,
}

/// One migration step: upgrades the manifest of version `from` to `from + 1`.
type MigrationStep = fn(Value) -> Result<Value, StoreError>;

/// Migration chain, indexed by the version each step upgrades **from**.
///
/// v1 → v2: a entrada da agenda deixa de ser um id solto e passa a ser execução,
/// com versão e tom. **Campo que não existe fica ausente**, nunca inventado: o
/// relatório diz "versão não declarada" em vez de chutar um arranjo.
const MIGRATIONS: &[(u32, MigrationStep)] = &[(1, migrar_1_para_2)];

/// v1 → v2. Mexe só onde há `entries`: manifesto e índice da biblioteca passam
/// sem alteração de forma, apenas com a versão carimbada.
fn migrar_1_para_2(mut value: Value) -> Result<Value, StoreError> {
    if let Some(entries) = value.get_mut("entries").and_then(Value::as_array_mut) {
        for entry in entries.iter_mut() {
            if let Some(id) = entry.as_str() {
                *entry = serde_json::json!({ "song": id, "versao": null, "tom": null });
            }
        }
    }
    value["schema_version"] = Value::from(2);
    Ok(value)
}

/// Reads `schema_version` out of an already-parsed file.
fn version_of(value: &Value, path: &Path) -> Result<u32, StoreError> {
    value
        .get("schema_version")
        .and_then(Value::as_u64)
        .and_then(|found| u32::try_from(found).ok())
        .ok_or_else(|| StoreError::InvalidSchemaVersion {
            path: path.to_path_buf(),
        })
}

/// Walks the migration chain until the file reaches [`SCHEMA_VERSION`].
///
/// A file from a newer build is refused, and so is a version with no step to
/// upgrade it: guessing would read a format this build does not know.
fn migrate(value: Value, path: &Path) -> Result<Value, StoreError> {
    let mut current = value;
    let mut version = version_of(&current, path)?;

    while version < SCHEMA_VERSION {
        match MIGRATIONS.iter().find(|(from, _)| *from == version) {
            Some((_, step)) => {
                current = step(current)?;
                version += 1;
            }
            None => {
                return Err(StoreError::UnsupportedSchema {
                    found: version,
                    supported: SCHEMA_VERSION,
                });
            }
        }
    }

    if version > SCHEMA_VERSION {
        return Err(StoreError::UnsupportedSchema {
            found: version,
            supported: SCHEMA_VERSION,
        });
    }

    Ok(current)
}

fn write_file(path: &Path, contents: &str) -> Result<(), StoreError> {
    fs::write(path, contents).map_err(|source| StoreError::Io {
        path: path.to_path_buf(),
        source,
    })
}

/// Reads a file that must exist, reporting absence as [`StoreError::MissingFile`]
/// instead of a generic I/O error.
fn read_required(path: &Path) -> Result<String, StoreError> {
    fs::read_to_string(path).map_err(|source| {
        if source.kind() == io::ErrorKind::NotFound {
            StoreError::MissingFile {
                path: path.to_path_buf(),
            }
        } else {
            StoreError::Io {
                path: path.to_path_buf(),
                source,
            }
        }
    })
}

fn parse_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, StoreError> {
    let raw = read_required(path)?;
    let value: Value = serde_json::from_str(&raw).map_err(|source| StoreError::Json {
        path: path.to_path_buf(),
        source,
    })?;
    let value = migrate(value, path)?;
    serde_json::from_value(value).map_err(|source| StoreError::Json {
        path: path.to_path_buf(),
        source,
    })
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), StoreError> {
    let json = serde_json::to_string_pretty(value).map_err(|source| StoreError::Json {
        path: path.to_path_buf(),
        source,
    })?;
    write_file(path, &json)
}

/// Caminho declarado de uma tablatura dentro do bundle: `tabs/<nome>.atex`.
///
/// Sempre com `/`, mesmo no Windows: o manifesto é um arquivo JSON e o inventário
/// não pode depender do separador da plataforma que gravou.
#[must_use]
pub fn caminho_da_tablatura(nome: &str) -> String {
    format!("{TABLATURE_DIR}/{nome}.{TABLATURE_EXT}")
}

/// Writes a bundle into `dir`, creating the directory when needed.
///
/// Returns the manifest that was written. `updated_at` is stored verbatim — the
/// core owns no clock, so the caller decides the timestamp.
///
/// Writes everything the bundle carries, and declares it all in the manifest: the
/// ChordPro source (always, re-serialized from the model), `score.musicxml` when
/// [`SongBundle::musicxml`] is set, and `tabs/<nome>.atex` for each
/// [`SongBundle::tabs`] entry, in insertion order.
///
/// **Saving does not delete files it no longer writes.** A `score.musicxml` or
/// `tabs/*.atex` left behind by an earlier save of the same directory becomes an
/// orphan the manifest does not declare, and the next [`load_bundle`] refuses it
/// with [`StoreError::UndeclaredFile`]. Loud on purpose: deleting content the caller
/// did not name would be data loss, and ignoring the orphan would hide it. To drop a
/// source, remove the file (or save into a fresh directory).
pub fn save_bundle(
    dir: &Path,
    bundle: &SongBundle,
    updated_at: Option<&str>,
) -> Result<BundleManifest, StoreError> {
    fs::create_dir_all(dir).map_err(|source| StoreError::Io {
        path: dir.to_path_buf(),
        source,
    })?;
    write_file(&dir.join(SONG_FILE), &chordpro::write(bundle.song()))?;

    let mut files = vec![SONG_FILE.to_owned()];
    if let Some(musicxml) = bundle.musicxml() {
        write_file(&dir.join(MUSICXML_FILE), musicxml)?;
        files.push(MUSICXML_FILE.to_owned());
    }

    if !bundle.tabs().is_empty() {
        let tab_dir = dir.join(TABLATURE_DIR);
        fs::create_dir_all(&tab_dir).map_err(|source| StoreError::Io {
            path: tab_dir.clone(),
            source,
        })?;
        for tablatura in bundle.tabs() {
            let declarado = caminho_da_tablatura(tablatura.nome());
            write_file(&dir.join(&declarado), tablatura.atex())?;
            files.push(declarado);
        }
    }

    let manifest = BundleManifest {
        schema_version: SCHEMA_VERSION,
        id: bundle.id().as_str().to_owned(),
        updated_at: updated_at.map(str::to_owned),
        files,
    };
    write_json(&dir.join(MANIFEST_FILE), &manifest)?;
    Ok(manifest)
}

/// Like [`save_bundle`], but also writes an optional MusicXML source file.
///
/// Kept for callers that hold the score outside the bundle. `musicxml_source`
/// **overrides** whatever the bundle already carries; `None` means "keep the
/// bundle's own score", which is what [`save_bundle`] does.
pub fn save_bundle_with_musicxml(
    dir: &Path,
    bundle: &SongBundle,
    updated_at: Option<&str>,
    musicxml_source: Option<&str>,
) -> Result<BundleManifest, StoreError> {
    match musicxml_source {
        Some(source) => save_bundle(dir, &bundle.clone().com_musicxml(source), updated_at),
        None => save_bundle(dir, bundle, updated_at),
    }
}

/// Reads the optional MusicXML source from a bundle directory, if it exists.
///
/// This reads the file straight from disk and **does not consult the manifest** — it
/// is the raw helper, not the reconciling path. [`load_bundle`] is the one that
/// checks the inventory and hands the score back inside the model; a bundle that
/// passes through it always agrees with this function.
///
/// Returns `None` when no `score.musicxml` file is present.
pub fn load_musicxml_from_bundle(dir: &Path) -> Result<Option<String>, StoreError> {
    let path = dir.join(MUSICXML_FILE);
    if !path.exists() {
        return Ok(None);
    }
    read_required(&path).map(Some)
}

/// What a path declared in the manifest is, in this build.
enum Declarado {
    /// The ChordPro source, [`SONG_FILE`].
    Cancao,
    /// The MusicXML score, [`MUSICXML_FILE`].
    Partitura,
    /// A tablature; carries the name, without directory or extension.
    Tablatura(String),
}

/// Classifies a declared path; `Err` carries whatever this build does not read.
fn classificar(declarado: &str) -> Result<Declarado, String> {
    if declarado == SONG_FILE {
        return Ok(Declarado::Cancao);
    }
    if declarado == MUSICXML_FILE {
        return Ok(Declarado::Partitura);
    }
    if let [dir, arquivo] = declarado.split('/').collect::<Vec<_>>()[..] {
        if dir == TABLATURE_DIR {
            let sufixo = format!(".{TABLATURE_EXT}");
            if let Some(nome) = arquivo
                .strip_suffix(&sufixo)
                .filter(|nome| !nome.is_empty() && *nome != "." && *nome != "..")
            {
                return Ok(Declarado::Tablatura(nome.to_owned()));
            }
        }
    }
    Err(declarado.to_owned())
}

/// Confere que todo arquivo de conteúdo do diretório está declarado no manifesto.
///
/// O manifesto é o inventário: `score.musicxml` ou `tabs/*.atex` presente em disco e
/// fora dele **reprova**, em vez de ser ignorado em silêncio. Nome de arquivo que
/// não é UTF-8 também reprova — o manifesto é JSON, e o que não cabe nele não pode
/// ser declarado.
fn conferir_inventario(dir: &Path, manifest: &BundleManifest) -> Result<(), StoreError> {
    let declarado = |caminho: &str| manifest.files.iter().any(|entry| entry == caminho);

    let partitura = dir.join(MUSICXML_FILE);
    if partitura.is_file() && !declarado(MUSICXML_FILE) {
        return Err(StoreError::UndeclaredFile { path: partitura });
    }

    let tab_dir = dir.join(TABLATURE_DIR);
    if !tab_dir.is_dir() {
        return Ok(());
    }
    for entrada in fs::read_dir(&tab_dir).map_err(|source| StoreError::Io {
        path: tab_dir.clone(),
        source,
    })? {
        let entrada = entrada.map_err(|source| StoreError::Io {
            path: tab_dir.clone(),
            source,
        })?;
        let caminho = entrada.path();
        if !entrada
            .file_type()
            .map_err(|source| StoreError::Io {
                path: caminho.clone(),
                source,
            })?
            .is_file()
        {
            continue;
        }
        let nome = caminho
            .file_name()
            .and_then(|nome| nome.to_str())
            .ok_or_else(|| StoreError::UndeclaredFile {
                path: caminho.clone(),
            })?;
        if !declarado(&format!("{TABLATURE_DIR}/{nome}")) {
            return Err(StoreError::UndeclaredFile { path: caminho });
        }
    }
    Ok(())
}

/// Reads a bundle from `dir`, with everything the manifest declares.
///
/// Fails loudly when the manifest is absent, carries an unknown schema, does not
/// declare the ChordPro source, declares a file this build does not read or that is
/// not on disk, when the directory holds a content file the manifest omits, or when
/// the ChordPro source does not parse.
///
/// The model comes back with its sources: [`SongBundle::musicxml`] and
/// [`SongBundle::tabs`] carry the very bytes that were on disk.
pub fn load_bundle(dir: &Path) -> Result<SongBundle, StoreError> {
    let manifest: BundleManifest = parse_json(&dir.join(MANIFEST_FILE))?;
    conferir_inventario(dir, &manifest)?;

    let mut fonte: Option<String> = None;
    let mut musicxml: Option<String> = None;
    let mut tablaturas: Vec<Tablatura> = Vec::new();
    for declarado in &manifest.files {
        let caminho = dir.join(declarado);
        match classificar(declarado).map_err(|declarado| StoreError::UnsupportedFile {
            path: caminho.clone(),
            declarado,
        })? {
            Declarado::Cancao => fonte = Some(read_required(&caminho)?),
            Declarado::Partitura => musicxml = Some(read_required(&caminho)?),
            Declarado::Tablatura(nome) => {
                let atex = read_required(&caminho)?;
                tablaturas.push(Tablatura::nova(nome, atex)?);
            }
        }
    }

    let fonte = fonte.ok_or_else(|| StoreError::UndeclaredSong {
        path: dir.join(MANIFEST_FILE),
    })?;
    let song_path = dir.join(SONG_FILE);
    let song = ChordProParser::parse(&fonte).map_err(|message| StoreError::ChordPro {
        path: song_path,
        message,
    })?;

    let id = SongId::new(manifest.id)?;
    let mut bundle = SongBundle::new(id, song)?;
    if let Some(musicxml) = musicxml {
        bundle = bundle.com_musicxml(musicxml);
    }
    for tablatura in tablaturas {
        bundle = bundle.com_tablatura(tablatura)?;
    }
    Ok(bundle)
}

/// Writes the library index next to the bundle directories.
pub fn save_library(dir: &Path, library: &Library) -> Result<(), StoreError> {
    fs::create_dir_all(dir).map_err(|source| StoreError::Io {
        path: dir.to_path_buf(),
        source,
    })?;
    let file = LibraryFile {
        schema_version: SCHEMA_VERSION,
        songs: library
            .iter()
            .map(|bundle| bundle.id().as_str().to_owned())
            .collect(),
    };
    write_json(&dir.join(LIBRARY_FILE), &file)
}

/// Reads the library index and loads every bundle it names.
///
/// The bundle directory of an entry is the entry itself: `<library-dir>/<id>/`.
pub fn load_library(dir: &Path) -> Result<Library, StoreError> {
    let file: LibraryFile = parse_json(&dir.join(LIBRARY_FILE))?;
    let mut library = Library::new();
    for id in file.songs {
        let id = SongId::new(id)?;
        let bundle = load_bundle(&dir.join(id.as_str()))?;
        library.add(bundle)?;
    }
    Ok(library)
}

/// Writes an agenda (event setlist) file.
pub fn save_setlist(path: &Path, setlist: &Setlist) -> Result<(), StoreError> {
    let file = SetlistFile {
        schema_version: SCHEMA_VERSION,
        id: setlist.id().as_str().to_owned(),
        name: setlist.name().to_owned(),
        data: setlist.data().map(str::to_owned),
        tipo: setlist.tipo().map(str::to_owned),
        entries: setlist
            .entries()
            .iter()
            .map(|entrada| ExecucaoFile {
                song: entrada.song().as_str().to_owned(),
                versao: entrada.versao().map(str::to_owned),
                tom: entrada.tom().map(str::to_owned),
            })
            .collect(),
    };
    write_json(path, &file)
}

/// Reads an agenda, rebuilding it through the validated constructors.
///
/// Every entry goes back through `Setlist::push_execucao`, so a file referencing
/// a song the library does not hold fails loudly instead of loading an agenda
/// with dangling entries.
pub fn load_setlist(path: &Path, library: &Library) -> Result<Setlist, StoreError> {
    let file: SetlistFile = parse_json(path)?;
    let mut setlist = Setlist::new(SetlistId::new(file.id)?, file.name)?;
    if let Some(data) = file.data {
        setlist = setlist.com_data(data);
    }
    if let Some(tipo) = file.tipo {
        setlist = setlist.com_tipo(tipo);
    }
    for entrada in file.entries {
        let mut execucao = Execucao::new(SongId::new(entrada.song)?);
        if let Some(versao) = entrada.versao {
            execucao = execucao.com_versao(versao);
        }
        if let Some(tom) = entrada.tom {
            execucao = execucao.com_tom(tom);
        }
        setlist.push_execucao(library, execucao)?;
    }
    Ok(setlist)
}

/// Reads every agenda of a bundle directory (`<dir>/setlists/*.json`).
///
/// Order is by file name, so the usage report is comparable between runs.
/// Absent directory means no agenda yet — not an error, and not a silent empty
/// answer either: the caller sees zero events.
///
/// # Errors
///
/// Fails loudly on any agenda that cannot be read or that points at a song the
/// library no longer holds.
pub fn load_setlists(dir: &Path, library: &Library) -> Result<Vec<Setlist>, StoreError> {
    let agenda_dir = dir.join(SETLISTS_DIR);
    if !agenda_dir.exists() {
        return Ok(Vec::new());
    }
    let mut caminhos: Vec<PathBuf> = Vec::new();
    for entrada in fs::read_dir(&agenda_dir).map_err(|source| StoreError::Io {
        path: agenda_dir.clone(),
        source,
    })? {
        // Uma entrada ilegivel e' erro, nao um arquivo que some da lista: filtrada em
        // silencio, ela viraria agenda ausente no relatorio de uso.
        let entrada = entrada.map_err(|source| StoreError::Io {
            path: agenda_dir.clone(),
            source,
        })?;
        if entrada
            .path()
            .extension()
            .is_some_and(|extensao| extensao == "json")
        {
            caminhos.push(entrada.path());
        }
    }
    caminhos.sort();
    caminhos
        .iter()
        .map(|caminho| load_setlist(caminho, library))
        .collect()
}

/// Copies a bundle directory, file by file.
///
/// ADR-0006 keeps the single-file container for v1.1; v1 exports by copying the
/// directory, which is also the backup story for the pilot.
pub fn export_bundle(from: &Path, to: &Path) -> Result<(), StoreError> {
    copy_dir(from, to)
}

fn copy_dir(from: &Path, to: &Path) -> Result<(), StoreError> {
    fs::create_dir_all(to).map_err(|source| StoreError::Io {
        path: to.to_path_buf(),
        source,
    })?;
    let entries = fs::read_dir(from).map_err(|source| StoreError::Io {
        path: from.to_path_buf(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| StoreError::Io {
            path: from.to_path_buf(),
            source,
        })?;
        let source_path = entry.path();
        let target_path = to.join(entry.file_name());
        let kind = entry.file_type().map_err(|source| StoreError::Io {
            path: source_path.clone(),
            source,
        })?;
        if kind.is_dir() {
            copy_dir(&source_path, &target_path)?;
        } else {
            fs::copy(&source_path, &target_path).map_err(|source| StoreError::Io {
                path: target_path,
                source,
            })?;
        }
    }
    Ok(())
}
