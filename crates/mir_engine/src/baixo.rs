//! Baixo por quadro: a nota mais grave que o ouvido separa do resto.
//!
//! O croma dobra o espectro em 12 classes e **descarta o registro** — é o que
//! dá invariância a transposição, a dobramento e a voicing, e é também o que
//! torna `C`, `C/E` e `C/G` indistinguíveis: as três têm as mesmas três classes.
//! Aqui o registro é justamente o ponto, então a banda é a do baixo/contrabaixo
//! (40–250 Hz) e a decisão **não** usa o croma dobrado.
//!
//! A evidência vem de **soma harmônica**: o baixo costuma ter fundamental fraca
//! (a corda e a caixa do instrumento comem o fundamental, e o alto-falante de
//! celular não reproduz 41 Hz), e quem o revela são o 2º e o 3º harmônicos.
//! Somar os harmônicos de cada candidato identifica a fundamental mesmo quando
//! ela não aparece no espectro.
//!
//! **Viés declarado, não acidental:** entre candidatos com evidência parecida,
//! vence o **mais grave**. É o que o ouvido faz ao separar baixo de acorde: o
//! dó de um piano e o dó de um baixo têm a mesma classe e papéis diferentes, e
//! o que os separa é o registro.
//!
//! **Limite conhecido e medido:** com janela de 4096 a 44,1 kHz o bin vale
//! 10,77 Hz, e um semitom só fica mais largo que um bin acima de ~181 Hz. Na
//! banda do baixo o detector depende de **picos interpolados**, não de
//! resolução de bin — a mesma razão pela qual o croma usa interpolação
//! parabólica (ver `spectrum::chroma`).

use crate::spectrum::{Spectrogram, para_cada_pico};

/// Banda onde se procura a fundamental do baixo.
const BANDA_HZ: (f32, f32) = (40.0, 250.0);
/// Harmônicos somados na evidência de cada candidato.
const HARMONICOS: u32 = 4;
/// Candidatos a 85% ou mais do melhor vencem se forem mais graves.
const MARGEM_GRAVE: f32 = 0.85;
/// Fração mínima de quadros que precisam concordar com a nota do baixo para
/// que ela seja usada na cifra.
///
/// **Medido no repertório (2026-11-22):** abaixo de 0,40 o quadro discorda de
/// si mesmo na maior parte do tempo e a barra vira ruído — a nota sai da
/// maioria, mas por pouco. Acima disso a inversão é estável.
pub const CONFIANCA_MINIMA: f32 = 0.40;
/// Tolerância, em semitons, para casar um pico ao harmônico esperado.
const TOLERANCIA_SEMITONS: f32 = 0.5;

/// Nomes de tônica na mesma convenção do resto do motor (bemóis nas pretas).
const NOMES: [&str; 12] = [
    "C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B",
];

/// Baixo estimado por quadro, em nota MIDI.
pub struct Baixo {
    /// `None` quando o quadro não tem evidência suficiente na banda.
    pub quadros: Vec<Option<u8>>,
    /// Fração da energia da banda que está sobre os harmônicos da nota escolhida.
    ///
    /// **Perto de zero = o que há ali é percussivo, não é nota.** É o que separa
    /// o baixo do **bumbo**, que vive na mesma faixa e não tem altura. Sem isso,
    /// "tem energia no grave" não distingue "o baixo está tocando" de "o bumbo
    /// bateu" — e a abstenção do detector fica sem julgamento.
    pub tonalidade: Vec<f32>,
    pub fps: f32,
}

/// Nome da nota com oitava, como o músico lê (`E2`, `Bb1`).
#[must_use]
pub fn nome(midi: u8) -> String {
    let classe = usize::from(midi) % 12;
    let oitava = i32::from(midi) / 12 - 1;
    format!("{}{}", NOMES[classe], oitava)
}

/// Estima a fundamental mais grave por quadro.
///
/// Devolve `quadros` vazios quando o áudio é curto demais para uma janela.
#[must_use]
pub fn extrair(spectro: &Spectrogram, taxa: u32) -> Baixo {
    let hz_por_bin = taxa as f32 / spectro.janela as f32;
    let (de_hz, ate_hz) = BANDA_HZ;
    // Candidatos: toda nota MIDI cuja fundamental cai na banda.
    let primeira = (69.0 + 12.0 * (de_hz / 440.0).log2()).ceil() as u8;
    let ultima = (69.0 + 12.0 * (ate_hz / 440.0).log2()).floor() as u8;
    let mut quadros = Vec::with_capacity(spectro.quadros.len());
    let mut tonalidades: Vec<f32> = Vec::with_capacity(spectro.quadros.len());
    for mags in &spectro.quadros {
        let mut picos: Vec<(f32, f32)> = Vec::new();
        para_cada_pico(mags, hz_por_bin, |hz, magnitude| {
            // Os harmônicos passam da banda: um baixo em 41 Hz tem o 4º em
            // 165 Hz, mas um em 220 Hz tem o 4º em 880 Hz.
            if (de_hz..=ate_hz * HARMONICOS as f32).contains(&hz) {
                picos.push((69.0 + 12.0 * (hz / 440.0).log2(), magnitude));
            }
        });
        if picos.is_empty() {
            quadros.push(None);
            tonalidades.push(0.0);
            continue;
        }
        let mut notas: Vec<(u8, f32)> = Vec::new();
        for midi in primeira..=ultima {
            let mut evidencia = 0.0f32;
            for h in 1..=HARMONICOS {
                let alvo = f32::from(midi) + 12.0 * (h as f32).log2();
                let peso = 1.0 / h as f32;
                for &(midi_pico, magnitude) in &picos {
                    if (midi_pico - alvo).abs() <= TOLERANCIA_SEMITONS {
                        evidencia += peso * magnitude;
                    }
                }
            }
            if evidencia > 0.0 {
                notas.push((midi, evidencia));
            }
        }
        if notas.is_empty() {
            quadros.push(None);
            tonalidades.push(0.0);
            continue;
        }
        let melhor = notas.iter().map(|&(_, e)| e).fold(0.0f32, f32::max);
        // Viés para o grave: entre os que empatam na margem, o mais grave vence.
        let escolhido = notas
            .iter()
            .filter(|&&(_, e)| e >= MARGEM_GRAVE * melhor)
            .map(|&(m, _)| m)
            .min();
        quadros.push(escolhido);
        tonalidades.push(escolhido.map_or(0.0, |nota| tonalidade(mags, &picos, hz_por_bin, nota)));
    }
    Baixo {
        quadros,
        tonalidade: tonalidades,
        fps: spectro.quadros_por_segundo(taxa),
    }
}

/// Fração da energia da banda (40–250 Hz) que cai sobre os harmônicos da nota.
///
/// Um baixo emite série harmônica: quase toda a energia da banda fica sobre os
/// harmônicos dela. Um **bumbo** tem energia na mesma faixa e **não tem altura**
/// — a energia se espalha e sobra pouco sobre qualquer série. É o que distingue
/// "o baixo está tocando" de "o bumbo bateu", e sem isso a abstenção do detector
/// fica sem julgamento: energia no grave não é o mesmo que nota no grave.
fn tonalidade(mags: &[f32], picos: &[(f32, f32)], hz_por_bin: f32, nota: u8) -> f32 {
    let (de_hz, ate_hz) = BANDA_HZ;
    if mags.is_empty() {
        return 0.0;
    }
    let lo = (de_hz / hz_por_bin).ceil() as usize;
    let hi = ((ate_hz / hz_por_bin).floor() as usize).min(mags.len() - 1);
    if lo >= hi {
        return 0.0;
    }
    let total: f32 = mags[lo..=hi].iter().sum();
    if total <= f32::EPSILON {
        return 0.0;
    }
    let midi_lo = 69.0 + 12.0 * (de_hz / 440.0).log2();
    let midi_hi = 69.0 + 12.0 * (ate_hz / 440.0).log2();
    let mut harmonica = 0.0f32;
    for &(midi_pico, magnitude) in picos {
        if !(midi_lo..=midi_hi).contains(&midi_pico) {
            continue;
        }
        for h in 1..=HARMONICOS {
            let alvo = f32::from(nota) + 12.0 * (h as f32).log2();
            if (midi_pico - alvo).abs() <= TOLERANCIA_SEMITONS {
                harmonica += magnitude;
                break;
            }
        }
    }
    (harmonica / total).min(1.0)
}

impl Baixo {
    /// Tonalidade média num intervalo, de 0 a 1.
    #[must_use]
    pub fn tonalidade_do_trecho(&self, inicio_s: f32, fim_s: f32) -> f32 {
        if self.tonalidade.is_empty() || self.fps <= 0.0 {
            return 0.0;
        }
        let inicio = ((inicio_s * self.fps) as usize).min(self.tonalidade.len());
        let fim = ((fim_s * self.fps) as usize).min(self.tonalidade.len());
        if inicio >= fim {
            return 0.0;
        }
        self.tonalidade[inicio..fim].iter().sum::<f32>() / (fim - inicio) as f32
    }

    /// Nota mais frequente num intervalo, e a fração de quadros que a sustenta.
    ///
    /// `None` quando não há um único quadro com evidência no intervalo.
    #[must_use]
    pub fn do_trecho(&self, inicio_s: f32, fim_s: f32) -> Option<(u8, f32)> {
        if self.quadros.is_empty() || self.fps <= 0.0 {
            return None;
        }
        let inicio = ((inicio_s * self.fps) as usize).min(self.quadros.len());
        let fim = ((fim_s * self.fps) as usize).min(self.quadros.len());
        if inicio >= fim {
            return None;
        }
        // Conta por **classe de altura**, não por nota MIDI: o baixo alternando
        // E1 e E2 é a mesma nota para a harmonia, e contar MIDI exato dividia a
        // confiança pela metade — erro medido no repertório em 2026-11-22. A
        // oitava continua sendo devolvida (é registro), mas não decide a nota.
        let mut classes: Vec<(u8, Vec<(u8, usize)>)> = Vec::new();
        let mut com_evidencia = 0usize;
        for nota in self.quadros[inicio..fim].iter().flatten() {
            com_evidencia += 1;
            let classe = nota % 12;
            let i = match classes.iter().position(|(c, _)| *c == classe) {
                Some(i) => i,
                None => {
                    classes.push((classe, Vec::new()));
                    classes.len() - 1
                }
            };
            let midis = &mut classes[i].1;
            match midis.iter_mut().find(|(m, _)| m == nota) {
                Some((_, n)) => *n += 1,
                None => midis.push((*nota, 1)),
            }
        }
        if com_evidencia == 0 {
            return None;
        }
        classes.sort_by_key(|(_, midis)| {
            std::cmp::Reverse(midis.iter().map(|(_, n)| n).sum::<usize>())
        });
        let (_, midis) = &mut classes[0];
        let total: usize = midis.iter().map(|(_, n)| n).sum();
        midis.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
        Some((midis[0].0, total as f32 / com_evidencia as f32))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use crate::wav::Audio;
    use std::f32::consts::PI;

    /// Soma de harmônicos com amplitudes 1/h, como um instrumento de corda.
    fn tom(freq: f32, duracao_s: f32, harmonicos: u32) -> Vec<f32> {
        let taxa = 44_100u32;
        let n = (duracao_s * taxa as f32) as usize;
        (0..n)
            .map(|i| {
                let t = i as f32 / taxa as f32;
                (1..=harmonicos)
                    .map(|h| (1.0 / h as f32) * (2.0 * PI * freq * h as f32 * t).sin())
                    .sum::<f32>()
                    / harmonicos as f32
            })
            .collect()
    }

    fn analisar(amostras: Vec<f32>) -> Baixo {
        let audio = Audio {
            metadados: HashMap::new(),
            amostras,
            taxa: 44_100,
            canais: 1,
        };
        let spectro = Spectrogram::new(&audio, 4096, 1024);
        extrair(&spectro, audio.taxa)
    }

    #[test]
    fn acha_o_baixo_em_mi1() {
        // E1 = 41,20 Hz, a corda mais grave do baixo.
        let baixo = analisar(tom(41.20, 2.0, 6));
        let (midi, confianca) = baixo.do_trecho(0.4, 1.6).expect("sem baixo");
        assert_eq!(midi, 28, "esperava E1 (28), veio {}", nome(midi));
        assert!(confianca > 0.5, "confiança baixa: {confianca}");
    }

    #[test]
    fn acha_o_baixo_em_la1() {
        let baixo = analisar(tom(55.00, 2.0, 6));
        let (midi, _) = baixo.do_trecho(0.4, 1.6).expect("sem baixo");
        assert_eq!(midi, 33, "esperava A1 (33), veio {}", nome(midi));
    }

    #[test]
    fn fundamental_fraca_nao_engana() {
        // 2º e 3º harmônicos fortes, fundamental quase ausente: o que a caixa do
        // instrumento e o alto-falante pequeno fazem com o grave. A soma
        // harmônica resolve sem o fundamental.
        let taxa = 44_100u32;
        let n = (2.0 * taxa as f32) as usize;
        let f = 41.20f32;
        let amostras: Vec<f32> = (0..n)
            .map(|i| {
                let t = i as f32 / taxa as f32;
                0.05 * (2.0 * PI * f * t).sin()
                    + 0.5 * (2.0 * PI * 2.0 * f * t).sin()
                    + 0.4 * (2.0 * PI * 3.0 * f * t).sin()
            })
            .collect();
        let baixo = analisar(amostras);
        let (midi, _) = baixo.do_trecho(0.4, 1.6).expect("sem baixo");
        assert_eq!(midi, 28, "esperava E1 (28), veio {}", nome(midi));
    }

    #[test]
    fn distingue_as_tres_posicoes_do_mesmo_acorde() {
        // A tríade de Dó é a mesma nas três; o que muda é o baixo. O croma não
        // separa (determinístico: mesmas 3 classes), o registro separa.
        // C2 = 36, E2 = 40, G2 = 43 — fundamental, 1ª e 2ª inversão.
        for &(baixo_hz, esperado) in &[(65.41f32, 36u8), (82.41, 40), (98.00, 43)] {
            // Baixo e tríade no mesmo nível: com o baixo abafado o detector
            // escolhe a nota grave mais forte do acorde, que é o que o ouvido
            // também faz — mas isso não é o caso de teste.
            let mut amostras = tom(baixo_hz, 2.0, 8);
            let mut triade = vec![0.0f32; amostras.len()];
            for f in [130.81f32, 164.81, 196.00] {
                for (soma, nota) in triade.iter_mut().zip(tom(f, 2.0, 6)) {
                    *soma += nota / 3.0;
                }
            }
            for (a, b) in amostras.iter_mut().zip(triade) {
                *a += b;
            }
            let baixo = analisar(amostras);
            let (midi, _) = baixo.do_trecho(0.4, 1.6).expect("sem baixo");
            assert_eq!(
                midi,
                esperado,
                "esperava {} como baixo, veio {}",
                nome(esperado),
                nome(midi)
            );
        }
    }

    #[test]
    fn alternar_oitava_nao_derruba_a_confianca() {
        // E1 (41,20) e E2 (82,41) são a mesma nota para a harmonia. Contar nota
        // MIDI exata dividia a confiança pela metade; contar classe de altura
        // não. A oitava devolvida é a mais frequente.
        let mut amostras = tom(41.20, 2.0, 6);
        amostras.extend(tom(82.41, 2.0, 6));
        let baixo = analisar(amostras);
        let (midi, confianca) = baixo.do_trecho(0.4, 3.6).expect("sem baixo");
        assert_eq!(midi % 12, 4, "esperava a classe de Mi, veio {}", nome(midi));
        assert!(
            confianca > 0.85,
            "alternar oitavas não pode derrubar a confiança: {confianca}"
        );
    }

    #[test]
    fn tonalidade_separa_nota_de_ruido() {
        // Série harmônica: quase toda a energia da banda cai sobre os harmônicos.
        let com_nota = analisar(tom(82.41, 2.0, 8));
        let t_nota = com_nota.tonalidade_do_trecho(0.4, 1.6);
        // Ruído de banda larga: energia espalhada, sobra pouco sobre qualquer série.
        // É o que um bumbo faz na mesma faixa — energia sem altura.
        let ruido: Vec<f32> = (0..(44_100 * 2))
            .map(|i| {
                let x = (i as f32 * 12.9898).sin() * 43_758.547;
                x - x.floor() - 0.5
            })
            .collect();
        let t_ruido = analisar(ruido).tonalidade_do_trecho(0.4, 1.6);
        assert!(t_nota > 0.40, "nota tonal devia passar de 0,40: {t_nota}");
        assert!(
            t_ruido < 0.30,
            "ruído não pode parecer nota, veio {t_ruido}"
        );
        assert!(
            t_nota > t_ruido * 1.5,
            "separação pequena: nota {t_nota} vs ruído {t_ruido}"
        );
    }

    #[test]
    fn silencio_nao_inventa_baixo() {
        let baixo = analisar(vec![0.0; 44_100 * 2]);
        assert!(baixo.do_trecho(0.4, 1.6).is_none());
    }

    #[test]
    fn nome_escreve_oitava() {
        assert_eq!(nome(28), "E1");
        assert_eq!(nome(33), "A1");
        assert_eq!(nome(36), "C2");
        assert_eq!(nome(40), "E2");
    }
}
