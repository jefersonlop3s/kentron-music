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
    /// `6` e `sus2` **não são detectados** — não estão em `QUALIDADES`. Existem
    /// como nome porque o mesmo conjunto de notas tem dois nomes legítimos e
    /// **o baixo decide qual**: `Dm7` com o baixo na terça é `F6` em estado
    /// fundamental, e `Bbsus4` com o baixo na quarta é `Ebsus2`. Medido no
    /// repertório (2026-11-22): 39,5% das inversões são esse par de nomes, não
    /// inversão.
    Seis,
    SusDois,
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
            Self::Seis => "6",
            Self::SusDois => "sus2",
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
            Self::Seis => &[0, 4, 7, 9],
            Self::SusDois => &[0, 2, 7],
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

/// Qualidades que podem **nomear** o conjunto quando o baixo vira a raiz.
///
/// Inclui `6` e `sus2`, que não são detectados mas são o outro nome legítimo do
/// mesmo conjunto de notas.
const QUALIDADES_NOME: [Qualidade; 10] = [
    Qualidade::Maior,
    Qualidade::Menor,
    Qualidade::Sete,
    Qualidade::MenorSete,
    Qualidade::MaiorSete,
    Qualidade::SusQuatro,
    Qualidade::SusDois,
    Qualidade::Diminuto,
    Qualidade::Potencia,
    Qualidade::Seis,
];

/// O mesmo conjunto de notas lido com o **baixo como raiz**, se isso for um
/// acorde conhecido.
///
/// É a decisão que o croma não pode tomar: `Dm7` e `F6` têm as mesmas quatro
/// classes de altura, então a correlação é idêntica e a escolha seria arbitrária.
/// O baixo desempata — e só ele.
#[must_use]
pub fn nome_pelo_baixo(
    tonica: usize,
    qualidade: Qualidade,
    baixo_midi: u8,
) -> Option<(usize, Qualidade)> {
    let classe = usize::from(baixo_midi) % 12;
    let notas: Vec<usize> = qualidade
        .intervalos()
        .iter()
        .map(|i| (tonica + i) % 12)
        .collect();
    if classe == tonica || !notas.contains(&classe) {
        return None;
    }
    let mut reordenado: Vec<usize> = notas.iter().map(|n| (n + 12 - classe) % 12).collect();
    reordenado.sort_unstable();
    QUALIDADES_NOME
        .iter()
        .find(|q| {
            let mut ivs = q.intervalos().to_vec();
            ivs.sort_unstable();
            ivs == reordenado
        })
        .map(|&q| (classe, q))
}

/// Cifra com o baixo: **renomeia** quando o conjunto tem outro nome, e escreve
/// a barra quando é inversão de verdade.
///
/// `None` quando o baixo não informa nada: fora do acorde, igual à fundamental,
/// quarta ou segunda (que são adição, não inversão) e acordes sem terça.
#[must_use]
pub fn com_baixo(tonica: usize, qualidade: Qualidade, baixo_midi: u8) -> Option<String> {
    let classe = usize::from(baixo_midi) % 12;
    // 1. O conjunto lido pelo baixo é acorde conhecido: o baixo é a raiz.
    if let Some((t, q)) = nome_pelo_baixo(tonica, qualidade, baixo_midi) {
        return Some(nome(t, q));
    }
    // 2. Senão é inversão — e a barra só entra onde a identidade não muda.
    let intervalo = (classe + 12 - tonica) % 12;
    if intervalo == 0 || !qualidade.tem_terca() || !qualidade.intervalos().contains(&intervalo) {
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
    /// De onde veio a terça que o nome carrega. Publicado porque **a terça é a
    /// decisão de que o croma mais erra**: sem isto, um `C` que veio do croma
    /// (razão 0,45 da fundamental) e um `C` que veio da melodia são o mesmo
    /// texto na tela, e nenhum dos dois é conferível.
    pub origem_terca: OrigemDaTerca,
    /// Razão terça/fundamental medida no croma. É o número que a regra usa
    /// (`LIMIAR_DA_TERCA`); `0,0` quando o template já tinha terça.
    pub terca_razao: f32,
    /// A maioria dos quadros do trecho **cru** era potência: só aí a regra pôde
    /// agir. É o denominador honesto de "quantos trechos ganharam terça".
    pub potencia_no_croma: bool,
}

/// De onde veio a terça (ou a ausência dela) no nome do trecho.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum OrigemDaTerca {
    /// O template já tinha terça: nada foi acrescentado.
    Template,
    /// O croma tinha terça acima do limiar e a regra devolveu o acorde inteiro.
    Croma,
    /// O croma perdeu a terça e a **melodia** a trouxe (`com_a_melodia`).
    Melodia,
    /// O nome final **não tem terça** — ficou em potência (ou `sus`). É a leitura
    /// honesta quando a terça não está no áudio, e é o denominador do risco:
    /// "ganhou terça" só significa alguma coisa contra quantos ficaram sem ela.
    Ausente,
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

/// Melhor acorde para um croma, com a margem sobre o segundo colocado — e o que
/// a terça mediu: a razão terça/fundamental, a qualidade **crua** do template e a
/// margem.
///
/// A razão sai junto porque a decisão da terça era invisível: o nome final não
/// diz se veio do template, do croma ou da melodia, nem com que número. Sem isso
/// não há como medir "quantos trechos ganharam terça que não têm".
fn melhor(croma: &[f32; 12]) -> (usize, Qualidade, f32, Qualidade, f32) {
    let norma = (croma.iter().map(|v| v * v).sum::<f32>()).sqrt();
    if norma <= f32::EPSILON {
        return (0, Qualidade::Maior, 0.0, Qualidade::Maior, 0.0);
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
    let (final_q, razao) = com_terca(croma, tonica, qualidade);
    (
        tonica,
        final_q,
        razao,
        qualidade,
        (primeira - segunda).max(0.0),
    )
}

/// Terça mínima (fração da fundamental) para o croma devolver o acorde inteiro.
///
/// **0,40 é medido, não escolhido** (2026-11-22): nos 91 trechos rotulados `5`
/// da primeira música real, a razão terça/fundamental tem mediana **0,28** e fica
/// abaixo de 0,4 em **80%** deles — ali a terça não está audível e `5` é a
/// leitura honesta. Nos 20% restantes o template de duas notas ganhava por
/// diluição do croma, e o motor volta a dizer o acorde completo. **Limite
/// declarado:** medido em uma música; o número de cada trecho passou a sair em
/// `ChordSegment::terca_razao` para a decisão ser conferível no repertório.
const LIMIAR_DA_TERCA: f32 = 0.4;

/// Diz o acorde inteiro quando a terça **está** no áudio; senão, fica em potência.
///
/// Devolve também **a razão medida**: um número ao lado da decisão é o que
/// permite ver se o limiar corta perto da fronteira (risco de inventar terça) ou
/// no vazio (decisão folgada). `(qualidade, razao)`, com `razao = 0,0` quando o
/// template já tinha terça.
fn com_terca(croma: &[f32; 12], tonica: usize, qualidade: Qualidade) -> (Qualidade, f32) {
    if qualidade != Qualidade::Potencia {
        return (qualidade, 0.0);
    }
    let fundamental = croma[tonica].max(1e-6);
    let terca_maior = croma[(tonica + 4) % 12];
    let terca_menor = croma[(tonica + 3) % 12];
    let razao = terca_maior.max(terca_menor) / fundamental;
    if razao > LIMIAR_DA_TERCA {
        let decidida = if terca_maior >= terca_menor {
            Qualidade::Maior
        } else {
            Qualidade::Menor
        };
        (decidida, razao)
    } else {
        (Qualidade::Potencia, razao)
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
    // 1. Voto por quadro. Cada voto carrega o **cru** (qualidade do template) e a
    //    razão da terça: a agregação precisa deles para dizer de onde veio a terça.
    let votos: Vec<Voto> = chroma.quadros.iter().map(melhor).collect();
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
        suave[i] = (t, q, votos[i].2, votos[i].3, votos[i].4);
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
        let confianca = suave[inicio..fim].iter().map(|v| v.4).sum::<f32>() / (fim - inicio) as f32;
        let potencia_no_croma = fatia_potencia(&votos[inicio..fim]);
        let crua = qualidade_mais_frequente(&votos[inicio..fim]);
        let razoes: Vec<f32> = votos[inicio..fim]
            .iter()
            .filter(|v| v.3 == Qualidade::Potencia)
            .map(|v| v.2)
            .collect();
        let terca_razao = if razoes.is_empty() {
            0.0
        } else {
            razoes.iter().sum::<f32>() / razoes.len() as f32
        };
        trechos.push(Bruto {
            inicio_s: inicio as f32 / chroma.fps,
            fim_s: fim as f32 / chroma.fps,
            tonica: suave[inicio].0,
            qualidade: suave[inicio].1,
            confianca,
            potencia_no_croma,
            crua,
            terca_razao,
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
            // Quem acrescentou a terça: ninguém (o template já a tinha), o croma
            // (razão acima do limiar) ou a melodia. Sem isto, os três casos
            // chegam à tela com o mesmo nome.
            let origem_terca = if qualidade != trecho.qualidade {
                OrigemDaTerca::Melodia
            } else if trecho.qualidade != trecho.crua {
                OrigemDaTerca::Croma
            } else if trecho.crua.tem_terca() {
                OrigemDaTerca::Template
            } else {
                OrigemDaTerca::Ausente
            };
            ChordSegment {
                inicio_s: trecho.inicio_s,
                fim_s: trecho.fim_s,
                acorde: nome(trecho.tonica, qualidade),
                confianca: trecho.confianca,
                tonica: trecho.tonica,
                qualidade,
                origem_terca,
                terca_razao: trecho.terca_razao,
                potencia_no_croma: trecho.potencia_no_croma,
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
    /// A maioria dos quadros tinha qualidade **crua** de potência: só aí a regra
    /// da terça pôde agir. É o denominador de "quantos trechos ganharam terça".
    potencia_no_croma: bool,
    /// Qualidade mais frequente **no template**, antes da regra da terça.
    crua: Qualidade,
    /// Razão terça/fundamental média dos quadros de potência do trecho.
    terca_razao: f32,
}

/// Voto de um quadro: tônica, qualidade final, razão da terça, qualidade **crua**
/// do template e margem sobre o segundo colocado.
type Voto = (usize, Qualidade, f32, Qualidade, f32);

/// A maioria dos quadros da fatia era potência no template.
fn fatia_potencia(fatia: &[Voto]) -> bool {
    if fatia.is_empty() {
        return false;
    }
    fatia.iter().filter(|v| v.3 == Qualidade::Potencia).count() * 2 > fatia.len()
}

/// Qualidade crua mais frequente da fatia: o que o template escolheu, sem regra.
fn qualidade_mais_frequente(fatia: &[Voto]) -> Qualidade {
    let mut contagem: Vec<(Qualidade, usize)> = Vec::new();
    for voto in fatia {
        match contagem.iter_mut().find(|(q, _)| *q == voto.3) {
            Some((_, n)) => *n += 1,
            None => contagem.push((voto.3, 1)),
        }
    }
    contagem
        .iter()
        .max_by_key(|(_, n)| *n)
        .map_or(Qualidade::Maior, |&(q, _)| q)
}

/// Quantos quadros com F0 a melodia precisa ter no trecho para opinar.
pub const MINIMO_DE_QUADROS: usize = 3;

/// Fração dos quadros com F0 que a terça precisa somar para decidir.
///
/// **0,20 é medido, não escolhido** (2026-11-22): nos três acordes de `Me Derramar`
/// com verdade conhecida, a terça aparece em **21%** (`G5` → `G9`, terça B),
/// **29%** (`Gsus2` → `G9`) e **20%** (`C5` → `C9`, terça E) dos quadros com F0.
/// O 0,40 anterior só servia para melodia parada — **melodia de verdade se move**,
/// e toca a fundamental, a quinta, a nona e a terça no mesmo trecho. **Limite
/// declarado:** calibrado em três acordes de uma música; o piso precisa de casos
/// negativos para não disparar terça onde não há. É o mesmo critério que
/// `Report.melodia[].terca_confirmada` usa para confirmar a terça do croma.
pub const FRACAO_DA_TERCA: f32 = 0.2;

/// Promove `5` para maior/menor quando a melodia canta a terça.
///
/// Só toca em `Potencia`: qualidade já decidida não é sobrescrita. É o mesmo
/// princípio do `com_terca` — na dúvida, dizer menos. Se a melodia errar, o
/// custo é um `5` que continua `5`, não um acorde que vira o oposto.
/// Promove a maior/menor o acorde **sem terça** quando a melodia traz a terça.
///
/// Vale para `5` (potência), `sus4` e `sus2`: são os três casos em que o croma
/// perdeu a terça e o nome ficou sem qualidade. Medido em `Me Derramar`
/// (2026-11-22): o motor dizia `G5` e `Gsus2` onde é `G9` — e a melodia tocava
/// **B (a terça)** e **A (a nona)** no mesmo trecho. A terça vem da melodia; é a
/// mesma tese do bloco 3 da ADR-0014, aplicada a quem não é potência pura.
fn com_a_melodia(
    tonica: usize,
    qualidade: Qualidade,
    inicio_s: f32,
    fim_s: f32,
    melodia: &Melodia,
) -> Qualidade {
    if !matches!(
        qualidade,
        Qualidade::Potencia | Qualidade::SusQuatro | Qualidade::SusDois
    ) {
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
    fn a_terca_fraca_da_melodia_promove_potencia_e_sus() {
        // Medido em `Me Derramar` (2026-11-22): a melodia toca a terça em **21%**
        // dos quadros com F0, porque no mesmo trecho ela também toca a
        // fundamental, a quinta e a nona. Com o limiar antigo de 0,40 o motor
        // ficava em `G5` onde a verdade é `G` — melodia de verdade se move.
        let mut classes = vec![Some(11usize); 7]; // B = terça maior de G
        classes.extend(vec![Some(7usize); 20]); // G = fundamental
        classes.extend(vec![Some(2usize); 4]); // D = quinta
        classes.push(None); // um quadro sem F0
        let melodia = Melodia { classes, fps: 10.0 };
        assert_eq!(
            com_a_melodia(7, Qualidade::Potencia, 0.0, 3.1, &melodia),
            Qualidade::Maior,
            "terça a 23% dos quadros com F0 tem de promover"
        );
        assert_eq!(
            com_a_melodia(7, Qualidade::SusQuatro, 0.0, 3.1, &melodia),
            Qualidade::Maior,
            "sus4 sem terça no croma também é promovido pela melodia"
        );
    }

    #[test]
    fn sem_terca_na_melodia_nao_inventa_qualidade() {
        // Prova negativa: melodia que só toca fundamental e quinta não decide.
        let classes = vec![Some(7usize); 20]
            .into_iter()
            .chain(vec![Some(2usize); 5])
            .collect();
        let melodia = Melodia { classes, fps: 10.0 };
        assert_eq!(
            com_a_melodia(7, Qualidade::Potencia, 0.0, 2.5, &melodia),
            Qualidade::Potencia,
            "sem terça o motor não inventa qualidade"
        );
    }

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
        // O baixo **renomeia** quando o conjunto tem dois nomes legítimos:
        // Dm7 (D-F-A-C) com o baixo na terça é F6 (F-A-C-D) em fundamental, e
        // Bbsus4 com o baixo na quarta é Ebsus2. O croma empata nos dois.
        assert_eq!(
            com_baixo(2, Qualidade::MenorSete, 41).as_deref(),
            Some("F6"),
            "Dm7 com F no baixo é F6"
        );
        assert_eq!(
            com_baixo(10, Qualidade::SusQuatro, 51).as_deref(),
            Some("Ebsus2"),
            "Bbsus4 com Eb no baixo é Ebsus2"
        );
        assert_eq!(
            nome_pelo_baixo(2, Qualidade::MenorSete, 41),
            Some((5, Qualidade::Seis)),
            "a leitura pelo baixo devolve Fá com sexta"
        );
        // Sem barra e sem renome: fundamental, nota estranha ao acorde, e
        // potência (que não tem outro nome nem terça).
        assert_eq!(com_baixo(0, Qualidade::Maior, 36), None, "fundamental");
        assert_eq!(com_baixo(0, Qualidade::Maior, 41), None, "F não é de C");
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
        let (qualidade, razao) = com_terca(&valor, 0, Qualidade::Potencia);
        assert_eq!(
            qualidade,
            Qualidade::Maior,
            "terça acima de 0,4 da fundamental tem de virar acorde completo"
        );
        // A razão medida sai junto da decisão: é ela que diz o quanto a decisão
        // está perto do limiar.
        assert!(
            (razao - 0.45).abs() < 1e-5,
            "a razão medida tem de acompanhar a decisão: {razao}"
        );
        assert_eq!(melhor(&valor).1, Qualidade::Maior);
    }

    #[test]
    fn terca_ausente_ou_fraca_fica_potencia_e_nao_inventa() {
        // Sem terça nenhuma: potência é o que se pode defender — e a razão zero é
        // o que prova que não havia nada para promover.
        assert_eq!(
            com_terca(&croma(&[(0, 1.0), (7, 0.9)]), 0, Qualidade::Potencia),
            (Qualidade::Potencia, 0.0)
        );
        // 0,3 da fundamental fica abaixo do limiar medido (0,4): não vira menor —
        // e a razão 0,3 é o que mostra que a decisão ali estava perto da borda.
        let (q, r) = com_terca(
            &croma(&[(0, 1.0), (3, 0.3), (7, 0.9)]),
            0,
            Qualidade::Potencia,
        );
        assert_eq!(q, Qualidade::Potencia, "0,3 fica abaixo do limiar de 0,4");
        assert!((r - 0.3).abs() < 1e-5, "razão medida 0,3: {r}");
        // Terça menor presente: escolhe menor, não maior.
        assert_eq!(
            com_terca(
                &croma(&[(0, 1.0), (3, 0.6), (7, 0.9)]),
                0,
                Qualidade::Potencia
            )
            .0,
            Qualidade::Menor
        );
        // Qualidade já decidida não é mexida — e a razão sai 0, porque a regra não
        // foi consultada.
        assert_eq!(
            com_terca(&croma(&[(0, 1.0)]), 0, Qualidade::MaiorSete),
            (Qualidade::MaiorSete, 0.0)
        );
    }

    /// Croma sintético de potência pura: fundamental e quinta, sem terça nenhuma.
    fn chroma_de_potencia() -> Chroma {
        Chroma {
            quadros: (0..40).map(|_| croma(&[(0, 1.0), (7, 1.2)])).collect(),
            fps: 10.0,
        }
    }

    #[test]
    fn o_trecho_publica_de_onde_veio_a_terca() {
        // Potência pura: o nome tem de sair sem terça e o trecho tem de DIZER
        // isso (`Ausente`), com a razão medida perto de zero. Sem este campo, o
        // relatório mostrava o mesmo `C5` sem dizer que a terça foi procurada.
        let trechos = detect(&chroma_de_potencia());
        assert!(!trechos.is_empty(), "croma de potência não gerou trecho");
        for trecho in &trechos {
            assert_eq!(trecho.qualidade, Qualidade::Potencia, "{}", trecho.acorde);
            assert_eq!(trecho.origem_terca, OrigemDaTerca::Ausente);
            assert!(trecho.potencia_no_croma, "{}", trecho.acorde);
            assert!(
                trecho.terca_razao < LIMIAR_DA_TERCA,
                "razão acima do limiar sem terça no croma: {}",
                trecho.terca_razao
            );
        }

        // Com a terça no croma — fraca o bastante para o template de potência
        // ainda ganhar no cru, que é o caso medido na música real —, o MESMO
        // caminho tem de dizer `Croma` e publicar a razão: é a prova de que o
        // campo mede a decisão, e não repete uma constante.
        let mut com_terca_no_croma = chroma_de_potencia();
        for quadro in &mut com_terca_no_croma.quadros {
            *quadro = croma(&[(0, 1.0), (4, 0.45), (7, 1.5)]);
        }
        let trechos = detect(&com_terca_no_croma);
        assert!(!trechos.is_empty());
        for trecho in &trechos {
            assert_eq!(
                trecho.origem_terca,
                OrigemDaTerca::Croma,
                "{}",
                trecho.acorde
            );
            assert_eq!(
                trecho.acorde, "C",
                "a terça presente devolve o acorde inteiro"
            );
            assert!(
                trecho.terca_razao > LIMIAR_DA_TERCA,
                "a razão publicada tem de estar acima do limiar: {}",
                trecho.terca_razao
            );
        }
    }
}
