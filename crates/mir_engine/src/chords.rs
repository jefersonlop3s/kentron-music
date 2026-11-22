//! Acordes: casamento por template sobre o croma, com suavização temporal.
//!
//! Cada quadro recebe o acorde de maior similaridade de cosseno e a **margem**
//! sobre o segundo colocado vira a confiança. Quadros vizinhos iguais são
//! agrupados em trechos; trechos curtos demais são absorvidos pelo vizinho
//! (flicker de detector não é mudança de acorde).
//!
//! A saída é **candidata**. Em mixagem real, instrumentação e percussão
//! deslocam o croma; por isso o trecho carrega confiança e o músico confirma.

use serde::Serialize;

use crate::melody::Melodia;
use crate::spectrum::Chroma;

/// Qualidade do acorde reconhecida por este incremento.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Qualidade {
    Maior,
    Menor,
    Sete,
    MenorSete,
    MaiorSete,
    SusQuatro,
    Diminuto,
    Potencia,
}

impl Qualidade {
    fn sufixo(self) -> &'static str {
        match self {
            Self::Maior => "",
            Self::Menor => "m",
            Self::Sete => "7",
            Self::MenorSete => "m7",
            Self::MaiorSete => "maj7",
            Self::SusQuatro => "sus4",
            Self::Diminuto => "dim",
            Self::Potencia => "5",
        }
    }

    fn intervalos(self) -> &'static [usize] {
        match self {
            Self::Maior => &[0, 4, 7],
            Self::Menor => &[0, 3, 7],
            Self::Sete => &[0, 4, 7, 10],
            Self::MenorSete => &[0, 3, 7, 10],
            Self::MaiorSete => &[0, 4, 7, 11],
            Self::SusQuatro => &[0, 5, 7],
            Self::Diminuto => &[0, 3, 6],
            Self::Potencia => &[0, 7],
        }
    }

    /// Qualidades com terça definida.
    ///
    /// Só nelas a inversão não muda a identidade: em `sus4` e em potência a
    /// "fundamental" é convenção, e `Csus4/F` e `Fsus2` são o mesmo conjunto
    /// de notas — escrever a barra ali seria decidir nomenclatura, não medir.
    fn tem_terca(self) -> bool {
        !matches!(self, Self::SusQuatro | Self::Potencia)
    }
}

/// Cifra com o baixo quando ele é **terça, quinta ou sétima** do acorde.
///
/// Devolve `None` quando a barra não informa nada: baixo que não é nota do
/// acorde (passagem, pedal, erro), quarta ou segunda (que são adição, não
/// inversão) e acordes sem terça. O nome da qualidade nunca muda — a barra só
/// acrescenta registro, que é o que o croma descartou.
#[must_use]
pub fn com_baixo(tonica: usize, qualidade: Qualidade, baixo_midi: u8) -> Option<String> {
    if !qualidade.tem_terca() {
        return None;
    }
    let classe = usize::from(baixo_midi) % 12;
    let intervalo = (classe + 12 - tonica) % 12;
    if intervalo == 0 || !qualidade.intervalos().contains(&intervalo) {
        return None;
    }
    Some(format!("{}/{}", nome(tonica, qualidade), TONICAS[classe]))
}

const QUALIDADES: [Qualidade; 8] = [
    Qualidade::Maior,
    Qualidade::Menor,
    Qualidade::Sete,
    Qualidade::MenorSete,
    Qualidade::MaiorSete,
    Qualidade::SusQuatro,
    Qualidade::Diminuto,
    Qualidade::Potencia,
];

/// Nomes de tônica na convenção da tela: bemóis nas teclas pretas.
const TONICAS: [&str; 12] = [
    "C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B",
];

/// Um trecho com acorde detectado.
#[derive(Debug, Clone, Serialize)]
pub struct ChordSegment {
    pub inicio_s: f32,
    pub fim_s: f32,
    pub acorde: String,
    /// Margem sobre o segundo colocado, média do trecho. Baixa = incerto.
    pub confianca: f32,
    /// Tônica e qualidade ficam fora do JSON: existem para a inversão ser
    /// montada sem reparsear o nome (`crate::chords::com_baixo`).
    #[serde(skip)]
    pub tonica: usize,
    #[serde(skip)]
    pub qualidade: Qualidade,
}

/// Vetor do template: 1.0 nas classes do acorde, 0.0 no resto (normalizado).
fn template(tonica: usize, qualidade: Qualidade) -> [f32; 12] {
    let mut vetor = [0.0f32; 12];
    for intervalo in qualidade.intervalos() {
        vetor[(tonica + intervalo) % 12] = 1.0;
    }
    let norma = (vetor.iter().map(|v| v * v).sum::<f32>()).sqrt();
    if norma > 0.0 {
        for valor in &mut vetor {
            *valor /= norma;
        }
    }
    vetor
}

/// Nome do acorde como a tela escreve.
#[must_use]
pub fn nome(tonica: usize, qualidade: Qualidade) -> String {
    format!("{}{}", nome_da_classe(tonica), qualidade.sufixo())
}

/// Nome da classe de altura na convenção da tela: bemóis nas teclas pretas.
///
/// É a mesma convenção que o acorde usa — quem nomeia tonalidade e quem nomeia
/// acorde não podem divergir num `Db` que virou `C#`.
#[must_use]
pub fn nome_da_classe(classe: usize) -> &'static str {
    TONICAS[classe % 12]
}

/// Melhor acorde para um croma, com a margem sobre o segundo colocado.
fn melhor(croma: &[f32; 12]) -> (usize, Qualidade, f32) {
    let norma = (croma.iter().map(|v| v * v).sum::<f32>()).sqrt();
    if norma <= f32::EPSILON {
        return (0, Qualidade::Maior, 0.0);
    }
    let normalizado: [f32; 12] = std::array::from_fn(|i| croma[i] / norma);
    let mut pontuacoes: Vec<(usize, Qualidade, f32)> = Vec::new();
    for tonica in 0..12 {
        for qualidade in QUALIDADES {
            let t = template(tonica, qualidade);
            let similaridade: f32 = (0..12).map(|i| t[i] * normalizado[i]).sum();
            pontuacoes.push((tonica, qualidade, similaridade));
        }
    }
    pontuacoes.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
    let (tonica, qualidade, primeira) = pontuacoes[0];
    let segunda = pontuacoes[1].2;
    (
        tonica,
        com_terca(croma, tonica, qualidade),
        (primeira - segunda).max(0.0),
    )
}

/// Diz o acorde inteiro quando a terça **está** no áudio; senão, fica em potência.
///
/// Medido na primeira música real (91 trechos rotulados `5`): em **80%** deles a
/// terça não passa de 40% da fundamental — razão mediana 0,28. Inventar
/// maior/menor ali seria mentira; `5` é o que se pode defender. Nos 20% em que a
/// terça existe, o template de duas notas ganhava por diluição do croma, e aí o
/// motor volta a dizer o acorde completo.
fn com_terca(croma: &[f32; 12], tonica: usize, qualidade: Qualidade) -> Qualidade {
    if qualidade != Qualidade::Potencia {
        return qualidade;
    }
    let fundamental = croma[tonica].max(1e-6);
    let terca_maior = croma[(tonica + 4) % 12];
    let terca_menor = croma[(tonica + 3) % 12];
    if terca_maior.max(terca_menor) > 0.4 * fundamental {
        if terca_maior >= terca_menor {
            Qualidade::Maior
        } else {
            Qualidade::Menor
        }
    } else {
        Qualidade::Potencia
    }
}

/// Detecta a sequência de acordes e agrupa em trechos.
#[must_use]
pub fn detect(chroma: &Chroma) -> Vec<ChordSegment> {
    detect_com_melodia(chroma, None)
}

/// Detecta os acordes e deixa a **melodia** arbitrar a terça que o croma perdeu.
///
/// A melodia só é consultada para promover `5` (potência) a maior/menor; o que
/// já foi decidido pelo croma não é sobrescrito.
#[must_use]
pub fn detect_com_melodia(chroma: &Chroma, melodia: Option<&Melodia>) -> Vec<ChordSegment> {
    if chroma.quadros.is_empty() || chroma.fps <= 0.0 {
        return Vec::new();
    }
    // 1. Voto por quadro.
    let votos: Vec<(usize, Qualidade, f32)> = chroma.quadros.iter().map(melhor).collect();
    // 2. Suavização: moda do rótulo numa janela de ~0,5 s.
    let meia = ((0.25 * chroma.fps) as usize).max(1);
    let mut suave = votos.clone();
    for i in 0..votos.len() {
        let inicio = i.saturating_sub(meia);
        let fim = (i + meia + 1).min(votos.len());
        let mut contagem: Vec<(usize, Qualidade, usize)> = Vec::new();
        for voto in &votos[inicio..fim] {
            match contagem
                .iter_mut()
                .find(|(t, q, _)| *t == voto.0 && *q == voto.1)
            {
                Some((_, _, n)) => *n += 1,
                None => contagem.push((voto.0, voto.1, 1)),
            }
        }
        contagem.sort_by_key(|&(_, _, n)| std::cmp::Reverse(n));
        let (t, q, _) = contagem[0];
        suave[i] = (t, q, votos[i].2);
    }
    // 3. Agrupa quadros iguais em trechos, guardando a **estrutura** — tônica e
    //    qualidade — até o fim. O nome é a última coisa montada, porque o
    //    árbitro da melodia ainda pode mudar a qualidade.
    let mut trechos: Vec<Bruto> = Vec::new();
    let mut inicio = 0usize;
    while inicio < suave.len() {
        let mut fim = inicio + 1;
        while fim < suave.len()
            && suave[fim].0 == suave[inicio].0
            && suave[fim].1 == suave[inicio].1
        {
            fim += 1;
        }
        let confianca = suave[inicio..fim].iter().map(|v| v.2).sum::<f32>() / (fim - inicio) as f32;
        trechos.push(Bruto {
            inicio_s: inicio as f32 / chroma.fps,
            fim_s: fim as f32 / chroma.fps,
            tonica: suave[inicio].0,
            qualidade: suave[inicio].1,
            confianca,
        });
        inicio = fim;
    }
    // 4. Trecho curto demais deixa de existir: absorve o anterior, porque
    //    detector não muda de acorde a cada 200 ms.
    let minimo = 0.6f32;
    let mut limpos: Vec<Bruto> = Vec::with_capacity(trechos.len());
    for trecho in trechos {
        let curto = trecho.fim_s - trecho.inicio_s < minimo;
        match limpos.last_mut() {
            Some(anterior) if curto => anterior.fim_s = trecho.fim_s,
            _ => limpos.push(trecho),
        }
    }
    // 5. A melodia fecha a conta do que ficou em `5`.
    limpos
        .into_iter()
        .map(|trecho| {
            let qualidade = melodia.map_or(trecho.qualidade, |melodia| {
                com_a_melodia(
                    trecho.tonica,
                    trecho.qualidade,
                    trecho.inicio_s,
                    trecho.fim_s,
                    melodia,
                )
            });
            ChordSegment {
                inicio_s: trecho.inicio_s,
                fim_s: trecho.fim_s,
                acorde: nome(trecho.tonica, qualidade),
                confianca: trecho.confianca,
                tonica: trecho.tonica,
                qualidade,
            }
        })
        .collect()
}

/// Trecho antes de virar nome: a estrutura sobrevive até a última decisão.
struct Bruto {
    inicio_s: f32,
    fim_s: f32,
    tonica: usize,
    qualidade: Qualidade,
    confianca: f32,
}

/// Quantos quadros com F0 a melodia precisa ter no trecho para opinar.
const MINIMO_DE_QUADROS: usize = 3;

/// Fração dos quadros com F0 que a terça precisa somar para decidir.
const FRACAO_DA_TERCA: f32 = 0.4;

/// Promove `5` para maior/menor quando a melodia canta a terça.
///
/// Só toca em `Potencia`: qualidade já decidida não é sobrescrita. É o mesmo
/// princípio do `com_terca` — na dúvida, dizer menos. Se a melodia errar, o
/// custo é um `5` que continua `5`, não um acorde que vira o oposto.
fn com_a_melodia(
    tonica: usize,
    qualidade: Qualidade,
    inicio_s: f32,
    fim_s: f32,
    melodia: &Melodia,
) -> Qualidade {
    if qualidade != Qualidade::Potencia {
        return qualidade;
    }
    let com_f0 = melodia.quadros_com_f0(inicio_s, fim_s);
    if com_f0 < MINIMO_DE_QUADROS {
        return qualidade;
    }
    let contagem = melodia.contagem(inicio_s, fim_s);
    let maior = contagem[(tonica + 4) % 12];
    let menor = contagem[(tonica + 3) % 12];
    let limite = FRACAO_DA_TERCA * com_f0 as f32;
    if maior as f32 >= limite && maior > menor {
        Qualidade::Maior
    } else if menor as f32 >= limite && menor > maior {
        Qualidade::Menor
    } else {
        qualidade
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_barra_entra_so_quando_o_baixo_informa() {
        // C2=36, E2=40, F2=41, G2=43
        assert_eq!(
            com_baixo(0, Qualidade::Maior, 40).as_deref(),
            Some("C/E"),
            "terça no baixo é 1ª inversão"
        );
        assert_eq!(
            com_baixo(0, Qualidade::Maior, 43).as_deref(),
            Some("C/G"),
            "quinta no baixo é 2ª inversão"
        );
        assert_eq!(
            com_baixo(7, Qualidade::Sete, 41).as_deref(),
            Some("G7/F"),
            "sétima no baixo"
        );
        assert_eq!(
            com_baixo(9, Qualidade::Menor, 48).as_deref(),
            Some("Am/C"),
            "terça menor no baixo"
        );
        // Sem barra: fundamental, nota estranha ao acorde, e acordes sem terça,
        // onde a inversão é decisão de nome (Csus4/F = Fsus2).
        assert_eq!(com_baixo(0, Qualidade::Maior, 36), None, "fundamental");
        assert_eq!(com_baixo(0, Qualidade::Maior, 41), None, "F não é de C");
        assert_eq!(com_baixo(0, Qualidade::SusQuatro, 41), None, "sus4");
        assert_eq!(com_baixo(0, Qualidade::Potencia, 43), None, "potência");
    }

    /// Croma sintético: só as classes dadas têm energia.
    fn croma(pares: &[(usize, f32)]) -> [f32; 12] {
        let mut vetor = [0.0f32; 12];
        for &(classe, valor) in pares {
            vetor[classe] = valor;
        }
        vetor
    }

    #[test]
    fn terca_presente_troca_potencia_por_acorde_completo() {
        // Dó com terça maior em 0,45 e quinta forte: o casamento cru prefere a
        // potência (é o caso que a medição na música real mostrou), e a regra da
        // terça devolve o acorde inteiro.
        let valor = croma(&[(0, 1.0), (4, 0.45), (7, 1.5)]);
        assert_eq!(
            com_terca(&valor, 0, Qualidade::Potencia),
            Qualidade::Maior,
            "terça acima de 0,4 da fundamental tem de virar acorde completo"
        );
        assert_eq!(melhor(&valor).1, Qualidade::Maior);
    }

    #[test]
    fn terca_ausente_ou_fraca_fica_potencia_e_nao_inventa() {
        // Sem terça nenhuma: potência é o que se pode defender.
        assert_eq!(
            com_terca(&croma(&[(0, 1.0), (7, 0.9)]), 0, Qualidade::Potencia),
            Qualidade::Potencia
        );
        // 0,3 da fundamental fica abaixo do limiar medido (0,4): não vira menor.
        assert_eq!(
            com_terca(
                &croma(&[(0, 1.0), (3, 0.3), (7, 0.9)]),
                0,
                Qualidade::Potencia
            ),
            Qualidade::Potencia
        );
        // Terça menor presente: escolhe menor, não maior.
        assert_eq!(
            com_terca(
                &croma(&[(0, 1.0), (3, 0.6), (7, 0.9)]),
                0,
                Qualidade::Potencia
            ),
            Qualidade::Menor
        );
        // Qualidade já decidida não é mexida.
        assert_eq!(
            com_terca(&croma(&[(0, 1.0)]), 0, Qualidade::MaiorSete),
            Qualidade::MaiorSete
        );
    }
}
