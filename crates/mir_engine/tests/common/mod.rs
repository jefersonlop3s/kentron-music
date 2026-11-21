//! Áudio "cantado" sintetizado para os testes de letra (em `tests/`).
//!
//! Sintetizar é o ponto: a posição esperada de cada linha é conhecida, então o
//! teste mede o motor e não a música. Nenhum material de terceiros entra aqui.
#![allow(dead_code)]

pub const TAXA: u32 = 44_100;

/// Frequência da nota MIDI (69 = A4 = 440 Hz).
pub fn hz(midi: i32) -> f32 {
    440.0 * 2f32.powf((midi as f32 - 69.0) / 12.0)
}

/// "Canta" um bloco: tríade **sustentada** com a voz por cima.
///
/// A tríade soa **junta** — na primeira versão eu a escrevia em sequência (C,
/// depois E, depois G) e o detector lia três acordes onde havia um: a fixture
/// saía com `[F][A5][Dsus4]` em vez de `C`. Acorde é simultâneo.
pub fn bloco(saida: &mut Vec<f32>, triade: [i32; 3], voz: i32, duracao: f32) {
    let n = (duracao * TAXA as f32) as usize;
    // Ruído determinístico do sopro (nada de `rand`: fixture tem de ser idêntica).
    let mut estado = 0x2545_F491u32;
    for i in 0..n {
        let t = i as f32 / TAXA as f32;
        let ataque = (i as f32 / (0.05 * TAXA as f32)).min(1.0);
        let mut som = 0.0f32;
        for midi in triade {
            som += 0.5 * (2.0 * std::f32::consts::PI * hz(midi) * t).sin();
            som += 0.15 * (2.0 * std::f32::consts::PI * hz(midi + 12) * t).sin();
        }
        // A voz: fundamental + sopro de banda larga. O sopro é **ruído**, não uma
        // senoide: uma senoide de formante é uma nota, e o primeiro teste lia um
        // Lá que não existia na música (900 Hz ≈ A5) e virava `Am7` no croma.
        som += (2.0 * std::f32::consts::PI * hz(voz) * t).sin();
        estado = estado.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        som += 0.12 * ((estado >> 9) as f32 / (1 << 23) as f32 * 2.0 - 1.0);
        saida.push(0.45 * ataque * som / 3.0);
    }
}

pub fn silencio(saida: &mut Vec<f32>, duracao: f32) {
    saida.extend(std::iter::repeat_n(0.0, (duracao * TAXA as f32) as usize));
}

/// Quatro blocos cantados, separados por pausa longa: C, G, Am, F.
///
/// Cada bloco é o que uma linha cantada é — acorde sustentado e voz por cima, com
/// silêncio entre as linhas. É o material da fixture dourada.
pub fn cantar_quatro_frases() -> mir_engine::wav::Audio {
    let mut amostras = Vec::new();
    let acordes = [[48, 52, 55], [43, 47, 50], [45, 48, 52], [41, 45, 48]];
    let vozes = [72, 67, 69, 65];
    for (i, (triade, voz)) in acordes.iter().zip(vozes).enumerate() {
        if i > 0 {
            silencio(&mut amostras, 0.8);
        }
        bloco(&mut amostras, *triade, voz, 4.8);
    }
    mir_engine::wav::Audio {
        taxa: TAXA,
        canais: 1,
        amostras,
    }
}

/// Ruído determinístico de banda larga — sem voz, para o teste do negativo.
pub fn ruido(segundos: usize) -> mir_engine::wav::Audio {
    let mut estado = 0x2545_F491u32;
    let amostras: Vec<f32> = (0..TAXA as usize * segundos)
        .map(|_| {
            estado = estado.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (estado >> 8) as f32 / (1 << 24) as f32 * 0.02 - 0.01
        })
        .collect();
    mir_engine::wav::Audio {
        taxa: TAXA,
        canais: 1,
        amostras,
    }
}
