//! Relatório de uso classificado: a agenda (ADR-0006) cruzada com a ficha da
//! obra (ADR-0012).
//!
//! É a resposta ao "é impossível enumerar o repertório": **não se enumera,
//! classifica-se o que se canta**. O relatório sai das agendas que a equipe já
//! mantém, e a lista de obras **sem ficha** é a lista de trabalho — finita e
//! priorizada pelo uso real, em vez dos 400 hinos do hinário.
//!
//! Limite declarado: sem ficha, o motor **não** afirma domínio público nem
//! protegido. Afirma `sem ficha`.
//!
//! A classificação consulta a **ficha do hinário** (`crate::fichas`) antes do
//! corpus de letras: é ela que carrega a tradução, a camada que manda. Título que
//! casa com mais de uma ficha não vira veredito — vira `Indeterminado` nomeando
//! as candidatas, porque o motor não escolhe qual "Louvor" a igreja cantou.

use std::path::Path;

use notation_bundle::bundle::{Library, RelatorioUso, relatorio_de_uso};
use notation_bundle::bundle_store;

use crate::lyrics::{self, Jurisdicao, Origem, Veredito};

/// Uma obra do repertório, com o uso medido e o veredito da ficha (quando há).
#[derive(Debug, Clone)]
pub struct ObraClassificada {
    pub titulo: String,
    pub execucoes: usize,
    pub versoes: Vec<(String, usize)>,
    /// `None` = sem ficha: o motor não afirma nada sobre esta obra.
    pub veredito: Option<Veredito>,
}

impl ObraClassificada {
    /// A obra tem ficha que resolve domínio público.
    #[must_use]
    pub fn eh_publica(&self) -> bool {
        self.veredito
            .as_ref()
            .is_some_and(lyrics::Veredito::eh_dominio_publico)
    }
}

/// Uso do repertório (agendas) + classificação por obra (fichas do corpus).
#[derive(Debug, Clone)]
pub struct RelatorioClassificado {
    pub uso: RelatorioUso,
    pub obras: Vec<ObraClassificada>,
}

impl RelatorioClassificado {
    /// Quantas obras do repertório são de domínio público por ficha.
    #[must_use]
    pub fn publicas(&self) -> usize {
        self.obras.iter().filter(|obra| obra.eh_publica()).count()
    }

    /// Quantas obras ficam sem ficha — **a lista de trabalho**.
    #[must_use]
    pub fn sem_ficha(&self) -> usize {
        self.obras
            .iter()
            .filter(|obra| obra.veredito.is_none())
            .count()
    }

    /// Relatório legível: uso primeiro, classificação depois, e a lista das
    /// obras que precisam de ficha em ordem decrescente de uso.
    #[must_use]
    pub fn texto(&self) -> String {
        let mut texto = self.uso.texto();
        texto.push_str(&format!(
            "\nclassificação (ADR-0012) — domínio público por ficha: {} · sem ficha: {}\n",
            self.publicas(),
            self.sem_ficha()
        ));
        let mut ordenadas = self.obras.clone();
        ordenadas.sort_by(|a, b| {
            b.execucoes
                .cmp(&a.execucoes)
                .then_with(|| a.titulo.cmp(&b.titulo))
        });
        for obra in &ordenadas {
            let linha = match &obra.veredito {
                Some(veredito) => veredito.resumo(),
                None => "SEM FICHA — monte a ficha (--ficha) para o veredito".to_owned(),
            };
            texto.push_str(&format!(
                "  {:>3}×  {}  [{}]\n",
                obra.execucoes, obra.titulo, linha
            ));
        }
        texto
    }
}

/// Classifica cada obra do relatório de uso pela ficha do corpus embutido.
#[must_use]
pub fn classificar(uso: &RelatorioUso, ano_corrente: u16) -> Vec<ObraClassificada> {
    uso.por_obra
        .iter()
        .map(|resumo| ObraClassificada {
            titulo: resumo.titulo.clone(),
            execucoes: resumo.execucoes,
            versoes: resumo.versoes.clone(),
            veredito: veredito_por_titulo(&resumo.titulo, ano_corrente),
        })
        .collect()
}

/// Veredito para um título, consultando o corpus de domínio público (ADR-0012).
///
/// Sem ficha no corpus, devolve `None` — e `None` é resposta, não falha: o motor
/// não afirma domínio público do que não tem ficha.
/// Veredito para um título: **ficha do hinário primeiro**, corpus de letras depois.
///
/// A ficha é a camada que carrega a tradução — que é a obra que manda. Título
/// ambíguo **não** vira veredito: dois "Louvor" no repertório têm vereditos
/// opostos (135, domínio público; 385, protegido), e escolher o primeiro seria
/// mentir sobre o que a igreja canta.
#[must_use]
pub fn veredito_por_titulo(titulo: &str, ano_corrente: u16) -> Option<Veredito> {
    match crate::fichas::resposta_por_titulo(titulo, ano_corrente) {
        crate::fichas::RespostaLetra::Livre {
            ano_dominio_publico,
        } => {
            return Some(Veredito::DominioPublico {
                regra: format!("ficha do hinário: domínio público desde {ano_dominio_publico}"),
            });
        }
        crate::fichas::RespostaLetra::RequerLetraDoUsuario {
            ano_dominio_publico,
        } => {
            return Some(Veredito::Protegido {
                regra: format!(
                    "ficha do hinário: a camada que manda está protegida até {ano_dominio_publico}"
                ),
            });
        }
        crate::fichas::RespostaLetra::Gap { motivo } => {
            return Some(Veredito::Indeterminado { motivo });
        }
        crate::fichas::RespostaLetra::Ambiguo { candidatas } => {
            return Some(Veredito::Indeterminado {
                motivo: format!(
                    "título ambíguo entre fichas ({}) — use (hinário, número)",
                    candidatas.join("; ")
                ),
            });
        }
        crate::fichas::RespostaLetra::SemFicha => {}
    }
    let letra = lyrics::do_corpus(titulo)?;
    match &letra.origem {
        Origem::DominioPublico { obra } => {
            Some(lyrics::veredito_pd(obra, Jurisdicao::Brasil, ano_corrente))
        }
        _ => Some(Veredito::Indeterminado {
            motivo: "origem inesperada no corpus".to_owned(),
        }),
    }
}

/// Lê biblioteca e agendas de um diretório de bundle e devolve o relatório.
///
/// # Errors
///
/// Erro legível quando a biblioteca ou uma agenda não pode ser lida, ou quando
/// uma agenda aponta para obra fora da biblioteca.
pub fn do_diretorio(dir: &Path, ano_corrente: u16) -> Result<RelatorioClassificado, String> {
    let library: Library = bundle_store::load_library(dir).map_err(|erro| {
        format!(
            "biblioteca em {dir:?}: {erro} ({}). Rode o índice primeiro.",
            erro.message_key()
        )
    })?;
    let agendas = bundle_store::load_setlists(dir, &library)
        .map_err(|erro| format!("agendas em {dir:?}: {erro} ({})", erro.message_key()))?;
    let uso = relatorio_de_uso(&agendas, &library)
        .map_err(|erro| format!("relatório: {erro} ({})", erro.message_key()))?;
    let obras = classificar(&uso, ano_corrente);
    Ok(RelatorioClassificado { uso, obras })
}
