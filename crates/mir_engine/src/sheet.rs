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

use crate::lyrics::{Fonograma, LinhaAlinhada, Origem, Veredito, silabas, veredito_pd};
use crate::report::Report;

/// Ficha da cifra: a **obra** e a **gravação**, que são coisas diferentes
/// (ADR-0012). O título batiza o arquivo; a origem responde pelo texto; o
/// fonograma diz de quem é o áudio analisado.
#[derive(Debug, Clone)]
pub struct Ficha {
    pub titulo: String,
    pub origem: Origem,
    /// Gravação analisada. `None` = não declarada, e a cifra diz isso.
    pub fonograma: Option<Fonograma>,
    /// Ano corrente, para a regra de domínio público ser explícita e testável.
    pub ano_corrente: u16,
}

/// Monta a cifra com letra e acordes.
///
/// A procedência viaja na cifra em **dois eixos** (ADR-0012): `lyrics_source` +
/// `work_status` para a obra, `recording_source` para a gravação, e
/// `lyrics_scope` para onde ela pode circular. Ninguém deve confundir texto do
/// usuário com domínio público, nem obra de 1822 com a gravação de ontem.
#[must_use]
pub fn montar(relatorio: &Report, alinhadas: &[LinhaAlinhada], ficha: &Ficha) -> Song {
    let mut metadata: HashMap<String, String> = HashMap::new();
    metadata.insert("title".to_owned(), ficha.titulo.clone());
    metadata.insert("key".to_owned(), relatorio.tonalidade.rotulo().to_owned());
    metadata.insert("tempo".to_owned(), format!("{:.0}", relatorio.tempo.bpm));
    metadata.insert(
        "duration".to_owned(),
        format!("{:.0}", relatorio.audio.duracao_s),
    );
    metadata.insert("source".to_owned(), "mir_engine".to_owned());
    // O compasso **não** entra como `time`. O nome do motor acerta 1 de 8 no
    // gabarito independente de `David Bennett` — gravá-lo como fato seria pior
    // que não gravar. A leitura vai como **comentário**, e `{time: …}` fica para
    // o músico: a diretiva é a decisão dele, não o palpite do motor.
    let leitura = relatorio.compasso.leitura.sugestao();
    if !leitura.is_empty() {
        metadata.insert("comment".to_owned(), leitura);
    }
    metadata.insert("lyrics_source".to_owned(), ficha.origem.descricao());
    metadata.insert(
        "lyrics_scope".to_owned(),
        ficha.origem.escopo().nome().to_owned(),
    );
    if let Some(obra) = obra_da_origem(&ficha.origem) {
        // O veredito sai **com a regra** que o produziu: quem lê a cifra sabe por
        // que ela é de domínio público, e em que jurisdição.
        let veredito = veredito_pd(obra, crate::lyrics::Jurisdicao::Brasil, ficha.ano_corrente);
        metadata.insert("work_status".to_owned(), veredito.resumo());
        if let Veredito::Indeterminado { motivo } = veredito {
            metadata.insert("work_note".to_owned(), motivo);
        }
    }
    match &ficha.fonograma {
        Some(fonograma) => {
            let mut descricao = fonograma.descricao.clone();
            if let Some(ano) = fonograma.ano {
                descricao = format!("{descricao}; {ano}");
            }
            if let Some(titular) = &fonograma.titular {
                descricao = format!("{descricao}; titular: {titular}");
            }
            metadata.insert("recording_source".to_owned(), descricao);
        }
        // Sem fonograma declarado, a cifra **diz que não sabe** — não omite.
        None => {
            metadata.insert(
                "recording_source".to_owned(),
                "não declarada (gravação não é obra: tem titular próprio)".to_owned(),
            );
        }
    }

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

/// A ficha da obra, quando a origem carrega uma.
fn obra_da_origem(origem: &Origem) -> Option<&crate::lyrics::Obra> {
    match origem {
        Origem::DominioPublico { obra } => Some(obra),
        Origem::ExtraidaPeloUsuario { obra, .. } => obra.as_ref(),
        Origem::DoUsuario { .. } => None,
    }
}

/// Diz se a cifra pode circular publicamente.
///
/// Cifra nascida de **extração pelo usuário** é privada do grupo (ADR-0012):
/// publicar é o que a licença coletiva do usuário cobre, e o app não substitui a
/// licença dele. Aqui é portão, não aviso: reprova.
///
/// # Errors
///
/// Erro com o motivo quando o escopo declarado é `grupo`.
pub fn pode_publicar(musica: &Song) -> Result<(), String> {
    let escopo = musica
        .metadata
        .get("lyrics_scope")
        .map_or("público", String::as_str);
    if escopo == "grupo" {
        let origem = musica
            .metadata
            .get("lyrics_source")
            .map_or("origem não declarada", String::as_str);
        return Err(format!(
            "cifra com escopo privado do grupo não vai para publicação ({origem}); \
             publicar depende da licença coletiva do usuário"
        ));
    }
    Ok(())
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
