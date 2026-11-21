# ADR-0010 · Piloto automatizado antes do uso ao vivo

- **Status:** aceito (2026-11-21) — decisão do dono do projeto
- **Emenda:** substitui o portão de release da [ADR-0002](0002-primeira-release-cifra-e-projecao.md) e o R1-D da [ADR-0009](0009-portao-inicial-da-r1-alpha-antes-do-piloto.md)

## Contexto

A [ADR-0002](0002-primeira-release-cifra-e-projecao.md) fixa o portão da R1 como *"uma igreja piloto usa no culto, offline"*; a [ADR-0009](0009-portao-inicial-da-r1-alpha-antes-do-piloto.md) manteve esse portão como R1-D, depois do R1.1.

Dois fatos medidos em 2026-11-21:

- O portão depende de **calendário de igreja** — dois domingos consecutivos. Cada verificação vira uma espera de duas semanas, e a release fica refém de um compromisso externo.
- O ambiente de prova **já existe e é automatizável**: há uma máquina Windows virtualizada com OBS Studio instalado, e a projeção é entregue por URL (Browser Source). Dá para carregar a projeção no OBS e verificar toque a toque sem um culto real.

## Decisão

O portão de release passa a ser **automatizado**; o uso ao vivo vira o passo seguinte, não o bloqueio:

1. **Piloto automatizado (portão):** em ambiente controlado (VM Windows + OBS), carregar a projeção pelo Browser Source e executar a bateria — abrir a cifra, transpor, projetar, avançar e voltar de seção, conferir que a projeção muda e que o app sobrevive a fechar/reabrir. Falha bloqueia a release.
2. **Duas rodadas consecutivas simuladas** substituem os "dois cultos" como *espera*.
3. **Uso ao vivo depois da release:** o teste em culto continua acontecendo, sem descartar os testes locais; o resultado realimenta a próxima release.

A faixa 1 de idiomas (pt-BR, en-US, es-419) permanece requisito, com a revisão por falantes nativos em paralelo — não como portão de calendário.

## Consequências

- **Ganho:** o feedback deixa de custar duas semanas; uma falha aparece antes de chegar ao culto, e o portão roda quando se quiser.
- **Custo:** automatizar o OBS exige preparar a VM (instalar o que faltar) e escrever o roteiro — trabalho que não existia.
- **Risco assumido:** o piloto automatizado prova o **fluxo**, não a operação humana sob pressão. Esse risco só o uso ao vivo cobre; por isso ele não foi descartado.
- **Escopo da prova:** "dois cultos" deixa de ser espera e passa a ser execução simulada em duas rodadas consecutivas na VM.

## Referências

- [ADR-0002](0002-primeira-release-cifra-e-projecao.md) — portão original, emendado aqui.
- [ADR-0009](0009-portao-inicial-da-r1-alpha-antes-do-piloto.md) — R1-D, emendado aqui.
- [ROADMAP](../ROADMAP.md) — R1-D.
- [PENDING](../PENDING.md) — roteiro do piloto automatizado.
