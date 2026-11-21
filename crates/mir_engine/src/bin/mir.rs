//! CLI do motor MIR.
//!
//! Uso:
//!
//! ```text
//! cargo run -p mir_engine -- musica.wav                                # resumo legível
//! cargo run -p mir_engine -- musica.wav --json                         # relatório completo
//! cargo run -p mir_engine -- musica.wav --chordpro                     # cifra (só acordes)
//! cargo run -p mir_engine -- musica.wav --chordpro --hino Amazing      # cifra com letra
//! cargo run -p mir_engine -- musica.wav --chordpro --letra letra.txt   # letra do usuário
//! cargo run -p mir_engine -- musica.wav --frases-vocais            # onde a voz soa
//! cargo run -p mir_engine -- --listar-hinos                            # corpus de domínio público
//! ```
//!
//! O `ffmpeg` converte antes; este binário lê WAV.
//!
//! A letra vem de **fonte declarada** (ADR-0012): `--letra` é texto do próprio
//! usuário e `--hino` busca no corpus de domínio público embutido. Raspagem de
//! site de letra não existe aqui — nem no teste.

use std::path::PathBuf;
use std::process::ExitCode;

use mir_engine::lyrics::Letra;
use mir_engine::{Opts, sheet, wav};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--listar-hinos") {
        for (titulo, autor, ano) in mir_engine::lyrics::titulos_do_corpus() {
            println!("{titulo} · {autor} · {ano}");
        }
        return ExitCode::SUCCESS;
    }

    let json = args.iter().any(|a| a == "--json");
    let chordpro = args.iter().any(|a| a == "--chordpro");
    let valor_de = |bandeira: &str| -> Option<String> {
        let posicao = args.iter().position(|a| a == bandeira)?;
        args.get(posicao + 1).cloned()
    };

    let caminho = match args
        .iter()
        .find(|a| !a.starts_with("--") && !eh_valor_de(&args, a))
    {
        Some(c) => PathBuf::from(c),
        None => {
            eprintln!(
                "uso: mir <arquivo.wav> [--json|--chordpro] [--letra arquivo | --hino titulo]"
            );
            return ExitCode::FAILURE;
        }
    };

    // Fonte da letra: declarada, e o erro é explícito quando não existe.
    let letra: Option<Letra> = if let Some(caminho_letra) = valor_de("--letra") {
        match std::fs::read_to_string(&caminho_letra) {
            Ok(texto) => Some(mir_engine::lyrics::ler_do_usuario(&texto)),
            Err(erro) => {
                eprintln!("erro ao ler a letra {caminho_letra:?}: {erro}");
                return ExitCode::FAILURE;
            }
        }
    } else if let Some(titulo) = valor_de("--hino") {
        match mir_engine::lyrics::do_corpus(&titulo) {
            Some(letra) => {
                eprintln!("letra: {}", letra.origem.descricao());
                Some(letra)
            }
            None => {
                eprintln!(
                    "hino {titulo:?} não está no corpus de domínio público (veja --listar-hinos)"
                );
                return ExitCode::FAILURE;
            }
        }
    } else {
        None
    };

    let audio = match wav::ler_arquivo(&caminho) {
        Ok(audio) => audio,
        Err(erro) => {
            eprintln!("erro ao ler {caminho:?}: {erro}");
            return ExitCode::FAILURE;
        }
    };
    if audio.amostras.is_empty() {
        eprintln!("erro: áudio vazio");
        return ExitCode::FAILURE;
    }

    // Esqueleto de letra: onde a voz soa. Serve para o músico ver **onde** cada
    // linha vai cair antes de fornecer a letra — e para medir em música cuja
    // letra não se tem (ADR-0012).
    if args.iter().any(|a| a == "--frases-vocais") {
        let opts = Opts::default();
        let spectro = mir_engine::spectrum::Spectrogram::new(&audio, opts.janela, opts.salto);
        let frases = mir_engine::lyrics::frases_vocais(&spectro, audio.taxa);
        let cobertura: f32 = frases.iter().map(|&(inicio, fim)| fim - inicio).sum();
        println!(
            "frases vocais: {} · cobertura {:.1} s de {:.1} s",
            frases.len(),
            cobertura,
            audio.duracao()
        );
        for (i, (inicio, fim)) in frases.iter().enumerate() {
            println!(
                "  {:>3}  {:>7.1}s–{:>7.1}s  ({:.1} s)",
                i + 1,
                inicio,
                fim,
                fim - inicio
            );
        }
        return ExitCode::SUCCESS;
    }

    let (relatorio, alinhadas) =
        mir_engine::analyze_com_letra(&audio, &Opts::default(), letra.as_ref());
    if json {
        match relatorio.para_json() {
            Ok(texto) => println!("{texto}"),
            Err(erro) => {
                eprintln!("erro ao serializar: {erro}");
                return ExitCode::FAILURE;
            }
        }
    } else if chordpro {
        let titulo = caminho.file_stem().map_or_else(
            || "sem título".to_owned(),
            |s| s.to_string_lossy().into_owned(),
        );
        let procedencia = letra.as_ref().map_or_else(
            || "sem letra (só acordes)".to_owned(),
            |l| l.origem.descricao(),
        );
        if alinhadas.is_empty() {
            print!("{}", relatorio.para_chordpro(&titulo));
        } else {
            let musica = sheet::montar(&relatorio, &alinhadas, &titulo, &procedencia);
            print!("{}", notation_bundle::chordpro::write(&musica));
            eprintln!(
                "letra: {} · {} linha(s) alinhada(s), confiança média {:.2}",
                procedencia,
                alinhadas.len(),
                alinhadas.iter().map(|l| l.confianca).sum::<f32>() / alinhadas.len() as f32
            );
        }
    } else {
        print!("{}", relatorio.resumo());
        for linha in &alinhadas {
            println!(
                "  {:>6.1}s–{:>6.1}s  {:>3} {:<20} {:<34} {:>6.1} LUFS  conf {:.2}{}",
                linha.inicio_s,
                linha.fim_s,
                linha.trecho,
                linha.papel,
                linha.texto,
                linha.lufs_medio,
                linha.confianca,
                if linha.repetida { "  (repete)" } else { "" }
            );
        }
    }
    ExitCode::SUCCESS
}

/// Diz se o argumento é o valor de uma bandeira (`--letra x`), não o arquivo.
fn eh_valor_de(args: &[String], valor: &str) -> bool {
    args.iter().enumerate().any(|(i, argumento)| {
        matches!(argumento.as_str(), "--letra" | "--hino")
            && i + 1 < args.len()
            && args[i + 1] == valor
    })
}
