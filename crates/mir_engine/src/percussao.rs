//! A relação entre o **bumbo** e a **caixa** — o que separa `6/8` de `3/4`.
//!
//! A pergunta do dono (2026-11-24) expôs a falha da medida anterior: a `dobra`
//! somava bumbo, caixa, chimbau, violão e voz num fluxo espectral só, e **para
//! essa soma o acento do bumbo e o da caixa são o mesmo tipo de evento**. Não
//! são: o bumbo marca o **tempo 1**, a caixa marca o **contratempo**. Somados,
//! `6/8` (bumbo em 1, caixa em 4) e `3/4` (bumbo em 1, caixa em 2 e 3) produzem
//! o mesmo envelope — contraste **0,42 nos dois**, empate exato medido.
//!
//! Aqui os dois são medidos **separados**, e o metro sai da relação:
//!
//! - **`6/8`**: a caixa cai na **metade** do ciclo do bumbo (dois grupos de três).
//! - **`3/4`**: a caixa cai em **um terço e dois terços** (três grupos de dois).
//!
//! **Este módulo pressupõe o stem de bateria** (ADR-0017). No mix o número sai
//! contaminado, e isso está medido: a banda de 30–110 Hz carrega o baixo, e a
//! "barra do bumbo" sai **0,49–1,28 s** em vez dos 2–3 s de um compasso.

use crate::spectrum::Spectrogram;
use serde::Serialize;

/// Faixa do bumbo. O baixo vive aqui também — ver o cabeçalho.
const BUMBO_DE_HZ: f32 = 30.0;
const BUMBO_ATE_HZ: f32 = 110.0;
/// Faixa da caixa: a **pele**, não os fios.
///
/// Medido em 2026-11-24, no `binky`: com os fios (1,5–5 kHz) o histograma da
/// caixa fica `52% 21% 13% 6% 4% 4%` — massa empilhada logo depois do bumbo. A
/// causa **não é timbre**: o ataque do bumbo é um transiente banda-larga e dispara
/// em **todas** as bandas no mesmo instante (medido: pele `52% 21% 13%`, fios
/// `32% 29% 17%`, prato `25% 35% 18%` — a mesma forma). A pele da caixa
/// (150–450 Hz) é onde o corpo dela vive e o prato não tem energia.
const CAIXA_DE_HZ: f32 = 150.0;
const CAIXA_ATE_HZ: f32 = 450.0;
/// Tolerância para considerar que uma caixa **coincide** com o bumbo e descartá-la.
///
/// O ataque do bumbo vaza para a banda da caixa, e a caixa real fica encoberta
/// por ele. Medido: sem a supressão o gabarito fica em 5 de 6; com 0,06 s (e com
/// 0,10 s) fica em **6 de 6**.
const COINCIDENCIA_S: f32 = 0.06;
/// Ciclo plausível do bumbo, em segundos.
const CICLO_MIN_S: f32 = 0.4;
const CICLO_MAX_S: f32 = 5.0;
/// Faixa em que uma **barra real** cabe: 70 BPM em `4/4` dá 3,43 s, 176 BPM em
/// `3/4` dá 1,02 s. O que passa disso é o **hipercompasso** — medido no corpus de
/// 40 músicas: barra mediana de 5,08 s contra troca de acorde mediana de 1,90 s.
const BARRA_MIN_S: f32 = 0.85;
const BARRA_MAX_S: f32 = 3.60;
/// Em quantas partes o ciclo bruto pode estar subdividido: `T`, `T/2`, `T/3`, `T/4`.
const DIVISORES: [f32; 4] = [1.0, 2.0, 3.0, 4.0];
/// Banda em que dois candidatos contam como **empatados** na distância ao alvo.
const EMPATE_ANCORA: f32 = 1.05;
/// Tolerância para uma troca de acorde contar como **alinhada** ao ciclo.
const ALINHAMENTO: f32 = 0.10;
/// Piso do ataque, em desvios-padrão acima da média da própria banda.
const PISO_ATAQUE: f32 = 1.5;
/// Espaçamento mínimo entre ataques da mesma peça.
const ESPACO_MIN_S: f32 = 0.08;
/// Quantas fatias do ciclo do bumbo o histograma da caixa usa.
pub const FATIAS: usize = 6;

/// A relação medida entre o bumbo e a caixa.
#[derive(Debug, Clone, Serialize, Default)]
pub struct BumboCaixa {
    /// Período do bumbo — a barra quando ele toca uma vez por compasso.
    pub periodo_bumbo_s: f32,
    /// Ataques de bumbo e de caixa encontrados.
    pub bumbos: usize,
    pub caixas: usize,
    /// Onde cada caixa cai no ciclo do bumbo, em [`FATIAS`] fatias normalizadas.
    pub caixa_no_ciclo: [f32; FATIAS],
    /// Contraste da hipótese **`6/8`**: polos em `s1` e `s4` (a metade) contra o
    /// resto, normalizado. Ver [`BumboCaixa::dobra`].
    pub escore_68: f32,
    /// Contraste da hipótese **`3/4`**: polos em `s1`, `s3` e `s5` (os terços)
    /// contra o resto, normalizado.
    pub escore_34: f32,
    /// Em quantas partes a caixa dobra o ciclo do bumbo: **2** (metade, `6/8`),
    /// **3** (terços, `3/4`) ou **0** quando não há evidência.
    ///
    /// **Competição de hipóteses, não limiar.** Medido em 2026-11-24, no gabarito
    /// independente: a comparação anterior (`fatia 3` contra `fatias 2+4`) dava
    /// **4 de 6**; esta dá **5 de 6**. A diferença é usar **as seis fatias
    /// normalizadas** em vez de duas posições soltas — o limiar local confundia
    /// ruído com contraste.
    ///
    /// A **rotação circular** foi testada e **piorou** (5 → 4): a origem do ciclo
    /// é o último bumbo, que **já é um polo** nos dois metros, então rotacionar só
    /// deixa a hipótese errada encontrar alinhamento espúrio.
    pub dobra: u8,
    /// Por que esta leitura.
    pub regra: String,
}

/// Novidade (fluxo positivo) da energia de uma banda, normalizada pela média.
fn novidade_da_banda(spectro: &Spectrogram, taxa: u32, de_hz: f32, ate_hz: f32) -> Vec<f32> {
    let energia = spectro.energia_banda(taxa, de_hz, ate_hz);
    let mut saida = Vec::with_capacity(energia.len());
    let mut anterior = energia.first().copied().unwrap_or(0.0);
    for &e in &energia {
        saida.push((e - anterior).max(0.0));
        anterior = e;
    }
    let media = saida.iter().sum::<f32>() / (saida.len().max(1) as f32);
    if media > f32::EPSILON {
        for x in &mut saida {
            *x /= media;
        }
    }
    saida
}

/// Período dominante por autocorrelação, com **interpolação parabólica** — a
/// mesma que o `structure::estimate_tempo` usa, e pelo mesmo motivo: sem ela o
/// período só assume valores de lag inteiro.
fn periodo_dominante(novidade: &[f32], fps: f32, min_s: f32, max_s: f32) -> f32 {
    let media = novidade.iter().sum::<f32>() / (novidade.len().max(1) as f32);
    let curva: Vec<f32> = novidade.iter().map(|v| (v - media).max(0.0)).collect();
    let min_lag = (fps * min_s).round() as usize;
    let max_lag = ((fps * max_s).round() as usize)
        .min(curva.len() / 2)
        .max(min_lag + 2);
    if curva.len() < 8 || max_lag <= min_lag + 1 {
        return 0.0;
    }
    let correlacao = |lag: usize| -> f32 {
        (0..curva.len() - lag)
            .map(|i| curva[i] * curva[i + lag])
            .sum()
    };
    let mut melhor = (min_lag, f32::MIN);
    for lag in min_lag..max_lag {
        let soma = correlacao(lag);
        if soma > melhor.1 {
            melhor = (lag, soma);
        }
    }
    // Correlação nula é ausência de periodicidade, não período no menor lag —
    // sem esta guarda o silêncio devolvia `min_lag` e o módulo inventava uma
    // dobra.
    if melhor.1 <= f32::EPSILON {
        return 0.0;
    }
    let lag = if melhor.0 > min_lag && melhor.0 + 1 < max_lag {
        let (a, b, c) = (correlacao(melhor.0 - 1), melhor.1, correlacao(melhor.0 + 1));
        let denominador = a - 2.0 * b + c;
        if denominador.abs() > f32::EPSILON {
            (melhor.0 as f32 + 0.5 * (a - c) / denominador)
                .clamp(melhor.0 as f32 - 0.5, melhor.0 as f32 + 0.5)
        } else {
            melhor.0 as f32
        }
    } else {
        melhor.0 as f32
    };
    lag / fps
}

/// Ataques de uma novidade: máximo local acima do piso, com espaçamento mínimo.
///
/// **Piso relativo à própria banda**, e é isso que faz o bumbo e a caixa serem
/// comparáveis: o piso absoluto deixaria a banda mais energética dominar.
fn ataques(novidade: &[f32], fps: f32) -> Vec<f32> {
    if novidade.len() < 3 {
        return Vec::new();
    }
    let media = novidade.iter().sum::<f32>() / novidade.len() as f32;
    let variancia = novidade
        .iter()
        .map(|v| (v - media) * (v - media))
        .sum::<f32>()
        / novidade.len() as f32;
    let piso = media + PISO_ATAQUE * variancia.sqrt();
    // Sem variância não há ataque: a novidade constante (silêncio, ou banda
    // vazia) passaria no piso e cada quadro viraria um ataque.
    if variancia <= f32::EPSILON {
        return Vec::new();
    }
    let espaco = (ESPACO_MIN_S * fps).round() as usize;
    let mut saida: Vec<f32> = Vec::new();
    let mut ultimo: Option<usize> = None;
    for i in 1..novidade.len() - 1 {
        if novidade[i] < piso || novidade[i] < novidade[i - 1] || novidade[i] < novidade[i + 1] {
            continue;
        }
        if let Some(u) = ultimo {
            if i - u < espaco {
                continue;
            }
        }
        ultimo = Some(i);
        saida.push(i as f32 / fps);
    }
    saida
}

/// O intervalo mediano entre trocas de acorde — **a escala em que a música anda**.
///
/// É o relógio da harmonia, não o da percussão: medido no corpus de 40 músicas,
/// ele dá 1,90 s enquanto o ciclo bruto da percussão dá 5,08 s. Os dois discordam
/// por um fator de 2 a 5 — e é essa discordância que denuncia o hipercompasso.
fn intervalo_harmonico(cortes: &[f32]) -> Option<f32> {
    let mut intervalos: Vec<f32> = cortes
        .windows(2)
        .map(|par| par[1] - par[0])
        .filter(|d| (0.3..15.0).contains(d))
        .collect();
    if intervalos.len() < 2 {
        return None;
    }
    intervalos.sort_by(f32::total_cmp);
    Some(intervalos[intervalos.len() / 2])
}

/// A fração das trocas de acorde que caem **na grade** de um ciclo.
///
/// A troca de acorde acontece na fronteira da barra (ou num múltiplo dela), então
/// um ciclo que não tem as trocas nas suas fronteiras é fração ou múltiplo da
/// barra. Medido nos três critérios possíveis, em 2026-11-24: o alinhamento
/// **sozinho** separa mal (custos todos entre 0,40 e 0,50, e ele quebra o
/// `fallin`, levando a barra para a metade); ele serve como **desempate**.
fn alinhamento(periodo: f32, cortes: &[f32]) -> f32 {
    let (Some(&origem), true) = (cortes.first(), periodo > 0.0 && cortes.len() >= 3) else {
        return 0.0;
    };
    let na_grade = cortes
        .iter()
        .filter(|&&t| {
            let voltas = (t - origem) / periodo;
            let fracao = voltas - voltas.floor();
            fracao.min(1.0 - fracao) <= ALINHAMENTO
        })
        .count();
    na_grade as f32 / cortes.len() as f32
}

/// Escolhe a barra entre os divisores do ciclo bruto.
///
/// **A bateria dá a fase; a harmonia dá a escala.** O ciclo bruto da percussão
/// pode estar certo (`T`) ou em submúltiplo (`T/2`, `T/3`, `T/4`), e o que decide
/// **não** é o contraste do histograma: medido, ele degenera quando o
/// denominador é nulo (`5e8` contra `3,3e8`) e prefere o ciclo longo — foi assim
/// que o corpus ficou em 5,08 s de barra mediana.
///
/// O que decide é o **ritmo harmônico**: a troca de acorde cai na fronteira da
/// barra, então a barra é o candidato mais próximo do intervalo mediano entre
/// trocas. Medido nos 40 do corpus e nos 6 do gabarito: a barra mediana cai de
/// 5,75 s para **1,77 s**, 29 das 34 músicas com acordes entram na faixa de
/// 1,5–2,5 s, todas as 34 ficam na faixa fisiológica — e o `fallin` (6/8, barra
/// de 1,88 s) **não regride**.
fn escolher_barra(periodo: f32, cortes: &[f32]) -> Option<f32> {
    let alvo = intervalo_harmonico(cortes)?;
    let mut avaliados: Vec<(f32, f32, f32)> = DIVISORES
        .iter()
        .map(|&divisor| periodo / divisor)
        .filter(|&p| (BARRA_MIN_S..=BARRA_MAX_S).contains(&p))
        .map(|p| (p, (p - alvo).abs() / alvo, alinhamento(p, cortes)))
        .collect();
    if avaliados.is_empty() {
        return None;
    }
    let melhor = avaliados.iter().map(|a| a.1).fold(f32::INFINITY, f32::min);
    avaliados.retain(|a| a.1 <= melhor * EMPATE_ANCORA);
    // Empate vai para o **maior** período: a barra é o maior ciclo que cabe na
    // faixa, e o menor é a sua fração. Sobrevindo empate exato, o melhor alinhado.
    avaliados
        .into_iter()
        .max_by(|a, b| a.0.total_cmp(&b.0).then(a.2.total_cmp(&b.2)))
        .map(|a| a.0)
}

/// Mede a relação entre o bumbo e a caixa.
///
/// `cortes` são as fronteiras de acorde — o relógio da harmonia, que dá a escala
/// da barra. Sem eles a escala cai no contraste do histograma, que é o que
/// errava no corpus.
///
/// Devolve `dobra = 0` sem evidência — o mesmo princípio de abstenção do resto
/// do motor: pouco ataque ou período incoerente não viram palpite.
#[must_use]
pub fn medir(spectro: &Spectrogram, taxa: u32, cortes: &[f32]) -> BumboCaixa {
    let fps = spectro.quadros_por_segundo(taxa).max(1.0);
    let bumbo = novidade_da_banda(spectro, taxa, BUMBO_DE_HZ, BUMBO_ATE_HZ);
    let caixa = novidade_da_banda(spectro, taxa, CAIXA_DE_HZ, CAIXA_ATE_HZ);
    let periodo = periodo_dominante(&bumbo, fps, CICLO_MIN_S, CICLO_MAX_S);
    let ataques_bumbo = ataques(&bumbo, fps);
    let ataques_caixa = ataques(&caixa, fps);
    // A caixa que **coincide com o bumbo** é descartada: o ataque do bumbo é
    // banda-larga e vaza para a banda da caixa, encobertos a caixa real. Sem
    // isto o gabarito fica em 5 de 6; com isto, 6 de 6 (medido 2026-11-24).
    let ataques_caixa: Vec<f32> = ataques_caixa
        .into_iter()
        .filter(|&c| {
            !ataques_bumbo
                .iter()
                .any(|&b| (c - b).abs() <= COINCIDENCIA_S)
        })
        .collect();
    let mut saida = BumboCaixa {
        periodo_bumbo_s: periodo,
        bumbos: ataques_bumbo.len(),
        caixas: ataques_caixa.len(),
        ..BumboCaixa::default()
    };
    if periodo <= 0.0 || ataques_bumbo.is_empty() || ataques_caixa.len() < 4 {
        saida.regra = format!(
            "sem evidência: período {periodo:.2} s, {} bumbo(s), {} caixa(s)",
            ataques_bumbo.len(),
            ataques_caixa.len()
        );
        return saida;
    }
    // O histograma da caixa no ciclo do bumbo, em `FATIAS` fatias centradas.
    let histograma = |periodo: f32| -> Option<[f32; FATIAS]> {
        if periodo <= 0.0 {
            return None;
        }
        let mut contagem = [0usize; FATIAS];
        let mut total = 0usize;
        for &c in &ataques_caixa {
            // A caixa pertence ao ciclo do último bumbo que a precede.
            let Some(&ultimo) = ataques_bumbo.iter().rfind(|&&b| b <= c + 0.02) else {
                continue;
            };
            let fracao = (c - ultimo) / periodo;
            if !(0.0..1.0).contains(&fracao) {
                continue;
            }
            // A fatia é a **mais próxima** do centro, não o intervalo em que a
            // fração cai: com `floor` a metade exata (T/2) cai na fronteira entre
            // duas fatias e a leitura se divide ao meio — medido no sintético,
            // 71% numa fatia e 29% na seguinte, quando a verdade era o ponto.
            let fatia = ((fracao * FATIAS as f32).round() as usize) % FATIAS;
            contagem[fatia] += 1;
            total += 1;
        }
        if total < 4 {
            return None;
        }
        let mut perfil = [0.0f32; FATIAS];
        for (i, fatia) in contagem.iter().enumerate() {
            perfil[i] = *fatia as f32 / total as f32;
        }
        Some(perfil)
    };

    let Some(perfil) = histograma(periodo) else {
        saida.regra = "sem evidência: caixas de menos no ciclo do bumbo".to_owned();
        return saida;
    };

    // **A dobra é do contraste; a escala é da harmonia — e elas são ortogonais.**
    //
    // O ciclo bruto pode estar em submúltiplo da barra, e há dois consertos
    // diferentes para dois erros diferentes:
    //
    // 1. **A dobra** (`×1,5`, `×2`) conserta o bumbo que toca **menos** de uma vez
    //    por compasso — `3/4` em que ele toca dois de três tempos (`binky`: bruto
    //    0,93 s, barra 1,39 s), `6/8` em que ele toca dois de seis. Quem decide é o
    //    **contraste** do histograma da caixa, e é o que acerta o gabarito.
    // 2. **A subdivisão** (`÷2`, `÷3`, `÷4`) conserta o **hipercompasso** — a
    //    bateria discreta faz o período disparar para 2 a 4 barras (medido no
    //    corpus de 40: barra mediana de 5,08 s contra troca de acorde mediana de
    //    1,90 s). Quem decide é a **harmonia**, e o contraste **não serve**: ele
    //    degenera quando o denominador é nulo (`5e8` contra `3,3e8`) e prefere o
    //    ciclo longo.
    //
    // A ordem importa: a dobra vem **primeiro** (é o que o gabarito precisa) e a
    // subdivisão depois, sobre o ciclo já dobrado. Aplicar a harmonia ao ciclo
    // bruto **quebra o `binky`** (o alvo harmônico dele é 4,46 s, três barras
    // adiante, e a escolha caía na metade da barra) — medido em 2026-11-24.
    let media = |perfil: &[f32; FATIAS], idx: [usize; 4], n: usize| -> f32 {
        idx[..n].iter().map(|&i| perfil[i]).sum::<f32>() / n as f32
    };
    let contraste = |perfil: &[f32; FATIAS]| -> (f32, f32) {
        let s68 = (perfil[0] + perfil[3]) / 2.0 / (media(perfil, [1, 2, 4, 5], 4) + 1e-9);
        let s34 =
            (perfil[0] + perfil[2] + perfil[4]) / 3.0 / (media(perfil, [1, 3, 5, 0], 3) + 1e-9);
        (s68, s34)
    };
    let inicial = contraste(&perfil);
    let mut melhor = (periodo, perfil, inicial);
    for fator in [1.5f32, 2.0] {
        if let Some(candidato) = histograma(periodo * fator) {
            let escore = contraste(&candidato);
            if escore.0.max(escore.1) > melhor.2.0.max(melhor.2.1) {
                melhor = (periodo * fator, candidato, escore);
            }
        }
    }
    if let Some(barra) = escolher_barra(melhor.0, cortes) {
        if let Some(perfil_barra) = histograma(barra) {
            melhor = (barra, perfil_barra, contraste(&perfil_barra));
        }
    }
    let (periodo, perfil, (escore_68, escore_34)) = melhor;
    saida.caixa_no_ciclo = perfil;
    saida.periodo_bumbo_s = periodo;
    saida.escore_68 = escore_68;
    saida.escore_34 = escore_34;
    saida.dobra = if escore_68 > escore_34 {
        2
    } else if escore_34 > escore_68 {
        3
    } else {
        0
    };
    saida.regra = format!(
        "período do bumbo {periodo:.2} s · {} bumbo(s) e {} caixa(s) · \
         caixa no ciclo {} · contraste 6/8 {escore_68:.2} contra 3/4 {escore_34:.2} → \
         dobra em {}",
        ataques_bumbo.len(),
        ataques_caixa.len(),
        perfil
            .iter()
            .map(|v| format!("{:.0}%", v * 100.0))
            .collect::<Vec<_>>()
            .join(" "),
        saida.dobra
    );
    saida
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spectrum::Spectrogram;
    use crate::wav::Audio;

    const TAXA: u32 = 22_050;

    /// Bateria sintética: bumbo em `bumbo_em` e caixa em `caixa_em` (1-based,
    /// dentro do compasso de `compasso` batidas), a 120 BPM. O bumbo é um seno
    /// grave curto; a caixa, ruído branco curto — as duas bandas que este módulo
    /// mede, sem nada mais no sinal.
    fn bateria(compasso: u8, bumbo_em: &[u8], caixa_em: &[u8]) -> Vec<f32> {
        let duracao_s = 24.0f32;
        let n = (duracao_s * TAXA as f32) as usize;
        let mut v = vec![0.0f32; n];
        let periodo = 0.5f32; // 120 BPM
        let mut k = 0usize;
        while (k as f32) * periodo < duracao_s {
            let pos = (k % usize::from(compasso)) as u8 + 1;
            let i0 = (k as f32 * periodo * TAXA as f32) as usize;
            if bumbo_em.contains(&pos) {
                for j in 0..(TAXA as usize / 40) {
                    if i0 + j < n {
                        let t = j as f32 / TAXA as f32;
                        v[i0 + j] +=
                            (-25.0 * t).exp() * (2.0 * std::f32::consts::PI * 60.0 * t).sin();
                    }
                }
            }
            if caixa_em.contains(&pos) {
                for j in 0..(TAXA as usize / 60) {
                    if i0 + j < n {
                        let t = j as f32 / TAXA as f32;
                        let x = (((i0 + j) as f32 * 12.9898).sin()) * 43_758.547;
                        v[i0 + j] += 0.8 * (-40.0 * t).exp() * (x - x.floor() - 0.5);
                    }
                }
            }
            k += 1;
        }
        v
    }

    fn medir_amostras(amostras: Vec<f32>) -> BumboCaixa {
        let audio = Audio {
            amostras,
            taxa: TAXA,
            canais: 1,
        };
        let spectro = Spectrogram::new(&audio, 4096, 1024);
        medir(&spectro, TAXA, &[])
    }

    #[test]
    fn a_caixa_na_metade_e_composto_e_nos_tercos_e_simples() {
        // `6/8`: bumbo no 1, caixa no 4 de seis — a caixa cai na **metade** do
        // ciclo do bumbo. É a medida que a `dobra` não fazia: lá os dois acentos
        // eram somados e davam o mesmo envelope (0,42 nos dois, empate exato).
        let composto = medir_amostras(bateria(6, &[1], &[4]));
        assert_eq!(
            composto.dobra, 2,
            "6/8 não dobrou em dois: {}",
            composto.regra
        );
        assert!(
            (composto.periodo_bumbo_s - 3.0).abs() < 0.3,
            "a barra do bumbo devia ser 3,0 s: {}",
            composto.regra
        );

        // `3/4`: bumbo no 1, caixa no 2 e no 3 — a caixa cai em **um terço e
        // dois terços**.
        let simples = medir_amostras(bateria(3, &[1], &[2, 3]));
        assert_eq!(
            simples.dobra, 3,
            "3/4 não dobrou em três: {}",
            simples.regra
        );
        assert!(
            (simples.periodo_bumbo_s - 1.5).abs() < 0.25,
            "a barra do bumbo devia ser 1,5 s: {}",
            simples.regra
        );
    }

    #[test]
    fn a_harmonia_traz_a_barra_de_volta_do_hipercompasso() {
        // A bateria é **discreta**: o bumbo toca uma vez a cada **duas** barras, e
        // o ciclo bruto sai no hipercompasso — 4,0 s para uma barra de 2,0 s. É o
        // modo de falha medido no corpus de 40 (barra mediana de 5,08 s contra
        // 1,90 s de troca de acorde, razão de 2,16 a 5,19).
        let amostras = bateria(8, &[1], &[2, 4, 6, 8]);
        let sem_acordes = medir_amostras(amostras.clone());
        // Sem acordes o ciclo fica **fora da faixa fisiológica** — e a dobra por
        // contraste ainda o **piora** (dobra de 4,0 para 8,0 s), que é exatamente
        // o defeito medido no corpus: o contraste degenera e prefere o ciclo longo.
        assert!(
            sem_acordes.periodo_bumbo_s > BARRA_MAX_S,
            "sem acordes o ciclo devia ficar no hipercompasso (acima de {BARRA_MAX_S} s): {}",
            sem_acordes.regra
        );

        // Com as trocas de acorde **na barra** (2,0 s), a escala volta — a
        // subdivisão é da harmonia, e o contraste não a faria: medido, ele
        // degenera no denominador nulo e prefere o ciclo longo.
        let audio = Audio {
            amostras,
            taxa: TAXA,
            canais: 1,
        };
        let spectro = Spectrogram::new(&audio, 4096, 1024);
        let cortes: Vec<f32> = (1..12).map(|i| i as f32 * 2.0).collect();
        let com_acordes = medir(&spectro, TAXA, &cortes);
        assert!(
            (com_acordes.periodo_bumbo_s - 2.0).abs() < 0.4,
            "com acordes o ciclo devia voltar a 2,0 s: {}",
            com_acordes.regra
        );
    }

    #[test]
    fn silencio_nao_inventa_relacao() {
        let silencio = medir_amostras(vec![0.0; TAXA as usize * 12]);
        assert_eq!(silencio.dobra, 0, "inventou dobra no silêncio");
        assert!(
            silencio.regra.contains("sem evidência"),
            "{}",
            silencio.regra
        );
    }
}
