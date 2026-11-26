//! A superfície que o Dart enxerga (ADR-0007).
//!
//! Aqui só há Rust puro: funções e tipos. O código de cola (`frb_generated.rs`
//! e o equivalente em Dart) é gerado por `flutter_rust_bridge_codegen` a partir
//! deste módulo — não se escreve FFI à mão.

use notation_bundle::bundle::{Library, Setlist};
use notation_bundle::chord_naming::{Chord, Note};
use notation_bundle::chordpro::{ChordProParser, Song};
use notation_bundle::harmony::HarmonyEngine;
use notation_bundle::musicxml;

/// O resumo de uma cifra: o que a tela precisa para listar e abrir a música.
#[derive(Debug, Clone)]
pub struct SongSummary {
    /// Título vindo de `{title: ...}`; vazio quando a cifra não declara um.
    pub title: String,
    /// Tom original (`{key: ...}`), quando declarado.
    pub key: Option<String>,
    /// Artista (`{artist: ...}`), quando declarado.
    pub artist: Option<String>,
    /// Compasso (`{time: ...}`), quando declarado.
    pub time: Option<String>,
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
    Ok(summary(&song))
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
    /// Compasso (`{time: ...}`), quando declarado.
    pub time: Option<String>,
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

/// Lê uma cifra MusicXML e devolve o resumo que a interface usa.
///
/// Converte o MusicXML para o modelo interno [`Song`] e extrai o mesmo
/// resumo de [`parse_song`].
///
/// # Errors
///
/// Devolve erro quando o XML é inválido ou o formato não é score-partwise.
pub fn parse_musicxml(input: String) -> Result<SongSummary, String> {
    let song = musicxml::parse(&input)?;
    Ok(summary(&song))
}

/// Lê um MusicXML e devolve a estrutura completa, linha a linha.
///
/// # Errors
///
/// Devolve erro quando o XML é inválido ou o formato não é score-partwise.
pub fn read_musicxml(input: String) -> Result<SongDetail, String> {
    let song = musicxml::parse(&input)?;
    Ok(detail(&song))
}

/// O portão de publicação da letra: o motor decide, o app obedece.
///
/// A regra mora em `mir_engine::sheet::pode_publicar` — não é copiada aqui. O
/// app pergunta antes de exportar ou de gerar um link de projeção: cifra cuja
/// letra nasceu de extração pelo usuário circula no grupo, não em publicação.
///
/// # Errors
///
/// Devolve o motivo do bloqueio, pronto para a tela mostrar.
pub fn pode_publicar(input: String) -> Result<(), String> {
    if input.trim().is_empty() {
        return Err("cifra vazia: nada a publicar".to_owned());
    }
    let song = ChordProParser::parse(&input)?;
    mir_engine::sheet::pode_publicar(&song)
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
/// Salva uma cifra no diretório da biblioteca local.
///
/// Cria o diretório `base_dir/id/` com `meta.json` e `song.cho`, e **mantém o
/// índice** (`library.json`) que a biblioteca e a agenda leem. O `id` é um
/// identificador estável (slug do título, por exemplo); salvar de novo o mesmo
/// `id` atualiza a obra em vez de duplicá-la no índice.
///
/// # Errors
///
/// Devolve erro quando o diretório não pode ser criado, o JSON não pode
/// ser escrito, o ChordPro não é válido ou o índice existente está corrompido
/// (índice ilegível não é recriado em silêncio: recriá-lo apagaria as obras que
/// ele lista).
pub fn salvar_na_biblioteca(base_dir: String, id: String, chordpro: String) -> Result<(), String> {
    let song = ChordProParser::parse(&chordpro).map_err(|e| format!("ChordPro inválido: {e}"))?;
    let song_id =
        notation_bundle::bundle::SongId::new(&id).map_err(|e| format!("id inválido: {e}"))?;
    let bundle = notation_bundle::bundle::SongBundle::new(song_id.clone(), song)
        .map_err(|e| format!("bundle inválido: {e}"))?;

    let path: &std::path::Path = base_dir.as_ref();
    notation_bundle::bundle_store::save_bundle(&path.join(&id), &bundle, None)
        .map_err(|e| format!("erro ao salvar: {e}"))?;

    // Sem este passo o app salva a obra e a lista continua dizendo que não há
    // índice nenhum: `listarBiblioteca`/`listarAgenda` respondem
    // "missing file: .../library.json" (medido com o cdylib real).
    let mut library = indice_atual(path)?;
    library.remove(&song_id);
    library
        .add(bundle)
        .map_err(|e| format!("erro ao indexar a obra: {e}"))?;
    notation_bundle::bundle_store::save_library(path, &library)
        .map_err(|e| format!("erro ao salvar o índice: {e}"))?;
    Ok(())
}

/// Lê o índice da biblioteca tratando **apenas** o índice ausente como vazio.
///
/// A primeira gravação acontece num diretório sem `library.json`, então ausência
/// ali é o caso normal. Qualquer outro erro — JSON corrompido, `schema_version`
/// desconhecido, bundle que o índice lista e não existe — sobe como erro: tratar
/// isso como "biblioteca vazia" recriaria o índice por cima e perderia as obras.
fn indice_atual(dir: &std::path::Path) -> Result<Library, String> {
    let ausente = notation_bundle::bundle_store::LIBRARY_FILE;
    match notation_bundle::bundle_store::load_library(dir) {
        Ok(library) => Ok(library),
        Err(notation_bundle::bundle_store::StoreError::MissingFile { path })
            if path == dir.join(ausente) =>
        {
            Ok(Library::new())
        }
        Err(erro) => Err(format!(
            "erro ao ler a biblioteca: {erro} ({})",
            erro.message_key()
        )),
    }
}

/// Carrega uma cifra da biblioteca local e devolve o ChordPro.
///
/// # Errors
///
/// Devolve erro quando o diretório não existe ou o bundle está corrompido.
pub fn carregar_da_biblioteca(base_dir: String, id: String) -> Result<String, String> {
    let path: &std::path::Path = base_dir.as_ref();
    let bundle = notation_bundle::bundle_store::load_bundle(&path.join(&id))
        .map_err(|e| format!("erro ao carregar: {e}"))?;
    Ok(notation_bundle::chordpro::write(bundle.song()))
}

/// Lista os IDs das músicas na biblioteca local.
///
/// Lê o arquivo `library.json` no diretório base.
///
/// # Errors
///
/// Devolve erro quando o índice não existe ou está corrompido.
pub fn listar_biblioteca(base_dir: String) -> Result<Vec<SongSummary>, String> {
    let path: &std::path::Path = base_dir.as_ref();
    let library = notation_bundle::bundle_store::load_library(path)
        .map_err(|e| format!("erro ao listar: {e}"))?;

    Ok(library
        .iter()
        .map(|bundle| summary(bundle.song()))
        .collect())
}

/// Remove uma cifra da biblioteca local.
///
/// Recusa a remoção enquanto uma agenda citar a obra: apagar o bundle deixaria
/// `load_setlists` falhando alto e a agenda inteira ilegível. O índice é
/// reescrito sem a obra, senão a leitura seguinte acusa bundle ausente.
///
/// # Errors
///
/// Devolve erro quando a obra é usada por um evento, quando o índice não pode
/// ser lido ou quando o diretório não pode ser removido.
pub fn remover_da_biblioteca(base_dir: String, id: String) -> Result<(), String> {
    let path: &std::path::Path = base_dir.as_ref();
    let song_id =
        notation_bundle::bundle::SongId::new(&id).map_err(|e| format!("id inválido: {e}"))?;

    let mut library = indice_atual(path)?;
    let agendas = notation_bundle::bundle_store::load_setlists(path, &library).map_err(|erro| {
        format!(
            "erro ao ler a agenda em {}: {erro} ({})",
            path.join("setlists").display(),
            erro.message_key()
        )
    })?;
    if let Some(evento) = agendas.iter().find(|agenda| {
        agenda
            .entries()
            .iter()
            .any(|execucao| execucao.song() == &song_id)
    }) {
        return Err(format!(
            "obra usada na agenda: {} — remova a execução do evento primeiro",
            evento.name()
        ));
    }

    let dir = path.join(&id);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).map_err(|e| format!("erro ao remover: {e}"))?;
    }
    library.remove(&song_id);
    notation_bundle::bundle_store::save_library(path, &library)
        .map_err(|e| format!("erro ao salvar o índice: {e}"))?;
    Ok(())
}

/// Uma execução da agenda: a obra **e a versão tocada naquele evento**.
///
/// A obra vem identificada pelo `song_id` (não pelo título): é esse identificador
/// que a tela passa de volta à biblioteca para abrir a cifra. O título acompanha
/// porque a lista da agenda o mostra na ordem do evento.
#[derive(Debug, Clone)]
pub struct ExecucaoAgenda {
    /// Identificador da obra na biblioteca — por ele a tela abre a cifra.
    pub song_id: String,
    /// Título da obra, lido do bundle correspondente.
    pub titulo: String,
    /// Versão declarada para o evento (arranjo, gravação), quando existe.
    pub versao: Option<String>,
    /// Tom declarado para o evento, quando existe. É o tom **daquele culto**,
    /// não o tom original da cifra.
    pub tom: Option<String>,
}

/// Um evento da agenda: a playlist do culto, do ensaio ou do evento.
#[derive(Debug, Clone)]
pub struct EventoAgenda {
    /// Identificador do evento, como declarado na agenda.
    pub id: String,
    /// Nome do evento.
    pub nome: String,
    /// Data do evento (`AAAA-MM-DD`), quando declarada.
    pub data: Option<String>,
    /// Tipo do evento (`culto`, `ensaio`, `evento`), quando declarado.
    pub tipo: Option<String>,
    /// As execuções na ordem da agenda. O **número de músicas é
    /// `execucoes.len()`** — a contagem não é um campo à parte para não poder
    /// divergir da lista que a tela desenha.
    pub execucoes: Vec<ExecucaoAgenda>,
    /// Qual núcleo respondeu, no formato `kentron_ffi <versão>`.
    pub core: String,
}

/// Lista a agenda local: os eventos e as execuções de cada um.
///
/// Lê a biblioteca (`library.json` + um diretório por obra) e as agendas
/// (`setlists/*.json`) do diretório base, reusando `notation_bundle` — a ponte
/// não reconstrói o modelo. A ordem dos eventos e das músicas é a do núcleo
/// (nome do arquivo da agenda; ordem das execuções dentro dela).
///
/// # Errors
///
/// Falha — em vez de devolver lista vazia — quando o diretório base não existe,
/// quando a biblioteca não pode ser lida (índice ausente, JSON corrompido,
/// `schema_version` não suportado) ou quando uma agenda é ilegível ou aponta
/// para obra fora da biblioteca. **Lista vazia significa, e só significa,
/// biblioteca sem agenda**: é o estado "nenhum evento ainda", não um erro
/// disfarçado.
pub fn listar_agenda(base_dir: String) -> Result<Vec<EventoAgenda>, String> {
    let path: &std::path::Path = base_dir.as_ref();
    if !path.is_dir() {
        return Err(format!(
            "diretório da biblioteca não existe: {base_dir} — nada para listar"
        ));
    }

    let library = notation_bundle::bundle_store::load_library(path)
        .map_err(|erro| format!("erro ao ler a biblioteca: {erro} ({})", erro.message_key()))?;
    // `load_setlists` já recusa qualquer agenda ilegível ou pendurada; o `map_err`
    // acrescenta onde a agenda foi procurada, o que o erro do núcleo não traz (o
    // `StoreError::Bundle` de uma execução pendurada não carrega caminho).
    let agendas = notation_bundle::bundle_store::load_setlists(path, &library).map_err(|erro| {
        format!(
            "erro ao ler a agenda em {}: {erro} ({})",
            path.join("setlists").display(),
            erro.message_key()
        )
    })?;

    agendas
        .iter()
        .map(|agenda| evento(agenda, &library))
        .collect()
}

fn evento(agenda: &Setlist, library: &Library) -> Result<EventoAgenda, String> {
    // `Setlist::songs` falha alto quando a agenda aponta para obra ausente; o
    // erro nomeia o evento para a tela não precisar adivinhar qual quebrou.
    let obras = agenda
        .songs(library)
        .map_err(|erro| format!("agenda {}: {erro}", agenda.id().as_str()))?;

    let execucoes = agenda
        .entries()
        .iter()
        .zip(obras)
        .map(|(entrada, obra)| ExecucaoAgenda {
            song_id: entrada.song().as_str().to_owned(),
            titulo: obra.title().to_owned(),
            versao: entrada.versao().map(str::to_owned),
            tom: entrada.tom().map(str::to_owned),
        })
        .collect();

    Ok(EventoAgenda {
        id: agenda.id().as_str().to_owned(),
        nome: agenda.name().to_owned(),
        data: agenda.data().map(str::to_owned),
        tipo: agenda.tipo().map(str::to_owned),
        execucoes,
        core: format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
    })
}

fn summary(song: &Song) -> SongSummary {
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

    SongSummary {
        title: song.metadata.get("title").cloned().unwrap_or_default(),
        key: song.metadata.get("key").cloned(),
        artist: song.metadata.get("artist").cloned(),
        time: song.metadata.get("time").cloned(),
        sections: song.sections.len() as u32,
        chords,
        core: format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
    }
}

fn detail(song: &Song) -> SongDetail {
    SongDetail {
        title: song.metadata.get("title").cloned().unwrap_or_default(),
        key: song.metadata.get("key").cloned(),
        artist: song.metadata.get("artist").cloned(),
        time: song.metadata.get("time").cloned(),
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
    fn blocks_publication_of_private_scope_lyrics() {
        // Escopo `grupo` é a letra extraída pelo usuário: privada por desenho.
        let privada = "{title: X}\n{lyrics_scope: grupo}\n[C]texto\n";
        let erro = pode_publicar(privada.to_owned()).expect_err("escopo privado deve bloquear");
        assert!(erro.contains("privado"), "o motivo nomeia o escopo: {erro}");
    }

    #[test]
    fn allows_publication_of_public_scope_lyrics() {
        let publica = "{title: X}\n{lyrics_scope: público}\n[C]texto\n";
        assert!(pode_publicar(publica.to_owned()).is_ok());
    }

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

    // ── Agenda: a leitura passa pelo armazenamento de verdade ──────────────
    //
    // Cada teste monta uma biblioteca real num diretório único do temp do
    // sistema (índice + bundles + arquivos de agenda), chama a função da ponte e
    // confere o que voltou. Não há dublê de armazenamento: o que se prova é o
    // caminho `bridge → notation_bundle → disco`.

    use notation_bundle::bundle::{Execucao, SetlistId, SongBundle, SongId};
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU32, Ordering};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    /// Diretório único sob o temp do sistema, removido ao cair.
    struct Scratch(PathBuf);

    impl Scratch {
        fn new(name: &str) -> Self {
            let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "kentron-ffi-{}-{unique}-{name}",
                std::process::id()
            ));
            std::fs::create_dir_all(&path).unwrap_or_else(|erro| panic!("scratch: {erro}"));
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }

        fn base_dir(&self) -> String {
            self.0.to_string_lossy().into_owned()
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// Escreve uma biblioteca de verdade: os bundles e o índice `library.json`.
    fn biblioteca(dir: &Path) {
        let obras = [
            ("santo", "Santo"),
            ("hino-3", "Hino 3"),
            ("arranjo-coral", "Santo (arranjo do coral)"),
        ];
        let mut library = Library::new();
        for (id, titulo) in obras {
            let fonte = format!("{{title: {titulo}}}\n{{key: C}}\n\n[Verse]\n[C]texto\n");
            let song = ChordProParser::parse(&fonte).expect("fixture deve parsear");
            let bundle = SongBundle::new(SongId::new(id).expect("id válido"), song)
                .expect("fixture tem título");
            notation_bundle::bundle_store::save_bundle(&dir.join(id), &bundle, None)
                .expect("salvar bundle");
            library.add(bundle).expect("biblioteca sem duplicata");
        }
        notation_bundle::bundle_store::save_library(dir, &library).expect("salvar índice");
    }

    /// Escreve uma agenda de verdade (`<dir>/setlists/<arquivo>`).
    fn agenda(dir: &Path, arquivo: &str, setlist: &Setlist) {
        let agenda_dir = dir.join("setlists");
        std::fs::create_dir_all(&agenda_dir).expect("criar setlists/");
        notation_bundle::bundle_store::save_setlist(&agenda_dir.join(arquivo), setlist)
            .expect("salvar agenda");
    }

    fn nova_agenda(id: &str, nome: &str) -> Setlist {
        Setlist::new(SetlistId::new(id).expect("id da agenda"), nome).expect("nome da agenda")
    }

    #[test]
    fn lists_every_event_with_titles_in_event_order() {
        let scratch = Scratch::new("agenda-cheia");
        biblioteca(scratch.path());
        let library =
            notation_bundle::bundle_store::load_library(scratch.path()).expect("biblioteca");

        let mut culto = nova_agenda("culto-1", "Culto da manhã")
            .com_data("2026-11-22")
            .com_tipo("culto");
        culto
            .push(&library, SongId::new("santo").expect("id"))
            .expect("primeira execução");
        culto
            .push_execucao(
                &library,
                Execucao::new(SongId::new("arranjo-coral").expect("id"))
                    .com_versao("versão do coral")
                    .com_tom("D"),
            )
            .expect("segunda execução");
        agenda(scratch.path(), "culto-1.json", &culto);

        let mut ensaio = nova_agenda("ensaio-0", "Ensaio de quinta").com_tipo("ensaio");
        ensaio
            .push(&library, SongId::new("hino-3").expect("id"))
            .expect("execução");
        agenda(scratch.path(), "ensaio-0.json", &ensaio);

        let eventos = listar_agenda(scratch.base_dir()).expect("listar agenda");

        assert_eq!(eventos.len(), 2, "duas agendas no disco");
        // Ordem do núcleo: nome do arquivo — `culto-1.json` antes de `ensaio-0.json`.
        assert_eq!(eventos[0].id, "culto-1");
        assert_eq!(eventos[0].nome, "Culto da manhã");
        assert_eq!(eventos[0].data.as_deref(), Some("2026-11-22"));
        assert_eq!(eventos[0].tipo.as_deref(), Some("culto"));
        assert_eq!(eventos[0].execucoes.len(), 2);

        let santo = &eventos[0].execucoes[0];
        assert_eq!(santo.song_id, "santo");
        assert_eq!(santo.titulo, "Santo");
        assert_eq!(santo.versao, None, "versão não declarada não é inventada");
        assert_eq!(santo.tom, None);

        let coral = &eventos[0].execucoes[1];
        assert_eq!(coral.song_id, "arranjo-coral");
        assert_eq!(coral.titulo, "Santo (arranjo do coral)");
        assert_eq!(coral.versao.as_deref(), Some("versão do coral"));
        assert_eq!(coral.tom.as_deref(), Some("D"));

        assert!(
            eventos[0].core.starts_with("kentron_ffi "),
            "{}",
            eventos[0].core
        );

        assert_eq!(eventos[1].id, "ensaio-0");
        assert_eq!(eventos[1].data, None, "ensaio sem data declarada");
        assert_eq!(eventos[1].execucoes[0].song_id, "hino-3");
        assert_eq!(eventos[1].execucoes[0].titulo, "Hino 3");
    }

    #[test]
    fn a_library_without_agenda_is_zero_events_and_not_an_error() {
        let scratch = Scratch::new("agenda-ausente");
        biblioteca(scratch.path());

        let eventos = listar_agenda(scratch.base_dir()).expect("sem agenda não é erro");
        assert!(eventos.is_empty(), "nenhum evento, e nenhum erro");
    }

    // ── A gravação mantém o índice ─────────────────────────────────────────
    //
    // Medido antes da correção: `salvar_na_biblioteca` deixava só
    // `<base>/<id>/meta.json` e `<base>/<id>/song.cho`, e a leitura seguinte
    // respondia "missing file: .../library.json". Sem índice, o app não lista o
    // que ele mesmo salvou — nem a biblioteca, nem a agenda.

    #[test]
    fn saving_keeps_the_index_the_readers_use() {
        let scratch = Scratch::new("indice-apos-salvar");
        let base = scratch.base_dir();

        salvar_na_biblioteca(
            base.clone(),
            "santo".to_owned(),
            "{title: Santo}\n{key: C}\n\n[Verse]\n[C]texto\n".to_owned(),
        )
        .expect("salvar a primeira obra");
        salvar_na_biblioteca(
            base.clone(),
            "hino-3".to_owned(),
            "{title: Hino 3}\n{key: G}\n\n[Verse]\n[G]texto\n".to_owned(),
        )
        .expect("salvar a segunda obra");

        assert!(
            scratch.path().join("library.json").is_file(),
            "o índice existe depois de salvar"
        );
        let titulos: Vec<String> = listar_biblioteca(base.clone())
            .expect("listar o que foi salvo")
            .into_iter()
            .map(|musica| musica.title)
            .collect();
        assert_eq!(titulos, ["Hino 3", "Santo"], "ordem por id");

        // E a agenda do mesmo diretório já é legível (sem evento ainda).
        assert!(listar_agenda(base).expect("agenda vazia").is_empty());
    }

    #[test]
    fn saving_the_same_id_updates_the_song_instead_of_duplicating_it() {
        let scratch = Scratch::new("indice-atualizacao");
        let base = scratch.base_dir();

        salvar_na_biblioteca(
            base.clone(),
            "santo".to_owned(),
            "{title: Santo}\n{key: C}\n\n[Verse]\n[C]texto\n".to_owned(),
        )
        .expect("salvar");
        salvar_na_biblioteca(
            base.clone(),
            "santo".to_owned(),
            "{title: Santo}\n{key: D}\n\n[Verse]\n[D]texto novo\n".to_owned(),
        )
        .expect("salvar de novo o mesmo id");

        let musicas = listar_biblioteca(base.clone()).expect("listar");
        assert_eq!(musicas.len(), 1, "o mesmo id não vira duas obras");
        assert_eq!(
            musicas[0].key.as_deref(),
            Some("D"),
            "a cifra foi atualizada"
        );
    }

    #[test]
    fn saving_refuses_a_corrupted_index_instead_of_recreating_it() {
        let scratch = Scratch::new("indice-corrompido");
        std::fs::write(scratch.path().join("library.json"), "{ não é JSON")
            .expect("gravar índice torto");

        let erro = salvar_na_biblioteca(
            scratch.base_dir(),
            "santo".to_owned(),
            "{title: Santo}\n{key: C}\n\n[Verse]\n[C]texto\n".to_owned(),
        )
        .expect_err("índice ilegível não pode ser recriado por cima");
        assert!(erro.contains("biblioteca"), "{erro}");
        assert!(erro.contains("store.json"), "o erro traz a chave: {erro}");
    }

    #[test]
    fn removing_a_song_drops_it_from_the_index() {
        let scratch = Scratch::new("indice-remocao");
        let base = scratch.base_dir();
        for (id, titulo) in [("santo", "Santo"), ("hino-3", "Hino 3")] {
            salvar_na_biblioteca(
                base.clone(),
                id.to_owned(),
                format!("{{title: {titulo}}}\n{{key: C}}\n\n[Verse]\n[C]texto\n"),
            )
            .expect("salvar");
        }

        remover_da_biblioteca(base.clone(), "hino-3".to_owned()).expect("remover");

        let titulos: Vec<String> = listar_biblioteca(base)
            .expect("o índice continua legível")
            .into_iter()
            .map(|musica| musica.title)
            .collect();
        assert_eq!(titulos, ["Santo"]);
    }

    #[test]
    fn removing_a_song_used_by_an_event_is_refused() {
        let scratch = Scratch::new("indice-remocao-bloqueada");
        let base = scratch.base_dir();
        salvar_na_biblioteca(
            base.clone(),
            "santo".to_owned(),
            "{title: Santo}\n{key: C}\n\n[Verse]\n[C]texto\n".to_owned(),
        )
        .expect("salvar");

        let library = notation_bundle::bundle_store::load_library(scratch.path())
            .expect("biblioteca com índice");
        let mut culto = nova_agenda("culto-1", "Culto da manhã");
        culto
            .push(&library, SongId::new("santo").expect("id"))
            .expect("execução");
        agenda(scratch.path(), "culto-1.json", &culto);

        let erro = remover_da_biblioteca(base.clone(), "santo".to_owned())
            .expect_err("obra em uso não sai da biblioteca");
        assert!(
            erro.contains("Culto da manhã"),
            "o erro nomeia o evento: {erro}"
        );
        assert!(
            listar_agenda(base).expect("a agenda segue legível").len() == 1,
            "a obra continua na biblioteca e o evento continua lendo"
        );
    }

    #[test]
    fn refuses_an_absent_base_directory_instead_of_returning_an_empty_list() {
        let scratch = Scratch::new("agenda-sem-base");
        let ausente = scratch.path().join("nao-existe");

        let erro = listar_agenda(ausente.to_string_lossy().into_owned())
            .expect_err("pasta inexistente deve falhar");
        assert!(erro.contains("não existe"), "{erro}");
        assert!(
            erro.contains("nao-existe"),
            "o erro nomeia o caminho: {erro}"
        );
    }

    #[test]
    fn refuses_a_library_without_index_instead_of_listing_nothing() {
        let scratch = Scratch::new("agenda-sem-indice");

        // O diretório existe, mas não tem `library.json`: lista vazia seria mentira.
        let erro = listar_agenda(scratch.base_dir()).expect_err("índice ausente deve falhar");
        assert!(erro.contains("biblioteca"), "{erro}");
        assert!(
            erro.contains("library.json"),
            "o erro nomeia o arquivo: {erro}"
        );
    }

    #[test]
    fn refuses_a_corrupted_agenda_file_instead_of_swallowing_it() {
        let scratch = Scratch::new("agenda-corrompida");
        biblioteca(scratch.path());
        let agenda_dir = scratch.path().join("setlists");
        std::fs::create_dir_all(&agenda_dir).expect("criar setlists/");
        std::fs::write(agenda_dir.join("culto-1.json"), "{ isto não é JSON")
            .expect("gravar arquivo torto");

        let erro = listar_agenda(scratch.base_dir()).expect_err("JSON torto deve falhar");
        assert!(erro.contains("agenda"), "{erro}");
        assert!(
            erro.contains("culto-1.json"),
            "o erro nomeia o arquivo: {erro}"
        );
        assert!(erro.contains("store.json"), "o erro traz a chave: {erro}");
    }

    #[test]
    fn refuses_an_agenda_pointing_at_a_song_outside_the_library() {
        let scratch = Scratch::new("agenda-pendurada");
        biblioteca(scratch.path());
        let agenda_dir = scratch.path().join("setlists");
        std::fs::create_dir_all(&agenda_dir).expect("criar setlists/");
        std::fs::write(
            agenda_dir.join("culto-1.json"),
            r#"{"schema_version":2,"id":"culto-1","name":"Culto","data":"2026-11-22",
                "tipo":"culto","entries":[{"song":"fantasma","versao":null,"tom":null}]}"#,
        )
        .expect("gravar agenda com obra fora da biblioteca");

        // A entrada pendurada não pode virar uma agenda mais curta e silenciosa.
        // O erro do núcleo nomeia a obra e a chave estável; a ponte acrescenta
        // onde procurou. O arquivo da agenda não entra na mensagem — o
        // `StoreError` de uma execução pendurada não carrega caminho.
        let erro = listar_agenda(scratch.base_dir()).expect_err("obra ausente deve falhar");
        assert!(erro.contains("fantasma"), "o erro nomeia a obra: {erro}");
        assert!(
            erro.contains("setlist.song.missing"),
            "o erro traz a chave estável: {erro}"
        );
        assert!(
            erro.contains("setlists"),
            "o erro diz onde a agenda foi procurada: {erro}"
        );
    }
}
