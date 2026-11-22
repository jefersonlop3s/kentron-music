//! Prova do motor MIR em áudio **sintetizado**: progressão conhecida
//! (C–G–Am–F), andamento conhecido (120 BPM) e um final que some.
//!
//! Sintetizar é o ponto: o resultado esperado é conhecido, então o teste mede o
//! motor e não a música. Material de terceiros não entra em fixture.
//!
//! A contraprova do LUFS (comparação com o filtro `ebur128` do `ffmpeg`) vive no
//! fim do arquivo: ela **avisa** quando o `ffmpeg` não está na máquina — aviso
//! visível, não pulo silencioso.

use mir_engine::wav::{self, Audio};
use mir_engine::{Opts, analyze};

const TAXA: u32 = 44_100;

/// Frequência da nota MIDI (69 = A4 = 440 Hz).
fn hz(midi: i32) -> f32 {
    440.0 * 2f32.powf((midi as f32 - 69.0) / 12.0)
}

/// Escreve um acorde (tríade + uma oitava acima) por `duracao` segundos.
fn acorde(saida: &mut Vec<f32>, tonicas: [i32; 3], duracao: f32, ganho: f32) {
    let n = (duracao * TAXA as f32) as usize;
    for i in 0..n {
        let t = i as f32 / TAXA as f32;
        // Envelope percussivo: ataque rápido e queda — dá onsets ao detector.
        let ataque = (i as f32 / (0.01 * TAXA as f32)).min(1.0);
        let queda = 1.0 - 0.6 * (i as f32 / n as f32);
        let mut amostra = 0.0;
        for midi in tonicas {
            amostra += (2.0 * std::f32::consts::PI * hz(midi) * t).sin();
            amostra += 0.3 * (2.0 * std::f32::consts::PI * hz(midi + 12) * t).sin();
        }
        saida.push(ganho * ataque * queda * amostra / 3.9);
    }
}

/// Progressão C–G–Am–F repetida duas vezes, com final que esmaece.
fn sintetizar() -> Audio {
    let mut amostras = Vec::new();
    // 8 s por volta, 4 acordes de 2 s. Duas voltas (16 s) + 2 s de final.
    for _ in 0..2 {
        acorde(&mut amostras, [48, 52, 55], 2.0, 0.7); // C  (C3 E3 G3)
        acorde(&mut amostras, [43, 47, 50], 2.0, 0.7); // G  (G2 B2 D3)
        acorde(&mut amostras, [45, 48, 52], 2.0, 0.7); // Am (A2 C3 E3)
        acorde(&mut amostras, [41, 45, 48], 2.0, 0.7); // F  (F2 A2 C3)
    }
    // Final: mesma fundamental de C, esmaecendo até o silêncio.
    let n = (2.0 * TAXA as f32) as usize;
    for i in 0..n {
        let t = i as f32 / TAXA as f32;
        let queda = 1.0 - i as f32 / n as f32;
        let mut amostra = 0.0;
        for midi in [48, 52, 55] {
            amostra += (2.0 * std::f32::consts::PI * hz(midi) * t).sin();
        }
        amostras.push(0.7 * queda * amostra / 3.9);
    }
    Audio {
        taxa: TAXA,
        canais: 1,
        amostras,
    }
}

/// Escreve WAV PCM 16 bits mono — usado para a contraprova externa.
///
/// A escala é 32768 nas duas pontas (escrita e leitura), como o leitor faz:
/// usar 32767 aqui criava erro de mais de um passo na volta.
fn como_wav(audio: &Audio) -> Vec<u8> {
    let dados: Vec<u8> = audio
        .amostras
        .iter()
        .flat_map(|s| {
            let q = (s.clamp(-1.0, 1.0) * 32_768.0).round();
            (q.clamp(-32_768.0, 32_767.0) as i16).to_le_bytes()
        })
        .collect();
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + dados.len() as u32).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes()); // PCM
    bytes.extend_from_slice(&1u16.to_le_bytes()); // mono
    bytes.extend_from_slice(&audio.taxa.to_le_bytes());
    bytes.extend_from_slice(&(audio.taxa * 2).to_le_bytes()); // byte rate
    bytes.extend_from_slice(&2u16.to_le_bytes()); // block align
    bytes.extend_from_slice(&16u16.to_le_bytes()); // bits
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&(dados.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&dados);
    bytes
}

#[test]
fn cifra_sai_em_chordpro_com_tonalidade_e_sem_acorde_repetido() {
    let audio = sintetizar();
    let opts = Opts {
        trecho_minimo_s: 4.0,
        ..Opts::default()
    };
    let relatorio = analyze(&audio, &opts);

    // O tom é nome de nota: nenhum dígito. `{key: B5}` era o defeito medido.
    let tom = relatorio.tonalidade.rotulo().to_owned();
    assert!(
        !tom.chars().any(|c| c.is_ascii_digit()),
        "tom com dígito: {tom}"
    );

    let chordpro = relatorio.para_chordpro("prova");
    assert!(chordpro.starts_with("{title: prova}"));
    assert!(chordpro.contains(&format!("{{key: {tom}}}")));
    // C–G–Am–F é Dó maior: a progressão I–V–vi–IV não deixa dúvida de centro
    // tonal, e a contagem antiga de fundamental de acorde não dava isso.
    assert_eq!(tom, "C", "regra: {}", relatorio.tonalidade.regra);
    // `Db` é bem (D + b); o nome tem no máximo três caracteres.
    assert!(tom.len() <= 3, "tom: {tom}");

    // Acorde não repete vizinho na mesma seção.
    for linha in chordpro.lines().filter(|l| l.starts_with('[')) {
        let acordes: Vec<&str> = linha
            .split('[')
            .skip(1)
            .filter_map(|t| t.split(']').next())
            .collect();
        for par in acordes.windows(2) {
            assert_ne!(par[0], par[1], "acorde repetido em sequência: {linha}");
        }
    }
}

#[test]
fn le_de_volta_o_wav_que_escreveu() {
    let audio = sintetizar();
    let bytes = como_wav(&audio);
    let lido = wav::ler(&bytes).expect("deve ler o próprio WAV");
    assert_eq!(lido.taxa, audio.taxa);
    assert_eq!(lido.canais, 1);
    assert_eq!(lido.amostras.len(), audio.amostras.len());
    // 16 bits quantiza: tolerância de um passo.
    let maior = lido
        .amostras
        .iter()
        .zip(&audio.amostras)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f32, f32::max);
    assert!(maior < 1.0 / 32_000.0, "diferença de quantização: {maior}");
}

#[test]
fn reconhece_a_progressao_o_andamento_e_a_estrutura() {
    let audio = sintetizar();
    // A fixture tem 18 s: o mínimo de trecho do default (15 s) é para música.
    // Aqui a escala é outra, então o teste escolhe o mínimo dela.
    let opts = Opts {
        trecho_minimo_s: 4.0,
        ..Opts::default()
    };
    let relatorio = analyze(&audio, &opts);

    // 1. Duração: 2 voltas de 8 s + 2 s de final.
    assert!(
        (relatorio.audio.duracao_s - 18.0).abs() < 0.5,
        "duração medida: {}",
        relatorio.audio.duracao_s
    );

    // 2. Acordes: a progressão tem de aparecer.
    let distintos: Vec<String> =
        relatorio
            .chords
            .iter()
            .map(|t| t.acorde.clone())
            .fold(Vec::new(), |mut acc, a| {
                if acc.last() != Some(&a) {
                    acc.push(a);
                }
                acc
            });
    for acorde in ["C", "G", "Am", "F"] {
        assert!(
            distintos.iter().any(|d| d == acorde),
            "acorde {acorde} não apareceu; detectado: {distintos:?}"
        );
    }

    // 3. Andamento: 120 BPM, com folga para o estimador.
    assert!(
        (110.0..=130.0).contains(&relatorio.tempo.bpm),
        "bpm medido: {}",
        relatorio.tempo.bpm
    );

    // 4. Estrutura: as duas voltas são o mesmo material, então o motor precisa
    //    enxergar repetição — é isso que ele chama de refrão candidato.
    assert!(
        relatorio.sections.len() >= 3,
        "trechos: {}",
        relatorio.sections.len()
    );
    assert_eq!(relatorio.sections[0].papel, "introducao");
    assert_eq!(
        relatorio.sections.last().expect("há trechos").papel,
        "final"
    );

    // 5. Dinâmica: o final esmaece, então a cauda é mais baixa que o começo.
    let curto = &relatorio.dynamics.curva_curto_prazo;
    let comeco = curto.first().expect("há curva").1;
    let fim = curto.last().expect("há curva").1;
    assert!(
        fim < comeco,
        "o final deveria ser mais baixo: {comeco:.1} -> {fim:.1} LUFS"
    );
    assert!(relatorio.dynamics.lufs_integrado.is_finite());
}

#[test]
fn contraprova_do_lufs_contra_o_ffmpeg() {
    let audio = sintetizar();
    // Hermeticidade (skill `hermeticidade-de-teste`): processo filho declara o
    // próprio ambiente — herança implícita é o defeito. E o nome do arquivo leva
    // o PID: nome fixo em `/tmp` colide se duas sessões rodarem o hook juntas.
    let caminho = std::env::temp_dir().join(format!(
        "kentron-mir-contraprova-{}.wav",
        std::process::id()
    ));
    let caminho_str = caminho.to_str().expect("caminho utf-8");
    // O `env` pode apontar `TMPDIR` para qualquer lugar; se o caminho começasse
    // com `-`, o ffmpeg o leria como flag. Não começa, e o teste diz por quê.
    assert!(
        caminho.is_absolute() && !caminho_str.starts_with('-'),
        "caminho de prova inadequado para argv: {caminho_str}"
    );
    std::fs::write(&caminho, como_wav(&audio)).expect("escrever o WAV de prova");

    let saida = match std::process::Command::new("ffmpeg")
        .env_clear()
        .env("PATH", std::env::var_os("PATH").unwrap_or_default())
        .env("LC_ALL", "C")
        .args([
            "-hide_banner",
            "-nostats",
            "-i",
            caminho_str,
            "-filter_complex",
            "ebur128",
            "-f",
            "null",
            "-",
        ])
        .output()
    {
        Ok(saida) => saida,
        Err(erro) => {
            // Aviso alto, não pulo silencioso: a contraprova não rodou.
            println!("AVISO: contraprova do LUFS NÃO executada (ffmpeg ausente: {erro})");
            return;
        }
    };
    let texto = String::from_utf8_lossy(&saida.stderr);
    // A ÚLTIMA ocorrência de `I:` é a do resumo; as primeiras são as linhas de
    // progresso do filtro, com o loudness corrente (medido: pegar a primeira
    // devolvia -70, o piso). `rsplit` caminha do fim para o começo.
    let medido = texto.rsplit("I:").find_map(|resto| {
        resto
            .split_whitespace()
            .next()
            .and_then(|valor| valor.parse::<f64>().ok())
    });

    let Some(ffmpeg_lufs) = medido else {
        panic!("não achei o LUFS integrado na saída do ffmpeg");
    };
    let nosso = analyze(&audio, &Opts::default()).dynamics.lufs_integrado;
    assert!(
        (nosso - ffmpeg_lufs).abs() < 1.0,
        "LUFS divergiu: nosso {nosso:.2}, ffmpeg {ffmpeg_lufs:.2}"
    );
    let _ = std::fs::remove_file(&caminho);
}
