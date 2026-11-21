//! CLI do motor MIR.
//!
//! Uso:
//!
//! ```text
//! cargo run -p mir_engine -- musica.wav            # resumo legível
//! cargo run -p mir_engine -- musica.wav --json     # relatório completo
//! ```
//!
//! O `ffmpeg` converte antes; este binário lê WAV.

use std::path::PathBuf;
use std::process::ExitCode;

use mir_engine::{Opts, wav};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let json = args.iter().any(|a| a == "--json");
    let caminho = match args.iter().find(|a| !a.starts_with("--")) {
        Some(c) => PathBuf::from(c),
        None => {
            eprintln!("uso: mir <arquivo.wav> [--json]");
            return ExitCode::FAILURE;
        }
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

    let relatorio = mir_engine::analyze(&audio, &Opts::default());
    if json {
        match relatorio.para_json() {
            Ok(texto) => println!("{texto}"),
            Err(erro) => {
                eprintln!("erro ao serializar: {erro}");
                return ExitCode::FAILURE;
            }
        }
    } else if args.iter().any(|a| a == "--chordpro") {
        let titulo = caminho.file_stem().map_or_else(
            || "sem título".to_owned(),
            |s| s.to_string_lossy().into_owned(),
        );
        print!("{}", relatorio.para_chordpro(&titulo));
    } else {
        print!("{}", relatorio.resumo());
    }
    ExitCode::SUCCESS
}
