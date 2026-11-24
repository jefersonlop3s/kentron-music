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

## As duas propostas de conserto — medidas, e nenhuma fecha ainda

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

## O que ficou estabelecido

- **A bateria define a fase; a harmonia define a escala; a subdivisão separaria
  `4/4` de `6/8`.** As três partes da tese estão certas — e **nenhuma das duas
  últimas tem medida que funcione ainda**.
- **A âncora harmônica é o caminho**, e ela está **medida como referência**
  (1,90 s de troca de acorde contra 5,08 s de barra), mas **não como critério**:
  falta escolher o ciclo por **alinhamento com as fronteiras de acorde**, e não
  por contraste de histograma.
- **O hipercompasso é o próximo problema de verdade** — e ele é de material
  (bateria discreta), não de código: o mesmo motor que acerta `6 de 6` no
  gabarito lê 2 a 5 barras no corpus.

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