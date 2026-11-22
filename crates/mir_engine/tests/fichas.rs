//! Prova da ficha do hinário: identidade, ambiguidade, derivação pelo ano e
//! auditoria contra a regra única (`veredito_pd`).
//!
//! O asset é o embutido — o mesmo que o app usa e que o gate `fichas` valida no
//! push. Nenhum material de terceiros entra: a ficha guarda fato, não letra.

use mir_engine::fichas::{self, RespostaLetra, VereditoFicha};
use mir_engine::lyrics::{self, Jurisdicao, Veredito};

const ANO: u16 = 2026;

#[test]
fn asset_embutido_tem_as_doze_fichas_e_o_placar_do_repertorio() {
    let todas = fichas::fichas();
    assert_eq!(todas.len(), 12);

    let contar = |alvo: VereditoFicha| todas.iter().filter(|f| f.veredito == alvo).count();
    assert_eq!(contar(VereditoFicha::DominioPublico), 8);
    assert_eq!(contar(VereditoFicha::Protegido), 3);
    assert_eq!(contar(VereditoFicha::Gap), 1);

    // O repertório do dono são 10; as outras duas fichas são do mesmo livro.
    assert_eq!(todas.iter().filter(|f| f.no_repertorio).count(), 10);
}

#[test]
fn identidade_e_o_par_e_o_numero_aceita_zero_a_esquerda() {
    let ficha = fichas::por_identidade("Cantor Cristão", "09").expect("09 do Cantor Cristão");
    assert_eq!(ficha.titulo, "Santo, Santo, Santo");
    assert_eq!(ficha.ano_dominio_publico, Some(2018));

    // Mesma ficha por nome parcial, sem acento e com o número sem zero:
    // quem consulta digita o nome que usa, não a linha do cabeçalho.
    let outra = fichas::por_identidade("cantor cristao", "9").expect("mesma ficha");
    assert_eq!(outra.numero, "09");

    // O 291 do Cantor Cristão não é o 291 da Harpa: o par é que identifica.
    assert!(fichas::por_identidade("Harpa Cristã", "09").is_none());
}

#[test]
fn titulo_ambiguo_vira_recusa_e_nao_veredito() {
    // O livro tem TRÊS títulos que começam com "Louvor": 03 "Louvor ao Senhor",
    // 135 "Louvor (Louvamos-te, ó Deus)" e 385 "Louvor". Dois deles estão no
    // repertório do dono, com vereditos opostos.
    let candidatas = fichas::por_titulo("Louvor");
    assert_eq!(candidatas.len(), 3, "três hinos que começam com Louvor");

    match fichas::resposta_por_titulo("Louvor", ANO) {
        RespostaLetra::Ambiguo { candidatas } => {
            assert_eq!(candidatas.len(), 3);
            let texto = candidatas.join(" ");
            for numero in ["03", "135", "385"] {
                assert!(texto.contains(numero), "cita o {numero}: {texto}");
            }
        }
        outra => panic!("esperava Ambiguo, veio {outra:?}"),
    }
}

#[test]
fn resposta_deriva_do_ano_e_nao_da_coluna_veredito() {
    let santo = fichas::por_identidade("Cantor Cristão", "09").expect("09");
    assert_eq!(santo.veredito, VereditoFicha::DominioPublico);
    assert!(
        matches!(
            fichas::resposta_para_letra(santo, 2017),
            RespostaLetra::RequerLetraDoUsuario { .. }
        ),
        "em 2017 a tradução de 1947 ainda estava protegida"
    );
    assert!(
        matches!(
            fichas::resposta_para_letra(santo, 2018),
            RespostaLetra::Livre { .. }
        ),
        "caiu em 2018, sem ninguém editar a ficha"
    );

    let cruz = fichas::por_identidade("Harpa Cristã", "291").expect("291");
    assert_eq!(cruz.ano_dominio_publico, Some(2040));
    assert!(matches!(
        fichas::resposta_para_letra(cruz, 2039),
        RespostaLetra::RequerLetraDoUsuario { .. }
    ));
    assert!(matches!(
        fichas::resposta_para_letra(cruz, 2040),
        RespostaLetra::Livre { .. }
    ));
}

#[test]
fn gap_nao_vira_livre_nem_protegido() {
    let substituicao = fichas::por_identidade("Cantor Cristão", "92").expect("92");
    assert_eq!(substituicao.veredito, VereditoFicha::Gap);
    assert_eq!(substituicao.ano_dominio_publico, None);
    match fichas::resposta_para_letra(substituicao, ANO) {
        RespostaLetra::Gap { motivo } => {
            assert!(
                motivo.to_lowercase().contains("falecimento"),
                "o motivo nomeia o dado que falta: {motivo}"
            );
        }
        outra => panic!("gap não pode virar {outra:?}"),
    }
}

#[test]
fn sem_ficha_e_resposta_explicita() {
    assert_eq!(
        fichas::resposta_por_identidade("Harpa Cristã", "999", ANO),
        RespostaLetra::SemFicha
    );
    assert_eq!(
        fichas::resposta_por_titulo("Obra que não existe", ANO),
        RespostaLetra::SemFicha
    );
}

#[test]
fn asset_malformado_falha_alto_em_vez_de_ignorar_linha() {
    let cabecalho = "hinario\tnumero\ttitulo\tautoria\ttraducao\tmelodia\tobra_original\tano_relevante\tveredito\tano_dominio_publico\tfonte_autoria\tfonte_traducao\tfonte_melodia\tno_repertorio\tobservacao";
    let torto = format!("{cabecalho}\nCantor Cristão\t09\n");
    let erro = fichas::carregar_do_texto(&torto).expect_err("linha torta tem de falhar");
    assert!(erro.contains("linha 2"), "o erro nomeia a linha: {erro}");
    assert!(erro.contains("2 campos"), "o erro diz o que houve: {erro}");
}

#[test]
fn auditoria_prova_a_fronteira_de_cada_linha_pela_regra_unica() {
    let aprovadas = fichas::auditar().expect("auditoria da ficha");
    assert_eq!(aprovadas.len(), fichas::fichas().len());
    for linha in &aprovadas {
        assert!(
            linha.contains("+71") || linha.contains("gap"),
            "linha auditada sem prova: {linha}"
        );
    }
}

#[test]
fn obra_da_ficha_usa_a_traducao_como_camada_que_manda() {
    let vencendo = fichas::por_identidade("Cantor Cristão", "112").expect("112");
    let obra = fichas::obra_da_ficha(vencendo);
    assert_eq!(obra.autor, "Julia Ward Howe");
    assert_eq!(obra.adaptador.as_deref(), Some("Ricardo Pitrowsky"));
    assert_eq!(obra.adaptador_morte, Some(1965));

    let em_2035 = lyrics::veredito_pd(&obra, Jurisdicao::Brasil, 2035);
    assert!(
        matches!(em_2035, Veredito::Protegido { .. }),
        "2035: {em_2035:?}"
    );
    let em_2036 = lyrics::veredito_pd(&obra, Jurisdicao::Brasil, 2036);
    assert!(em_2036.eh_dominio_publico(), "2036: {em_2036:?}");
}

#[test]
fn vinculo_resolve_a_obra_original_no_corpus() {
    let santo = fichas::por_identidade("Cantor Cristão", "09").expect("ficha do 09");
    let original = fichas::obra_original_no_corpus(santo).expect("Holy, Holy, Holy no corpus");
    assert_eq!(original.titulo, "Holy, Holy, Holy");
    assert_eq!(original.autor, "Reginald Heber");

    // O vínculo é de referência: a obra original responde pelo veredito dela.
    let veredito = mir_engine::uso::veredito_por_titulo(&original.titulo, ANO)
        .expect("o corpus responde pelo título original");
    assert!(
        veredito.eh_dominio_publico(),
        "Holy, Holy, Holy é domínio público: {veredito:?}"
    );
}

#[test]
fn ficha_sem_vinculo_nao_resolve_obra_original() {
    let louvamos = fichas::por_identidade("Cantor Cristão", "135").expect("ficha do 135");
    assert!(fichas::obra_original_no_corpus(louvamos).is_none());
}

#[test]
fn obra_da_ficha_carrega_o_vinculo_declarado() {
    let santo = fichas::por_identidade("Cantor Cristão", "09").expect("ficha do 09");
    let obra = fichas::obra_da_ficha(santo);
    assert_eq!(obra.obra_original.as_deref(), Some("Holy, Holy, Holy"));
    // A camada que manda continua sendo a tradução (Rocha), não o original (Heber).
    assert_eq!(obra.adaptador.as_deref(), Some("João Gomes da Rocha"));
}

#[test]
fn relatorio_de_uso_recusa_escolher_qual_louvor() {
    let veredito = mir_engine::uso::veredito_por_titulo("Louvor", ANO).expect("tem ficha");
    match veredito {
        Veredito::Indeterminado { motivo } => {
            assert!(motivo.contains("ambíguo"), "motivo: {motivo}");
            assert!(
                motivo.contains("385") && motivo.contains("135"),
                "cita as duas candidatas: {motivo}"
            );
        }
        outro => panic!("título ambíguo não pode virar {outro:?}"),
    }
}
