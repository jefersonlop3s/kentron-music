# ADR-0005 · Escopo da R1: ChordPro primeiro; MusicXML/ATEX em R1.1

- **Status:** proposto (2026-11-20) — aguarda aceite do dono do projeto
- **Decisão do plano:** escopo da primeira release
- **Emenda:** substitui a cláusula de formatos da ADR-0002

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

## Consequências

- **Ganho:** o primeiro release público fica centrado no que a igreja piloto realmente usa; menos superfície de teste antes do piloto.
- **Ganho:** o esforço de MusicXML/ATEX só é comprometido depois de medição real (S5).
- **Custo:** a R1 não atende bandas que precisam de partitura ou tablatura desde o primeiro dia; elas esperam a R1.1.
- **Custo:** a ADR-0002 deixa de valer integralmente para formatos; o histórico precisa apontar para esta emenda.
- **Dívida assumida:** a decisão depende do S5. Se o spike mostrar custo baixo, a redução pode ser revertida com uma nova ADR.

## Referências

- ADR-0002 · Primeira release pública: cifra, palco local e projeção (R1).
- `docs/plans/plano-mestre-v2.6.md`, seção 3 (R1).
- `docs/PENDING.md`, seção R1 (spike S5).
- Implementação: `crates/notation_bundle/src/harmony.rs`, `crates/notation_bundle/src/chord_naming.rs` e testes `harmony_tests.rs`, `golden_harmony.rs`.
