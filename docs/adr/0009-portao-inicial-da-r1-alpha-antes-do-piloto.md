# ADR-0009 · Portão inicial da R1: alpha assistido antes do piloto de culto

- **Status:** aceito (2026-11-21), **emendado pela [ADR-0010](0010-piloto-automatizado-antes-do-uso-ao-vivo.md)** — o portão R1-D passa a ser piloto automatizado
- **Decisão do plano:** sequência da R1 (plano v2.6, §3 e §5)

## Contexto

A [ADR-0002](0002-primeira-release-cifra-e-projecao.md) fixa o portão de release da R1: *"uma igreja piloto usa no culto, offline, em pt-BR ou es-419"*. O ROADMAP traduziu isso em `R1-C · Piloto e release`, com quatro critérios: dois cultos consecutivos sem assistência do desenvolvedor, zero perda de dados/crash, traduções revisadas por nativos e MusicXML/ATEX em R1.1.

O problema é de **ordem de execução**, não de escopo. Dois cultos consecutivos sem assistência exigem uma igreja disponível, dois domingos de calendário e uma equipe disposta a ser cobaia — e o resultado só aparece depois. Se o produto ainda não tem leitor nem projeção, esse portão não mede nada: ele só cria uma fila de espera. Na prática, o único portão da R1 fica atrás de um compromisso externo, e qualquer ajuste de UI espera duas semanas para saber se funcionou.

Estado medido em 2026-11-21: o app tem **uma tela** (`HomeScreen`) que prova a ponte; não há leitor de cifras, projeção, exportação nem setlist. O portão caro está a três entregas de distância.

## Decisão

Separar o portão da R1 em dois, na ordem do barato para o caro:

1. **R1-C · Alpha utilizável (portão inicial):** o fluxo completo roda em **um ensaio assistido** — leitor aberto, cifra transposta, projeção na tela secundária, com o desenvolvedor presente. Critérios: fluxo completo executado, falhas registradas com diagnóstico, dados sobrevivem a fechar e reabrir o app.
2. **R1-D · Piloto e release (portão final):** os critérios da ADR-0002 permanecem intactos — dois cultos consecutivos sem assistência, zero perda de dados, traduções revisadas por nativos. Passa a ser o **último** marco da R1, depois do R1.1 (MusicXML/ATEX).

O portão de release da ADR-0002 **não muda**: continua sendo uso real em culto, offline. O que muda é o que vem antes dele.

## Consequências

- **Ganho:** o ciclo de feedback cai de dois domingos para um ensaio; o produto pode ser corrigido antes de chegar a um culto.
- **Ganho:** o piloto deixa de ser o primeiro contato com o produto — quando acontecer, leitor e projeção já terão sido exercitados.
- **Custo:** a R1 não prova uso real em culto até o R1-D. Um alpha em ensaio não substitui evidência de produção; o risco de falha em culto é **adiado, não eliminado**.
- **Dívida assumida:** o alpha depende de um ensaio disponível. Se não houver ensaio, o portão inicial escorrega — mas escorrega para um evento semanal, não para um domingo de culto.
- **Pressuposto de produto:** o alvo inicial é a equipe de mídia/louvor em ensaio, não a congregação em culto. A projeção pública continua sendo critério do R1-D.

## Referências

- [ADR-0002](0002-primeira-release-cifra-e-projecao.md) — portão de release da R1, inalterado.
- [ADR-0005](0005-escopo-da-r1.md) — rejeitada; MusicXML/ATEX seguem em R1.1 e não bloqueiam o piloto.
- [ROADMAP](../ROADMAP.md) — R1-C e R1-D.
- [plano-mestre-v2.6](../plans/plano-mestre-v2.6.md) — sequência executiva ajustada.
