# ADR-0005 · Escopo da R1: ChordPro primeiro; MusicXML/ATEX em R1.1

- **Status:** **rejeitado** (2026-11-20) — decisão do dono do projeto; a ADR-0002 permanece integral
- **Decisão do plano:** escopo da primeira release
- **Emenda:** nenhuma — a cláusula de formatos da ADR-0002 não foi emendada
- **Não vigente:** as seções Contexto, Decisão e Consequências registram a proposta como foi submetida. O motivo da rejeição e o que vale no lugar estão na seção Rejeição.

## Contexto

A ADR-0002 (aceita em 2026-11-18) define a R1 como leitor `.cho` / `.musicxml` / `.atex`, com transposição, quatro sistemas de notação, setlist local, estilos, exportação OpenLyrics/Holyrics e Browser Source do OBS. O portão de aceite da mesma ADR é mais estreito: "uma igreja piloto usa no culto, offline, em pt-BR ou es-419".

O plano de execução v2.6 já registra MusicXML e ATEX em **R1.1**, mas a ADR-0002 continua listando os três formatos na R1. Isso é uma contradição documental: o plano e a ADR não podem estar ambos corretos.

Achados concretos que sustentam a revisão:

- O parser ChordPro está implementado e com testes dourados na `main`; a transposição e a renderização nos quatro sistemas estão implementadas e cobertas por 29 testes.
- MusicXML e ATEX não são necessários para o portão do piloto: uma equipe de louvor projeta cifra e letra, não partitura orquestral.
- A ADR-0002 lista o teste fechado da Google Play (12 testadores por 14 dias) como consequência do R1. Esse requisito vale para contas pessoais criadas após 13/11/2023 — é uma condição a verificar, não um bloqueio universal.
- O spike S5 (renderização de partituras/tabs) ainda não foi executado. Sem ele, o esforço de MusicXML/ATEX é estimativa, não medição.

## Decisão

A R1 pública cobre **ChordPro** (`.cho`), transposição, quatro sistemas de notação, setlist local, estilos, exportação OpenLyrics/Holyrics e Browser Source do OBS com controle manual, nos idiomas da faixa 1. **MusicXML e ATEX passam para R1.1**, condicionados ao spike S5.

O portão do piloto não muda: dois cultos consecutivos, offline, sem assistência do desenvolvedor, sem perda de dados.

Se o dono do projeto rejeitar esta proposta, a alternativa é manter a ADR-0002 integralmente: MusicXML e ATEX voltam à R1 e o piloto público aguarda o S5. Nesse caso, a linha correspondente do plano v2.6 deve ser removida.

> **Rejeitada.** A instrução acima não se aplica: a linha do plano v2.6 permanece como ordem de execução, e o piloto não aguarda o S5 — ver Rejeição, abaixo.

## Consequências

- **Ganho:** o primeiro release público fica centrado no que a igreja piloto realmente usa; menos superfície de teste antes do piloto.
- **Ganho:** o esforço de MusicXML/ATEX só é comprometido depois de medição real (S5).
- **Custo:** a R1 não atende bandas que precisam de partitura ou tablatura desde o primeiro dia; elas esperam a R1.1.
- **Custo:** a ADR-0002 deixa de valer integralmente para formatos; o histórico precisa apontar para esta emenda.
- **Dívida assumida:** a decisão depende do S5. Se o spike mostrar custo baixo, a redução pode ser revertida com uma nova ADR.

## Rejeição

**Rejeitada pelo dono do projeto em 2026-11-20.** Três motivos, medidos na sessão que a propôs:

1. **Tratava ordem de execução como corte de escopo.** O plano v2.6 já sequenciava MusicXML/ATEX depois da cifra. Formalizar isso como emenda de uma ADR já aceita criou a contradição documental que a própria proposta dizia estar resolvendo — em vez de resolvê-la.
2. **A cláusula de fallback era inferência do autor, não da ADR-0002.** A ADR-0002 fixa o portão do piloto como "uma igreja piloto usa no culto, offline, em pt-BR ou es-419". Ela não condiciona o piloto ao S5. A frase "o piloto público aguarda o S5" foi escrita nesta proposta e não tem lastro na ADR que ela pretendia emendar.
3. **Escopo de release aceita não se reduz por emenda unilateral.** O custo cai em quem planeja (bandas que precisam de partitura no primeiro dia) e cria precedente de reescrita de escopo sem nova decisão de produto.

### O que vale no lugar

- **ADR-0002 integral:** a R1 é o release com `.cho` / `.musicxml` / `.atex`, transposição, quatro sistemas, setlist local, estilos, exportação OpenLyrics/Holyrics e Browser Source do OBS (manual). Portão: uma igreja piloto usa no culto, offline, em pt-BR ou es-419.
- **A sequência continua, como sequência:** cifra primeiro, MusicXML/ATEX depois, no marco R1.1 do plano v2.6. É ordem de trabalho, não escopo de release.
- **O S5 mede esforço, não bloqueia o piloto:** ele dimensiona MusicXML/ATEX. O piloto da R1 não depende dele.
- **Reintroduzir a redução exige ADR nova** com decisão do dono. Esta fica registrada como rejeitada para que nenhuma sessão futura a proponha como melhoria óbvia.

## Referências

- ADR-0002 · Primeira release pública: cifra, palco local e projeção (R1).
- [Plano de execução v2.6](../plans/plano-mestre-v2.6.md), seção 3 (R1).
- [Pendências](../PENDING.md), seção R1 (spike S5).
- Implementação: `crates/notation_bundle/src/harmony.rs`, `crates/notation_bundle/src/chord_naming.rs` e testes `harmony_tests.rs`, `golden_harmony.rs`.
