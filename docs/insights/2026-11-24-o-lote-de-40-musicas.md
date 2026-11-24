# O lote de 40 músicas: o que o motor vê num repertório que não é o meu (2026-11-24)

**Origem:** o dono pediu para rodar o motor em `~/Music/MP3` — 40 faixas, 242
minutos de gospel/worship, quase todo de banda — e medir tudo.
**Uso aqui:** é o **primeiro teste em corpus real**, e ele **pegou um erro que o
gabarito não pegou**.

## O corpus

```
40 faixas · 242 minutos · mediana de 5,2 min por faixa
andamento: 70 a 176 BPM, mediana 112
tons: 17 distintos — D (9), A (6), G (5), C (3), E (3), Cm (3), ...
por faixa: mediana de 15 seções e 122 trechos de acorde
```

## O que o lote expôs

**28 das 40 músicas foram lidas como `6/8`.** Num repertório de gospel — que é
quase todo `4/4` — isso é implausível, e o número é a denúncia.

**A causa, e ela é de arquitetura:** a medida da percussão responde *"a caixa cai
na metade do ciclo do bumbo?"*. Isso é verdade em **`6/8`** (dois grupos de três)
**e em `4/4`** (dois grupos de dois). Num `4/4` com bumbo em 1 e 3 e caixa em 2 e
4, a caixa cai exatamente na metade do ciclo do bumbo — **indistinguível do `6/8`**
por esta medida.

**E o gabarito não pegou porque não tinha nenhum `4/4`.** As nove músicas do
David Bennett são `3/4` ou `6/8` — a ambiguidade `6/8` × `4/4` nunca foi
exercitada. **O corpus exercitou.**

## O que a medida realmente separa

| o que a percussão mede | resposta |
|---|---|
| **ternário** (caixa nos terços) | **`3/4`** — decidido, e acerta os três do gabarito |
| **binário** (caixa na metade) | **`6/8` ou `4/4`** — **não separa** |

**O que faltaria** para separar os dois: a **subdivisão por batida** — três
colcheias por tempo (`6/8`) ou duas (`4/4`). Tentei medir pela taxa de ataques na
banda do chimbau e **não funcionou**: a "batida" medida sai contaminada pelo
decaimento do bumbo (0,50 a 1,41 de razão, todos inconclusivos).

## O conserto

`3/4` quando os terços aparecem; **`6/8` ou `4/4` oferecidos** quando a metade
aparece. O motor **não afirma o que não separa** — e o placar honesto do gabarito
continua **6 de 6**: três decididos, três oferecidos com o certo entre as opções.

## A lição

**Gabarito escolhido por quem construiu não cobre o que ele não pensou em testar.**
O conjunto do Bennett tinha nove músicas e nenhuma `4/4` — e foi o corpus de outra
pessoa que mostrou o exagero. É a segunda vez no mesmo dia que um conjunto de fora
corrige um número que parecia bom: o `5 de 6` era viés de seleção, e o `6 de 6`
era **viés de cobertura**.

**Um placar de 100% num conjunto de nove não é 100% no mundo.** O tamanho importa
menos que a **variedade**: nove músicas que cobrem os casos que existem valem mais
que nove que cobrem só os que eu imaginei.