//! Prova da letra: corpus com procedência, sílaba, alinhamento ao áudio e a
//! cifra que sai pronta para o app desenhar.
//!
//! O áudio é **sintetizado** ("canto" de teste: rajadas de tom com formante na
//! banda da voz), então a posição esperada de cada linha é conhecida. Material
//! de terceiros não entra em fixture; a letra é do corpus de **domínio público**
//! embutido (ADR-0012).

use mir_engine::lyrics::{self, Origem};
use mir_engine::sheet;
use mir_engine::{Opts, analyze_com_letra};
use notation_bundle::chordpro::{ChordProParser, write};

mod common;
use common::cantar_quatro_frases;

#[test]
fn corpus_de_dominio_publico_tem_procedencia_e_recusa_o_que_nao_pode() {
    let letra = lyrics::do_corpus("amazing").expect("Amazing Grace está no corpus");
    assert_eq!(letra.linhas.len(), 4);
    match &letra.origem {
        Origem::DominioPublico { autor, ano, .. } => {
            assert_eq!(autor, "John Newton");
            assert_eq!(*ano, 1779);
        }
        outro => panic!("procedência errada: {outro:?}"),
    }
    // O hino que NÃO pode entrar (tradução inglesa de 1949, protegida) não
    // entrou — provado no sentido negativo.
    assert!(lyrics::do_corpus("how great").is_none());
    assert!(lyrics::do_corpus("great is thy").is_none());
    // Todo título do corpus tem autor e ano anteriores a 1900.
    for (titulo, autor, ano) in lyrics::titulos_do_corpus() {
        assert!(
            ano < 1900,
            "{titulo} ({autor}) com ano {ano} não é pré-1900"
        );
        assert!(!autor.is_empty());
    }
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
    assert_eq!(letra.origem, Origem::DoUsuario);
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
    let (relatorio, alinhadas) = analyze_com_letra(&audio, &opts, Some(&letra));

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
    let musica = sheet::montar(&relatorio, &alinhadas, "prova", &letra.origem.descricao());
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
