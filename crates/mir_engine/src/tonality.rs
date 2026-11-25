//! Centro tonal por perfil de tonalidade (ADR-0014, bloco 1).
//!
//! O motor antigo devolvia a **fundamental mais frequente** entre os acordes — e
//! errou de forma documentada: numa música em **E maior** devolveu `B`, e na
//! fixture em Dó devolveu `Am`. Frequência de acorde não é centro tonal.
//!
//! Aqui a decisão usa **perfil de tonalidade** (Krumhansl–Schmuckler, 1982),
//! correlacionado com o croma médio da peça e repetido nas 12 tônicas em modo
//! maior e menor — 24 candidatos. O perfil já ordena os graus pelo **ciclo de
//! quintas**, que é o mesmo eixo que o músico usa para achar tom de ouvido.
//!
//! O resultado carrega a **regra que o produziu** (correlação e margem sobre a
//! segunda colocada), como a ADR-0012 exige de veredito. Sem energia suficiente
//! para sustentar tonalidade, a resposta é `Indeterminada` — fail-loud, não
//! palpite.

use serde::Serialize;

use crate::chords;
use crate::spectrum::Chroma;

/// Modo do centro tonal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Modo {
    Maior,
    Menor,
}

impl Modo {
    fn nome(self) -> &'static str {
        match self {
            Self::Maior => "maior",
            Self::Menor => "menor",
        }
    }
}

/// Perfil de Krumhansl–Kessler para modo maior, na ordem dos graus da escala.
///
/// O peso é o do estudo original: a tônica domina, a quinta e a terça vêm
/// depois, e os graus fora da escala ficam baixos. É isso que faz a correlação
/// seguir o ciclo de quintas sem escrever o ciclo à mão.
const PERFIL_MAIOR: [f32; 12] = [
    6.35, 2.23, 3.48, 2.33, 4.38, 4.09, 2.52, 5.19, 2.39, 3.66, 2.29, 2.88,
];

/// Perfil para modo menor natural, na mesma ordem.
const PERFIL_MENOR: [f32; 12] = [
    6.33, 2.68, 3.52, 5.38, 2.60, 3.53, 2.54, 4.75, 3.98, 2.69, 3.34, 3.17,
];

/// Fração do pico abaixo da qual a classe de altura não conta como presente.
const LIMIAR_PRESENCA: f32 = 0.05;

/// Menos classes que isto não sustenta tonalidade: é ruído ou trecho vazio.
///
/// Com menos de quatro classes, qualquer perfil correlaciona bem com qualquer
/// coisa — e o motor estaria dando veredito sobre um intervalo.
const MINIMO_DE_CLASSES: u8 = 4;

/// Resultado do centro tonal.
#[derive(Debug, Clone, Serialize)]
pub struct Tonalidade {
    /// Nome na convenção da tela (`C`, `Am`, `Bbm`). `None` = indeterminada.
    pub nome: Option<String>,
    /// Correlação do vencedor com o perfil (0 a 1; na prática 0,5–0,95).
    pub correlacao: f32,
    /// Quanto o vencedor passou a segunda colocada. Baixa = decisão frágil.
    pub margem: f32,
    /// Nome da segunda melhor tônica — `None` quando indeterminada.
    pub segunda_nome: Option<String>,
    /// Correlação da segunda colocada.
    pub segunda_correlacao: f32,
    /// Classes de altura com energia relevante (0–12).
    pub classes_presentes: u8,
    /// A regra que produziu o resultado — não é decoração.
    pub regra: String,
}

impl Tonalidade {
    /// Indeterminada com o motivo dito: quem lê sabe por que não houve resposta.
    fn indeterminada(motivo: &str, classes_presentes: u8) -> Self {
        Self {
            nome: None,
            correlacao: 0.0,
            margem: 0.0,
            segunda_nome: None,
            segunda_correlacao: 0.0,
            classes_presentes,
            regra: format!("indeterminada: {motivo}"),
        }
    }

    /// Nome para a tela; `?` quando indeterminada.
    #[must_use]
    pub fn rotulo(&self) -> &str {
        self.nome.as_deref().unwrap_or("?")
    }
}

/// Correlação de Pearson entre duas séries de 12 valores.
fn correlacao(a: &[f32; 12], b: &[f32; 12]) -> f32 {
    let n = 12.0f32;
    let media_a = a.iter().sum::<f32>() / n;
    let media_b = b.iter().sum::<f32>() / n;
    let (mut covariancia, mut var_a, mut var_b) = (0.0f32, 0.0f32, 0.0f32);
    for i in 0..12 {
        let da = a[i] - media_a;
        let db = b[i] - media_b;
        covariancia += da * db;
        var_a += da * da;
        var_b += db * db;
    }
    let denominador = (var_a * var_b).sqrt();
    if denominador <= f32::EPSILON {
        0.0
    } else {
        covariancia / denominador
    }
}

/// Croma médio da peça, contando só quadros com energia.
///
/// Quadro silencioso é vetor zero; se entrar na média, dilui tudo e o perfil
/// perde o sinal. Contar só quadro com energia é o mínimo de higiene — e o
/// número de quadros aproveitados volta no retorno, para o caso degenerado.
fn croma_medio(chroma: &Chroma) -> ([f32; 12], usize) {
    let mut soma = [0.0f32; 12];
    let mut com_energia = 0usize;
    for quadro in &chroma.quadros {
        if quadro.iter().sum::<f32>() <= f32::EPSILON {
            continue;
        }
        com_energia += 1;
        for (i, valor) in quadro.iter().enumerate() {
            soma[i] += valor;
        }
    }
    if com_energia > 0 {
        let n = com_energia as f32;
        for valor in &mut soma {
            *valor /= n;
        }
    }
    (soma, com_energia)
}

/// Centro tonal pelo perfil correlacionado nas 24 tônicas.
#[must_use]
pub fn analisar(chroma: &Chroma) -> Tonalidade {
    let (media, quadros_com_energia) = croma_medio(chroma);
    let pico = media.iter().copied().fold(0.0f32, f32::max);
    if quadros_com_energia == 0 || pico <= f32::EPSILON {
        return Tonalidade::indeterminada("sem quadro com energia", 0);
    }
    let classes_presentes = media
        .iter()
        .filter(|valor| **valor >= LIMIAR_PRESENCA * pico)
        .count() as u8;
    if classes_presentes < MINIMO_DE_CLASSES {
        return Tonalidade::indeterminada(
            "poucas classes de altura com energia (menos de quatro)",
            classes_presentes,
        );
    }

    // 24 candidatos: 12 tônicas × 2 modos. O perfil é rotacionado para a classe
    // de altura absoluta — o grau `d` da escala vira a classe `(tonica + d) % 12`.
    let mut candidatos: Vec<(f32, usize, Modo)> = Vec::with_capacity(24);
    for tonica in 0..12 {
        for (perfil, modo) in [(&PERFIL_MAIOR, Modo::Maior), (&PERFIL_MENOR, Modo::Menor)] {
            let esperado: [f32; 12] = std::array::from_fn(|i| perfil[(i + 12 - tonica) % 12]);
            candidatos.push((correlacao(&media, &esperado), tonica, modo));
        }
    }
    candidatos.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    let (melhor, tonica, modo) = candidatos[0];
    let (segunda_corr, segunda_tonica, segunda_modo) = candidatos[1];
    let margem = (melhor - segunda_corr).max(0.0);
    let sufixo = if modo == Modo::Menor { "m" } else { "" };
    let segunda_sufixo = if segunda_modo == Modo::Menor { "m" } else { "" };
    Tonalidade {
        nome: Some(format!("{}{sufixo}", chords::nome_da_classe(tonica))),
        correlacao: melhor,
        margem,
        segunda_nome: Some(format!("{}{}", chords::nome_da_classe(segunda_tonica), segunda_sufixo)),
        segunda_correlacao: segunda_corr,
        classes_presentes,
        regra: format!(
            "perfil de Krumhansl–Schmuckler ({}) em 24 candidatos: correlação {melhor:.3}, \
             margem {margem:.3} sobre {}{} ({segunda_corr:.3})",
            modo.nome(),
            chords::nome_da_classe(segunda_tonica),
            segunda_sufixo
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Croma de um único quadro, somando os pares (classe, peso) iguais.
    fn croma(pares: &[(usize, f32)]) -> Chroma {
        let mut vetor = [0.0f32; 12];
        for &(classe, valor) in pares {
            vetor[classe % 12] += valor;
        }
        Chroma {
            quadros: vec![vetor],
            fps: 86.13,
        }
    }

    #[test]
    fn escala_de_do_maior_vira_do_maior() {
        // C D E F G A B — a escala inteira, com a tônica mais forte.
        let tonalidade = analisar(&croma(&[
            (0, 6.0),
            (2, 3.0),
            (4, 4.0),
            (5, 3.5),
            (7, 5.0),
            (9, 3.2),
            (11, 2.8),
        ]));
        assert_eq!(tonalidade.rotulo(), "C", "regra: {}", tonalidade.regra);
        assert!(tonalidade.margem > 0.0);
    }

    #[test]
    fn menor_natural_vira_la_menor() {
        // A B C D E F G — o relativo de Dó, com Lá dominando.
        let tonalidade = analisar(&croma(&[
            (9, 6.0),
            (11, 3.0),
            (0, 3.4),
            (2, 3.0),
            (4, 4.6),
            (5, 2.6),
            (7, 2.5),
        ]));
        assert_eq!(tonalidade.rotulo(), "Am", "regra: {}", tonalidade.regra);
    }

    #[test]
    fn progressao_em_mi_maior_nao_vira_si() {
        // E–B–C#m–A contado por ocorrência: o defeito medido devolvia `B`
        // porque contava fundamental de acorde. D# presente e D ausente é o
        // par que separa Mi maior de Lá maior (ADR-0014).
        let tonalidade = analisar(&croma(&[
            (4, 3.0),  // E
            (8, 2.0),  // G#
            (11, 2.0), // B
            (3, 1.0),  // D#
            (6, 1.0),  // F#
            (1, 2.0),  // C#
            (9, 1.0),  // A
        ]));
        assert_eq!(tonalidade.rotulo(), "E", "regra: {}", tonalidade.regra);
    }

    #[test]
    fn progressao_em_la_maior_com_d_nao_vira_mi() {
        // A–E–F#m–D: mesmo formato de acordes, mas D natural e nenhum D#.
        // É o discriminador de grau 4 que a ADR-0014 nomeia.
        let tonalidade = analisar(&croma(&[
            (9, 3.0),  // A
            (1, 2.0),  // C#
            (4, 2.0),  // E
            (8, 1.0),  // G#
            (11, 1.0), // B
            (6, 2.0),  // F#
            (2, 1.0),  // D
        ]));
        assert_eq!(tonalidade.rotulo(), "A", "regra: {}", tonalidade.regra);
    }

    #[test]
    fn sem_energia_e_indeterminada_e_diz_por_que() {
        let vazio = Chroma {
            quadros: vec![[0.0f32; 12]],
            fps: 86.13,
        };
        let tonalidade = analisar(&vazio);
        assert!(tonalidade.nome.is_none());
        assert_eq!(tonalidade.rotulo(), "?");
        assert!(tonalidade.regra.contains("sem quadro com energia"));
    }

    #[test]
    fn duas_notas_nao_sustentam_tonalidade() {
        // Um intervalo não é tonalidade: qualquer perfil correlaciona bem.
        let tonalidade = analisar(&croma(&[(0, 1.0), (7, 1.0)]));
        assert!(tonalidade.nome.is_none());
        assert_eq!(tonalidade.classes_presentes, 2);
        assert!(tonalidade.regra.contains("poucas classes"));
    }

    #[test]
    fn quadro_silencioso_nao_dilui_a_media() {
        // Metade dos quadros em silêncio: se eles entrassem na média, o croma
        // cairia pela metade e o pico relativo mudaria. A resposta tem de ser a
        // mesma com ou sem eles.
        let mut com_silencio = croma(&[(0, 6.0), (4, 4.0), (7, 5.0), (2, 3.0)]);
        com_silencio.quadros.push([0.0f32; 12]);
        com_silencio.quadros.push([0.0f32; 12]);
        let sem_silencio = croma(&[(0, 6.0), (4, 4.0), (7, 5.0), (2, 3.0)]);
        assert_eq!(
            analisar(&com_silencio).rotulo(),
            analisar(&sem_silencio).rotulo()
        );
    }
}
