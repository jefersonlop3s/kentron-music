//! Fichas de hino do hinário: o dado que decide se uma versão pode ter letra.
//!
//! O asset `assets/fichas_hinario.tsv` guarda **fatos** de cada hino (autoria,
//! tradução, melodia, anos, veredito e procedência por campo), e a identidade é o
//! **par (hinário, número)** mais o título. O corpus `hinos_dominio_publico.tsv`
//! guarda **letra**; a ficha não guarda letra — guarda o que decide se a letra
//! **daquela versão** pode ser exibida.
//!
//! Três decisões de desenho, todas vindas do material real:
//!
//! 1. **A resposta é derivada do ano, não da coluna `veredito`.** O ano de queda
//!    (`ano_dominio_publico`) é o fato durável; `veredito` é o retrato do dia da
//!    conferência. Em 2040 o 291 fica livre sem ninguém editar o asset, e a ficha
//!    continua dizendo a verdade sobre o que se sabia em 2026.
//! 2. **Título não é identidade.** O repertório tem dois hinos chamados "Louvor"
//!    com vereditos opostos (135, domínio público; 385, protegido até 2033).
//!    Consulta por título devolve **todas** as candidatas; o motor recusa chutar.
//! 3. **A ficha é julgada pela camada que manda** — a tradução brasileira do
//!    século XX, não a letra original. `ano_relevante` já é o falecimento dessa
//!    camada, e `auditar` reconstrói a `Obra` para a regra única
//!    (`lyrics::veredito_pd`) provar a fronteira `morte + 71` linha a linha.

use std::sync::OnceLock;

use serde::Serialize;

use crate::lyrics::{self, Jurisdicao, Obra, Veredito};

/// O asset embutido — validado pelo gate `fichas` no push e por `auditar` no teste.
const ASSET: &str = include_str!("../assets/fichas_hinario.tsv");

/// As colunas obrigatórias do asset.
const COLUNAS: [&str; 14] = [
    "hinario",
    "numero",
    "titulo",
    "autoria",
    "traducao",
    "melodia",
    "ano_relevante",
    "veredito",
    "ano_dominio_publico",
    "fonte_autoria",
    "fonte_traducao",
    "fonte_melodia",
    "no_repertorio",
    "observacao",
];

/// Estados da ficha. `SemFicha` existe na API (e no gate) como **resposta**:
/// ausência de ficha não é falha, é o motor se recusando a afirmar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum VereditoFicha {
    DominioPublico,
    Protegido,
    Gap,
    SemFicha,
}

impl VereditoFicha {
    /// O rótulo do asset (`pd`, `protegido`, `gap`, `sem_ficha`).
    #[must_use]
    pub fn nome(self) -> &'static str {
        match self {
            Self::DominioPublico => "pd",
            Self::Protegido => "protegido",
            Self::Gap => "gap",
            Self::SemFicha => "sem_ficha",
        }
    }
}

/// Uma ficha de hino, com procedência **por campo** — o fato e de onde ele veio.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FichaHino {
    pub hinario: String,
    pub numero: String,
    pub titulo: String,
    pub autoria: String,
    pub traducao: String,
    pub melodia: String,
    /// Falecimento da camada que manda (autoria, ou a tradução quando há).
    /// Ausente em `gap`: é justamente o dado que falta.
    pub ano_relevante: Option<u16>,
    /// Retrato do dia da conferência; a resposta sai de `ano_dominio_publico`.
    pub veredito: VereditoFicha,
    /// Ano em que a camada que manda cai em domínio público (`morte + 71`, Brasil).
    pub ano_dominio_publico: Option<u16>,
    pub fonte_autoria: String,
    pub fonte_traducao: String,
    pub fonte_melodia: String,
    pub no_repertorio: bool,
    pub observacao: String,
}

impl FichaHino {
    /// Título legível para o relatório: `Cantor Cristão · 09 Santo, Santo, Santo`.
    #[must_use]
    pub fn rotulo(&self) -> String {
        format!("{} · {} {}", self.hinario, self.numero, self.titulo)
    }
}

/// Lê fichas de um texto no formato do asset (`#` comenta).
///
/// # Errors
///
/// Erro legível com a linha e o motivo. Linha torta **nunca** é ignorada em
/// silêncio: um asset malformado vira erro, não "nenhuma ficha".
pub fn carregar_do_texto(texto: &str) -> Result<Vec<FichaHino>, String> {
    let mut linhas = texto.lines().filter(|linha| {
        let t = linha.trim();
        !t.is_empty() && !t.starts_with('#')
    });
    let cabecalho = linhas.next().ok_or("asset de fichas vazio")?;
    let colunas: Vec<&str> = cabecalho.split('\t').collect();
    for obrigatoria in COLUNAS {
        if !colunas.contains(&obrigatoria) {
            return Err(format!("coluna obrigatória ausente: {obrigatoria}"));
        }
    }

    let mut fichas = Vec::new();
    for (indice, linha) in linhas.enumerate() {
        let numero_da_linha = indice + 2; // o cabeçalho é a linha 1
        let campos: Vec<&str> = linha.split('\t').collect();
        if campos.len() != colunas.len() {
            return Err(format!(
                "linha {numero_da_linha}: {} campos contra {} colunas",
                campos.len(),
                colunas.len()
            ));
        }
        let campo = |nome: &str| -> &str {
            let posicao = colunas
                .iter()
                .position(|c| *c == nome)
                .expect("coluna conferida acima");
            campos[posicao].trim()
        };
        let ano = |nome: &str| -> Result<Option<u16>, String> {
            let bruto = campo(nome);
            if bruto.is_empty() {
                return Ok(None);
            }
            bruto
                .parse()
                .map(Some)
                .map_err(|_| format!("linha {numero_da_linha}: \"{nome}\" não é ano: {bruto:?}"))
        };
        let veredito = match campo("veredito") {
            "pd" => VereditoFicha::DominioPublico,
            "protegido" => VereditoFicha::Protegido,
            "gap" => VereditoFicha::Gap,
            "sem_ficha" => VereditoFicha::SemFicha,
            outro => {
                return Err(format!(
                    "linha {numero_da_linha}: veredito inválido {outro:?}"
                ));
            }
        };
        let no_repertorio = match campo("no_repertorio") {
            "sim" => true,
            "nao" | "não" | "" => false,
            outro => {
                return Err(format!(
                    "linha {numero_da_linha}: no_repertorio inválido {outro:?}"
                ));
            }
        };
        fichas.push(FichaHino {
            hinario: campo("hinario").to_owned(),
            numero: campo("numero").to_owned(),
            titulo: campo("titulo").to_owned(),
            autoria: campo("autoria").to_owned(),
            traducao: campo("traducao").to_owned(),
            melodia: campo("melodia").to_owned(),
            ano_relevante: ano("ano_relevante")?,
            veredito,
            ano_dominio_publico: ano("ano_dominio_publico")?,
            fonte_autoria: campo("fonte_autoria").to_owned(),
            fonte_traducao: campo("fonte_traducao").to_owned(),
            fonte_melodia: campo("fonte_melodia").to_owned(),
            no_repertorio,
            observacao: campo("observacao").to_owned(),
        });
    }
    Ok(fichas)
}

/// As fichas do asset embutido. O parse acontece uma vez.
///
/// # Panics
///
/// Se o asset embutido estiver malformado. É fail-loud por desenho: o asset é
/// versionado e o gate `fichas` o valida no push; um arquivo torto não pode
/// virar "nenhuma ficha" em silêncio.
#[must_use]
pub fn fichas() -> &'static [FichaHino] {
    static CACHE: OnceLock<Vec<FichaHino>> = OnceLock::new();
    CACHE.get_or_init(|| {
        carregar_do_texto(ASSET).unwrap_or_else(|erro| panic!("asset de fichas malformado: {erro}"))
    })
}

/// Normaliza para busca: sem acento e em minúsculas.
fn normalizar(texto: &str) -> String {
    let dobrado: String = texto
        .chars()
        .map(|c| match c {
            'á' | 'à' | 'â' | 'ã' | 'ä' | 'Á' | 'À' | 'Â' | 'Ã' | 'Ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' | 'É' | 'È' | 'Ê' | 'Ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' | 'Í' | 'Ì' | 'Î' | 'Ï' => 'i',
            'ó' | 'ò' | 'ô' | 'õ' | 'ö' | 'Ó' | 'Ò' | 'Ô' | 'Õ' | 'Ö' => 'o',
            'ú' | 'ù' | 'û' | 'ü' | 'Ú' | 'Ù' | 'Û' | 'Ü' => 'u',
            'ç' | 'Ç' => 'c',
            outro => outro,
        })
        .collect();
    dobrado.to_lowercase()
}

/// `"09"` e `"9"` são o mesmo número; `"0"` não vira vazio.
fn numero_normalizado(numero: &str) -> String {
    let sem_zeros = numero.trim().trim_start_matches('0');
    if sem_zeros.is_empty() {
        numero.trim().to_owned()
    } else {
        sem_zeros.to_owned()
    }
}

/// Ficha pelo **par (hinário, número)** — a identidade de verdade.
///
/// O hinário casa por trecho normalizado (`"cantor cristao"` encontra
/// `"Cantor Cristão (Cifrado, EME/JUERP 2011)"`), porque quem consulta digita o
/// nome que usa.
#[must_use]
pub fn por_identidade(hinario: &str, numero: &str) -> Option<&'static FichaHino> {
    let alvo_hinario = normalizar(hinario);
    if alvo_hinario.trim().is_empty() {
        return None;
    }
    let alvo_numero = numero_normalizado(numero);
    fichas().iter().find(|ficha| {
        normalizar(&ficha.hinario).contains(&alvo_hinario)
            && numero_normalizado(&ficha.numero) == alvo_numero
    })
}

/// Todas as fichas cujo título começa pelo texto buscado (normalizado).
///
/// **Mais de uma é resposta**, não erro: "Louvor" casa com o 135 (domínio
/// público) e com o 385 (protegido até 2033), e escolher o primeiro seria mentir.
#[must_use]
pub fn por_titulo(titulo: &str) -> Vec<&'static FichaHino> {
    let alvo = normalizar(titulo);
    if alvo.trim().is_empty() {
        return Vec::new();
    }
    fichas()
        .iter()
        .filter(|ficha| normalizar(&ficha.titulo).starts_with(&alvo))
        .collect()
}

/// O que a ficha autoriza para a letra **desta versão**.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum RespostaLetra {
    /// A camada que manda já caiu: a letra pode ser exibida.
    Livre { ano_dominio_publico: u16 },
    /// A versão que se canta ainda está protegida: só com letra do usuário.
    RequerLetraDoUsuario { ano_dominio_publico: u16 },
    /// Falta o dado que permitiria afirmar — não se chuta (`gap`).
    Gap { motivo: String },
    /// O título casa com mais de uma ficha: a consulta precisa da identidade.
    Ambiguo { candidatas: Vec<String> },
    /// Não há ficha para esta obra.
    SemFicha,
}

impl RespostaLetra {
    /// Texto curto para o relatório e para a CLI.
    #[must_use]
    pub fn resumo(&self) -> String {
        match self {
            Self::Livre {
                ano_dominio_publico,
            } => format!("letra LIVRE — domínio público desde {ano_dominio_publico}"),
            Self::RequerLetraDoUsuario {
                ano_dominio_publico,
            } => format!("letra só do usuário — protegida até {ano_dominio_publico}"),
            Self::Gap { motivo } => format!("GAP — {motivo}"),
            Self::Ambiguo { candidatas } => format!(
                "AMBÍGUO — o título casa com {} fichas ({}): consulte por (hinário, número)",
                candidatas.len(),
                candidatas.join("; ")
            ),
            Self::SemFicha => "SEM FICHA — monte a ficha para o veredito".to_owned(),
        }
    }
}

/// Deriva a resposta do **ano**, não da coluna `veredito`.
///
/// Na virada do ano a resposta muda sozinha — e a ficha continua registrando o
/// que se sabia no dia da conferência.
#[must_use]
pub fn resposta_para_letra(ficha: &FichaHino, ano_corrente: u16) -> RespostaLetra {
    match ficha.veredito {
        VereditoFicha::Gap => RespostaLetra::Gap {
            motivo: if ficha.observacao.trim().is_empty() {
                format!(
                    "{}: falta o dado que decide o domínio (o livro não traz a data)",
                    ficha.rotulo()
                )
            } else {
                ficha.observacao.clone()
            },
        },
        VereditoFicha::SemFicha => RespostaLetra::SemFicha,
        VereditoFicha::DominioPublico | VereditoFicha::Protegido => {
            match ficha.ano_dominio_publico {
                Some(ano) if ano_corrente >= ano => RespostaLetra::Livre {
                    ano_dominio_publico: ano,
                },
                Some(ano) => RespostaLetra::RequerLetraDoUsuario {
                    ano_dominio_publico: ano,
                },
                None => RespostaLetra::Gap {
                    motivo: format!(
                        "{}: ficha sem ano de domínio público — o gate deveria ter reprovado",
                        ficha.rotulo()
                    ),
                },
            }
        }
    }
}

/// Resposta pela identidade. Sem ficha, `SemFicha` — nunca "provavelmente livre".
#[must_use]
pub fn resposta_por_identidade(hinario: &str, numero: &str, ano_corrente: u16) -> RespostaLetra {
    por_identidade(hinario, numero).map_or(RespostaLetra::SemFicha, |ficha| {
        resposta_para_letra(ficha, ano_corrente)
    })
}

/// Resposta por título, **recusando** ambiguidade.
#[must_use]
pub fn resposta_por_titulo(titulo: &str, ano_corrente: u16) -> RespostaLetra {
    match por_titulo(titulo).as_slice() {
        [] => RespostaLetra::SemFicha,
        [unica] => resposta_para_letra(unica, ano_corrente),
        varias => RespostaLetra::Ambiguo {
            candidatas: varias
                .iter()
                .map(|ficha| {
                    format!(
                        "{} {} — {}",
                        ficha.hinario,
                        ficha.numero,
                        resposta_para_letra(ficha, ano_corrente).resumo()
                    )
                })
                .collect(),
        },
    }
}

/// Separa `"Maxwell Wright (1849-1931)"` em nome e falecimento.
///
/// `"D. M. Hazlett (1852-?)"` devolve nome e `None`: o livro não traz a data.
fn nome_e_morte(campo: &str) -> (String, Option<u16>) {
    let (Some(abre), Some(fecha)) = (campo.rfind('('), campo.rfind(')')) else {
        return (campo.trim().to_owned(), None);
    };
    if fecha < abre {
        return (campo.trim().to_owned(), None);
    }
    let dentro = &campo[abre + 1..fecha];
    let morte = dentro
        .rsplit('-')
        .next()
        .and_then(|token| token.trim().parse::<u16>().ok());
    (campo[..abre].trim().to_owned(), morte)
}

/// Reconstrói a `Obra` do domínio a partir da ficha, para a regra única valer.
///
/// A ficha não carrega ano de publicação (o hinário não imprime), então `ano` é
/// `0` — e o veredito brasileiro não depende dele. A camada que manda é a
/// tradução, quando há: é ela que vira `adaptador` aqui.
#[must_use]
pub fn obra_da_ficha(ficha: &FichaHino) -> Obra {
    let (autor, _) = nome_e_morte(&ficha.autoria);
    let (adaptador, _) = nome_e_morte(&ficha.traducao);
    Obra {
        titulo: ficha.titulo.clone(),
        autor,
        ano: 0,
        autor_morte: ficha.ano_relevante,
        adaptador: Some(adaptador).filter(|nome| !nome.is_empty()),
        adaptador_morte: ficha.ano_relevante,
        obra_original: None,
        base: [
            ficha.fonte_autoria.as_str(),
            ficha.fonte_traducao.as_str(),
            ficha.fonte_melodia.as_str(),
        ]
        .iter()
        .filter(|fonte| !fonte.trim().is_empty())
        .copied()
        .collect::<Vec<_>>()
        .join(" · "),
    }
}

/// Auditoria da ficha: a aritmética tem de fechar e a fronteira tem de valer.
///
/// Para cada linha `pd`/`protegido`, prova com a regra única do domínio
/// (`lyrics::veredito_pd`, Brasil) que a obra está **protegida** em
/// `ano_dominio_publico - 1` e **livre** em `ano_dominio_publico`. É
/// independente da data de hoje, então não vira bomba-relógio.
///
/// # Errors
///
/// Lista de problemas encontrados (vazia = auditoria limpa na forma `Ok`).
pub fn auditar() -> Result<Vec<String>, Vec<String>> {
    let mut aprovadas = Vec::new();
    let mut problemas = Vec::new();
    for ficha in fichas() {
        let rotulo = ficha.rotulo();
        match ficha.veredito {
            VereditoFicha::Gap => {
                if ficha.observacao.trim().is_empty() {
                    problemas.push(format!("{rotulo}: gap sem observação do dado que falta"));
                } else {
                    aprovadas.push(format!("{rotulo}: gap declarado"));
                }
            }
            VereditoFicha::SemFicha => {
                problemas.push(format!("{rotulo}: sem_ficha dentro do asset de fichas"));
            }
            VereditoFicha::DominioPublico | VereditoFicha::Protegido => {
                let (Some(morte), Some(ano)) = (ficha.ano_relevante, ficha.ano_dominio_publico)
                else {
                    problemas.push(format!("{rotulo}: sem ano de morte ou de domínio público"));
                    continue;
                };
                let calculado = morte.saturating_add(71);
                if ano != calculado {
                    problemas.push(format!(
                        "{rotulo}: ano de domínio público {ano} não é morte {morte} + 71 ({calculado})"
                    ));
                    continue;
                }
                let obra = obra_da_ficha(ficha);
                let antes = lyrics::veredito_pd(&obra, Jurisdicao::Brasil, ano - 1);
                let no_ano = lyrics::veredito_pd(&obra, Jurisdicao::Brasil, ano);
                let fronteira = matches!(antes, Veredito::Protegido { .. })
                    && matches!(no_ano, Veredito::DominioPublico { .. });
                if fronteira {
                    aprovadas.push(format!(
                        "{rotulo}: {} — {morte}+71 = {ano} (protegida até {}, livre em {ano})",
                        ficha.veredito.nome(),
                        ano - 1
                    ));
                } else {
                    problemas.push(format!(
                        "{rotulo}: a regra não fecha na fronteira — em {}: {}, em {ano}: {}",
                        ano - 1,
                        antes.resumo(),
                        no_ano.resumo()
                    ));
                }
            }
        }
    }
    if problemas.is_empty() {
        Ok(aprovadas)
    } else {
        Err(problemas)
    }
}
