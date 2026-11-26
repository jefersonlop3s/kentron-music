//! Áudio "cantado" sintetizado para os testes de letra (em `tests/`).
//!
//! Sintetizar é o ponto: a posição esperada de cada linha é conhecida, então o
//! teste mede o motor e não a música. Nenhum material de terceiros entra aqui.
#![allow(dead_code)]

use std::collections::HashMap;

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

/// Power chord (fundamental + quinta, **sem terça**) com a voz por cima.
///
/// É o material que prova o árbitro da terça (ADR-0014, bloco 3): o acorde não
/// tem terça nenhuma, então se o motor disser `C` ou `Cm` foi a **melodia** que
/// decidiu. Sem esta fixture o teste passaria sem provar nada — a fixture
/// dourada canta a fundamental de cada acorde e nunca traz a terça.
///
/// A voz é uma senoide com segundo e terceiro harmônicos fracos: sem eles, a
/// soma harmônica não teria como distinguir a voz de um pico qualquer.
pub fn power_chord_com_voz(terca: i32) -> mir_engine::wav::Audio {
    power_chord_com_voz_em(terca, GANHO_DA_VOZ)
}

/// Ganho calibrado para a voz ficar **abaixo** do limiar do croma e **acima** do
/// piso da melodia. Medido: acima disto o croma decide sozinho e o teste do
/// árbitro passa sem provar nada; abaixo, a melodia não acha a voz.
pub const GANHO_DA_VOZ: f32 = 0.12;

/// Mesma fixture com o ganho da voz aberto — usada para calibrar e para medir o
/// limite do método.
pub fn power_chord_com_voz_em(terca: i32, ganho: f32) -> mir_engine::wav::Audio {
    let mut amostras = Vec::new();
    // Dó3 (48) + Sol3 (55) — tríade sem a terça. As duas ficam **abaixo** de
    // 200 Hz de propósito: a banda da melodia começa ali, e assim a voz é o
    // único candidato dentro dela.
    for _ in 0..2 {
        bloco_power(&mut amostras, [48, 55], terca, ganho, 2.0);
    }
    mir_engine::wav::Audio {
        metadados: HashMap::new(),
        taxa: TAXA,
        canais: 1,
        amostras,
    }
}

fn bloco_power(saida: &mut Vec<f32>, notas: [i32; 2], voz: i32, ganho: f32, duracao: f32) {
    let n = (duracao * TAXA as f32) as usize;
    let mut estado = 0x2545_F491u32;
    for i in 0..n {
        let t = i as f32 / TAXA as f32;
        let ataque = (i as f32 / (0.05 * TAXA as f32)).min(1.0);
        let mut som = 0.0f32;
        for midi in notas {
            som += 0.50 * (2.0 * std::f32::consts::PI * hz(midi) * t).sin();
        }
        // Voz: fundamental mais os dois primeiros harmônicos, todos com o ganho
        // de calibração — é a regularidade harmônica que a soma harmônica lê.
        let f = hz(voz);
        som += ganho * (2.0 * std::f32::consts::PI * f * t).sin();
        som += ganho * 0.30 * (2.0 * std::f32::consts::PI * 2.0 * f * t).sin();
        som += ganho * 0.15 * (2.0 * std::f32::consts::PI * 3.0 * f * t).sin();
        estado = estado.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        som += 0.05 * ((estado >> 9) as f32 / (1 << 23) as f32 * 2.0 - 1.0);
        saida.push(0.45 * ataque * som / 1.2);
    }
}

/// Duração da introdução instrumental de [`intro_quieta_e_canto`].
pub const INTRO_S: f32 = 3.2;

/// Introdução instrumental **quieta** seguida de canto forte.
///
/// Regressão de 2026-11-25, medida com gabarito humano numa música real: o
/// detector de frase vocal abria frase na introdução — energia de banda
/// sustentada, sem canto nenhum — porque o piso era 2% do pico, **abaixo** do
/// nível da própria introdução. As primeiras linhas da letra caíam ~12 s
/// adiantadas. O piso passou a ser a **mediana** da faixa vocal na música
/// inteira; a proporção aqui imita a música real (introdução curta contra muito
/// canto), porque com introdução dominante a mediana cai dentro dela e o teste
/// mediria uma fixture, não o mecanismo.
pub fn intro_quieta_e_canto() -> mir_engine::wav::Audio {
    let mut amostras: Vec<f32> = Vec::new();
    for _ in 0..2 {
        senoides(&mut amostras, 1.6, 0.03); // introdução: 1/25 da energia do canto
    }
    silencio(&mut amostras, 0.8);
    for _ in 0..3 {
        senoides(&mut amostras, 4.8, 0.15); // canto
    }
    mir_engine::wav::Audio {
        metadados: HashMap::new(),
        taxa: TAXA,
        canais: 1,
        amostras,
    }
}

/// Introdução instrumental quieta + estrutura cantada **com repetição**.
///
/// Refrão e verso cantados **duas vezes cada** (A B A B) — o caso do GAP das
/// repetições: a cifra/letra de qualquer fonte traz as linhas **distintas** e a
/// gravação repete. Com duas linhas distintas para quatro ocorrências, metade do
/// canto fica sem letra e não existe linha nenhuma onde o refrão volta. Devolve
/// o áudio e os inícios **conhecidos** de cada ocorrência (4 · 6,8 · 9,6 · 12,4 s).
pub fn intro_e_canto_repetido() -> (mir_engine::wav::Audio, Vec<f32>) {
    let mut amostras: Vec<f32> = Vec::new();
    for _ in 0..2 {
        senoides(&mut amostras, INTRO_S / 2.0, 0.03); // introdução: 1/25 da energia do canto
    }
    let mut inicios = Vec::new();
    for _ in 0..4 {
        silencio(&mut amostras, 0.8);
        inicios.push(amostras.len() as f32 / TAXA as f32);
        senoides(&mut amostras, LINHA_S, 0.15);
    }
    (
        mir_engine::wav::Audio {
            metadados: HashMap::new(),
            taxa: TAXA,
            canais: 1,
            amostras,
        },
        inicios,
    )
}

/// Senoides PURAS dentro da banda da voz (300/600/900 Hz).
///
/// A intensidade da banda passa a ser o quadrado da amplitude — sem o ruído de
/// banda larga que `bloco_power` acrescenta com ganho fixo, e que fazia a
/// "introdução quieta" ter a mesma energia da banda que o canto.
fn senoides(saida: &mut Vec<f32>, duracao: f32, amp: f32) {
    let taxa_f = TAXA as f32;
    let n = (duracao * taxa_f) as usize;
    for i in 0..n {
        let t = i as f32 / taxa_f;
        let ataque = (i as f32 / (0.05 * taxa_f)).min(1.0);
        let mut s = 0.0f32;
        for (k, hz) in [300.0f32, 600.0, 900.0].iter().enumerate() {
            s += (2.0 * std::f32::consts::PI * hz * t).sin() / (k as f32 + 1.0);
        }
        saida.push(amp * ataque * s / 1.8);
    }
}

/// Duração de uma linha cantada nas fixtures de alinhamento.
pub const LINHA_S: f32 = 2.0;

/// Linhas cantadas **sem pausa** entre elas — o caso que o detector lê como
/// frase única, e onde o alinhamento proporcional mais erra.
///
/// Devolve o áudio e os inícios **conhecidos** de cada linha.
pub fn canto_sem_pausa(n: usize) -> (mir_engine::wav::Audio, Vec<f32>) {
    let mut amostras: Vec<f32> = Vec::new();
    let mut inicios = Vec::new();
    for _ in 0..n {
        inicios.push(amostras.len() as f32 / TAXA as f32);
        senoides(&mut amostras, LINHA_S, 0.15);
        senoides(&mut amostras, 0.10, 0.0); // respiração mínima
    }
    (
        mir_engine::wav::Audio {
            metadados: HashMap::new(),
            taxa: TAXA,
            canais: 1,
            amostras,
        },
        inicios,
    )
}

/// Estrutura com **repetição**: A B A B, cada bloco com pausa antes.
///
/// Devolve o áudio e os inícios conhecidos.
pub fn canto_com_repeticao() -> (mir_engine::wav::Audio, Vec<f32>) {
    let mut amostras: Vec<f32> = Vec::new();
    let mut inicios = Vec::new();
    for _ in 0..2 {
        for amp in [0.15f32, 0.15] {
            silencio(&mut amostras, 0.8);
            inicios.push(amostras.len() as f32 / TAXA as f32);
            senoides(&mut amostras, LINHA_S, amp);
        }
    }
    (
        mir_engine::wav::Audio {
            metadados: HashMap::new(),
            taxa: TAXA,
            canais: 1,
            amostras,
        },
        inicios,
    )
}

/// Canto **contínuo com linhas de durações muito desiguais**.
///
/// É o caso que separa: as linhas têm o **mesmo texto** (mesmo número de
/// sílabas), mas durações de 1 s e 5 s alternadas. O modelo proporcional — que
/// reparte o tempo pelo número de sílabas — **tem** de errar aqui, e é isso que
/// um benchmark precisa de ter. As pausas de 0,4 s ficam abaixo do limiar de
/// fusão das frases (0,6 s), então o detector lê tudo como **uma** frase.
///
/// Devolve o áudio e os inícios conhecidos.
pub fn canto_continuo_desigual() -> (mir_engine::wav::Audio, Vec<f32>) {
    let mut amostras: Vec<f32> = Vec::new();
    let mut inicios = Vec::new();
    for i in 0..8 {
        if i > 0 {
            senoides(&mut amostras, 0.4, 0.0);
        }
        inicios.push(amostras.len() as f32 / TAXA as f32);
        senoides(&mut amostras, if i % 2 == 0 { 1.0 } else { 5.0 }, 0.15);
    }
    (
        mir_engine::wav::Audio {
            metadados: HashMap::new(),
            taxa: TAXA,
            canais: 1,
            amostras,
        },
        inicios,
    )
}

/// Linhas com **ataques densos** (sílabas), como no áudio real.
///
/// Os outros casos deste banco têm **um** ataque por linha (tom contínuo) — e foi
/// por isso que ele aprovou um mecanismo (fronteiras pelos maiores intervalos)
/// que o gabarito humano reprovou: com ataques limpos e bem separados, "o maior
/// intervalo" acha a fronteira; no áudio real, com sílabas, não acha.
///
/// Aqui cada linha tem **seis** sílabas (ataques) e as sílabas duram diferente
/// por linha — então o número de sílabas é **igual** (o prior proporcional não
/// ajuda) e a duração da linha é **diferente**.
///
/// Devolve o áudio e os inícios conhecidos.
pub fn canto_com_silabas() -> (mir_engine::wav::Audio, Vec<f32>) {
    let mut amostras: Vec<f32> = Vec::new();
    let mut inicios = Vec::new();
    for (i, dur_silaba) in [0.12f32, 0.30, 0.15, 0.40].iter().enumerate() {
        if i > 0 {
            senoides(&mut amostras, 0.5, 0.0); // pausa de linha (< 0.6 s: funde)
        }
        inicios.push(amostras.len() as f32 / TAXA as f32);
        for _ in 0..6 {
            senoides(&mut amostras, *dur_silaba, 0.15);
            senoides(&mut amostras, 0.08, 0.0);
        }
    }
    (
        mir_engine::wav::Audio {
            metadados: HashMap::new(),
            taxa: TAXA,
            canais: 1,
            amostras,
        },
        inicios,
    )
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
        metadados: HashMap::new(),
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
        metadados: HashMap::new(),
        taxa: TAXA,
        canais: 1,
        amostras,
    }
}
