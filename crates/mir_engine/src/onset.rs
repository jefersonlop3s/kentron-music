//! Detector de onset compartilhado entre percussão e peças.
//!
//! Consolida o detector que antes estava duplicado em `percussao::ataques()`
//! e `pecas::ataques()`. A melhoria principal é a **interpolação parabólica**
//! para localização sub-quadro dos picos — a mesma técnica de
//! `crate::percussao::periodo_dominante`. Sem ela, o ataque só é localizado
//! com a resolução do quadro salto (`1024 / 44100 ≈ 23 ms`), e o histograma
//! de caixa no ciclo do bumbo fica espalhado — o que mantém o `binky` em
//! aberto (ADR-0017).

use crate::spectrum::Spectrogram;

/// Piso do ataque, em desvios-padrão acima da média do envelope.
const PISO_ATAQUE: f32 = 1.5;
/// Espaçamento mínimo entre ataques. Abaixo disso conta como o mesmo golpe.
const ESPACO_MIN_S: f32 = 0.08;

/// Detecta onsets num envelope de energia ou novidade espectral.
///
/// 1. Calcula o piso adaptativo: `média + PISO_ATAQUE × desvio-padrão`.
/// 2. Acha máximos locais acima do piso.
/// 3. Interpola o instante do pico parabolicamente (sub-quadro).
/// 4. Remove candidatos a menos de `ESPACO_MIN_S` do anterior.
///
/// Devolve pares `(tempo_s, amplitude_do_pico)`.
///
/// Devolve vetor vazio quando a variância é zero — envelope constante não
/// tem ataque, e sem esta guarda cada quadro viraria um falso positivo
/// (medido em 2026-11-24).
#[must_use]
pub fn detectar(envelope: &[f32], fps: f32) -> Vec<(f32, f32)> {
    if envelope.len() < 3 {
        return Vec::new();
    }
    let media: f32 = envelope.iter().sum::<f32>() / envelope.len() as f32;
    let variancia: f32 = envelope
        .iter()
        .map(|v| (v - media) * (v - media))
        .sum::<f32>()
        / envelope.len() as f32;
    if variancia <= f32::EPSILON {
        return Vec::new();
    }
    let piso = media + PISO_ATAQUE * variancia.sqrt();
    let espaco = (ESPACO_MIN_S * fps).round() as usize;
    let mut saida: Vec<(f32, f32)> = Vec::new();
    let mut ultimo: Option<usize> = None;

    for i in 1..envelope.len() - 1 {
        if envelope[i] < piso || envelope[i] < envelope[i - 1] || envelope[i] < envelope[i + 1] {
            continue;
        }
        if let Some(u) = ultimo {
            if i - u < espaco {
                continue;
            }
        }
        ultimo = Some(i);

        // Interpolação parabólica para localização sub-quadro do pico.
        // Sem ela, a resolução é `1/fps` (23 ms a 44100 Hz, salto 1024) —
        // que é o jitter que espalha o histograma da caixa no ciclo do
        // bumbo (ADR-0017, P5).
        let a = if i > 0 { envelope[i - 1] } else { 0.0 };
        let b = envelope[i];
        let c = if i + 1 < envelope.len() {
            envelope[i + 1]
        } else {
            0.0
        };
        let t_bruto = i as f32 / fps;
        let denom = a - 2.0 * b + c;
        let t_fino = if denom.abs() > f32::EPSILON {
            let desloc = 0.5 * (a - c) / denom;
            (i as f32 + desloc.clamp(-0.5, 0.5)) / fps
        } else {
            t_bruto
        };
        saida.push((t_fino, envelope[i]));
    }
    saida
}

/// Cobre uma banda do espectro com a novidade da energia (derivada positiva)
/// e detecta os onsets nela.
///
/// É o atalho mais comum: pega a energia de uma banda, calcula a novidade
/// (diferença positiva entre quadros consecutivos), normaliza pela média da
/// própria banda e detecta os picos.
///
/// Devolve pares `(tempo_s, amplitude_do_pico)`.
#[must_use]
pub fn detectar_banda(
    spectro: &Spectrogram,
    taxa: u32,
    de_hz: f32,
    ate_hz: f32,
) -> Vec<(f32, f32)> {
    let fps = spectro.quadros_por_segundo(taxa).max(1.0);
    let energia = spectro.energia_banda(taxa, de_hz, ate_hz);
    let novidade = fluxo_espectral(&energia);
    detectar(&novidade, fps)
}

/// Novidade espectral: derivada de meia-onda (diferença positiva) normalizada
/// pela média.
///
/// Medido (2026-11-24): sem a normalização, a banda grave (que tem magnitude
/// muito maior) domina o perfil e os ataques das bandas agudas desaparecem.
#[must_use]
pub fn fluxo_espectral(energia: &[f32]) -> Vec<f32> {
    let mut saida = Vec::with_capacity(energia.len());
    let mut anterior = energia.first().copied().unwrap_or(0.0);
    for e in energia {
        saida.push((*e - anterior).max(0.0));
        anterior = *e;
    }
    let media: f32 = saida.iter().sum::<f32>() / (saida.len().max(1) as f32);
    if media > f32::EPSILON {
        for x in &mut saida {
            *x /= media;
        }
    }
    saida
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encontra_picos_simples() {
        // Dois picos bem acima do limiar adaptativo
        let envelope = vec![
            0.1, 0.2, 0.6, 1.5, 1.8, 0.7, 0.3, 0.1, 0.1, 0.1, 0.1, 0.2, 0.8, 2.0, 1.5, 0.6, 0.2,
            0.1,
        ];
        let encontrados = detectar(&envelope, 10.0);
        assert_eq!(encontrados.len(), 2, "deveria achar dois picos");
        assert!((encontrados[0].0 - 0.4).abs() < 0.05, "pico 1 ~0,4 s");
        assert!((encontrados[1].0 - 1.3).abs() < 0.05, "pico 2 ~1,3 s");
    }

    #[test]
    fn silencio_nao_inventa() {
        let encontrados = detectar(&[0.0; 100], 10.0);
        assert!(encontrados.is_empty(), "silêncio não produz ataques");
    }

    #[test]
    fn interpolacao_parabolica_desloca_pico() {
        // Um pico exatamente no índice 4 (0,4 s a 10 fps), vizinhos
        // assimétricos: a parábola deve deslocar o pico para a **esquerda**
        // (vizinho esquerdo 0,9 é mais forte que o direito 0,6).
        let envelope = vec![0.0, 0.1, 0.5, 0.9, 1.0, 0.6, 0.2, 0.0];
        let encontrados = detectar(&envelope, 10.0);
        assert_eq!(encontrados.len(), 1, "um pico só");
        // A parábola deve deslocar o pico para < 0,4 s (índice 4 / 10 Hz)
        assert!(
            encontrados[0].0 < 0.40,
            "interpolação devia deslocar pico para antes de 0,4 s: {:.4}",
            encontrados[0].0
        );
    }

    #[test]
    fn espacamento_minimo_funde_grupos() {
        // Três máximos locais, mas o segundo e o terceiro estão a menos de
        // 0,08 s — só o primeiro e o último devem ser reportados.
        let fps = 100.0;
        let mut envelope = vec![0.1; 50];
        envelope[10] = 1.0; // pico em 0,10 s
        envelope[12] = 0.9; // 0,12 s — colado
        envelope[14] = 0.8; // 0,14 s — colado
        envelope[30] = 2.0; // 0,30 s — longe
        let encontrados = detectar(&envelope, fps);
        assert_eq!(encontrados.len(), 2, "dois grupos de ataque");
    }
}
