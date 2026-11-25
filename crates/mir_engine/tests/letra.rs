//! Prova da letra: corpus com procedência, sílaba, alinhamento ao áudio e a
//! cifra que sai pronta para o app desenhar.
//!
//! O áudio é **sintetizado** ("canto" de teste: rajadas de tom com formante na
//! banda da voz), então a posição esperada de cada linha é conhecida. Material
//! de terceiros não entra em fixture; a letra é do corpus de **domínio público**
//! embutido (ADR-0012).

use mir_engine::lyrics::{self, Escopo, Fonograma, Jurisdicao, Obra, Origem, Veredito};
use mir_engine::sheet::{self, Ficha};
use mir_engine::{Opts, analyze_com_letra};
use notation_bundle::chordpro::{ChordProParser, write};

mod common;
use common::cantar_quatro_frases;

#[test]
fn corpus_de_dominio_publico_tem_procedencia_e_recusa_o_que_nao_pode() {
    let letra = lyrics::do_corpus("amazing").expect("Amazing Grace está no corpus");
    assert_eq!(letra.linhas.len(), 4);
    match &letra.origem {
        Origem::DominioPublico { obra } => {
            assert_eq!(obra.autor, "John Newton");
            assert_eq!(obra.ano, 1779);
            // A ficha traz o que decide a regra — não só o título e o ano.
            assert_eq!(obra.autor_morte, Some(1807));
            assert!(!obra.base.is_empty());
        }
        outro => panic!("procedência errada: {outro:?}"),
    }
    // O hino que NÃO pode entrar (tradução inglesa de 1949, protegida) não
    // entrou — provado no sentido negativo.
    assert!(lyrics::do_corpus("how great").is_none());
    assert!(lyrics::do_corpus("great is thy").is_none());
    // Toda ficha do corpus resolve domínio público nas duas jurisdições.
    let relatorio = lyrics::auditar_corpus(2026).expect("corpus audita limpo");
    assert!(!relatorio.is_empty());
}

#[test]
fn ficha_incompleta_nao_vira_dominio_publico() {
    // Sem ano de falecimento, vida+70 não se calcula: `Indeterminado`. E
    // indeterminado **não** é domínio público — é o fail-loud da função.
    let sem_morte = Obra {
        titulo: "Hino antigo".to_owned(),
        autor: "Autor Desconhecido".to_owned(),
        ano: 1822,
        autor_morte: None,
        adaptador: None,
        adaptador_morte: None,
        obra_original: None,
        base: "Harpa Cristã".to_owned(),
    };
    let veredito = lyrics::veredito_pd(&sem_morte, Jurisdicao::Brasil, 2026);
    assert!(!veredito.eh_dominio_publico());
    assert!(matches!(veredito, Veredito::Indeterminado { .. }));
}

#[test]
fn obra_de_1822_com_gravacao_de_ontem_nao_vira_gravacao_livre() {
    // O caso do dono: letra de 1822, gravação de 2024. A **obra** é de domínio
    // público; a **gravação** não — é fonograma de quem gravou, e `Fonograma`
    // não tem veredito de domínio público de propósito.
    let obra = Obra {
        titulo: "Hino de 1822".to_owned(),
        autor: "Autor Falecido".to_owned(),
        ano: 1822,
        autor_morte: Some(1868),
        adaptador: None,
        adaptador_morte: None,
        obra_original: None,
        base: "edição de 1822".to_owned(),
    };
    assert_eq!(
        lyrics::veredito_pd(&obra, Jurisdicao::Brasil, 2026),
        Veredito::DominioPublico {
            regra: "vida de Autor Falecido (1868) + 70 anos".to_owned()
        }
    );
    assert_eq!(
        lyrics::veredito_pd(&obra, Jurisdicao::Eua, 2026),
        Veredito::DominioPublico {
            regra: "publicado em 1822 (antes de 1930)".to_owned()
        }
    );

    let gravacao = Fonograma {
        descricao: "regravação publicada em vídeo".to_owned(),
        ano: Some(2024),
        titular: Some("canal que gravou".to_owned()),
    };
    assert_eq!(gravacao.ano, Some(2024));
    assert!(gravacao.titular.is_some());

    // Tradução recente da mesma obra: outra obra, protegida no Brasil.
    let traduzida = Obra {
        adaptador: Some("Adaptador Recente".to_owned()),
        adaptador_morte: Some(1995),
        obra_original: Some("Hino de 1822".to_owned()),
        ..obra
    };
    assert!(matches!(
        lyrics::veredito_pd(&traduzida, Jurisdicao::Brasil, 2026),
        Veredito::Protegido { .. }
    ));
    // E nos EUA a tradução é Indeterminada: falta o ano de publicação dela.
    assert!(matches!(
        lyrics::veredito_pd(&traduzida, Jurisdicao::Eua, 2026),
        Veredito::Indeterminado { .. }
    ));
}

#[test]
fn extracao_pelo_usuario_nasce_privada_e_nao_publica() {
    let origem = Origem::ExtraidaPeloUsuario {
        responsavel: "o próprio músico".to_owned(),
        obra: None,
    };
    assert_eq!(origem.escopo(), Escopo::Grupo);
    assert!(origem.descricao().contains("circulação privada"));

    let outro = Origem::DoUsuario { licenca: None };
    assert_eq!(outro.escopo(), Escopo::Publico);
    assert_eq!(
        Origem::DoUsuario {
            licenca: Some("licença coletiva da igreja".to_owned())
        }
        .descricao(),
        "fornecida pelo usuário; licença declarada: licença coletiva da igreja"
    );
}

#[test]
fn adaptacao_com_frases_novas_e_obra_propria_e_nao_herda_o_veredito() {
    // Caso do dono: "Maravilhosa Graça" (obra antiga) e a versão "Tremenda
    // Graça" — mesma música, mesmo sentido, mas **título e frases mudam**.
    // A original é de domínio público; a adaptação é obra própria, e o veredito
    // de uma **não empresta** para a outra.
    let original = Obra {
        titulo: "Maravilhosa Graça".to_owned(),
        autor: "Autor Antigo".to_owned(),
        ano: 1779,
        autor_morte: Some(1807),
        adaptador: None,
        adaptador_morte: None,
        obra_original: None,
        base: "edição de domínio público".to_owned(),
    };
    assert!(
        lyrics::veredito_pd(&original, Jurisdicao::Brasil, 2026).eh_dominio_publico(),
        "a obra antiga é de domínio público"
    );

    let adaptacao = Obra {
        titulo: "Tremenda Graça".to_owned(),
        autor: "Autor Antigo".to_owned(),
        ano: 1779,
        autor_morte: Some(1807),
        adaptador: Some("Adaptador da equipe".to_owned()),
        adaptador_morte: Some(2015),
        obra_original: Some("Maravilhosa Graça".to_owned()),
        base: "versão cantada pela equipe".to_owned(),
    };
    let veredito = lyrics::veredito_pd(&adaptacao, Jurisdicao::Brasil, 2026);
    assert!(
        matches!(veredito, Veredito::Protegido { .. }),
        "a adaptação recente é protegida: {veredito:?}"
    );
    let regra = veredito.resumo();
    assert!(
        regra.contains("Adaptador da equipe"),
        "a regra julga pelo adaptador: {regra}"
    );
    // E a ficha guarda o vínculo — sem fundir as duas obras numa só.
    assert_eq!(
        adaptacao.obra_original.as_deref(),
        Some("Maravilhosa Graça")
    );
    assert_ne!(adaptacao.titulo, original.titulo);
}

#[test]
fn obra_recente_nao_passa_por_dominio_publico() {
    let recente = Obra {
        titulo: "Obra recente".to_owned(),
        autor: "Autor Vivo".to_owned(),
        ano: 1980,
        autor_morte: Some(2010),
        adaptador: None,
        adaptador_morte: None,
        obra_original: None,
        base: "edição recente".to_owned(),
    };
    assert!(matches!(
        lyrics::veredito_pd(&recente, Jurisdicao::Brasil, 2026),
        Veredito::Protegido { .. }
    ));
}

#[test]
fn silaba_recua_so_o_ataque_que_existe() {
    // "San-to" e não "Sa-nto": o `nt` não é ataque válido, o `t` é.
    assert_eq!(lyrics::silabas("Santo, santo"), vec![0, 3, 7, 10]);
    // Ataque de duas consoantes com líquida fica inteiro: "Bra-vo". A regra
    // ingênua (recuar tudo) daria [1, 3], com o `r` na sílaba errada.
    assert_eq!(lyrics::silabas("Bravo"), vec![0, 3]);
    // "ch" também é ataque válido; consoante isolada também ("gra-ça", "chu-va").
    assert_eq!(lyrics::silabas("Chuva"), vec![0, 3]);
    assert_eq!(lyrics::silabas("Graca"), vec![0, 3]);
}

#[test]
fn letra_do_usuario_perde_acorde_e_metadado() {
    let letra =
        lyrics::ler_do_usuario("{title: X}\n[C]Santo, [G]santo\n# comentário\n\n[Verse]\nDeus\n");
    assert_eq!(letra.linhas, vec!["Santo, santo", "Deus"]);
    assert_eq!(letra.origem, Origem::DoUsuario { licenca: None });
}

#[test]
fn alinha_a_letra_as_frases_e_poe_o_acorde_na_silaba() {
    let audio = cantar_quatro_frases();
    let letra = lyrics::do_corpus("amazing").expect("corpus");
    // O áudio de prova tem 22 s: o mínimo de trecho do default (15 s) é para
    // música, então aqui a escala é a da fixture.
    let opts = Opts {
        trecho_minimo_s: 4.0,
        ..Opts::default()
    };
    let (relatorio, alinhadas) = analyze_com_letra(&audio, &opts, Some(&letra), None, None);

    // 1. As quatro linhas cantadas cabem no áudio.
    assert_eq!(
        alinhadas.len(),
        4,
        "linhas: {:?}",
        alinhadas.iter().map(|l| &l.texto).collect::<Vec<_>>()
    );

    // 2. Cada linha cai em ordem, sem sobreposição, e a primeira não se atrasa.
    for par in alinhadas.windows(2) {
        assert!(
            par[0].fim_s <= par[1].inicio_s + 0.01,
            "linhas fora de ordem: {:?} e {:?}",
            par[0].texto,
            par[1].texto
        );
    }
    assert!(
        alinhadas[0].inicio_s < 1.0,
        "primeira linha tarde demais: {}",
        alinhadas[0].inicio_s
    );

    // 3. A confiança é útil e não é propaganda: fica na faixa média, não em 1.
    let media = alinhadas.iter().map(|l| l.confianca).sum::<f32>() / alinhadas.len() as f32;
    assert!(
        (0.3..=0.95).contains(&media),
        "confiança média fora do razoável: {media}"
    );

    // 4. A cifra sai como objeto do domínio e volta pelo parser do app.
    let ficha = Ficha {
        titulo: "prova".to_owned(),
        origem: letra.origem.clone(),
        fonograma: None,
        ano_corrente: 2026,
    };
    let musica = sheet::montar(&relatorio, &alinhadas, &ficha);
    // Origem de corpus público circula; e a ficha diz por quê, com a regra.
    assert!(sheet::pode_publicar(&musica).is_ok());
    let veredito_na_cifra = musica
        .metadata
        .get("work_status")
        .expect("a cifra carrega o veredito da obra");
    assert!(
        veredito_na_cifra.contains("domínio público"),
        "{veredito_na_cifra}"
    );
    assert!(veredito_na_cifra.contains("1807"), "{veredito_na_cifra}");
    assert_eq!(
        musica.metadata.get("lyrics_scope").map(String::as_str),
        Some("público")
    );
    let texto = write(&musica);
    let devolta = ChordProParser::parse(&texto).expect("o app precisa entender o que saímos");
    assert_eq!(devolta, musica, "ida e volta de ChordPro mudou a música");

    // 5. A letra sobrevive inteira. Nos acordes inline o texto é **interrompido
    //    pelos colchetes** (`[F]Ama[A5]zing`), então a conferência é no texto que
    //    o parser separa — exatamente o que o app desenha. Buscar a frase
    //    contígua no arquivo bruto seria procurar o que a cifra nunca contém.
    let textos: Vec<String> = musica
        .sections
        .iter()
        .flat_map(|s| &s.lines)
        .map(|l| l.text.clone())
        .collect();
    assert_eq!(
        textos.len(),
        letra.linhas.len(),
        "linha de letra perdida na cifra: {textos:?}"
    );
    assert!(
        textos
            .iter()
            .any(|t| t == "Amazing grace! how sweet the sound"),
        "a letra não sobreviveu à cifra: {textos:?}"
    );
    // Seções diferentes para trechos diferentes: o mapeamento linha→trecho vale.
    assert!(
        musica.sections.len() >= 2,
        "a cifra ficou com uma seção só: {:?}",
        musica.sections.iter().map(|s| &s.name).collect::<Vec<_>>()
    );
    for secao in &musica.sections {
        for linha in &secao.lines {
            let limite = linha.text.chars().count();
            for acorde in &linha.chords {
                assert!(
                    acorde.position <= limite,
                    "acorde fora da linha: {:?} em {:?}",
                    acorde.chord,
                    linha.text
                );
            }
        }
    }

    // 6. O primeiro acorde da primeira linha com letra cai no começo dela — é o
    //    que o áudio mostra (acorde e voz entram juntos).
    let linha_com_letra = musica
        .sections
        .iter()
        .flat_map(|s| &s.lines)
        .find(|l| !l.text.is_empty())
        .expect("há linha com letra");
    assert_eq!(
        linha_com_letra.chords.first().map(|c| c.position),
        Some(0),
        "acorde não caiu no começo de {:?}",
        linha_com_letra.text
    );
    assert!(
        relatorio.chords.len() >= 3,
        "o detector de acorde não achou a progressão: {}",
        relatorio.chords.len()
    );
}
