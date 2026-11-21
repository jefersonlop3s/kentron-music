//! Monta a cifra: `Song` do domínio a partir do relatório e da letra alinhada.
//!
//! A colocação do acorde é **mecânica e independente de o nome do acorde estar
//! certo**: o tempo de cada acorde dentro da linha alinhada vira índice de sílaba
//! (ADR-0012 §3). Se o acorde detectado é candidato, a *posição* dele sobre a
//! letra continua sendo a posição que o áudio indica.
//!
//! A saída é `notation_bundle::chordpro::Song` — o mesmo objeto que o app já
//! desenha. Não há formato paralelo a manter.

use std::collections::HashMap;

use notation_bundle::chordpro::{ChordPlacement, Line, Section, Song};

use crate::lyrics::{LinhaAlinhada, silabas};
use crate::report::Report;

/// Monta a cifra com letra e acordes.
///
/// `origem_letra` entra nos metadados como procedência (ADR-0012): a cifra
/// carrega de onde a letra veio, para ninguém confundir texto do usuário com
/// domínio público.
#[must_use]
pub fn montar(
    relatorio: &Report,
    alinhadas: &[LinhaAlinhada],
    titulo: &str,
    origem_letra: &str,
) -> Song {
    let mut metadata: HashMap<String, String> = HashMap::new();
    metadata.insert("title".to_owned(), titulo.to_owned());
    metadata.insert("key".to_owned(), relatorio.tom_estimado());
    metadata.insert("tempo".to_owned(), format!("{:.0}", relatorio.tempo.bpm));
    metadata.insert(
        "duration".to_owned(),
        format!("{:.0}", relatorio.audio.duracao_s),
    );
    metadata.insert("source".to_owned(), "mir_engine".to_owned());
    metadata.insert("lyrics_source".to_owned(), origem_letra.to_owned());

    let mut sections: Vec<Section> = Vec::new();

    // Instrumental antes da primeira linha cantada: aparece como seção, porque
    // o músico precisa saber que não errou a contagem de compasso.
    if let Some(primeira) = alinhadas.first() {
        let introducao = acordes_no_intervalo(relatorio, 0.0, primeira.inicio_s);
        if !introducao.is_empty() {
            sections.push(Section {
                name: Some("Introdução · só instrumento".to_owned()),
                lines: vec![Line {
                    chords: introducao,
                    text: String::new(),
                }],
            });
        }
    }

    // Agrupa linhas consecutivas do mesmo trecho numa seção da cifra.
    let mut inicio = 0usize;
    while inicio < alinhadas.len() {
        let mut fim = inicio + 1;
        while fim < alinhadas.len()
            && alinhadas[fim].trecho == alinhadas[inicio].trecho
            && alinhadas[fim].papel == alinhadas[inicio].papel
        {
            fim += 1;
        }
        let grupo = &alinhadas[inicio..fim];
        let nome = format!("{} · {}", grupo[0].trecho, grupo[0].papel);
        sections.push(Section {
            name: Some(nome),
            lines: grupo
                .iter()
                .map(|linha| Line {
                    chords: acordes_da_linha(relatorio, linha),
                    text: linha.texto.clone(),
                })
                .collect(),
        });
        inicio = fim;
    }

    Song { metadata, sections }
}

/// Acordes de um intervalo qualquer, já sem repetir vizinho.
fn acordes_no_intervalo(relatorio: &Report, inicio_s: f32, fim_s: f32) -> Vec<ChordPlacement> {
    let mut colocados: Vec<ChordPlacement> = Vec::new();
    let mut anterior = String::new();
    let mut position = 0usize;
    for trecho in &relatorio.chords {
        if trecho.fim_s <= inicio_s || trecho.inicio_s >= fim_s || trecho.acorde == anterior {
            continue;
        }
        colocados.push(ChordPlacement {
            position,
            chord: trecho.acorde.clone(),
        });
        position += trecho.acorde.chars().count() + 1;
        anterior.clone_from(&trecho.acorde);
    }
    colocados
}

/// Coloca cada acorde sobre a sílaba que o tempo dele indica.
fn acordes_da_linha(relatorio: &Report, linha: &LinhaAlinhada) -> Vec<ChordPlacement> {
    let inicios = silabas(&linha.texto);
    if inicios.is_empty() {
        return Vec::new();
    }
    let duracao = (linha.fim_s - linha.inicio_s).max(0.001);
    let limite = linha.texto.chars().count();
    let mut colocados: Vec<ChordPlacement> = Vec::new();
    let mut anterior = String::new();
    for trecho in &relatorio.chords {
        if trecho.fim_s <= linha.inicio_s || trecho.inicio_s >= linha.fim_s {
            continue;
        }
        if trecho.acorde == anterior {
            continue;
        }
        let relativo = ((trecho.inicio_s - linha.inicio_s) / duracao).clamp(0.0, 0.999);
        let indice = ((relativo * inicios.len() as f32) as usize).min(inicios.len() - 1);
        colocados.push(ChordPlacement {
            position: inicios[indice].min(limite),
            chord: trecho.acorde.clone(),
        });
        anterior.clone_from(&trecho.acorde);
    }
    colocados
}
