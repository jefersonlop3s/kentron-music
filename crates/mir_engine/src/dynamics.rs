//! Dinâmica e pressão sonora por EBU R128.
//!
//! Implementa a pesagem K (dois biquads: prateleira de agudos + passa-alta RLB),
//! o loudness em blocos de 400 ms com 75% de sobreposição, o **portão** absoluto
//! (−70 LUFS) e relativo (−10 LU), a faixa de volume (LRA) por percentis e o
//! pico de amostra.
//!
//! Verificação: o valor integrado é comparado com o filtro `ebur128` do
//! `ffmpeg` no teste de integração — dois medidores independentes no mesmo sinal.
//! O campo de pico é **pico de amostra** (`sample peak`), não pico verdadeiro:
//! pico verdadeiro exige sobreamostragem com filtro polifásico, que este
//! incremento não faz. Nomear diferente seria mentir no relatório.

use serde::Serialize;

const OFFSET: f64 = -0.691;

/// Biquad direto, forma II transposta.
#[derive(Debug, Clone, Copy)]
struct Biquad {
    b: [f64; 3],
    a: [f64; 3],
}

impl Biquad {
    fn aplicar(self, entrada: &[f32]) -> Vec<f32> {
        let mut saida = Vec::with_capacity(entrada.len());
        let (mut z1, mut z2) = (0.0f64, 0.0f64);
        for &x in entrada {
            let x = f64::from(x);
            let y = self.b[0] * x + z1;
            z1 = self.b[1] * x - self.a[1] * y + z2;
            z2 = self.b[2] * x - self.a[2] * y;
            saida.push(y as f32);
        }
        saida
    }
}

/// Prateleira de agudos da pesagem K (parâmetros do ITU-R BS.1770).
fn prateleira(taxa: f64) -> Biquad {
    let (f0, g, q) = (1681.974450955533, 3.999843853973347, 0.7071752369554196);
    let k = (std::f64::consts::PI * f0 / taxa).tan();
    let vh = 10f64.powf(g / 20.0);
    let vb = vh.powf(0.499_666_774_154_541_6);
    let a0 = 1.0 + k / q + k * k;
    Biquad {
        b: [
            (vh + vb * k / q + k * k) / a0,
            2.0 * (k * k - vh) / a0,
            (vh - vb * k / q + k * k) / a0,
        ],
        a: [1.0, 2.0 * (k * k - 1.0) / a0, (1.0 - k / q + k * k) / a0],
    }
}

/// Passa-alta RLB da pesagem K.
fn passa_alta(taxa: f64) -> Biquad {
    let (f0, q) = (38.135_470_876_024_44, 0.500_327_037_323_877_3);
    let k = (std::f64::consts::PI * f0 / taxa).tan();
    let a0 = 1.0 + k / q + k * k;
    Biquad {
        b: [1.0, -2.0, 1.0],
        a: [1.0, 2.0 * (k * k - 1.0) / a0, (1.0 - k / q + k * k) / a0],
    }
}

/// Loudness de cada bloco, em LUFS: `(segundo, LUFS)`.
fn blocos_lufs(pesado: &[f32], taxa: u32, janela_s: f64, passo_s: f64) -> Vec<(f64, f64)> {
    let janela = ((janela_s * f64::from(taxa)) as usize).max(1);
    let passo = ((passo_s * f64::from(taxa)) as usize).max(1);
    let mut saida = Vec::new();
    let mut inicio = 0usize;
    while inicio + janela <= pesado.len() {
        let soma: f64 = pesado[inicio..inicio + janela]
            .iter()
            .map(|&x| f64::from(x) * f64::from(x))
            .sum();
        let media = soma / janela as f64;
        if media > 0.0 {
            // Um canal (mono), peso G = 1.0.
            saida.push((
                inicio as f64 / f64::from(taxa),
                OFFSET + 10.0 * media.log10(),
            ));
        }
        inicio += passo;
    }
    saida
}

/// Integrado com portão absoluto e relativo (EBU R128).
fn integrado(blocos: &[(f64, f64)]) -> f64 {
    let acima_absoluto: Vec<f64> = blocos
        .iter()
        .map(|&(_, l)| l)
        .filter(|&l| l > -70.0)
        .collect();
    if acima_absoluto.is_empty() {
        return f64::NEG_INFINITY;
    }
    let media_abs: f64 = acima_absoluto.iter().sum::<f64>() / acima_absoluto.len() as f64;
    let relativo = media_abs - 10.0;
    let acima_relativo: Vec<f64> = acima_absoluto
        .into_iter()
        .filter(|&l| l > relativo)
        .collect();
    if acima_relativo.is_empty() {
        return f64::NEG_INFINITY;
    }
    acima_relativo.iter().sum::<f64>() / acima_relativo.len() as f64
}

/// Faixa de volume (LRA) a partir dos blocos de curto prazo.
fn lra(curto: &[(f64, f64)]) -> f64 {
    let mut valores: Vec<f64> = curto
        .iter()
        .map(|&(_, l)| l)
        .filter(|&l| l > -70.0)
        .collect();
    if valores.len() < 2 {
        return 0.0;
    }
    valores.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let media: f64 = valores.iter().sum::<f64>() / valores.len() as f64;
    let gate = media - 20.0;
    let filtrados: Vec<f64> = valores.iter().copied().filter(|&v| v > gate).collect();
    if filtrados.len() < 2 {
        return 0.0;
    }
    let baixo = filtrados[(filtrados.len() as f64 * 0.10) as usize];
    let alto = filtrados[((filtrados.len() as f64 * 0.95) as usize).min(filtrados.len() - 1)];
    (alto - baixo).max(0.0)
}

/// Resultado da análise dinâmica.
#[derive(Debug, Clone, Serialize)]
pub struct Dynamics {
    /// Loudness integrado, em LUFS.
    pub lufs_integrado: f64,
    /// Faixa de volume, em LU.
    pub lra_lu: f64,
    /// Pico de amostra, em dBFS (não é pico verdadeiro — ver nota do módulo).
    pub pico_amostra_dbfs: f64,
    /// Curva de curto prazo (3 s), amostrada a cada segundo: `(segundo, LUFS)`.
    pub curva_curto_prazo: Vec<(f64, f64)>,
    /// Instante do trecho mais alto, em segundos.
    pub mais_alto_s: f64,
    /// Instante do trecho mais baixo (acima do portão), em segundos.
    pub mais_baixo_s: f64,
}

/// Analisa a dinâmica do sinal mono.
#[must_use]
pub fn analyze(amostras: &[f32], taxa: u32) -> Dynamics {
    let taxa_f = f64::from(taxa.max(1));
    let pesado = passa_alta(taxa_f).aplicar(&prateleira(taxa_f).aplicar(amostras));
    let curtos = blocos_lufs(&pesado, taxa, 3.0, 1.0);
    let integrado_lufs = integrado(&blocos_lufs(&pesado, taxa, 0.4, 0.1));

    let mut mais_alto = (0.0f64, f64::NEG_INFINITY);
    let mut mais_baixo = (0.0f64, f64::INFINITY);
    for &(t, l) in &curtos {
        if l > mais_alto.1 {
            mais_alto = (t, l);
        }
        if l < mais_baixo.1 {
            mais_baixo = (t, l);
        }
    }

    let pico = amostras
        .iter()
        .fold(0.0f64, |m, &x| m.max(f64::from(x).abs()));
    Dynamics {
        lufs_integrado: integrado_lufs,
        lra_lu: lra(&curtos),
        pico_amostra_dbfs: if pico > 0.0 {
            20.0 * pico.log10()
        } else {
            f64::NEG_INFINITY
        },
        curva_curto_prazo: curtos,
        mais_alto_s: mais_alto.0,
        mais_baixo_s: if mais_baixo.1.is_finite() {
            mais_baixo.0
        } else {
            0.0
        },
    }
}
