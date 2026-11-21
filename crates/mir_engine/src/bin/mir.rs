//! CLI do motor MIR.
//!
//! Uso:
//!
//! ```text
//! cargo run -p mir_engine -- musica.wav                                 # resumo legível
//! cargo run -p mir_engine -- musica.wav --json                          # relatório completo
//! cargo run -p mir_engine -- musica.wav --frases-vocais                 # onde a voz soa
//! cargo run -p mir_engine -- musica.wav --chordpro                      # cifra (só acordes)
//! cargo run -p mir_engine -- musica.wav --chordpro --hino Amazing       # cifra com letra
//! cargo run -p mir_engine -- musica.wav --chordpro --letra letra.txt    # letra do usuário
//!   [--licenca "<o que o usuário informar>"] [--gravacao "<descrição>|<ano>|<titular>"]
//! cargo run -p mir_engine -- musica.wav --chordpro --letra l.txt --extraida-por "José"
//! cargo run -p mir_engine -- --listar-hinos                             # fichas do corpus
//! cargo run -p mir_engine -- --auditar-corpus                            # confere as fichas
//! cargo run -p mir_engine -- --ficha "Título|Autor|1822|1868|||Harpa Cristã"
//! cargo run -p mir_engine -- --relatorio-uso <pasta-do-bundle>   # uso + fichas
//! ```
//!
//! O `ffmpeg` converte antes; este binário lê WAV.
//!
//! **Procedência em dois eixos (ADR-0012):** a *obra* (letra e música) e o
//! *fonograma* (a gravação analisada) são declarados separadamente. Gravação de
//! ontem de obra de 1822 continua protegida, e a saída nunca afirma domínio
//! público sobre a gravação. Letra extraída **pelo usuário** nasce com escopo
//! privado do grupo: o app é a ferramenta da licença dele, não a fonte de letra.

use std::path::PathBuf;
use std::process::ExitCode;

use mir_engine::lyrics::{self, Fonograma, Jurisdicao, Letra, Obra, Origem};
use mir_engine::sheet::{self, Ficha};
use mir_engine::{Opts, wav};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let ano_corrente: u16 = 2026;

    if args.iter().any(|a| a == "--listar-hinos") {
        for obra in lyrics::fichas_do_corpus() {
            let veredito = lyrics::veredito_pd(&obra, Jurisdicao::Brasil, ano_corrente);
            println!(
                "{} · {} · obra {} · autor falecido em {:?} · BR: {}",
                obra.titulo,
                obra.autor,
                obra.ano,
                obra.autor_morte
                    .map_or_else(|| "?".to_owned(), |m| m.to_string()),
                veredito.resumo()
            );
        }
        return ExitCode::SUCCESS;
    }

    if args.iter().any(|a| a == "--auditar-corpus") {
        return match lyrics::auditar_corpus(ano_corrente) {
            Ok(relatorio) => {
                for linha in relatorio {
                    println!("✅ {linha}");
                }
                ExitCode::SUCCESS
            }
            Err(problemas) => {
                for problema in problemas {
                    eprintln!("❌ {problema}");
                }
                ExitCode::FAILURE
            }
        };
    }

    let valor_de = |bandeira: &str| -> Option<String> {
        let posicao = args.iter().position(|a| a == bandeira)?;
        args.get(posicao + 1).cloned()
    };

    if let Some(ficha_bruta) = valor_de("--ficha") {
        let campos: Vec<&str> = ficha_bruta.split('|').collect();
        let campo = |i: usize| campos.get(i).map_or("", |v| v.trim());
        let obra = Obra {
            titulo: campo(0).to_owned(),
            autor: campo(1).to_owned(),
            ano: campo(2).parse().unwrap_or(0),
            autor_morte: campo(3).parse().ok(),
            tradutor: Some(campo(4).to_owned()).filter(|t| !t.is_empty()),
            tradutor_morte: campo(5).parse().ok(),
            base: campo(6).to_owned(),
        };
        println!(
            "obra:  {} — {} ({}), base: {}",
            obra.titulo, obra.autor, obra.ano, obra.base
        );
        for jurisdicao in [Jurisdicao::Brasil, Jurisdicao::Eua] {
            println!(
                "  {}: {}",
                jurisdicao.nome(),
                lyrics::veredito_pd(&obra, jurisdicao, ano_corrente).resumo()
            );
        }
        println!(
            "  (veredito de OBRA; a gravação analisada tem titular próprio — declarada com --gravacao)"
        );
        return ExitCode::SUCCESS;
    }

    if let Some(pasta) = valor_de("--relatorio-uso") {
        return match mir_engine::uso::do_diretorio(std::path::Path::new(&pasta), ano_corrente) {
            Ok(relatorio) => {
                print!("{}", relatorio.texto());
                ExitCode::SUCCESS
            }
            Err(erro) => {
                eprintln!("erro: {erro}");
                ExitCode::FAILURE
            }
        };
    }

    let json = args.iter().any(|a| a == "--json");
    let chordpro = args.iter().any(|a| a == "--chordpro");

    let caminho = match args
        .iter()
        .find(|a| !a.starts_with("--") && !eh_valor_de(&args, a))
    {
        Some(c) => PathBuf::from(c),
        None => {
            eprintln!(
                "uso: mir <arquivo.wav> [--json|--chordpro|--frases-vocais] \
                 [--letra arquivo | --hino titulo] [--licenca texto] [--gravacao \"desc|ano|titular\"] \
                 [--extraida-por nome] | --listar-hinos | --auditar-corpus | --ficha \"campos\""
            );
            return ExitCode::FAILURE;
        }
    };

    // Fonte da letra: declarada, e o erro é explícito quando não existe.
    let extraida_por = valor_de("--extraida-por");
    let licenca = valor_de("--licenca");
    let letra: Option<Letra> = if let Some(caminho_letra) = valor_de("--letra") {
        match std::fs::read_to_string(&caminho_letra) {
            Ok(texto) => {
                let mut letra = lyrics::do_usuario(&texto, licenca.clone());
                if let Some(responsavel) = &extraida_por {
                    // Ato do usuário: a cifra nasce privada no grupo (ADR-0012).
                    letra.origem = Origem::ExtraidaPeloUsuario {
                        responsavel: responsavel.clone(),
                        obra: None,
                    };
                }
                Some(letra)
            }
            Err(erro) => {
                eprintln!("erro ao ler a letra {caminho_letra:?}: {erro}");
                return ExitCode::FAILURE;
            }
        }
    } else if let Some(titulo) = valor_de("--hino") {
        match lyrics::do_corpus(&titulo) {
            Some(letra) => {
                eprintln!("letra: {}", letra.origem.descricao());
                Some(letra)
            }
            None => {
                eprintln!(
                    "obra {titulo:?} não está no corpus (veja --listar-hinos). Se é domínio público, \
                     monte a ficha com --ficha e acrescente a entrada com autor, falecimento e base."
                );
                return ExitCode::FAILURE;
            }
        }
    } else {
        None
    };

    // Fonograma: o outro eixo. Gravação não é obra.
    let fonograma = valor_de("--gravacao").map(|bruto| {
        let campos: Vec<&str> = bruto.split('|').collect();
        Fonograma {
            descricao: campos
                .first()
                .map_or("".to_owned(), |v| v.trim().to_owned()),
            ano: campos.get(1).and_then(|v| v.trim().parse().ok()),
            titular: campos
                .get(2)
                .map(|v| v.trim().to_owned())
                .filter(|t| !t.is_empty()),
        }
    });

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

    // Esqueleto de letra: onde a voz soa.
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
        if alinhadas.is_empty() {
            let titulo = caminho.file_stem().map_or_else(
                || "sem título".to_owned(),
                |s| s.to_string_lossy().into_owned(),
            );
            print!("{}", relatorio.para_chordpro(&titulo));
            return ExitCode::SUCCESS;
        }
        let titulo = caminho.file_stem().map_or_else(
            || "sem título".to_owned(),
            |s| s.to_string_lossy().into_owned(),
        );
        let ficha = Ficha {
            titulo,
            origem: letra
                .as_ref()
                .map_or(Origem::DoUsuario { licenca: None }, |l| l.origem.clone()),
            fonograma,
            ano_corrente,
        };
        let musica = sheet::montar(&relatorio, &alinhadas, &ficha);
        print!("{}", notation_bundle::chordpro::write(&musica));
        eprintln!(
            "obra: {} · {} linha(s), confiança média {:.2}",
            ficha.origem.descricao(),
            alinhadas.len(),
            alinhadas.iter().map(|l| l.confianca).sum::<f32>() / alinhadas.len() as f32
        );
        eprintln!("escopo: {}", ficha.origem.escopo().nome());
        if let Err(motivo) = sheet::pode_publicar(&musica) {
            eprintln!("⚠ publicação bloqueada: {motivo}");
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
    const VALORADAS: [&str; 7] = [
        "--letra",
        "--hino",
        "--licenca",
        "--gravacao",
        "--extraida-por",
        "--ficha",
        "--relatorio-uso",
    ];
    args.iter().enumerate().any(|(i, argumento)| {
        VALORADAS.contains(&argumento.as_str()) && i + 1 < args.len() && args[i + 1] == valor
    })
}
