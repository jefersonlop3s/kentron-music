# O lote de 40 músicas: o que o motor vê num repertório que não é o meu (2026-11-24)

**Origem:** o dono pediu para rodar o motor em `~/Music/MP3` — 40 faixas, 242
minutos de gospel/worship, quase todo de banda — e medir tudo.
**Uso aqui:** é o **primeiro teste em corpus real**, e ele **pegou dois erros que o
gabarito não pegou**: um exagero meu, e o limite da medida.

## O corpus

```
40 faixas · 242 minutos · mediana de 5,2 min por faixa
andamento: 70 a 176 BPM, mediana 112
tons: 17 distintos — D (9), A (6), G (5), C (3), E (3), Cm (3), ...
por faixa: mediana de 15 seções e 122 trechos de acorde
áudio: 38 em 48 kHz e 2 em 44,1 kHz · LUFS mediano −13,0 · LRA mediano 5,4 LU
```

## Erro 1 — o exagero, e ele é de arquitetura

**28 das 40 músicas foram lidas como `6/8`.** Num repertório de gospel — quase todo
`4/4` — isso é implausível, e **o número implausível é a denúncia**.

A medida da percussão responde *"a caixa cai na metade do ciclo do bumbo?"*. Isso é
verdade em **`6/8`** (dois grupos de três) **e em `4/4`** (dois grupos de dois). Num
`4/4` com bumbo em 1 e 3 e caixa em 2 e 4, a caixa cai exatamente na metade do ciclo
do bumbo — **indistinguível do `6/8`** por esta medida.

**E o gabarito não pegou porque não tinha nenhum `4/4`.** As nove músicas do David
Bennett são `3/4` ou `6/8`; a ambiguidade `6/8` × `4/4` nunca foi exercitada.

**Corrigido:** três terços → `3/4` decidido; duas metades → **`6/8` ou `4/4`
oferecidos**. Distribuição corrigida no corpus: **34 oferecidas e 6 decididas**.

## Erro 2 — o limite da medida, e ele é maior

**A "barra" mediana saiu 5,08 s** — a 112 BPM isso seriam nove tempos. E **17 das
40** ficaram acima de 6 s, com nove exatamente em 7,0 s (`3,5 × 2`: a correção de
ciclo empilhada). O ciclo está **saturando no teto** do estimador.

**Hipótese testada e confirmada:** o módulo `percussao` **pressupõe o stem de
bateria** — e o corpus foi analisado **nos MP3 crus, que são o mix**. Mas separar
os stems **não resolveu**: em oito faixas do corpus, com o stem de bateria, as
barras continuaram implausíveis (**5,30 s · 7,10 s · 5,06 s · 5,64 s**) e as
leituras ficaram em 4 oferecidas, 3 `3/4` e 1 `4/4`.

**Conclusão honesta: a medida é validada no gabarito e NÃO generaliza para o
corpus.** O gabarito é material guiado por bateria, e foi escolhido por mim; o
corpus é gospel com bateria discreta, e foi escolhido pelo dono.

## O que o corpus estabeleceu

| | gabarito (9 músicas, sem `4/4`) | corpus (40 músicas) |
|---|---|---|
| leitura da percussão | **6 de 6** (3 decididas, 3 oferecidas) | 6 decididas de 40 |
| ciclo (barra) | plausível (0,9 a 2,5 s) | **satura** (mediana 5,08 s) |
| `fonte` da leitura | percussão | percussão nas 40 |

**A medida separa ternário de binário quando o bumbo é detectável.** No corpus ele
não é — a mesma parede de sempre: **o ataque da bateria em mixagem discreta**.

## As duas propostas de conserto — a primeira não fecha, a segunda fechou

### Proposta 1 · a subdivisão interna separa `4/4` de `6/8` (`T/4` contra `T/6`)

**A tese:** em `4/4` os pulsos internos caem em `T/4` e `3T/4`; em `6/8` caem em
`1/6`, `2/6`, `4/6` e `5/6`, e `T/4` é vazio.

**Medido em oito músicas rotuladas (cinco `6/8`, três `4/4`):**

| variante | acertos |
|---|---|
| energia da banda média/aguda nas posições | **3 de 8** |
| densidade de ataques do chimbau, fase ancorada no bumbo | 5 de 8 |
| idem, com supressão da coincidência com o bumbo | **3 de 8** |

**Não separa.** Dois defeitos medidos: a banda média/aguda é **contínua** (violão,
voz, prato), então a energia em `T/4` é alta para tudo; e os ataques do chimbau
**não se alinham ao ciclo** — o histograma sai espalhado, com a massa empilhada
logo depois do bumbo (o ataque dele vaza, o mesmo defeito já corrigido na
`percussao`).

### Proposta 2 · o teto fisiológico e a âncora harmônica

**A tese:** nenhuma barra real entre 70 e 176 BPM dura mais de 3,60 s; o que passa
disso é o **hipercompasso**, e o ritmo harmônico diz a escala.

**A parte da escala está confirmada e medida** — e é o achado mais forte do lote:

```
barra mediana (percussao)   5,08 s
troca de acorde mediana     1,90 s     <- a escala real
razao                       2,16 a 5,19
fator de dobra necessario   2 em 24 das 40 · 3 em 5 · 1 em 11
```

**O período da percussão é 2 a 5× a escala harmônica.** O hipercompasso existe, é
grande, e o ritmo harmônico o mede.

**Mas o conserto não fecha.** Implementado o filtro fisiológico — os ciclos
candidatos (`T`, `T/2`, `T/3`, `T/4`, `1,5·T`, `2·T`) disputando **só dentro da
faixa plausível** — o resultado **não mudou no corpus** e **regrediu o gabarito**:

| | antes | depois do filtro |
|---|---|---|
| corpus (barra mediana) | 5,08 s | **4,81 s** (não resolveu) |
| gabarito | **6 de 6** | 5 de 6 (`fallin` quebrou) |
| sintético | 2 testes ok | 1 falhava (a barra caía para `T/2`) |

**Duas causas medidas:**

1. **O contraste é degenerado quando o histograma tem denominador nulo** — e aí a
   comparação vira ruído numérico (`5e8` contra `3,3e8`). O ciclo longo e a sua
   fração empatam, e o desempate é arbitrário.
2. **Desempatar pelo mais longo conserta o sintético e quebra o `fallin`** — o
   candidato `1,5·T` dele carrega contraste comparável e vence por ser maior.

**Revertido.** Nada entrou: sem ganho medido e com regressão no gabarito, a
mudança não se justifica — e o filtro fisiológico, sozinho, **não é o conserto**.

### Proposta 3 · o critério harmônico — e este fechou

**A tese:** a troca de acorde cai na fronteira da barra. Então a barra é o
candidato `T/k` (`k ∈ {1,2,3,4}`) **mais próximo do intervalo mediano entre
trocas** — o relógio da harmonia, que no corpus marca 1,90 s enquanto o da
percussão marca 5,08 s.

**Os critérios possíveis, medidos nos dois conjuntos antes de tocar no motor:**

| critério | corpus (mediana) | na faixa 1,5–2,5 s | gabarito |
|---|---|---|---|
| A · alinhamento médio às trocas | 2,13 s | 11 de 34 | **quebra o `fallin`** (0,94 s) |
| B · coincidência das fronteiras | 1,80 s | 16 de 34 | 1,88 s ✓ |
| C · distância ao intervalo de troca | 1,77 s | 28 de 34 | 1,88 s ✓ |
| **C, com B como desempate** | **1,77 s** | **29 de 34** | **1,88 s ✓** |

**O alinhamento sozinho não serve — e isso é medido, não suposto:** os custos dele
saem todos entre **0,40 e 0,50** (ele quase não discrimina) e ele **quebra o
`fallin`**, levando a barra para a metade. Ele entra como **desempate**, não como
juiz.

**A dobra e a subdivisão são ortogonais — e a ordem importa.**

- A **dobra** (`×1,5`, `×2`) conserta o bumbo que toca **menos** de uma vez por
  compasso. Quem decide é o **contraste** do histograma — e é o que acerta o
  gabarito.
- A **subdivisão** (`÷2`, `÷3`, `÷4`) conserta o **hipercompasso**. Quem decide é
  a **harmonia** — o contraste **não serve**, ele degenera no denominador nulo e
  prefere o ciclo longo.

**O bug do porte, e a lição:** apliquei a harmonia ao ciclo **bruto**, e o `binky`
caiu de `3/4` **decidido** para binário oferecido errado. O ciclo bruto dele é
0,93 s; a **dobra** (×1,5) dá a barra de 1,39 s — e o alvo harmônico dele é
4,46 s, **três barras adiante**. Com a dobra primeiro e a subdivisão depois, o
`binky` volta a 1,39 s e o **6 de 6** se mantém.

**A simulação em Python tinha previsto isso** — ela usava o ciclo **pós-dobra** (é
o que o JSON carrega). O porte trocou a entrada e o erro só apareceu no gabarito.
**Testar fora do motor não basta: o porte é um lugar onde o erro pode entrar.**

### O resultado, medido no motor

| | antes | depois |
|---|---|---|
| corpus · barra mediana | 5,08 s | **1,76 s** |
| corpus · na faixa 1,5–2,5 s | — | **29 de 40** |
| corpus · acima de 4 s | 17 de 40 (acima de 6 s) | **0 de 40** |
| corpus · razão barra/troca | 2,2 a 5,2 | **0,93** |
| gabarito independente | 6 de 6 | **6 de 6** |
| testes | 80 | **150 no workspace** |

**Os dois critérios de aceite foram batidos:** o gabarito ficou em 6 de 6 (sem
regressão no `fallin`) e a barra do corpus caiu de 5,08 s para 1,76 s, dentro da
faixa pedida de 1,5 a 2,5 s. **Zero das 40 ficou acima de 4 s.**

**E o que isto NÃO consertou — medido, não suposto:** o veredito ficou **mais
confiante**. As leituras `3/4` **decididas** passaram de 6 para **15 de 40**
(9 músicas saíram da abstenção), e a caixa nos terços **não distingue um compasso
ternário de uma subdivisão ternária de um binário** — é a própria skill
`medida-que-não-separa`. O corpus é louvor contemporâneo, majoritariamente
binário: 15 de 40 como `3/4` é implausível na ordem de grandeza.

### O veredito do dono: as dez são `4/4` — e o contraste não separa

**Dez músicas ouvidas e adjudicadas pelo dono (2026-11-24):**

| música | o motor | veredito | barra | contraste `3/4` |
|---|---|---|---|---|
| Agnus Dei | **decidiu `3/4`** | **`4/4`** ✗ | 0,64 s | 1,09 |
| Oficina G3 · Meus Passos | **decidiu `3/4`** | **`4/4`** ✗ | 0,43 s | **2,04** |
| Tudo é teu · Eu Sou Livre · Tua Graça · Abraça-me · Um Lugar para 2 · Te Adorar · Faz chover | ofereceu | `4/4` ✓ | 0,75–1,78 s | 0,54–0,86 |

**O piso de contraste não basta, e isto é medido:** `Meus Passos` tem contraste
**2,04** — **maior que o `binky` do gabarito (1,24)** — e é `4/4`. **Nenhum limiar
de contraste separa o certo do errado.**

**O que separa é a plausibilidade da barra:**

```
gabarito (3/4 certos)      0,90 · 1,39 · 1,93 s
falsos positivos           0,64 · 0,43 s        <- abaixo do piso fisiológico
```

As duas erradas têm barra de **283 e 421 BPM por batida de `3/4`**. O bumbo toca em
**todo tempo**, o ciclo cai na **batida** em vez da barra, e o conjunto de
candidatos só desce (`÷k`). **Uma barra fora da faixa não é uma barra.**

**O que o dono descreveu, e que o motor não vê:** *"contagem para não se perder…
`1 e 2 e 3 e 4 e`"*. As dez são `4/4` com a caixa em contratempo, repiques no `e`,
sextinas na guitarra, bumbo duplo, e — no caso do `Meus Passos` — andamentos e
compassos que mudam no meio. **O motor estava lendo o `e` como se fosse um tempo.**

### A regra final: a barra tem de ser plausível

`reconciliar` só afirma `3/4` com **as duas** condições: contraste confirmado
(**> 1**) **e** barra dentro da faixa fisiológica. Sem qualquer uma, a estrutura
ternária é **medida** e **oferecida**, com `3/4` entre as alternativas — e a
`regra` **nomeia o motivo**.

| | antes | depois do piso | depois do gate da barra |
|---|---|---|---|
| gabarito · `3/4` decididos | 3 de 3 | 3 de 3 | **3 de 3** |
| gabarito (total) | 6 de 6 | 6 de 6 | **6 de 6** |
| corpus · `3/4` decididos | 15 de 40 | 2 de 40 | **0 de 40** |
| corpus · falsos positivos medidos | 2 de 2 | 2 de 2 | **0 de 2** |
| corpus · abstenções corretas (amostra do dono) | — | — | **7 de 7** |

**A leitura honesta do resultado:** o motor **não** separa `4/4` de `6/8` — mas nas
dez músicas adjudicadas a resposta certa (`4/4`) está **dentro da oferta em 10 de
10**, e ele **não afirma** nenhuma delas. **É o que a R1 entrega: a resposta certa
entre as alternativas, sem vender palpite como medida.**

**Comparar dois contrastes e ficar com o maior não é decidir** — e foi daí que o
piso nasceu. Medido: as 15 vinham de contrastes **abaixo de 1** — `0,65` contra
`0,60`, `0,57` contra `0,45`. Não havia evidência nenhuma; havia um número maior
que o outro.

### O piso de contraste — o que ele fez, e por que não bastou

**Dois discriminadores foram testados e nenhum separa** — o que sobra é o
contraste absoluto:

| métrica | gabarito (3/4, devem decidir) | corpus (as 15) | separa? |
|---|---|---|---|
| alinhamento às trocas de acorde | 0,00 · 0,17 · 0,50 | até 0,31 | **não** |
| margem de contraste (`34 − 68`) | +0,11 · +1,41 · +1,44 | até +0,63 | **não** |
| **contraste `3/4` absoluto** | **1,24 · 1,76 · 2,08** | 0,47 a 0,86 · **e 2,04** | **quase** |

**A regra: um `3/4` só é decidido com o contraste `3/4` acima de 1** — a hipótese
tem de se confirmar (os terços carregarem mais que o resto), não apenas vencer a
alternativa. O que fica abaixo é **oferecido**, com `3/4` entre as alternativas:
**a estrutura ternária é medida; o nome do compasso é que não.**

| | antes do piso | depois |
|---|---|---|
| gabarito · `3/4` decididos | 3 de 3 | **3 de 3** |
| corpus · `3/4` decididos | 15 de 40 | **2 de 40** |
| corpus · ternário oferecido | — | **13** |

**O piso reduziu de 15 para 2 — e as 2 que sobraram eram as duas erradas.** O
contraste não bastou, e o defeito delas é o **inverso do hipercompasso**:
`Agnus Dei` dava barra de 0,64 s (**283 BPM** por batida de `3/4`) e `Meus Passos`,
0,43 s (**421 BPM**). O bumbo toca em **todo tempo**, o ciclo cai na **batida** em
vez da barra, e o conjunto de candidatos só **desce** (`÷k`), não sobe. **A
direção `×k` foi medida e quebra o gabarito** (o alvo harmônico do `fallin` é 1,5
barra, e a escolha ia para 2,82 s): não é um candidato a mais, é outro critério. É
o que o **gate da barra** resolve, sem tocar no gabarito.

## O que ficou estabelecido

- **A bateria define a fase; a harmonia define a escala; a subdivisão separaria
  `4/4` de `6/8`.** As três partes estão certas — e a terceira **continua sem
  medida que funcione** (3, 5 e 3 acertos de 8).
- **A âncora harmônica virou critério, e funcionou.** Ela era só referência
  (1,90 s de troca contra 5,08 s de barra); agora é o juiz da escala, com o
  alinhamento como desempate — e derruba a barra mediana do corpus para 1,76 s
  **sem tocar no gabarito**.
- **A dobra e a subdivisão são operações ortogonais, e a ordem importa:** a dobra
  (`×1,5`, `×2`) pelo contraste primeiro, a subdivisão (`÷2`, `÷3`, `÷4`) pela
  harmonia depois. Invertida, ela quebra o `binky`.
- **O hipercompasso é de material, não de código** — e agora está **medido e
  corrigido**: o mesmo motor que acerta `6 de 6` no gabarito lia 2 a 5 barras no
  corpus, e a harmonia traz a escala de volta.
- **A escala está travada; o veredito não.** 15 de 40 leem `3/4` **decidido**, e a
  caixa nos terços não separa compasso ternário de subdivisão ternária. **É o
  próximo problema — e ele é de honestidade, não de precisão.**

## As duas lições

**1. Gabarito escolhido por quem construiu não cobre o que ele não pensou em
testar.** O conjunto do Bennett tinha nove músicas e nenhuma `4/4` — e foi o corpus
de outra pessoa que mostrou o exagero. Segunda vez no mesmo dia: o `5 de 6` era
viés de **seleção**, o `6 de 6` era viés de **cobertura**.

**2. Precondição violada não é teste — mas é diagnóstico.** Rodar a percussão no
mix contraria o que o próprio módulo declara. O resultado (ciclos saturando) é
exatamente o que se espera quando a precondição falha — e o teste nos stems
mostrou que **só a precondição não bastava**: o corpus é mais difícil que o
gabarito por razões de material, não só de pipeline.

**Um placar de 100% num conjunto de nove não é 100% no mundo** — e rodar no mundo
é o que separa um instrumento de um resultado.