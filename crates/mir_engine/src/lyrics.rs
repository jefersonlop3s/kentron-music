//! Letra: fonte com procedência, corpus de domínio público e alinhamento ao áudio.
//!
//! **Não existe fonte oficial de letra gratuita** (ADR-0012): provedor licenciado
//! é pago e proíbe armazenar o texto. O que entra aqui é o que é barato e
//! legítimo — texto **do próprio usuário** e **obra em domínio público** (corpus
//! embutido, só obra pré-1900, com procedência linha a linha). Transcrição
//! automática (ASR) está declarada e **não implementada**: exige peso no
//! `models/MANIFEST.toml`.
//!
//! O alinhamento é o "cruzar com o que o áudio apresenta": frases vocais são
//! achadas por energia na banda da voz, as linhas são distribuídas por elas pela
//! ordem e pelo tamanho, e cada linha volta com **confiança** — alinhamento por
//! frase é aproximado, não é fonema a fonema.
//!
//! **Ordem cantada (o GAP das repetições).** A cifra/letra de qualquer fonte traz
//! as linhas **distintas**, e a gravação **repete seções**: com as linhas
//! distintas e sem repetição, o motor alinha o refrão a um trecho onde ele não é
//! cantado (medido em 2026-11-25: refrão na introdução instrumental, 0–27,6 s).
//! O texto da letra pode declarar a repetição, e o motor **expande** as linhas
//! para a ordem cantada antes de alinhar — vocabulário em [`do_usuario`].

use serde::Serialize;

use crate::report::Report;
use crate::spectrum::Spectrogram;

/// Corpus embutido (ADR-0012): obra pré-1900 com procedência declarada no
/// próprio arquivo. Nada entra sem autor, ano e base.
const CORPUS_DOMINIO_PUBLICO: &str = include_str!("../assets/hinos_dominio_publico.tsv");

/// Corte da regra norte-americana em 2026: obra **publicada antes** deste ano
/// está em domínio público (regra dos 95 anos rolantes).
const PRIMEIRO_ANO_PROTEGIDO_EUA: u16 = 1930;

/// Onde a pergunta está sendo feita. "Domínio público" sem jurisdição é meia
/// resposta: Brasil e EUA decidem por regras diferentes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Jurisdicao {
    Brasil,
    Eua,
}

impl Jurisdicao {
    #[must_use]
    pub fn nome(self) -> &'static str {
        match self {
            Self::Brasil => "Brasil",
            Self::Eua => "EUA",
        }
    }
}

/// A **obra** (letra e música): protegida por si, independente de quem gravou.
///
/// A ficha é o que sustenta o veredito — sem ela o motor **não afirma** domínio
/// público (fail-loud). `adaptador` existe porque **tradução e adaptação com
/// frases novas são obra própria**: hinário de 1922 pode conter tradução de 1980,
/// e "Tremenda Graça" é adaptação de frases novas sobre música antiga. A obra
/// derivada é julgada **pelo autor dela**.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Obra {
    pub titulo: String,
    pub autor: String,
    pub ano: u16,
    /// Ano de falecimento do autor: é o que decide vida+70 no Brasil.
    pub autor_morte: Option<u16>,
    /// Autor da tradução ou da adaptação, quando a letra é derivada.
    pub adaptador: Option<String>,
    /// Ano de falecimento do adaptador, quando há adaptação.
    pub adaptador_morte: Option<u16>,
    /// Obra de que esta deriva ("mesma música, outro título e outras frases").
    /// É referência, não fusão: o veredito de uma **não** empresta para a outra.
    pub obra_original: Option<String>,
    /// Procedência da ficha: edição, hinário, partitura.
    pub base: String,
}

/// O **fonograma**: a gravação analisada.
///
/// Tem vida própria e por isso **não carrega veredito de domínio público** — a
/// categoria não existe do lado da gravação. Uma gravação de ontem de obra de
/// 1822 continua protegida; este tipo existe para o app dizer de quem é o áudio,
/// em vez de tratar obra e gravação como a mesma coisa.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Fonograma {
    pub descricao: String,
    pub ano: Option<u16>,
    pub titular: Option<String>,
}

/// Veredito sobre a obra — três estados, porque "não sei" não é "pode".
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Veredito {
    DominioPublico { regra: String },
    Protegido { regra: String },
    Indeterminado { motivo: String },
}

impl Veredito {
    /// Texto curto para a cifra e para o relatório.
    #[must_use]
    pub fn resumo(&self) -> String {
        match self {
            Self::DominioPublico { regra } => format!("domínio público ({regra})"),
            Self::Protegido { regra } => format!("protegido ({regra})"),
            Self::Indeterminado { motivo } => format!("indeterminado — {motivo}"),
        }
    }

    #[must_use]
    pub fn eh_dominio_publico(&self) -> bool {
        matches!(self, Self::DominioPublico { .. })
    }
}

/// Onde a cifra pode circular. Extração feita pelo usuário nasce **privada**
/// (ADR-0012): o que publica é a licença coletiva do usuário, não o app.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Escopo {
    Grupo,
    Publico,
}

impl Escopo {
    #[must_use]
    pub fn nome(self) -> &'static str {
        match self {
            Self::Grupo => "grupo",
            Self::Publico => "público",
        }
    }
}

/// Calcula o veredito da obra numa jurisdição. `ano_corrente` entra por parâmetro
/// para o teste ser determinístico e para a regra rolante dos EUA ser explícita.
///
/// Não existe atalho: sem ano de falecimento (ou, na tradução, sem o do tradutor)
/// o resultado é `Indeterminado`. Chutar "domínio público" sem ficha seria o pior
/// erro possível nesta função.
#[must_use]
pub fn veredito_pd(obra: &Obra, jurisdicao: Jurisdicao, ano_corrente: u16) -> Veredito {
    // Tradução e adaptação são obra derivada: quem manda é o autor **dela**, não
    // o autor do original. PD + adaptação recente = adaptação protegida.
    let (nome, morte) = match &obra.adaptador {
        Some(adaptador) => (adaptador.clone(), obra.adaptador_morte),
        None => (obra.autor.clone(), obra.autor_morte),
    };

    match jurisdicao {
        Jurisdicao::Brasil => match morte {
            // Lei 9.610/98, art. 41: 70 anos contados de 1º de janeiro do ano
            // seguinte ao falecimento — livre a partir de morte + 71.
            Some(ano_morte) => {
                let regra = format!("vida de {nome} ({ano_morte}) + 70 anos");
                if ano_corrente >= ano_morte.saturating_add(71) {
                    Veredito::DominioPublico { regra }
                } else {
                    Veredito::Protegido { regra }
                }
            }
            None => Veredito::Indeterminado {
                motivo: format!(
                    "sem ano de falecimento de {nome}: vida+70 não se calcula e o motor não chuta"
                ),
            },
        },
        Jurisdicao::Eua => {
            if obra.ano == 0 {
                return Veredito::Indeterminado {
                    motivo: "sem ano de publicação".to_owned(),
                };
            }
            if obra.adaptador.is_some() {
                // Nos EUA quem decide é a data de publicação **da adaptação**, que
                // a ficha não carrega separada do original — e o original pode ser
                // um século mais antigo.
                return Veredito::Indeterminado {
                    motivo: "obra derivada: falta o ano de publicação da adaptação".to_owned(),
                };
            }
            if obra.ano < PRIMEIRO_ANO_PROTEGIDO_EUA {
                Veredito::DominioPublico {
                    regra: format!(
                        "publicado em {} (antes de {PRIMEIRO_ANO_PROTEGIDO_EUA})",
                        obra.ano
                    ),
                }
            } else {
                Veredito::Protegido {
                    regra: format!(
                        "publicado em {} (a partir de {PRIMEIRO_ANO_PROTEGIDO_EUA})",
                        obra.ano
                    ),
                }
            }
        }
    }
}

/// De onde veio a letra. O **ato tem dono** — e não existe variante para extração
/// feita *pelo produto*: ela exigiria licença e decisão própria (ADR-0012), e um
/// enum que a aceitasse seria convite a usá-la.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Origem {
    /// Texto fornecido pelo próprio usuário (biblioteca, transcrição dele, ou
    /// sob licença que ele declara).
    DoUsuario { licenca: Option<String> },
    /// Corpus de domínio público, com a ficha que sustenta o veredito.
    DominioPublico { obra: Obra },
    /// Letra **extraída do áudio pelo usuário**: o ato é dele e a cifra nasce
    /// privada no grupo. `obra` continua bem-vinda: extrair não transfere direito
    /// nem torna a obra pública.
    ExtraidaPeloUsuario {
        responsavel: String,
        obra: Option<Obra>,
    },
}

impl Origem {
    /// Onde esta cifra pode circular.
    #[must_use]
    pub fn escopo(&self) -> Escopo {
        match self {
            Self::ExtraidaPeloUsuario { .. } => Escopo::Grupo,
            Self::DoUsuario { .. } | Self::DominioPublico { .. } => Escopo::Publico,
        }
    }

    /// Descrição legível, para o relatório e para a cifra.
    #[must_use]
    pub fn descricao(&self) -> String {
        match self {
            Self::DoUsuario {
                licenca: Some(licenca),
            } => format!("fornecida pelo usuário; licença declarada: {licenca}"),
            Self::DoUsuario { licenca: None } => "fornecida pelo usuário".to_owned(),
            Self::DominioPublico { obra } => format!(
                "domínio público — {}, {} ({})",
                obra.titulo, obra.autor, obra.ano
            ),
            Self::ExtraidaPeloUsuario { responsavel, .. } => {
                format!("extraída do áudio por {responsavel} (circulação privada no grupo)")
            }
        }
    }
}

/// Letra com procedência declarada.
#[derive(Debug, Clone, Serialize)]
pub struct Letra {
    pub origem: Origem,
    /// Linhas de letra **na ordem cantada** (repetições já expandidas), uma por
    /// vez que a linha é cantada. Linha vazia não entra.
    pub linhas: Vec<String>,
}

/// Lê letra de texto livre ou de ChordPro, com licença declarada opcional.
///
/// Em ChordPro, o que está entre colchetes é acorde: sai. Metadados (`{...}`),
/// comentários (`#`) e linhas vazias também saem — sobra o texto cantado. As
/// diretivas de **ordem cantada** expandem a letra, na ordem em que aparecem:
///
/// ```text
/// {bloco refrao}        # abre um bloco nomeado (não aninha)
/// Todo ser que vive
/// Louve ao nome do Senhor
/// {fim}                 # fecha, e o bloco é cantado AQUI (uma vez)
/// {bloco verso}
/// E mesmo sendo pó
/// {fim}
/// {canta refrao}        # o refrão outra vez, uma vez
/// {canta refrao 3x}     # o refrão três vezes seguidas (1x–99x)
/// Teu santo nome (2x)   # sufixo na linha: esta linha, duas vezes
/// ```
///
/// O bloco é cantado **onde é definido** (como o `{soc}` do ChordPro) e cantado
/// de novo em cada `{canta nome}`. O vocabulário é do motor: qualquer outra
/// diretiva `{...}` continua sendo metadado (`{title: …}`, `{time: 4/4}`,
/// `{soc}`) e é ignorada.
///
/// Fail-loud: `{canta}` de bloco antes de ele ser definido, `{fim}` sem
/// `{bloco …}` aberto, nome de bloco repetido, `{bloco}` sem nome, bloco sem
/// linha, `{canta …}` dentro de um bloco e repetição fora de `1x–99x`
/// **reprovam** — ignorar em silêncio devolveria a letra com menos linhas que o
/// canto, que é exatamente o defeito que este vocabulário conserta.
///
/// # Errors
///
/// [`ErroDeLetra`] com a linha exata do texto e o motivo.
pub fn do_usuario(texto: &str, licenca: Option<String>) -> Result<Letra, ErroDeLetra> {
    Ok(Letra {
        origem: Origem::DoUsuario { licenca },
        linhas: ordem_cantada(texto)?,
    })
}

/// Lê letra do usuário sem declarar licença.
///
/// # Errors
///
/// Como [`do_usuario`].
pub fn ler_do_usuario(texto: &str) -> Result<Letra, ErroDeLetra> {
    do_usuario(texto, None)
}

/// Erro de leitura da letra: diretiva de ordem cantada malformada.
///
/// Existe para o erro ser **nomeado no arquivo e na linha**: a letra vem do
/// usuário e uma repetição mal escrita é erro dele, não do motor — o motor não
/// adivinha onde o refrão volta.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErroDeLetra {
    /// Linha do texto (1-based) onde o problema está.
    pub linha: usize,
    /// O que está errado, em texto para o usuário.
    pub motivo: String,
}

impl std::fmt::Display for ErroDeLetra {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "linha {}: {}", self.linha, self.motivo)
    }
}

impl std::error::Error for ErroDeLetra {}

/// Diretiva de ordem cantada que o motor entende.
enum Diretiva {
    /// `{bloco nome}` — abre um bloco nomeado.
    AbreBloco(String),
    /// `{fim}` — fecha o bloco aberto.
    FechaBloco,
    /// `{canta nome [Nx]}` — canta um bloco já definido.
    Canta { nome: String, vezes: usize },
}

/// Remove trechos entre colchetes (acordes e nomes de seção).
fn sem_colchetes(linha: &str) -> String {
    let mut saida = String::with_capacity(linha.len());
    let mut dentro = false;
    for caractere in linha.chars() {
        match caractere {
            '[' => dentro = true,
            ']' => dentro = false,
            _ if !dentro => saida.push(caractere),
            _ => {}
        }
    }
    saida
}

/// Lê uma diretiva `{...}`. `Ok(None)` = não é deste vocabulário (metadado de
/// ChordPro, que continua sendo ignorado).
fn ler_diretiva(conteudo: &str) -> Result<Option<Diretiva>, String> {
    let conteudo = conteudo.trim();
    let (palavra, resto) = conteudo
        .split_once(char::is_whitespace)
        .map_or((conteudo, ""), |(primeira, resto)| (primeira, resto.trim()));
    match palavra {
        "bloco" => {
            if resto.is_empty() {
                return Err("`{bloco}` sem nome — escreva `{bloco nome}`".to_owned());
            }
            if resto.split_whitespace().count() > 1 {
                return Err(format!(
                    "`{{bloco …}}` com nome composto ({resto:?}): o nome é uma palavra só"
                ));
            }
            Ok(Some(Diretiva::AbreBloco(resto.to_owned())))
        }
        "fim" => {
            if resto.is_empty() {
                Ok(Some(Diretiva::FechaBloco))
            } else {
                Err(format!("`{{fim}}` com sobra ({resto:?})"))
            }
        }
        "canta" => {
            let mut partes = resto.split_whitespace();
            let Some(nome) = partes.next() else {
                return Err(
                    "`{canta}` sem nome — escreva `{canta nome}` ou `{canta nome 3x}`".to_owned(),
                );
            };
            let vezes = match partes.next() {
                None => 1,
                Some(bruto) => vezes_marcadas(bruto)
                    .map_err(|motivo| format!("`{{canta {nome} {bruto}}}`: {motivo}"))?,
            };
            if let Some(sobra) = partes.next() {
                return Err(format!("`{{canta …}}` com sobra ({sobra:?})"));
            }
            Ok(Some(Diretiva::Canta {
                nome: nome.to_owned(),
                vezes,
            }))
        }
        _ => Ok(None),
    }
}

/// Lê `Nx` (ex.: `3x`) e devolve N. Fora do formato ou fora de `1x–99x`: erro.
fn vezes_marcadas(bruto: &str) -> Result<usize, String> {
    let Some(numero) = bruto.strip_suffix('x') else {
        return Err(format!("repetição fora do formato `Nx` ({bruto:?})"));
    };
    let vezes: usize = numero
        .parse()
        .map_err(|_| format!("repetição não numérica ({bruto:?})"))?;
    if vezes == 0 || vezes > 99 {
        return Err(format!("repetição {bruto} fora da faixa 1x–99x"));
    }
    Ok(vezes)
}

/// Separa o sufixo `(Nx)` do fim da linha: a linha é cantada `N` vezes seguidas.
///
/// Só o formato exato `(Nx)` é marcador — `(Aleluia)` e `(x)` ficam no texto,
/// porque parêntese é coisa que a letra tem de verdade.
fn separar_sufixo_de_repeticao(linha: &str) -> Result<(String, usize), String> {
    let limpa = linha.trim_end();
    if !limpa.ends_with(')') {
        return Ok((limpa.to_owned(), 1));
    }
    let Some(abre) = limpa.rfind('(') else {
        return Ok((limpa.to_owned(), 1));
    };
    let dentro = &limpa[abre + 1..limpa.len() - 1];
    // Sem `x` no fim, ou com o número vazio, não é marcador: é parêntese de letra.
    let Some(digitos) = dentro.strip_suffix('x') else {
        return Ok((limpa.to_owned(), 1));
    };
    if digitos.is_empty() || !digitos.chars().all(|c| c.is_ascii_digit()) {
        return Ok((limpa.to_owned(), 1));
    }
    let vezes = vezes_marcadas(dentro)?;
    let texto = limpa[..abre].trim_end().to_owned();
    if texto.is_empty() {
        return Err(format!("`({dentro})` sem linha para repetir"));
    }
    Ok((texto, vezes))
}

/// Expande a letra para a **ordem cantada**, resolvendo as diretivas de
/// repetição. Sem diretiva nenhuma, devolve as linhas como estão — o texto que
/// já vinha na ordem cantada continua valendo.
fn ordem_cantada(texto: &str) -> Result<Vec<String>, ErroDeLetra> {
    let mut blocos: Vec<(String, Vec<String>)> = Vec::new();
    let mut aberto: Option<(String, Vec<String>)> = None;
    let mut saida: Vec<String> = Vec::new();
    for (i, bruta) in texto.lines().enumerate() {
        let numero_da_linha = i + 1;
        let limpa = bruta.trim();
        if limpa.is_empty() || limpa.starts_with('#') {
            continue;
        }
        let erro = |motivo: String| ErroDeLetra {
            linha: numero_da_linha,
            motivo,
        };
        if limpa.starts_with('{') && limpa.ends_with('}') {
            let diretiva = match ler_diretiva(&limpa[1..limpa.len() - 1]) {
                Ok(diretiva) => diretiva,
                Err(motivo) => return Err(erro(motivo)),
            };
            match diretiva {
                None => continue, // metadado de ChordPro
                Some(Diretiva::AbreBloco(nome)) => {
                    if let Some((nome_aberto, _)) = &aberto {
                        return Err(erro(format!(
                            "`{{bloco {nome}}}` dentro do bloco {nome_aberto:?}: blocos não aninham"
                        )));
                    }
                    if blocos.iter().any(|(existente, _)| existente == &nome) {
                        return Err(erro(format!(
                            "bloco {nome:?} definido duas vezes — `{{canta {nome}}}` ficaria ambíguo"
                        )));
                    }
                    aberto = Some((nome, Vec::new()));
                }
                Some(Diretiva::FechaBloco) => {
                    let Some((nome, linhas)) = aberto.take() else {
                        return Err(erro("`{fim}` sem `{bloco …}` aberto".to_owned()));
                    };
                    if linhas.is_empty() {
                        return Err(erro(format!(
                            "bloco {nome:?} sem linha — `{{canta {nome}}}` não teria o que cantar"
                        )));
                    }
                    // O bloco é cantado onde é DEFINIDO, e de novo em cada
                    // `{canta …}`: a definição é posição na música, não só nome.
                    saida.extend(linhas.iter().cloned());
                    blocos.push((nome, linhas));
                }
                Some(Diretiva::Canta { nome, vezes }) => {
                    if aberto.is_some() {
                        return Err(erro(format!(
                            "`{{canta {nome}}}` dentro de um bloco: feche com `{{fim}}` primeiro"
                        )));
                    }
                    let Some((_, linhas)) = blocos.iter().find(|(existente, _)| existente == &nome)
                    else {
                        return Err(erro(format!(
                            "bloco {nome:?} cantado antes de ser definido — defina com `{{bloco {nome}}}`"
                        )));
                    };
                    for _ in 0..vezes {
                        saida.extend(linhas.iter().cloned());
                    }
                }
            }
            continue;
        }
        // Se for só um bloco entre colchetes (nome de seção), não é letra.
        let sem_acordes = sem_colchetes(limpa);
        if sem_acordes.trim().is_empty() {
            continue;
        }
        let (linha_de_letra, vezes) = separar_sufixo_de_repeticao(&sem_acordes).map_err(&erro)?;
        if let Some((_, linhas)) = aberto.as_mut() {
            for _ in 0..vezes {
                linhas.push(linha_de_letra.clone());
            }
        } else {
            for _ in 0..vezes {
                saida.push(linha_de_letra.clone());
            }
        }
    }
    if let Some((nome, _)) = aberto {
        return Err(ErroDeLetra {
            linha: texto.lines().count(),
            motivo: format!("`{{bloco {nome}}}` sem `{{fim}}`"),
        });
    }
    Ok(saida)
}

/// Busca no corpus de domínio público embutido, pelo título.
///
/// Comparação sem diferenciar maiúsculas nem acentos de caixa; aceita prefixo
/// (`{ titulo = "amazing" }`).
#[must_use]
pub fn do_corpus(titulo: &str) -> Option<Letra> {
    let alvo = titulo.trim().to_lowercase();
    for obra in obras_do_corpus() {
        if obra.titulo.to_lowercase().starts_with(&alvo) {
            // Ficha cuja obra não resolva domínio público na jurisdição de
            // referência não é servida como letra — ver `auditar_corpus`.
            let verso = verso_de(&obra.titulo)?;
            return Some(Letra {
                origem: Origem::DominioPublico { obra },
                linhas: verso,
            });
        }
    }
    None
}

/// Lê as fichas do corpus embutido. Uma linha por obra; campo vazio é ausente.
///
/// Formato: `titulo, autor, ano, autor_morte, adaptador, adaptador_morte,
/// obra_original, base, verso`.
fn obras_do_corpus() -> Vec<Obra> {
    CORPUS_DOMINIO_PUBLICO
        .lines()
        .filter(|l| {
            let l = l.trim();
            !l.is_empty() && !l.starts_with('#')
        })
        .filter_map(|linha| {
            let campos: Vec<&str> = linha.split('\t').collect();
            if campos.len() < 9 {
                return None;
            }
            Some(Obra {
                titulo: campos[0].trim().to_owned(),
                autor: campos[1].trim().to_owned(),
                ano: campos[2].trim().parse().unwrap_or(0),
                autor_morte: campos[3].trim().parse().ok(),
                adaptador: Some(campos[4].trim().to_owned()).filter(|t| !t.is_empty()),
                adaptador_morte: campos[5].trim().parse().ok(),
                obra_original: Some(campos[6].trim().to_owned()).filter(|t| !t.is_empty()),
                base: campos[7].trim().to_owned(),
            })
        })
        .collect()
}

/// Linhas da letra de uma obra do corpus.
fn verso_de(titulo: &str) -> Option<Vec<String>> {
    CORPUS_DOMINIO_PUBLICO.lines().find_map(|linha| {
        let campos: Vec<&str> = linha.split('\t').collect();
        if campos.len() < 9 || campos[0].trim() != titulo {
            return None;
        }
        let linhas: Vec<String> = campos[8]
            .split(" | ")
            .map(|l| l.trim().to_owned())
            .filter(|l| !l.is_empty())
            .collect();
        (!linhas.is_empty()).then_some(linhas)
    })
}

/// Auditoria do corpus: cada ficha tem de resolver **domínio público** nas duas
/// jurisdições declaradas, e a letra tem de existir.
///
/// É o verificador do corpus, e roda no teste: ficha incompleta ou tradução sem
/// data do tradutor **reprova** em vez de passar despercebida.
///
/// # Errors
///
/// Devolve a lista de problemas encontrados, um por linha.
pub fn auditar_corpus(ano_corrente: u16) -> Result<Vec<String>, Vec<String>> {
    let mut problemas: Vec<String> = Vec::new();
    let mut relatorio: Vec<String> = Vec::new();
    for obra in obras_do_corpus() {
        if obra.base.is_empty() {
            problemas.push(format!("{}: ficha sem base (procedência)", obra.titulo));
        }
        for jurisdicao in [Jurisdicao::Brasil, Jurisdicao::Eua] {
            let veredito = veredito_pd(&obra, jurisdicao, ano_corrente);
            if !veredito.eh_dominio_publico() {
                problemas.push(format!(
                    "{} ({}): {}",
                    obra.titulo,
                    jurisdicao.nome(),
                    veredito.resumo()
                ));
            } else {
                relatorio.push(format!(
                    "{} ({}): {}",
                    obra.titulo,
                    jurisdicao.nome(),
                    veredito.resumo()
                ));
            }
        }
        if verso_de(&obra.titulo).is_none() {
            problemas.push(format!("{}: ficha sem letra", obra.titulo));
        }
    }
    if problemas.is_empty() {
        Ok(relatorio)
    } else {
        Err(problemas)
    }
}

/// Fichas disponíveis no corpus embutido.
#[must_use]
pub fn fichas_do_corpus() -> Vec<Obra> {
    obras_do_corpus()
}

/// Vogal — inclusive `y`, que em inglês faz núcleo de sílaba ("every", "my").
fn eh_vogal(caractere: char) -> bool {
    matches!(
        caractere.to_lowercase().next(),
        Some(
            'a' | 'e'
                | 'i'
                | 'o'
                | 'u'
                | 'y'
                | 'á'
                | 'é'
                | 'í'
                | 'ó'
                | 'ú'
                | 'â'
                | 'ê'
                | 'ô'
                | 'ã'
                | 'õ'
                | 'à'
                | 'ü'
        )
    )
}

/// Início de cada sílaba, em **índices de caractere**.
///
/// Heurística barata e declarada: núcleo é grupo de vogais, e o início da sílaba
/// recua sobre as consoantes de ataque (é onde o acorde fica na cifra). Não é
/// silabação de dicionário — serve para posicionar acorde, não para cantar.
#[must_use]
pub fn silabas(texto: &str) -> Vec<usize> {
    let caracteres: Vec<char> = texto.chars().collect();
    let mut inicios: Vec<usize> = Vec::new();
    let mut i = 0;
    while i < caracteres.len() {
        if eh_vogal(caracteres[i]) {
            let limite = inicios.last().map_or(0, |&anterior| anterior + 1);
            let mut inicio = i;
            while inicio > limite
                && !eh_vogal(caracteres[inicio - 1])
                && !caracteres[inicio - 1].is_whitespace()
            {
                inicio -= 1;
            }
            // Ataque válido: consoante sozinha, ou o par cujo segundo elemento é
            // `l`, `r` ou `h` (gra, bra, ch, th). Sem isso, "San-to" saía
            // "Sa-nto" — o acorde cairia na consoante errada.
            let quantas_consoantes = i - inicio;
            if quantas_consoantes >= 2 {
                let ultima = caracteres[i - 1].to_lowercase().next();
                let par_valido = matches!(ultima, Some('l' | 'r' | 'h'));
                inicio = if par_valido {
                    (i - 2).max(limite)
                } else {
                    (i - 1).max(limite)
                };
            }
            inicios.push(inicio);
            while i < caracteres.len() && eh_vogal(caracteres[i]) {
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    inicios
}

/// Uma linha da letra posicionada no tempo, com o que o áudio diz dela.
#[derive(Debug, Clone, Serialize)]
pub struct LinhaAlinhada {
    pub indice: usize,
    pub inicio_s: f32,
    pub fim_s: f32,
    pub texto: String,
    /// Rótulo do trecho onde a linha caiu.
    pub trecho: String,
    /// Papel inferido do trecho (refrão candidato, final…).
    pub papel: String,
    /// Loudness local, em LUFS, pela curva de curto prazo.
    pub lufs_medio: f32,
    /// A mesma letra já apareceu antes — é refrão de fato, não rótulo inferido.
    pub repetida: bool,
    /// Quanta confiança merece este alinhamento (0–1). Baixa = conferir.
    pub confianca: f32,
}

/// Média móvel simples.
fn suavizar(serie: &[f32], janela: usize) -> Vec<f32> {
    if janela <= 1 || serie.is_empty() {
        return serie.to_vec();
    }
    let mut saida = Vec::with_capacity(serie.len());
    for i in 0..serie.len() {
        let inicio = i.saturating_sub(janela / 2);
        let fim = (i + janela / 2 + 1).min(serie.len());
        let fatia = &serie[inicio..fim];
        saida.push(fatia.iter().sum::<f32>() / fatia.len() as f32);
    }
    saida
}

/// Funde trechos separados por pausa menor que `pausa_s` (respiro não é fim de frase).
fn fundir(trechos: &mut Vec<(f32, f32)>, pausa_s: f32) {
    let mut fundidos: Vec<(f32, f32)> = Vec::with_capacity(trechos.len());
    for &(inicio, fim) in trechos.iter() {
        match fundidos.last_mut() {
            Some(anterior) if inicio - anterior.1 < pausa_s => anterior.1 = fim,
            _ => fundidos.push((inicio, fim)),
        }
    }
    *trechos = fundidos;
}

/// Frases vocais: onde a voz soa, por energia na banda 200–3500 Hz.
///
/// A referência é o **máximo recente** (4 s, causal), não a média local. Duas
/// medições levaram até aqui:
/// 1. limiar sobre o pico **global** devolveu **uma frase de 385 s em 400** no
///    primeiro áudio real — em mixagem densa a banda da voz nunca cai a 12% do
///    pico;
/// 2. limiar de 1,15× a **média local** (causal, 3 s) não achava o primeiro bloco
///    de um áudio cantado: nível constante nunca fica acima da própria média, e
///    as frases só abriam depois de cada pausa, truncadas em ~2,8 s.
///
/// O que marca presença de voz é **pausa** — a energia cai muito abaixo do
/// máximo recente —, e a frase é o que fica entre pausas.
#[must_use]
pub fn frases_vocais(spectro: &Spectrogram, taxa: u32) -> Vec<(f32, f32)> {
    let energia = spectro.energia_banda(taxa, 200.0, 3500.0);
    if energia.len() < 4 {
        return Vec::new();
    }
    let fps = spectro.quadros_por_segundo(taxa).max(1.0);
    let suave = suavizar(&energia, (0.15 * fps).max(1.0) as usize);
    let pico = suave.iter().copied().fold(0.0f32, f32::max);
    if pico <= f32::EPSILON {
        return Vec::new();
    }
    // Piso por PROEMINENCIA GLOBAL: a frase tem de ficar acima do nivel mediano
    // da propria faixa vocal NO CONJUNTO da musica, nao so acima do maximo local.
    // Sem isso, uma introducao instrumental sustentada abre frase — medido em
    // 2026-11-25: frase falsa em 7,2 s-15,4 s numa introducao sem canto (o canto
    // comeca a 19 s), e as primeiras linhas da letra caiam ~12 s adiantadas. O
    // piso antigo (2% do pico) ficava ABAIXO do nivel da introducao (que esta no
    // percentil 32 da faixa, contra o refrao no 49). Constante medida em UMA
    // musica com gabarito humano — reconferir no repertorio (PENDING).
    let mut ordem = suave.clone();
    ordem.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mediana = ordem[ordem.len() / 2];
    let piso = mediana.max(pico * 0.02);
    // Janela e limiares: medidos nos dois casos que existem (áudio sintetizado com
    // quatro blocos cantados e uma música real). Mais largos, o motor devolve
    // frases longas demais (`16,7 s` onde há verso); mais estreitos, ele picota a
    // voz. O repertório inteiro é o que falta medir (PENDING).
    let janela_referencia = (2.0 * fps).max(1.0) as usize;
    let abrir = 0.65f32;
    let fechar = 0.50f32;
    let mut brutos: Vec<(f32, f32)> = Vec::new();
    let mut dentro: Option<usize> = None;
    for (i, &valor) in suave.iter().enumerate() {
        let inicio_referencia = i.saturating_sub(janela_referencia);
        let referencia = suave[inicio_referencia..=i]
            .iter()
            .copied()
            .fold(piso, f32::max);
        match dentro {
            None if valor >= referencia * abrir => dentro = Some(i),
            Some(inicio) if valor < referencia * fechar => {
                brutos.push((inicio as f32 / fps, i as f32 / fps));
                dentro = None;
            }
            _ => {}
        }
    }
    if let Some(inicio) = dentro {
        brutos.push((inicio as f32 / fps, suave.len() as f32 / fps));
    }
    fundir(&mut brutos, 0.6);
    brutos.retain(|&(inicio, fim)| fim - inicio >= 0.5);
    brutos
}

/// Loudness local pela curva de curto prazo; sem bloco na janela, usa o mais próximo.
fn lufs_local(relatorio: &Report, inicio_s: f32, fim_s: f32) -> f32 {
    let curva = &relatorio.dynamics.curva_curto_prazo;
    let na_janela: Vec<f64> = curva
        .iter()
        .filter(|&&(t, _)| t >= f64::from(inicio_s) && t <= f64::from(fim_s))
        .map(|&(_, l)| l)
        .collect();
    if !na_janela.is_empty() {
        return (na_janela.iter().sum::<f64>() / na_janela.len() as f64) as f32;
    }
    curva
        .iter()
        .min_by(|a, b| {
            (a.0 as f32 - inicio_s)
                .abs()
                .partial_cmp(&(b.0 as f32 - inicio_s).abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map_or(f32::NAN, |&(_, l)| l as f32)
}

/// Alinha as linhas da letra ao áudio.
///
/// As linhas entram nas **frases vocais** na ordem, proporcional ao tempo de cada
/// frase; dentro da frase, o tempo é dividido pelo número de sílabas. Sem frase
/// detectada, a letra é distribuída pela duração inteira e a confiança cai para
/// o piso — o motor prefere dizer "não sei onde" a inventar precisão.
#[must_use]
pub fn alinhar(
    relatorio: &Report,
    letra: &Letra,
    spectro: &Spectrogram,
    taxa: u32,
) -> Vec<LinhaAlinhada> {
    let n = letra.linhas.len();
    if n == 0 {
        return Vec::new();
    }
    let frases = frases_vocais(spectro, taxa);
    let duracao = relatorio.audio.duracao_s;
    let cobertura: f32 = frases.iter().map(|&(inicio, fim)| fim - inicio).sum();
    // Uma frase só cobrindo (quase) a música inteira não diz **onde** cada linha
    // começa: é o mesmo caso de nenhuma frase. Aconteceu com ruído contínuo de
    // banda larga, que o limiar relativo ao pico lê como um bloco único.
    if frases.is_empty() || (frases.len() == 1 && cobertura > 0.85 * duracao) {
        let tudo = [(0.0f32, duracao)];
        return montar_linhas(relatorio, letra, &tudo, &[n], 0.15);
    }
    let pesos: Vec<f32> = frases.iter().map(|&(a, b)| (b - a).max(0.01)).collect();
    let mut quantas: Vec<usize> = vec![0; frases.len()];
    let mut restantes = n;
    for i in 0..frases.len() {
        if restantes == 0 {
            break;
        }
        if i == frases.len() - 1 {
            quantas[i] = restantes;
            break;
        }
        let peso_restante: f32 = pesos[i..].iter().sum();
        let alvo = (pesos[i] / peso_restante * restantes as f32).round() as usize;
        // Toda frase recebe ao menos uma linha, e cada frase seguinte ainda
        // precisa da sua — o resto não pode ser consumido de uma vez.
        let maximo = restantes.saturating_sub(frases.len() - i - 1).max(1);
        // Frase curta demais para caber uma linha (a letra canta ~2-4 s por
        // linha) NAO pode receber uma so por educacao: uma frase falsa de 1,0 s
        // na introducao consumia a primeira linha e empurrava TODAS as demais
        // (medido 2026-11-25). Sem linha, ela simplesmente nao aparece.
        let minimo = usize::from(pesos[i] >= 1.5);
        let escolhidas = alvo.clamp(minimo, maximo.min(restantes));
        quantas[i] = escolhidas;
        restantes -= escolhidas;
    }
    montar_linhas(relatorio, letra, &frases, &quantas, 0.6)
}

/// Distribui as linhas dentro das frases e mede a confiança de cada uma.
fn montar_linhas(
    relatorio: &Report,
    letra: &Letra,
    frases: &[(f32, f32)],
    quantas: &[usize],
    base_confianca: f32,
) -> Vec<LinhaAlinhada> {
    let mut saida: Vec<LinhaAlinhada> = Vec::new();
    let mut proxima = 0usize;
    for (i, &(inicio, fim)) in frases.iter().enumerate() {
        let k = quantas.get(i).copied().unwrap_or(0);
        if k == 0 || proxima >= letra.linhas.len() {
            continue;
        }
        let fim_indice = (proxima + k).min(letra.linhas.len());
        let linhas = &letra.linhas[proxima..fim_indice];
        let silabas_por_linha: Vec<usize> =
            linhas.iter().map(|l| silabas(l).len().max(1)).collect();
        let total_silabas: usize = silabas_por_linha.iter().sum();
        let mut instante = inicio;
        for (j, linha) in linhas.iter().enumerate() {
            let fatia = (fim - inicio) * (silabas_por_linha[j] as f32 / total_silabas as f32);
            let linha_inicio = instante;
            let linha_fim = if j + 1 == linhas.len() {
                fim
            } else {
                instante + fatia
            };
            instante = linha_fim;

            let indice = proxima + j;
            let secao = relatorio
                .sections
                .iter()
                .find(|s| linha_inicio >= s.inicio_s - 0.01 && linha_inicio < s.fim_s);
            let (trecho, papel) = secao.map_or_else(
                || ("?".to_owned(), "?".to_owned()),
                |s| (s.rotulo.clone(), s.papel.clone()),
            );

            let mut confianca = base_confianca;
            if secao.is_some_and(|s| (linha_inicio - s.inicio_s).abs() < 0.6) {
                confianca += 0.15;
            }
            if k > 2 {
                confianca -= 0.2;
            }
            let silabas_por_segundo =
                silabas_por_linha[j] as f32 / (linha_fim - linha_inicio).max(0.1);
            if !(1.0..=8.0).contains(&silabas_por_segundo) {
                confianca -= 0.2;
            }

            saida.push(LinhaAlinhada {
                indice,
                inicio_s: linha_inicio,
                fim_s: linha_fim,
                texto: linha.clone(),
                trecho,
                papel,
                lufs_medio: lufs_local(relatorio, linha_inicio, linha_fim),
                repetida: letra.linhas[..indice]
                    .iter()
                    .any(|anterior| anterior.trim().eq_ignore_ascii_case(linha.trim())),
                confianca: confianca.clamp(0.05, 0.95),
            });
        }
        proxima = fim_indice;
    }
    saida
}
