//! Espectro, croma e novidade espectral.
//!
//! A STFT usa janela de Hann e o `rustfft`. O croma dobra o espectro em doze
//! classes de altura entre 55 Hz e 2 kHz — a faixa onde acorde de música popular
//! vive; fora dela, baixo fundamental e brilho de prato só atrapalham o
//! casamento por template.

use rustfft::FftPlanner;
use rustfft::num_complex::Complex;

use crate::wav::Audio;

/// Espectrograma de magnitudes.
pub struct Spectrogram {
    /// Magnitude por quadro, com `janela / 2 + 1` valores.
    pub quadros: Vec<Vec<f32>>,
    pub janela: usize,
    pub salto: usize,
}

impl Spectrogram {
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
                    let hz = (i as f32 + delta) * hz_por_bin;
                    if !(55.0..=2000.0).contains(&hz) {
                        continue;
                    }
                    let midi = 69.0 + 12.0 * (hz / 440.0).log2();
                    let classe = midi.round().rem_euclid(12.0) as usize % 12;
                    vetor[classe] += pico;
                }
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
