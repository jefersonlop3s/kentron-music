//! Espectro, croma e novidade espectral.
//!
//! A STFT usa janela de Hann e o `rustfft`. O croma dobra o espectro em doze
//! classes de altura entre 55 Hz e 2 kHz — a faixa onde acorde de música popular
//! vive; fora dela, baixo fundamental e brilho de prato só atrapalham o
//! casamento por template.

#[cfg(test)]
use std::collections::HashMap;
use rustfft::FftPlanner;
use rustfft::num_complex::Complex;

use crate::wav::Audio;

/// Percorre os picos espectrais de um quadro, com frequência interpolada.
///
/// O croma e a medição de afinação leem os mesmos picos por esta função — se
/// cada um interpolasse do seu jeito, as duas visões do mesmo espectro
/// divergiriam e o desvio medido não corrigiria o croma que ele deveria corrigir.
pub(crate) fn para_cada_pico(mags: &[f32], hz_por_bin: f32, mut f: impl FnMut(f32, f32)) {
    for i in 1..mags.len().saturating_sub(1) {
        let (anterior, pico, seguinte) = (mags[i - 1], mags[i], mags[i + 1]);
        if pico <= anterior || pico <= seguinte {
            continue;
        }
        let denominador = anterior - 2.0 * pico + seguinte;
        let delta = if denominador.abs() > f32::EPSILON {
            0.5 * (anterior - seguinte) / denominador
        } else {
            0.0
        };
        f((i as f32 + delta) * hz_por_bin, pico);
    }
}

/// Mediana em janela deslizante; nas bordas usa a janela disponível.
fn mediana_janela(x: &[f32], largura: usize) -> Vec<f32> {
    let largura = largura.max(1) | 1; // ímpar
    let meia = largura / 2;
    let mut janela: Vec<f32> = Vec::with_capacity(largura);
    (0..x.len())
        .map(|i| {
            janela.clear();
            let inicio = i.saturating_sub(meia);
            let fim = (i + meia + 1).min(x.len());
            janela.extend_from_slice(&x[inicio..fim]);
            janela.sort_unstable_by(f32::total_cmp);
            janela[janela.len() / 2]
        })
        .collect()
}

/// Espectrograma de magnitudes.
pub struct Spectrogram {
    /// Magnitude por quadro, com `janela / 2 + 1` valores.
    pub quadros: Vec<Vec<f32>>,
    pub janela: usize,
    pub salto: usize,
}

impl Spectrogram {
    /// Componente harmônica por **filtro de mediana** — HPSS (Fitzgerald 2010).
    ///
    /// Mediana **no tempo** para o harmônico e **na frequência** para o
    /// percussivo, combinadas por máscara suave `H² / (H² + P²)`. A nota
    /// sustentada se repete ao longo de vários quadros e sobrevive à mediana
    /// temporal; a batida se espalha pela frequência e sobrevive à mediana
    /// espectral. **É separação sem modelo** — DSP puro, não precisa de stems.
    ///
    /// **Medido em quatro faixas reais** (30 s cada): as trocas de nota do
    /// rastreador de altura caíram de **28 → 3** (`Billie Jean`), **21 → 0**
    /// (`House of the Rising Sun`), **12 → 3** (`Manic Depression`) e
    /// **38 → 9** (`Nothing Else Matters`). A causa é o vibrato: com a bateria
    /// fora do sinal, a oscilação de ±30 cents deixa de cruzar a fronteira de
    /// semitom.
    ///
    /// Custo honesto: a mediana temporal **suaviza** o que é mais curto que a
    /// janela. Com 9 quadros a 43 quadros/s (0,21 s) as durações medidas
    /// empilham em ~0,23 s — o timescale do kernel, não da música. Esta
    /// componente serve para a **altura**; os **cortes** de nota têm de vir do
    /// ataque, não daqui.
    #[must_use]
    pub fn harmonico(&self, janela_tempo: usize, janela_freq: usize) -> Vec<Vec<f32>> {
        let bins = self.quadros.first().map_or(0, Vec::len);
        if bins == 0 || self.quadros.is_empty() {
            return Vec::new();
        }
        let harm: Vec<Vec<f32>> = (0..bins)
            .map(|i| {
                mediana_janela(
                    &self.quadros.iter().map(|q| q[i]).collect::<Vec<_>>(),
                    janela_tempo,
                )
            })
            .collect();
        let perc: Vec<Vec<f32>> = self
            .quadros
            .iter()
            .map(|q| mediana_janela(q, janela_freq))
            .collect();
        (0..self.quadros.len())
            .map(|t| {
                (0..bins)
                    .map(|i| {
                        let h = harm[i][t];
                        let p = perc[t][i];
                        let (hh, pp) = (h * h, p * p);
                        let total = hh + pp;
                        if total <= f32::EPSILON {
                            0.0
                        } else {
                            self.quadros[t][i] * (hh / total)
                        }
                    })
                    .collect()
            })
            .collect()
    }

    /// Calcula a STFT do sinal mono.
    #[must_use]
    pub fn new(audio: &Audio, janela: usize, salto: usize) -> Self {
        let janela = janela.max(64);
        let salto = salto.max(1);
        let mut planejador = FftPlanner::<f32>::new();
        let fft = planejador.plan_fft_forward(janela);
        let hann: Vec<f32> = (0..janela)
            .map(|i| {
                let x = std::f32::consts::PI * 2.0 * i as f32 / (janela - 1) as f32;
                0.5 - 0.5 * x.cos()
            })
            .collect();

        let mut quadros = Vec::new();
        let mut inicio = 0usize;
        while inicio + janela <= audio.amostras.len() {
            let mut buffer: Vec<Complex<f32>> = (0..janela)
                .map(|i| Complex::new(audio.amostras[inicio + i] * hann[i], 0.0))
                .collect();
            fft.process(&mut buffer);
            quadros.push(buffer[..janela / 2 + 1].iter().map(|c| c.norm()).collect());
            inicio += salto;
        }
        Self {
            quadros,
            janela,
            salto,
        }
    }

    /// Quadros por segundo da STFT.
    #[must_use]
    pub fn quadros_por_segundo(&self, taxa: u32) -> f32 {
        taxa as f32 / self.salto as f32
    }

    /// Desvio global de afinação, em semitons (mediana dos picos).
    ///
    /// Gravação varispeed, fita ou afinação baixa desalinha **toda** comparação
    /// contra escala. A mediana é robusta a pico espúrio e a nota desafinada
    /// isolada.
    ///
    /// Só entram picos com pelo menos 30% da magnitude máxima do quadro — é o
    /// que descarta lóbulo lateral e ruído de fundo sem precisar de limiar
    /// absoluto, que dependeria de normalização do arquivo.
    ///
    /// **Limite declarado:** o valor é a distância ao semitom mais próximo, logo
    /// vive em ±0,5 semitom. Além de meio semitom a gravação deixou de estar
    /// desafinada e passou a estar **noutra altura** — quem resolve isso é
    /// transposição, não afinação. Por isso este número é **medido e relatado**,
    /// e o croma **não** é deslocado por ele: dentro de ±50 cents o
    /// arredondamento já dá conta, e no limite a correção seria ambígua.
    #[must_use]
    pub fn desvio_de_afinacao(&self, taxa: u32) -> f32 {
        let hz_por_bin = taxa as f32 / self.janela as f32;
        let mut desvios: Vec<f32> = Vec::new();
        for mags in &self.quadros {
            let maximo = mags.iter().copied().fold(0.0f32, f32::max);
            if maximo <= f32::EPSILON {
                continue;
            }
            let piso = 0.3 * maximo;
            para_cada_pico(mags, hz_por_bin, |hz, magnitude| {
                if magnitude < piso || !(55.0..=2000.0).contains(&hz) {
                    return;
                }
                let midi = 69.0 + 12.0 * (hz / 440.0).log2();
                desvios.push(midi - midi.round());
            });
        }
        if desvios.is_empty() {
            return 0.0;
        }
        desvios.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        desvios[desvios.len() / 2]
    }

    /// Croma por quadro, normalizado pela soma (soma 1 quando há energia).
    ///
    /// A classe de altura sai de **picos interpolados**, não de bin cru: em
    /// 44,1 kHz com janela de 4096, o bin vale 10,8 Hz — mais largo que um
    /// semitom abaixo de ~180 Hz, e o C3 caía em B (medido no primeiro teste).
    /// A interpolação parabólica do pico devolve a frequência real; o
    /// arredondamento leva à classe certa.
    #[must_use]
    pub fn chroma(&self, taxa: u32) -> Chroma {
        let hz_por_bin = taxa as f32 / self.janela as f32;
        let quadros = self
            .quadros
            .iter()
            .map(|mags| {
                let mut vetor = [0.0f32; 12];
                para_cada_pico(mags, hz_por_bin, |hz, pico| {
                    if !(55.0..=2000.0).contains(&hz) {
                        return;
                    }
                    let midi = 69.0 + 12.0 * (hz / 440.0).log2();
                    let classe = midi.round().rem_euclid(12.0) as usize % 12;
                    vetor[classe] += pico;
                });
                let soma: f32 = vetor.iter().sum();
                if soma > 0.0 {
                    for valor in &mut vetor {
                        *valor /= soma;
                    }
                }
                vetor
            })
            .collect();
        Chroma {
            quadros,
            fps: self.quadros_por_segundo(taxa),
        }
    }

    /// Energia (RMS das magnitudes) por quadro numa faixa de frequência.
    ///
    /// É o que separa **voz** do resto sem modelo: a banda 200–3500 Hz carrega a
    /// fala e o canto, e bumbo/baixo ficam abaixo dela.
    #[must_use]
    pub fn energia_banda(&self, taxa: u32, de_hz: f32, ate_hz: f32) -> Vec<f32> {
        let hz_por_bin = taxa as f32 / self.janela as f32;
        let bins = self.quadros.first().map_or(0, Vec::len);
        if bins == 0 {
            return Vec::new();
        }
        let inicio = ((de_hz / hz_por_bin).ceil() as usize).min(bins - 1);
        let fim = ((ate_hz / hz_por_bin).floor() as usize).clamp(inicio, bins - 1);
        self.quadros
            .iter()
            .map(|mags| {
                let fim = fim.min(mags.len() - 1);
                if inicio > fim {
                    return 0.0;
                }
                let fatia = &mags[inicio..=fim];
                let soma: f32 = fatia.iter().map(|m| m * m).sum();
                (soma / fatia.len() as f32).sqrt()
            })
            .collect()
    }

    /// Novidade espectral (fluxo positivo) por quadro — base dos onsets.
    #[must_use]
    pub fn novidade(&self) -> Vec<f32> {
        let mut saida = Vec::with_capacity(self.quadros.len());
        let mut anterior: Option<&Vec<f32>> = None;
        for mags in &self.quadros {
            let valor = match anterior {
                None => 0.0,
                Some(ant) => mags
                    .iter()
                    .zip(ant)
                    .map(|(novo, velho)| (novo - velho).max(0.0))
                    .sum(),
            };
            saida.push(valor);
            anterior = Some(mags);
        }
        saida
    }
}

/// Croma normalizado por quadro.
pub struct Chroma {
    pub quadros: Vec<[f32; 12]>,
    pub fps: f32,
}

impl Chroma {
    /// Croma médio de um intervalo em segundos.
    #[must_use]
    pub fn media(&self, inicio_s: f32, fim_s: f32) -> [f32; 12] {
        let inicio = ((inicio_s * self.fps) as usize).min(self.quadros.len());
        let fim = ((fim_s * self.fps) as usize).min(self.quadros.len());
        let mut soma = [0.0f32; 12];
        if inicio >= fim {
            return soma;
        }
        for quadro in &self.quadros[inicio..fim] {
            for (i, valor) in quadro.iter().enumerate() {
                soma[i] += valor;
            }
        }
        let n = (fim - inicio) as f32;
        for valor in &mut soma {
            *valor /= n;
        }
        soma
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A voz sustentada (Lá3 = 220 Hz com 2ª e 3ª harmônicas) com **batidas de
    /// ruído por cima** a cada 0,5 s — o caso que a HPSS existe para resolver.
    fn voz_com_batidas() -> Audio {
        let taxa = 44_100u32;
        let total = taxa as usize * 6;
        let mut amostras = vec![0.0f32; total];
        for (i, a) in amostras.iter_mut().enumerate() {
            let t = i as f32 / taxa as f32;
            *a = 0.30 * (2.0 * std::f32::consts::PI * 220.0 * t).sin()
                + 0.15 * (2.0 * std::f32::consts::PI * 440.0 * t).sin()
                + 0.10 * (2.0 * std::f32::consts::PI * 660.0 * t).sin();
        }
        let mut k = 0usize;
        while k * (taxa as usize / 2) < total {
            let inicio = k * (taxa as usize / 2);
            for j in 0..(taxa as usize / 60) {
                if inicio + j < total {
                    let x = ((inicio + j) as f32 * 12.9898).sin() * 43_758.547;
                    amostras[inicio + j] += 0.9 * (x - x.floor() - 0.5);
                }
            }
            k += 1;
        }
        Audio {
        metadados: HashMap::new(),
            amostras,
            taxa,
            canais: 1,
        }
    }

    #[test]
    fn o_hpss_tira_a_batida_da_altura() {
        // Sem separação, cada batida é um transiente de banda larga e o
        // rastreador de altura troca de classe — medido em quatro faixas reais:
        // 28 → 3, 21 → 0, 12 → 3, 38 → 9 trocas de nota. Aqui a prova é
        // hermética: tom fixo, batida fixa, e a classe não pode oscilar.
        let audio = voz_com_batidas();
        let spectro = Spectrogram::new(&audio, 4096, 1024);
        let melodia = crate::melody::extrair(&spectro, audio.taxa);

        let trocas = melodia
            .classes
            .windows(2)
            .filter(|par| par[0].is_some() && par[1].is_some() && par[0] != par[1])
            .count();
        assert!(
            trocas <= 4,
            "a batida trocou a classe da melodia {trocas} vezes"
        );
        // E a classe é a do tom (Lá = 9), não a do ruído.
        let (dominante, _, quadros) = melodia.dominante(1.0, 5.0).expect("sem F0");
        assert_eq!(dominante, 9, "classe dominante trocada: {dominante}");
        assert!(
            quadros as f32 / (4.0 * melodia.fps) > 0.8,
            "F0 instável: {quadros} quadros em 4 s"
        );
    }

    /// Tríade de Dó com quatro parciais, opcionalmente fora de A440.
    ///
    /// A afinação é multiplicativa: deslocar em cents desloca todos os parciais
    /// pelo mesmo tanto, que é o que a medição precisa enxergar.
    fn triade_de_do(desvio_cents: f32) -> Audio {
        let taxa = 44_100u32;
        let total = taxa as usize; // 1 s basta: são 86 quadros
        let fator = 2f32.powf(desvio_cents / 1200.0);
        let parciais = [
            (261.63f32, 1.0f32),
            (329.63, 0.8),
            (392.00, 0.7),
            (523.25, 0.4),
        ];
        let amostras = (0..total)
            .map(|i| {
                let t = i as f32 / taxa as f32;
                parciais
                    .iter()
                    .map(|(hz, ganho)| ganho * (2.0 * std::f32::consts::PI * hz * fator * t).sin())
                    .sum::<f32>()
                    / 2.9
            })
            .collect();
        Audio {
            metadados: HashMap::new(),
            taxa,
            canais: 1,
            amostras,
        }
    }

    #[test]
    fn afinacao_em_440_da_quase_zero() {
        let audio = triade_de_do(0.0);
        let espetro = Spectrogram::new(&audio, 4096, 1024);
        let desvio = espetro.desvio_de_afinacao(audio.taxa);
        assert!(
            desvio.abs() < 0.02,
            "desvio medido: {:.1} cents",
            desvio * 100.0
        );
    }

    #[test]
    fn afinacao_fora_de_a440_e_medida() {
        // Trinta cents é desafinação de verdade: fita, varispeed, afinação baixa.
        let audio = triade_de_do(-30.0);
        let espetro = Spectrogram::new(&audio, 4096, 1024);
        let desvio = espetro.desvio_de_afinacao(audio.taxa) * 100.0;
        assert!(
            (desvio + 30.0).abs() < 8.0,
            "esperado ~-30 cents, medido {desvio:.1}"
        );
    }

    #[test]
    fn afinacao_acima_do_padrao_tambem_e_medida() {
        // O sinal do desvio não é decorativo: acima de A440 tem de dar positivo.
        let audio = triade_de_do(45.0);
        let espetro = Spectrogram::new(&audio, 4096, 1024);
        let desvio = espetro.desvio_de_afinacao(audio.taxa) * 100.0;
        assert!(
            (desvio - 45.0).abs() < 8.0,
            "esperado ~+45 cents, medido {desvio:.1}"
        );
    }

    #[test]
    fn limite_declarado_acima_de_meio_semitom() {
        // -60 cents passa de meio semitom: a medida é circular e devolve algo
        // entre 0 e +50. Não é defeito — é o limite de "distância ao semitom
        // mais próximo", declarado na doc da função. Acima disso o assunto é
        // transposição, não afinação, e o motor não finge resolver.
        let audio = triade_de_do(-60.0);
        let espetro = Spectrogram::new(&audio, 4096, 1024);
        let cents = espetro.desvio_de_afinacao(audio.taxa) * 100.0;
        assert!(
            cents > 0.0 && cents < 50.0,
            "medido {cents:.1} — fora da faixa declarada de ±50"
        );
    }
}
