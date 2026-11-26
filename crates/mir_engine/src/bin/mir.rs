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
//!
//! A letra do usuário pode declarar a **ordem cantada**: `{bloco refrao}` …
//! `{fim}` define um bloco (cantado onde é definido), `{canta refrao 3x}` canta
//! o bloco de novo e o sufixo `(2x)` repete a linha. A cifra/letra de qualquer
//! fonte traz as linhas **distintas** e a gravação **repete** — sem isso o
//! refrão é alinhado a um trecho onde ele não é cantado. Repetição malformada
//! **reprova** com a linha do arquivo, em vez de sumir em silêncio.
//! cargo run -p mir_engine -- --listar-hinos                             # fichas do corpus
//! cargo run -p mir_engine -- --auditar-corpus                            # confere as fichas
//! cargo run -p mir_engine -- --listar-fichas                             # fichas do hinário + resposta
//! cargo run -p mir_engine -- --ficha-hino "Cantor Cristão|09"            # ficha por (hinário, número)
//! cargo run -p mir_engine -- --auditar-fichas                            # aritmética morte+71, linha a linha
//! cargo run -p mir_engine -- --ficha "Título|Autor|1822|1868|Adaptador|morte do adaptador|obra original|base"
//! cargo run -p mir_engine -- --relatorio-uso <pasta-do-bundle>   # uso + fichas
//! ```
//!
//! O arquivo entra direto: WAV PCM pelo leitor próprio, mp3/AAC/FLAC/Vorbis/
//! Opus/ADPCM/WavPack pelo Symphonia (ADR-0016). `ffmpeg` não é necessário.
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
use mir_engine::{Opts, decode};

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

    if args.iter().any(|a| a == "--auditar-fichas") {
        return match mir_engine::fichas::auditar() {
            Ok(linhas) => {
                for linha in linhas {
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

    if args.iter().any(|a| a == "--listar-fichas") {
        for ficha in mir_engine::fichas::fichas() {
            println!(
                "{} · {} · {} — {} · {}",
                ficha.hinario,
                ficha.numero,
                ficha.titulo,
                ficha.veredito.nome(),
                mir_engine::fichas::resposta_para_letra(ficha, ano_corrente).resumo()
            );
        }
        return ExitCode::SUCCESS;
    }

    let valor_de = |bandeira: &str| -> Option<String> {
        let posicao = args.iter().position(|a| a == bandeira)?;
        args.get(posicao + 1).cloned()
    };

    if let Some(bruto) = valor_de("--ficha-hino") {
        let (hinario, numero) = bruto.split_once('|').unwrap_or((bruto.as_str(), ""));
        let Some(ficha) = mir_engine::fichas::por_identidade(hinario, numero) else {
            eprintln!(
                "sem ficha para ({hinario}, {numero}) — veja --listar-fichas. \
                 O motor não afirma domínio público nem proteção do que não tem ficha."
            );
            return ExitCode::FAILURE;
        };
        let vazio = "—";
        println!("{}", ficha.rotulo());
        println!("  autoria:  {}", ficha.autoria);
        println!(
            "  tradução: {}",
            if ficha.traducao.is_empty() {
                vazio
            } else {
                &ficha.traducao
            }
        );
        println!(
            "  melodia:  {}",
            if ficha.melodia.is_empty() {
                vazio
            } else {
                &ficha.melodia
            }
        );
        println!(
            "  ficha:    {} · morte relevante {} · domínio público {}",
            ficha.veredito.nome(),
            ficha
                .ano_relevante
                .map_or_else(|| "?".to_owned(), |ano| ano.to_string()),
            ficha
                .ano_dominio_publico
                .map_or_else(|| "?".to_owned(), |ano| ano.to_string())
        );
        for (campo, fonte) in [
            ("autoria", &ficha.fonte_autoria),
            ("tradução", &ficha.fonte_traducao),
            ("melodia", &ficha.fonte_melodia),
        ] {
            if !fonte.trim().is_empty() {
                println!("  fonte {campo}: {fonte}");
            }
        }
        if !ficha.observacao.trim().is_empty() {
            println!("  observação: {}", ficha.observacao);
        }
        if let Some(original) = mir_engine::fichas::obra_original_no_corpus(ficha) {
            println!(
                "  obra original no corpus: {} — {} ({}) [vínculo de referência: a letra do \
                 corpus é a original; esta ficha é a tradução]",
                original.titulo, original.autor, original.ano
            );
        }
        println!(
            "  resposta (letra): {}",
            mir_engine::fichas::resposta_para_letra(ficha, ano_corrente).resumo()
        );
        return ExitCode::SUCCESS;
    }

    if let Some(ficha_bruta) = valor_de("--ficha") {
        let campos: Vec<&str> = ficha_bruta.split('|').collect();
        let campo = |i: usize| campos.get(i).map_or("", |v| v.trim());
        let obra = Obra {
            titulo: campo(0).to_owned(),
            autor: campo(1).to_owned(),
            ano: campo(2).parse().unwrap_or(0),
            autor_morte: campo(3).parse().ok(),
            // Tradução e adaptação com frases novas: obra derivada, autor próprio.
            adaptador: Some(campo(4).to_owned()).filter(|t| !t.is_empty()),
            adaptador_morte: campo(5).parse().ok(),
            obra_original: Some(campo(6).to_owned()).filter(|t| !t.is_empty()),
            base: campo(7).to_owned(),
        };
        println!(
            "obra:  {} — {} ({}), base: {}",
            obra.titulo, obra.autor, obra.ano, obra.base
        );
        if let Some(adaptador) = &obra.adaptador {
            let morte = obra.adaptador_morte.map_or_else(
                || "falecimento não informado".to_owned(),
                |ano| format!("falecido em {ano}"),
            );
            let original = obra
                .obra_original
                .as_deref()
                .unwrap_or("obra original não informada");
            println!("  derivada: adaptação/tradução de {adaptador} ({morte}) sobre {original}");
        }
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
            // Sem áudio não há cifra, mas `--hino` sozinho ainda tem resposta:
            // a ficha do hinário diz se aquela versão pode ter letra e por quê.
            if let Some(titulo) = valor_de("--hino") {
                return explicar_letra_por_ficha(&titulo, ano_corrente);
            }
            eprintln!(
                "uso: mir <arquivo.wav> [--json|--chordpro|--frases-vocais] \
                 [--letra arquivo | --hino titulo] [--licenca texto] [--gravacao \"desc|ano|titular\"] \
                 [--extraida-por nome] [--stems dir] | --listar-hinos | --auditar-corpus | --ficha \"campos\" | \
                 --listar-fichas | --ficha-hino \"hinário|número\" | --auditar-fichas"
            );
            return ExitCode::FAILURE;
        }
    };

    // Fonte da letra: declarada, e o erro é explícito quando não existe.
    let extraida_por = valor_de("--extraida-por");
    let licenca = valor_de("--licenca");
    let letra: Option<Letra> = if let Some(caminho_letra) = valor_de("--letra") {
        match std::fs::read_to_string(&caminho_letra) {
            Ok(texto) => match lyrics::do_usuario(&texto, licenca.clone()) {
                Ok(mut letra) => {
                    if let Some(responsavel) = &extraida_por {
                        // Ato do usuário: a cifra nasce privada no grupo (ADR-0012).
                        letra.origem = Origem::ExtraidaPeloUsuario {
                            responsavel: responsavel.clone(),
                            obra: None,
                        };
                    }
                    // O que o motor entendeu da ordem cantada, dito em voz alta:
                    // sem isto, um marcador que não expande passa por acerto.
                    eprintln!(
                        "letra: {} linha(s) na ordem cantada ({} no texto)",
                        letra.linhas.len(),
                        texto.lines().count()
                    );
                    Some(letra)
                }
                Err(erro) => {
                    eprintln!("erro na letra {caminho_letra:?}: {erro}");
                    return ExitCode::FAILURE;
                }
            },
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
                // Sem letra no corpus, a ficha do hinário ainda responde o que
                // importa: se aquela versão pode ter letra, e por quê.
                return explicar_letra_por_ficha(&titulo, ano_corrente);
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

    let audio = match decode::abrir(&caminho) {
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

    // Diretório opcional com stems de bateria (drums.wav) para análise comparativa.
    // `--stems onnx` ativa a inferência ONNX (Demucs via crate ort).
    // `--stems <dir>` usa stems já separados em disco.
    let stems_dir_raw = valor_de("--stems");
    let stems_dir: Option<std::path::PathBuf> = if let Some(v) = stems_dir_raw {
        if v == "onnx" || v == "auto" {
            // Inferência ONNX: separa os stems, escreve drums.wav em temp
            let model_path = mir_engine::stems_onnx::caminho_padrao();
            if !model_path.exists() {
                eprintln!(
                    "modelo ONNX não encontrado em {:?}",
                    model_path.to_string_lossy()
                );
                eprintln!("baixe com: scripts/baixar-pesos.sh");
                return ExitCode::FAILURE;
            }
            let stems = mir_engine::stems_onnx::separar(&audio, &model_path);
            if let Err(erro) = stems {
                eprintln!("erro na separação ONNX: {erro}");
                return ExitCode::FAILURE;
            }
            let stems = stems.unwrap();
            let stem_name = caminho
                .file_stem()
                .map_or("audio".to_owned(), |s| s.to_string_lossy().into_owned());
            let tmp = std::path::Path::new("/tmp")
                .join("kentron-stems")
                .join(stem_name);
            std::fs::create_dir_all(&tmp).ok();
            for (nome, stem_audio) in stems {
                if nome == "drums" {
                    let drums_path = tmp.join("drums.wav");
                    mir_engine::wav::escrever(&stem_audio, &drums_path).ok();
                    eprintln!(
                        "stems ONNX: drums.wav escrito em {:?}",
                        drums_path.to_string_lossy()
                    );
                }
            }
            Some(tmp)
        } else {
            Some(std::path::PathBuf::from(v))
        }
    } else {
        None
    };

    let (relatorio, alinhadas) = mir_engine::analyze_com_letra(
        &audio,
        &Opts::default(),
        letra.as_ref(),
        stems_dir.as_deref(),
        Some(&caminho),
    );
    if json {
        match relatorio.para_json() {
            Ok(texto) => println!("{texto}"),
            Err(erro) => {
                eprintln!("erro ao serializar: {erro}");
                return ExitCode::FAILURE;
            }
        }
    } else if chordpro {
        // Título das tags do arquivo, com fallback para nome do arquivo.
        let titulo_src = audio
            .metadados
            .get("title")
            .or_else(|| audio.metadados.get("Title"));
        let titulo = titulo_src.map_or_else(
            || {
                caminho.file_stem().map_or_else(
                    || "sem título".to_owned(),
                    |s| s.to_string_lossy().into_owned(),
                )
            },
            String::clone,
        );
        if alinhadas.is_empty() {
            print!("{}", relatorio.para_chordpro(&titulo));
            return ExitCode::SUCCESS;
        }
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

/// Explica a fonte da letra pela ficha do hinário, quando o corpus não tem o hino.
///
/// É o caminho que evita o pior desfecho: dizer "não está no corpus" e deixar o
/// usuário sem saber se pode trazer a letra. A ficha responde — e quando o título
/// é ambíguo, ela nomeia as candidatas em vez de escolher uma.
fn explicar_letra_por_ficha(titulo: &str, ano_corrente: u16) -> ExitCode {
    let resposta = mir_engine::fichas::resposta_por_titulo(titulo, ano_corrente);
    match &resposta {
        mir_engine::fichas::RespostaLetra::SemFicha => eprintln!(
            "obra {titulo:?} não está no corpus nem nas fichas (veja --listar-hinos, \
             --listar-fichas). Se é domínio público, monte a ficha com --ficha."
        ),
        mir_engine::fichas::RespostaLetra::Livre { .. } => eprintln!(
            "obra {titulo:?} não tem letra no corpus — ficha do hinário: {}.\n\
             A letra desta versão é livre: traga o texto com --letra (a cifra vem do áudio).",
            resposta.resumo()
        ),
        _ => eprintln!(
            "obra {titulo:?} não tem letra no corpus — ficha do hinário: {}.\n\
             A letra é do usuário (--letra); a cifra continua vindo do áudio.",
            resposta.resumo()
        ),
    }
    ExitCode::FAILURE
}

/// Diz se o argumento é o valor de uma bandeira (`--letra x`), não o arquivo.
fn eh_valor_de(args: &[String], valor: &str) -> bool {
    const VALORADAS: [&str; 9] = [
        "--letra",
        "--hino",
        "--licenca",
        "--gravacao",
        "--extraida-por",
        "--ficha",
        "--ficha-hino",
        "--relatorio-uso",
        "--stems",
    ];
    args.iter().enumerate().any(|(i, argumento)| {
        VALORADAS.contains(&argumento.as_str()) && i + 1 < args.len() && args[i + 1] == valor
    })
}
