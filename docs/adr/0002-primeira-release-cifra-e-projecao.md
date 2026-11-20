# ADR-0002 · Primeira release pública: cifra, palco local e projeção (R1)

- **Status:** aceita (2026-11-18)
- **Decisão do plano:** D2

## Contexto

O motor de análise de áudio (MIR) é o diferencial do produto, mas é a parte de maior risco técnico (tempo e memória no celular, licenças de pesos, qualidade dos stems) e jurídico. Já a leitura de cifras com transposição, setlist e exportação limpa de letras para o telão resolve uma dor imediata das equipes de mídia e não depende de rede nem de IA.

## Decisão

A primeira release pública é o **R1**: leitor `.cho` / `.musicxml` / `.atex`, transposição, notação em 4 sistemas, setlist local, estilos, exportação OpenLyrics/Holyrics e Browser Source do OBS (controle manual), idiomas da faixa 1 (pt-BR, en-US, es-419). Sem rede.

O MIR entra como **experimento no desktop** assim que o spike S1 (MIR no aparelho) passar, sem bloquear o R1.

## Consequências

- Portão do R1: uma igreja piloto usa no culto, offline, em pt-BR ou es-419.
- O teste fechado da Google Play (12 testadores por 14 dias) deve começar durante o R1.
- Rede (Matrix), palco ao vivo (`stage_clock`) e MIR ficam para R2, R3 e R4.
