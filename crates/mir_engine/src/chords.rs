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
    // 3. Agrupa quadros iguais em trechos.
    let mut trechos: Vec<ChordSegment> = Vec::new();
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
        trechos.push(ChordSegment {
            inicio_s: inicio as f32 / chroma.fps,
            fim_s: fim as f32 / chroma.fps,
            acorde: nome(suave[inicio].0, suave[inicio].1),
            confianca,
        });
        inicio = fim;
    }
    // 4. Trecho curto demais deixa de existir: absorve o anterior, porque
    //    detector não muda de acorde a cada 200 ms.
    let minimo = 0.6f32;
    let mut limpos: Vec<ChordSegment> = Vec::with_capacity(trechos.len());
    for trecho in trechos {
        let curto = trecho.fim_s - trecho.inicio_s < minimo;
        match limpos.last_mut() {
            Some(anterior) if curto => anterior.fim_s = trecho.fim_s,
            _ => limpos.push(trecho),
        }
    }
    limpos
}

#[cfg(test)]
mod tests {
    use super::*;

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
