//! Terça pela melodia (ADR-0014, bloco 3).
//!
//! O croma soma tudo o que soa e, num acorde de fundamental + quinta, a própria
//! série harmônica da fundamental põe energia na terça maior (o 5º harmônico é
//! 5/4). Resultado medido: `Am` lido como A maior. A melodia é **outra fonte**,
//! com fundamental forte, e é ela que decide quando o croma empatou.
//!
//! Aqui só se extrai **uma coisa**: a classe de altura da voz por quadro. Não se
//! promete nota exata. Oitava errada não atrapalha — a decisão é por classe, e
//! oitava não muda classe. Isso permite um método simples (soma harmônica sobre
//! os picos do espectro) sem carregar um rastreador de pitch completo.
//!
//! Limite declarado: separar voz de mixagem sem stems é frágil. Por isso o
//! árbitro que consome isto (em `chords`) **só promove `5`** e nunca sobrescreve
//! qualidade já decidida — se a melodia errar, o custo é um `5` que continua `5`.

use crate::spectrum::{self, Spectrogram};

/// Faixa da voz: a mesma banda que a ADR-0011 usa para achar frase vocal.
const VOZ_DE_HZ: f32 = 200.0;
/// Teto da fundamental. Acima disso não é voz cantada, é harmônico.
const VOZ_ATE_HZ: f32 = 1000.0;
/// Até onde os harmônicos são procurados ao pontuar um candidato.
const HARMONICO_ATE_HZ: f32 = 3500.0;
/// Janela da mediana temporal do HPSS, em quadros: ~0,2 s a 43 quadros/s.
const JANELA_TEMPO_HPSS: usize = 9;
/// Janela da mediana espectral do HPSS, em bins.
const JANELA_FREQ_HPSS: usize = 17;
/// Pico abaixo disto (fração do maior do quadro) não é candidato a voz.
const PISO_DO_PICO: f32 = 0.20;

/// Classe de altura da melodia por quadro.
pub struct Melodia {
    /// `None` quando o quadro não deu F0 confiável (silêncio ou mixagem densa).
    pub classes: Vec<Option<usize>>,
    /// Quadros por segundo, o mesmo passo do espectrograma.
    pub fps: f32,
}

impl Melodia {
    /// Quantos quadros do intervalo tiveram F0.
    #[must_use]
    pub fn quadros_com_f0(&self, inicio_s: f32, fim_s: f32) -> usize {
        let (inicio, fim) = self.recorte(inicio_s, fim_s);
        self.classes[inicio..fim]
            .iter()
            .filter(|classe| classe.is_some())
            .count()
    }

    /// Histograma de classes de altura no intervalo.
    #[must_use]
    pub fn contagem(&self, inicio_s: f32, fim_s: f32) -> [usize; 12] {
        let (inicio, fim) = self.recorte(inicio_s, fim_s);
        let mut histograma = [0usize; 12];
        for classe in self.classes[inicio..fim].iter().flatten() {
            histograma[*classe] += 1;
        }
        histograma
    }

    /// Classe dominante no intervalo, com quantas vezes apareceu.
    #[must_use]
    pub fn dominante(&self, inicio_s: f32, fim_s: f32) -> Option<(usize, usize, usize)> {
        let histograma = self.contagem(inicio_s, fim_s);
        let total: usize = histograma.iter().sum();
        if total == 0 {
            return None;
        }
        let (classe, vezes) = histograma
            .iter()
            .copied()
            .enumerate()
            .max_by_key(|(_, vezes)| *vezes)?;
        Some((classe, vezes, total))
    }

    fn recorte(&self, inicio_s: f32, fim_s: f32) -> (usize, usize) {
        if self.fps <= 0.0 {
            return (0, 0);
        }
        let inicio = ((inicio_s * self.fps) as usize).min(self.classes.len());
        let fim = ((fim_s * self.fps) as usize).min(self.classes.len());
        if inicio >= fim {
            (inicio, inicio)
        } else {
            (inicio, fim)
        }
    }
}

/// Extrai a classe de altura da melodia, quadro a quadro.
#[must_use]
pub fn extrair(spectro: &Spectrogram, taxa: u32) -> Melodia {
    let hz_por_bin = taxa as f32 / spectro.janela as f32;
    // A altura vem da componente **harmônica**, não do mix: a bateria fora do
    // sinal é o que impede o vibrato de virar nota. Ver `Spectrogram::harmonico`.
    let harmonico = spectro.harmonico(JANELA_TEMPO_HPSS, JANELA_FREQ_HPSS);
    let classes = harmonico
        .iter()
        .map(|mags| classe_do_quadro(mags, hz_por_bin))
        .collect();
    Melodia {
        classes,
        fps: spectro.quadros_por_segundo(taxa),
    }
}

/// Classe de altura do candidato com maior soma harmônica no quadro.
///
/// Soma harmônica, e não a maior magnitude: a fundamental da voz vem acompanhada
/// de 2f e 3f, e é essa regularidade que a distingue de um pico isolado.
fn classe_do_quadro(mags: &[f32], hz_por_bin: f32) -> Option<usize> {
    let maior_pico = mags.iter().copied().fold(0.0f32, f32::max);
    if maior_pico <= f32::EPSILON {
        return None;
    }
    let piso = PISO_DO_PICO * maior_pico;
    let mut melhor = (0.0f32, 0.0f32); // pontuação, hz
    spectrum::para_cada_pico(mags, hz_por_bin, |hz, magnitude| {
        if !(VOZ_DE_HZ..=VOZ_ATE_HZ).contains(&hz) || magnitude < piso {
            return;
        }
        let mut pontuacao = magnitude;
        for harmonico in [2.0f32, 3.0] {
            let alvo = hz * harmonico;
            if alvo > HARMONICO_ATE_HZ {
                break;
            }
            let bin = (alvo / hz_por_bin).round() as usize;
            if let Some(valor) = mags.get(bin) {
                pontuacao += valor;
            }
        }
        if pontuacao > melhor.0 {
            melhor = (pontuacao, hz);
        }
    });
    if melhor.1 <= 0.0 {
        return None;
    }
    let midi = 69.0 + 12.0 * (melhor.1 / 440.0).log2();
    Some((midi.round().rem_euclid(12.0) as usize) % 12)
}
