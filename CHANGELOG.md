# Changelog

Todos os eventos notáveis do projeto Kentron Music Hub.

## [feat] A harmonia dá a escala da barra: o hipercompasso do corpus cai de 5,08 s para 1,76 s (2026-11-24)

### O problema, medido em 40 músicas reais

O motor lia **2 a 5 barras** no corpus: barra mediana de **5,08 s** contra troca de acorde mediana de **1,90 s** (razão 2,16 a 5,19), com 17 das 40 acima de 6 s. É o **hipercompasso** — a bateria discreta faz o período da percussão disparar. O mesmo motor que acerta **6 de 6** no gabarito independente.

**Duas tentativas anteriores foram medidas e refutadas:**

| tentativa | corpus | gabarito | veredito |
|---|---|---|---|
| subdivisão interna separa `4/4` de `6/8` | 3, 5 e 3 acertos de 8 | — | **não separa** |
| filtro fisiológico no contraste | 5,08 → 4,81 s | **6 → 5 de 6** | **revertido** |

### O conserto: a bateria dá a fase, a harmonia dá a escala

A troca de acorde cai na fronteira da barra. Então a barra é o candidato `T/k` (`k ∈ {1,2,3,4}`) **mais próximo do intervalo mediano entre trocas**, com o **alinhamento às fronteiras** como desempate e o maior ciclo como desempate final.

**Os três critérios possíveis foram medidos antes de tocar no motor:**

| critério | corpus (mediana) | na faixa 1,5–2,5 s | gabarito |
|---|---|---|---|
| alinhamento médio às trocas | 2,13 s | 11 de 34 | **quebra o `fallin`** |
| coincidência das fronteiras | 1,80 s | 16 de 34 | 1,88 s ✓ |
| distância ao intervalo de troca | 1,77 s | 28 de 34 | 1,88 s ✓ |
| **com o alinhamento como desempate** | **1,77 s** | **29 de 34** | **1,88 s ✓** |

**O alinhamento sozinho não serve — e isso é medido:** os custos dele saem todos entre 0,40 e 0,50 (quase não discrimina) e ele quebra o `fallin`.

### A ordem importa, e o porte quase estragou tudo

**A dobra e a subdivisão são ortogonais.** A **dobra** (`×1,5`, `×2`) conserta o bumbo que toca menos de uma vez por compasso — decidida pelo **contraste**, é o que acerta o gabarito. A **subdivisão** (`÷2`, `÷3`, `÷4`) conserta o hipercompasso — decidida pela **harmonia**, porque o contraste degenera no denominador nulo (`5e8` contra `3,3e8`) e prefere o ciclo longo.

**Aplicada ao ciclo bruto, a harmonia quebra o `binky`** (de `3/4` decidido para binário oferecido errado): o ciclo bruto dele é 0,93 s, a dobra dá a barra de 1,39 s, e o alvo harmônico dele é 4,46 s — **três barras adiante**. Com a dobra primeiro e a subdivisão depois, o `binky` volta a 1,39 s.

### Resultado

| | antes | depois |
|---|---|---|
| corpus · barra mediana | 5,08 s | **1,76 s** |
| corpus · na faixa 1,5–2,5 s | — | **29 de 40** |
| corpus · acima de 4 s | 17 de 40 | **0 de 40** |
| corpus · razão barra/troca | 2,2 a 5,2 | **0,93** |
| gabarito independente | 6 de 6 | **6 de 6** |
| testes | 80 | **150 no workspace** |

### O piso de confiança: a hipótese tem de se confirmar

O veredito ficou **mais confiante**: as leituras `3/4` **decididas** passaram de 6 para **15 de 40**. E **comparar dois contrastes e ficar com o maior não é decidir** — medido, as 15 vinham de contrastes **abaixo de 1** (`0,65` contra `0,60`; `0,57` contra `0,45`). Não havia evidência; havia um número maior que o outro.

**Dois discriminadores foram testados e nenhum separa:**

| métrica | gabarito (3/4, devem decidir) | corpus (as 15) | separa? |
|---|---|---|---|
| alinhamento às trocas de acorde | 0,00 · 0,17 · 0,50 | até 0,31 | **não** |
| margem de contraste (`34 − 68`) | +0,11 · +1,41 · +1,44 | até +0,63 | **não** |
| **contraste `3/4` absoluto** | **1,24 · 1,76 · 2,08** | 0,47 a 0,86 · **e 2,04** | **quase** |

**A regra: um `3/4` só é decidido com o contraste `3/4` acima de 1** — a hipótese tem de se confirmar (os terços carregarem mais que o resto), não apenas vencer a alternativa. O que fica abaixo é **oferecido**, com `3/4` entre as alternativas: **a estrutura ternária é medida; o nome do compasso é que não.**

| | antes do piso | depois |
|---|---|---|
| gabarito · `3/4` decididos | 3 de 3 | **3 de 3** |
| gabarito (total) | 6 de 6 | **6 de 6** |
| corpus · `3/4` decididos | 15 de 40 | **2 de 40** |
| corpus · ternário oferecido | — | **13** |

**As 2 que sobraram têm barra implausível — o defeito inverso do hipercompasso:** `Agnus Dei` dá barra de 0,64 s (**283 BPM** por batida de `3/4`) e `Oficina G3 · Meus Passos`, 0,43 s (**421 BPM**). O bumbo toca em **todo tempo**, então o ciclo cai na **batida** em vez da barra — e o conjunto de candidatos só **desce** (`÷k`), não sobe. **A direção `×k` foi medida e quebra o gabarito** (o alvo harmônico do `fallin` é 1,5 barra e a escolha ia para 2,82 s): não é um candidato a mais, é outro critério.

### O veredito do dono: as dez são `4/4` — e o piso não bastou

**Dez músicas ouvidas e adjudicadas pelo dono (2026-11-24):** as **duas** que o motor decidia `3/4` são **`4/4`** ✗ e as **sete** que ele ofereceu são **`4/4`** ✓.

**O piso de contraste não basta, e isto é medido:** `Meus Passos` tem contraste **2,04** — **maior que o `binky` do gabarito (1,24)** — e é `4/4`. **Nenhum limiar de contraste separa o certo do errado.**

**O que separa é a plausibilidade da barra:**

```
gabarito (3/4 certos)      0,90 · 1,39 · 1,93 s
falsos positivos           0,64 · 0,43 s        <- abaixo do piso fisiológico
```

As duas erradas dão **283 e 421 BPM por batida de `3/4`**. O que o dono descreveu, e que o motor não vê: *"contagem para não se perder… `1 e 2 e 3 e 4 e`"* — as dez são `4/4` com a caixa em contratempo e repiques no `e`. **O motor estava lendo o **tempo** (a batida) como se fosse a **barra**.

### A regra final: a barra tem de ser plausível

`reconciliar` só afirma `3/4` com **as duas** condições: contraste confirmado (**> 1**) **e** barra dentro da faixa fisiológica. Sem qualquer uma, a estrutura ternária é **medida** e **oferecida**, com `3/4` entre as alternativas — e a `regra` **nomeia o motivo**.

| | antes | piso | **gate da barra** |
|---|---|---|---|
| gabarito · `3/4` decididos | 3 de 3 | 3 de 3 | **3 de 3** |
| gabarito (total) | 6 de 6 | 6 de 6 | **6 de 6** |
| corpus · `3/4` decididos | 15 de 40 | 2 de 40 | **0 de 40** |
| corpus · falsos positivos medidos | 2 de 2 | 2 de 2 | **0 de 2** |

**A leitura honesta, corrigida:** a métrica "a resposta certa está na oferta em 10 de 10" é **vazia** — a oferta binária é sempre `[6/8, 4/4]`. O que se pode afirmar: **o motor não afirma um nome errado em nenhuma das 40**, a oferta estreita o espaço e **diz qual evidência falta**, e **onde ele afirma, acerta** (3 de 3 no gabarito).

**E a escala não está fechada em 40 de 40:** **6 das 40 ficam abaixo do piso fisiológico** — o ciclo caiu na batida — e a mediana de 1,76 s esconde isso.

### A tentativa de refutação — cinco contra-argumentos, nenhum vence

Antes de fechar a R1, o dono pediu que eu tentasse **derrubar o próprio diagnóstico**. Os cinco, e o que cada um mediu:

| contra-argumento | resultado |
|---|---|
| **A** · "10 de 10 na oferta" prova algo? | **Vence — contra mim.** A métrica é vazia. Retirada. |
| **B** · o ritmo harmônico em batidas decide `4/4` vs `3/4`? | **Falha.** Mede a *taxa* de troca (gabarito ~6,0; corpus ~3,2), não o metro. |
| **C** · a escala está fechada? | **Vence em parte.** 6 das 40 abaixo do piso; a mediana esconde duas populações. |
| **D** · o gate está ajustado em dois pontos? | **Vence em parte.** Margem de 0,05 s (`delilah` 0,90 contra o piso 0,85) — fina, não exercida pelo corpus. |
| **E** · a razão barra/batida decide o compasso? | **Falha.** As faixas se sobrepõem inteiramente (0,75–4,03 no corpus; 1,50–2,97 no gabarito). |

### O sexto contra-argumento — a regra do dono, testada e **refutada**

O dono propôs uma regra nova, **musicalmente correta**: *"se a música sustenta a batida em três, é `6/8`; se não sustenta, é `4/4`"*. Implementada e medida, ela **falha** — e o modo da falha é a lição do dia.

**O que parecia separação limpa** (os `6/8` do Bennett com `escore_68` de 12,00 · 2,89 · 1,06 contra os `4/4` do corpus em 0,26 a 0,78) era a **taxa-base**: o corpus é quase todo `4/4`, então "`4/4` por padrão" acerta por acidente.

| conjunto | taxa-base | resultado |
|---|---|---|
| corpus (gospel) | quase todo `4/4` | 13 de 15 — **acidente** |
| `EstudoMetro` — nove clássicos documentados, **3 de cada** | balanceado | **1 de 9** |
| hinos com compasso declarado | ternário | **0 de 4** |

E a falha tem **forma**: os hinos saem **invertidos** — os `6/8` viram `3/4` e os `3/4` viram `6/8`.

```text
back-in-black     (4/4)  escore_68 = 6,16  -> o motor diria 6/8   ✗
house-rising-sun  (6/8)  escore_68 = 0,74  -> o motor diria 4/4   ✗
Me Derramar       (6/8)  escore_34 = 1,01  -> o motor diria 3/4   ✗
Crer e observar   (3/4)  escore_68 = 2,14  -> o motor diria 6/8   ✗
```

**Os dois contrastes não medem o que a regra pede.** Revertido, sem exceção.

**A lição, e é de método:** um conjunto com a **mesma taxa-base** da classe que você quer prever **não mede nada**. Foi um conjunto **balanceado** — e um com a **taxa-base oposta** — que mostrou o erro. **Contar as classes antes de contar os acertos.**

## [feat] 6 de 6 no gabarito independente: a banda da caixa, a coincidência e o ciclo por contraste (2026-11-24)

### O que fechou o último erro

Três descobertas, cada uma medida:

**1. A banda da caixa estava errada — e não por timbre.** A hipótese era que os fios (1,5–5 kHz) pegavam o chimbau. **Medido: não.** O ataque do bumbo é um transiente **banda-larga** e dispara em **todas** as bandas no mesmo instante:

```
pele da caixa 150-450    52%  21%  13%   6%   4%   4%
fios 1.5-5k              32%  29%  17%  12%   4%   6%
prato 6.5-13k            25%  35%  18%  14%   4%   4%
```

**A mesma forma nas três** — a caixa real fica encoberta pelo ataque do bumbo. **Conserto: a banda da caixa passou a ser a PELE (150–450 Hz)**, onde o corpo dela vive e o prato não tem energia.

**2. A caixa que coincide com o bumbo é descartada** (tolerância 0,06 s). Sem a supressão: 5 de 6. Com ela: **6 de 6**.

**3. O ciclo é escolhido pelo contraste, não por limiar.** O bumbo pode ser capturado num **submúltiplo da barra**, e o fator depende do metro: `3/4` com o bumbo em dois de três tempos → `T` em **2/3** (fator **1,5**); `6/8` com ele em dois de seis → `T` na **metade** (fator **2**). Um limiar de energia órfã **não separa os dois** (`binky` 56%, `say-something` 36% — fatores diferentes). **Os três ciclos candidatos são avaliados e vence o de maior contraste** — a mesma moeda que decide a dobra.

### Resultado

| música | Bennett | o motor | ciclo |
|---|---|---|---|
| Mull of Kintyre | 3/4 | **3/4** | 1,93 s |
| We Are The Champions | 6/8 | **6/8** | 1,90 s |
| Delilah | 3/4 | **3/4** | 0,90 s |
| Fallin' | 6/8 | **6/8** | 1,88 s |
| Say Something | 6/8 | **6/8** | 1,26 → **2,52 s** |
| Binky | 3/4 | **3/4** | 0,93 → **1,39 s** |

**6 de 6 — o fator certo para cada metro, achado sozinho.**

### A lição

A regra da energia órfã era remédio para o sintoma. Com a banda certa o sintoma desapareceu, e a escolha do ciclo pôde virar **evidência** em vez de limiar. **Quando o front-end acerta, a decisão não precisa de exceção.**

### A trilha inteira

```
1 de 8   a dobra sozinha, no mix
2 de 7   a relação caixa×bumbo, no mix
4 de 6   no stem separado (ADR-0017)
5 de 6   competição de hipóteses sobre as fatias
6 de 6   banda da pele + supressão de coincidência + ciclo por contraste
```

## [feat] A decisão era o gargalo: competição de hipóteses leva o motor de 4 para 5 de 6 (2026-11-24)

### O diagnóstico que destravou

Oito hipóteses de **front-end acústico** falharam sem regredir o placar — banda mais larga, fluxo em log, espectrograma mais fino, âncora por amplitude, gate de entropia, `D_harm` a 30 ms, âncora harmônica, croma de ataque. **O sinal já chega limpo; o que faltava era modelagem temporal no nível do compasso.** Diagnóstico do dono, e ele acertou.

### O que mudou

A leitura da `percussao` deixou de ser um **limiar local** e passou a ser **competição de hipóteses** sobre as seis fatias normalizadas:

```
contraste 6/8 = (P0 + P3) / 2       ÷  média(P1, P2, P4, P5)    <- polos em s1 e s4
contraste 3/4 = (P0 + P2 + P4) / 3  ÷  média(P1, P3, P5)        <- polos em s1, s3 e s5
```

### Medido — no mesmo histograma do motor, no gabarito independente

| regra | acertos | erros |
|---|---|---|
| `fatia 3` vs `fatias 2+4` (a anterior) | 4 de 6 | `say-something`, `binky` |
| **competição de hipóteses** | **5 de 6** | `binky` |
| competição **com rotação circular** | 4 de 6 | `fallin`, `binky` |

**A rotação circular foi testada e piorou** — e o motivo é estrutural: a origem do ciclo é o **último bumbo**, que **já é um polo** nos dois metros. Rotacionar não corrige nada e deixa a hipótese errada achar alinhamento espúrio.

### A lição

**O limiar local comparava duas posições soltas e confundia ruído com contraste.** O contraste normalizado sobre o ciclo inteiro é o que separa — a mesma correção de forma que a `dobra` precisou: **normalizar antes de comparar**.

### O que falta

`binky`: o período do bumbo sai 0,93 s (dois tempos, não a barra de ~1,4 s) e o pico da caixa cai em **s2**, onde nenhuma hipótese tem polo. **É o ciclo de novo** — o mesmo fantasma que resistiu às quatro hipóteses de detector.

### A energia órfã corrigiu a escala — e o veredito não virou

`P1` e `P5` **nunca são polos** em nenhum dos dois metros. Massa concentrada ali é assinatura de período capturado no submúltiplo. A regra: se `max(P1,P5) > 0,35`, ampliar o ciclo por **1,5** e refazer.

- **Acertou a escala na mosca:** `binky` de **0,93 s para 1,39 s** — a barra real é **~1,40 s**. A razão `2/3` prevista, corrigida por `×1,5`.
- **Cirúrgica:** disparou **só no `binky`**. As órficas dos cinco acertos ficam entre 0,15 e 0,30; o limiar é 0,35, no vão.
- **Mas o veredito não virou:** o histograma no período corrigido é `23% 30% 22% 12% 5% 8%` — espalhado, não os terços limpos. **O período era um erro; o outro é a precisão dos onsets da caixa.**

**Placar: 5 de 6 mantido.** A regra entra porque corrige o período (ganho medido) e não regride — com o limite declarado de que não fecha o `binky` sozinha.

## [processo] A âncora por croma: uma peça é ouro, o gate é no-op por matemática (2026-11-24)

O dono propôs ancorar o ataque no **croma** em vez da energia — croma ponderado por transiente, gate por entropia, e `D_harm` separado de `D_reart`. Analisei e medi cada um dos quatro passos. **O motor não foi tocado: 4 de 6 intacto.**

| passo | veredito | evidência |
|---|---|---|
| **1 · croma de ataque** (derivada de meia-onda antes da projeção em 12 bins) | **funciona, e é o ouro** | só a energia nova entra; sustentado e reverberação desaparecem |
| **2 · gate por entropia** | **no-op no `D_harm` — provado** | abaixo |
| **3 · `D_harm` a τ=30 ms** | **mede nota, não acorde** | no grave: **11,4 ancoras/s** (saturação) |
| **4 · croma grave < 300 Hz** | **converge para o que o `compasso` já faz** | o baixo é a linha mais movimentada |

### O gate é no-op por matemática

`W(t)` é escalar por quadro, então `Cg(t) = W(t)·C(t)` e na distância cosseno o peso **cancela**:

```
D_harm = 1 - Cg(t)·Cg(t-τ) / (|Cg(t)|·|Cg(t-τ)|)
       = 1 - C(t)·C(t-τ) / (|C(t)|·|C(t-τ)|)
```

**Verificado numericamente: 1,18e-10 de diferença** entre com e sem gate. E na medição os picos são **idênticos** (17 · 72 · 71 · 148 · 99 · 180 · 64 · 39 · 14). O gate só afetaria o `D_reart`, que é soma.

### E ele está quase fechado de qualquer forma

Peso médio medido: **0,00–0,18**. A causa é de física — **o ataque de uma corda palhetada é banda-larga por natureza**, e é isso que "ataque" significa no espectro. **Ponderar por transiente e punir por entropia brigam entre si.**

### O `τ` é a escala errada para harmonia

`τ`=30 ms e `τ`=100 ms dão o mesmo (11,4 e 11,5 ancoras/s): o `D_harm` do grave dispara a cada **nota do baixo**. Mudança de acorde vive na escala de **0,5–2 s** — e ali o `D_harm` **é** a novidade harmônica que o `compasso` já usa para ancorar o tempo 1.

### O que fica da proposta

O **croma de ataque** é feature boa e ainda não usada pelo motor: a novidade de croma do `compasso` usa a diferença simples, não a derivada de meia-onda projetada. **É ali que ela pode render**, e é medível — a âncora do tempo 1 com o croma de ataque contra a atual.

### E o teste na âncora: não paga

| âncora | acertos |
|---|---|
| **último bumbo** (o que o motor faz) | **5 de 6** |
| harmônica, croma simples | 2 de 5 |
| harmônica, **croma de ataque** | 2 de 6 |

**A âncora harmônica é pior que o bumbo** — o que **corrige o meu diagnóstico anterior**. Eu tinha dito que "o último bumbo não é o tempo 1" e que a harmonia seria a âncora certa. **Medido: o bumbo é melhor.** Ele toca no tempo 1 na maior parte do repertório; a troca de acorde é mais esparsa e mais irregular.

E o **croma de ataque não melhora a âncora** (2 de 6 contra 2 de 5): a feature é boa, mas a âncora não é onde ela rende.

### A conclusão da linha

**A medida está no teto** com o material e as ferramentas atuais: quatro hipóteses de detector refutadas, a âncora harmônica refutada, o croma de ataque sem ganho na âncora. **O 4 de 6 (motor) e 5 de 6 (Python) seguem de pé — nada regrediu em nenhuma das oito tentativas.**

## [feat] A relação bumbo×caixa no motor — 4 de 6 no gabarito independente (2026-11-24)

### Adicionado
- **`crates/mir_engine/src/percussao.rs`** — mede o **bumbo** (30–110 Hz) e a **caixa** (1,5–5 kHz) separados e lê o metro da **relação** entre os dois: a caixa na **metade** do ciclo do bumbo (`6/8`) ou em **um terço e dois terços** (`3/4`). Período por autocorrelação com **interpolação parabólica**, a mesma do `estimate_tempo`.
- **`Report.percussao`** e uma linha no resumo.
- **2 testes:** o `6/8` sintético dobra em 2 com a barra em 3,00 s e o `3/4` em 3 com a barra em 1,50 s; o silêncio abstém.

### Medido

| material | resultado |
|---|---|
| sintético `6/8` (bumbo 1, caixa 4 de seis) | **dobra 2**, barra 3,00 s |
| sintético `3/4` (bumbo 1, caixa 2 e 3) | **dobra 3**, barra 1,50 s |
| `Me Derramar` (é `6/8`), no stem | barra **3,09 s**, caixa em 0 e na metade → **dobra 2** ✓ |
| **gabarito independente (6 stems)** | **4 de 6** — contra **1 de 8** da medida anterior |

### Por que funciona onde a `dobra` não funcionava

A `dobra` somava bumbo, caixa, chimbau, violão e voz num fluxo espectral só — e **para essa soma o acento do bumbo e o da caixa são o mesmo tipo de evento**. Somados, `6/8` (bumbo em 1, caixa em 4) e `3/4` (bumbo em 1, caixa em 2 e 3) dão o mesmo envelope: contraste 0,42 nos dois, empate exato. **Separados, não empatam.**

### Dois erros de contagem, achados pelos testes

- O **silêncio** devolvia `min_lag` como período: correlação nula é ausência de periodicidade, não período mínimo.
- A **novidade constante** fazia todo quadro virar ataque: variância zero não tem piso.

### O que falta

Nos 2 erros o **período do bumbo saiu pequeno** (`say-something` 1,26 s, `binky` 0,93 s) — é o ciclo de novo, não a relação. Próximo afinamento.

## [processo] O gabarito independente: o motor acerta 1 de 8, e o meu 5 de 6 era viés (2026-11-24)

O dono trouxe **quatro vídeos de músicos** sobre `3/4` × `6/8` e autorizou baixar. Legendas automáticas para o texto; **frames** do vídeo de 15 min para ler as respostas que aparecem na tela.

### Os quatro confirmam a regra — e ela é a `dobra`

- `Jay Fenichel`: *"it all has to do with **how you subdivide the six** — divide by **two** → two groups of three, that's 6/8; 3/4 is the opposite, divide by **three**"*.
- `Andrew Freed`: *"both have the same amount of eighth notes… **the underlying pulse is different**"*.
- `David Bennett`: `3/4` = **simple triple**; `6/8` = **compound duple** — *"68 has two strong beats in a bar, not six, but two"*.

### E confirmam a ambiguidade como fato musical

Bennett: em andamento **lento** o `6/8` *"can start to sound like 3/4 because our ear can't latch onto the two beats"* (`Gravity`, John Mayer); em andamento **rápido** o `3/4` soa como `6/8` (`America`, Simon & Garfunkel, *"I've seen differing opinions"*). Fecha: *"it comes down to the way you want to feel the music"*.

### O gabarito — nove exemplos com resposta na tela

| música | Bennett | o motor |
|---|---|---|
| Mull of Kintyre (Wings) | 3/4 | **3/4** ✓ |
| We Are The Champions (Queen) | 6/8 | sem nome |
| My Favorite Things | 3/4 | sem nome |
| Delilah (Tom Jones) | 3/4 | 6/8 ✗ |
| Fallin' (Alicia Keys) | 6/8 | 4/4 ✗ |
| Say Something | 6/8 | 3/4 ✗ |
| The Times They Are a-Changin' | 3/4 | sem nome (alternativas certas) |
| Manic Depression (Hendrix) | 3/4 | 3/4 ✓ |
| Binky (Snarky Puppy) | 3/4 | 4/4 ✗ |

**1 acerto, 4 erros, 4 sem nome.** (O `Manic Depression` já estava no conjunto anterior e já acertava; entre os oito novos, **1 de 8**.)

### O que isso obrigou a corrigir

- **O `5 de 6` era viés de seleção.** A medida **não generaliza** para material escolhido por outra pessoa.
- **A margem não é confiança:** no gabarito as margens de acertos e erros se sobrepõem (acerto 0,401; erros 0,243 · 0,190 · 0,053). O limiar de 0,05 foi calibrado no conjunto viciado.
- **O nome virou sugestão, e as alternativas vêm sempre.** A cifra carrega `{comment: …}` em vez de `{time: …}`: gravar um palpite de 1/8 como fato é pior que não gravar. `{time: …}` fica para o músico.
- **O fixture dourado pegou a mudança** antes de ela ser aceita.

## [processo] O metro: cinco medidas, uma funciona, um gargalo só (2026-11-22)

Fechamento da campanha. O placar do que foi medido:

| medida | resultado |
|---|---|
| perfil de acento no mix | chapado — contraste 0,42 em `m=3` **e** em `m=6`, empate exato |
| perfil por banda no mix | 0,5–1,5 em oito de nove — chapado |
| perfil por banda na componente percussiva | 0,43–1,35, sem estrutura consistente |
| figuras rítmicas da melodia | fase plana (11–27%) — **refutada para material cantado** |
| **dobra: `A(T/2)` contra `A(T/3)`** | **5 de 6** com a barra do motor · **5 de 6** com a barra verdadeira · 4 de 6 com o pico da autocorrelação |

**A única medida que separa é a dobra**, e o erro dela é sempre de **nível do `T`**, nunca da regra. Escolher o nível pela decisividade da dobra troca o erro de lugar (5 de 6 de novo, erro em 0,021).

**Recomendação:** aceitar **5 de 6 com abstenção** — o limiar de 0,05 já pega as margens finas — e parar de procurar medida nova. Cinco tentativas apontaram a mesma peça faltando: **a barra**.

## [processo] A melodia cantada não nomeia o metro — a sílaba não respeita a barra (2026-11-22)

Passo 2 da arquitetura do dono implementado: histerese de **0,6 semitom nos cents** (arredondar antes era o erro — transformava vibrato de ±30 cents em troca de classe), estabilidade de 4 quadros, e corte por **ataque harmônico** em 150–2500 Hz sobre a componente HPSS.

### Medido

| métrica | resultado |
|---|---|
| contagem de notas | **1,8–2,8/s** (antes 3–4/s, com 64–81% no piso) |
| **histograma de fase dos inícios** | **plano nas seis faixas — 11–27% por fatia** |
| notas ainda no piso (<0,15 s) | 11–34% |

### O que isso encerra

**Em música cantada, a melodia segue a sílaba, não o compasso** — o texto atravessa a barra. A hipótese de que as figuras da melodia nomeiam o metro **está certa para melodia instrumental e não vale para material cantado**.

**Corroboração:** na componente **percussiva** (o complemento do HPSS) o perfil acha o backbeat — `Billie Jean` com a caixa em **1,6 na posição 2** (amplitude 0,8), `Me Derramar` com 0,7–0,8. **O metro é do acompanhamento.**

### Decidido
- **Não publicar a fusão.** Terceira versão da segmentação reprovada na medição; mesma disciplina.
- **O próximo passo mudou de lado:** o perfil por banda passa a ser medido na **componente percussiva**, não no mix nem na melodia.

## [processo] As figuras da melodia não fecham a lacuna do metro — instrumento inexistente, medido (2026-11-22)

Ideia do dono: transcender para MIDI e ler as **figuras rítmicas** da melodia — mais semínimas seria `3/4`, mais semínima pontuada seria `6/8`. **A tese musical é correta** e é a saída natural do beco do envelope de novidade: o envelope soma tudo e é ambíguo, a melodia é uma linha só.

### Medido — e as duas rotas falharam

| rota | resultado |
|---|---|
| segmentar por mudança de classe de altura | **64–81% das notas entre 0,12 e 0,20 s** — pico colado no mínimo de duração |
| + filtro de moda de 0,12 s | 55–75% continuaram no piso; notas **aumentaram** (`house-rising-sun` 535 → 728) |
| cortar por ataque na banda da voz (200–1000 Hz) | **os mesmos ataques do espectro cheio** — 101 contra 101 (`Billie Jean`), 144 contra 146, 119 contra 122 |

**Causa:** a classe de altura oscila a ~7 Hz — vibrato de voz é 4–7 Hz e cruza a fronteira de semitom, e cada cruzamento corta a nota. A 43 quadros/s, 0,12 s = 5 quadros: a janela é **menor que o período do vibrato**, então nenhum filtro curto separa "vibrato numa nota" de "duas notas".

### Decidido
- **Não publicar a segmentação.** O código foi implementado, medido e **revertido**: uma lista de notas errada em 75% é pior que lista nenhuma, e um MIDI feito dela seria ficção com carimbo de transcrição.
- **O que falta não é limiar:** é **rastreador de nota** (ataque **e** altura juntos) ou **stems** — a parte com ML da R4 etapa 2, atrás do portão.

## [feat] A dobra do ciclo: `A(T/2)` contra `A(T/3)`, a regra do dono medida (2026-11-22)

### Adicionado
- **`Compasso.dobra`** (`emdois`/`emtres`/`indeterminada`), **`correlacao_meio`**, **`correlacao_terco`** e **`margem_dobra`** — a autocorrelação do envelope de novidade no **meio** e no **terço** do compasso, comparadas. É a regra proposta pelo dono: em `6/8` as metades são espelho e o envelope casa em `T/2`; em `3/4` os três tempos repetem e ele casa em `T/3`.
- **Limiar de abstenção `0,05`**, calibrado no material — margem fina não é evidência, e erro com margem fina seria pior que abstenção.
- **3 testes:** a correlação acha o período do envelope, a correção `n/(n-k)` compensa a queda de pares, e o silêncio abstém.

### Medido

| material | resultado |
|---|---|
| sintético `6/8` (bumbo 1, caixa 4) | `A(T/2)=0,532` contra `A(T/3)=-0,015` |
| sintético `3/4` (bumbo 1, caixa 2 e 3) | `-0,048` contra `0,763` |
| `Me Derramar` (é `6/8`) | **`emdois`, margem 0,332** |
| nove faixas de banda | 5 abstenções · 4 decisões · **3 certas, 1 errada** (`back-in-black`, 0,149) |

**A correção `n/(n-k)` é necessária:** sem ela o valor cai sempre com o lag e o `6/8` ideal era lido como simples.

### Limitações declaradas
- **O ciclo tem de ser a barra.** Em compasso composto o ciclo que o motor mede é o **grupo de acento** (3 subdivisões): ali a dobra conta subdivisões, não metro.
- **A margem do motor é menor que a do experimento em Python** (envelope de 11 ms, que deu 5 de 6): a janela 4096/salto 1024 suaviza o envelope. O caminho é um envelope dedicado mais fino, **não** um limiar mais frouxo.

## [docs] Faixas de Hz da bateria: o mapa medido e o que não separa (2026-11-22)

### Adicionado
- **`docs/insights/2026-11-22-faixas-de-hz-da-bateria.md`** — nove faixas com bateria de verdade (três `3/4`, três `6/8`, três `4/4`), baixadas sob autorização do dono e mantidas **fora** do repositório (ADR-0013). Método: novidade de energia por banda em cada ataque, normalizada pela média da própria banda, para não deixar a banda grave dominar.

### Medido — medianas das nove faixas

| peça | ≤300 Hz | 1,5–5 kHz | 6,5–13 kHz |
|---|---|---|---|
| **bumbo** | **53%** (25–68) | 1% | 0% |
| **caixa** | 13% | **65%** (48–76) | 25% |
| **chimbau** | 13% | 25% | **60%** (47–75) |

A distribuição por família é estável nas nove: **32–57% grave, 21–38% médio, 13–45% agudo**.

### Descoberto — o que a banda não separa
- **Bumbo e caixa se separam por banda** (53% contra 1% em 1,5–5 kHz).
- **Caixa e chimbau não:** a caixa tem **25%** da energia na faixa do chimbau e o chimbau **25%** na da caixa, simétrico. O que os separa é o **decaimento** (caixa 150–250 ms, chimbau fechado ~50 ms) — não a faixa. É o próximo passo da partitura de bateria.

### Resultado negativo registrado
- **O perfil de acento por instrumento é chapado em mix denso:** 0,5–1,5 por posição em oito das nove faixas; só `Nothing Else Matters` (mix esparso) dá 1,8–2,0. Em mix denso toda banda sobe em toda batida, e a banda de 30–110 Hz carrega o **baixo elétrico** junto com o bumbo.
- **O motor devolve `3` para o `3/4` e para o `6/8`**, com margem 0,00–0,04 em oito das nove — abstenção disfarçada. **A discriminação `3/4` × `6/8` continua aberta.**

## [feat] Acento por instrumento e o invariante da barra (2026-11-22)

### Adicionado
- **`Compasso.compasso_s`** — `periodo_s × batidas_por_compasso`, zero quando indeterminado. `(BPM, batidas)` é moldura; **a barra em segundos é o invariante**.
- **`Compasso.bandas`** (`PerfilBandas`) — acento por posição em **bumbo 30–110 Hz**, **caixa 1,5–5 kHz** e **chimbau 6,5–13 kHz**, dobrado em **dois** compassos, com `bandas.backbeat()`. A barra começa **onde o bumbo bate**: a fase do motor desliza uma batida quando há acento em quase toda batida.
- Normalização da novidade **pela média da própria banda** — sem isso a banda grave domina a medida.
- **3 testes:** o invariante da barra, caixa no tempo 4 de 6 (bate na virada da metade) e caixa nos tempos 2 e 3 (não bate).

### Medido — `Me Derramar`, 39–59 s, áudio original
- **Hierarquia limpa:** chimbau **0,52 s** → pé **1,55 s** → compasso **3,095 s**; a **caixa tem período de um compasso inteiro** — uma caixa por dois pés, a assinatura do `6/8` de banda.
- **O "meio-forte" existe:** bumbo **4,38** no tempo 1 e **1,32** na posição 6 — 12× os 0,08–0,11 das demais.

### Corrigido
- **A conclusão de que `3/4` e `6/8` seriam o mesmo objeto acústico estava errada.** O erro era somar bumbo, caixa e chimbau numa novidade espectral só — é essa soma que apaga o backbeat. Refutada pelo dono e registrada no `PENDING`.

## [feat] Melodia no JSON e arbitragem estendida a `sus` (2026-11-22)

### Adicionado
- **`Report.melodia`** — `MelodiaSegment` por trecho: quadros com F0, classe dominante, fração e histograma das 12 classes. A melodia era extraída e usada, mas **invisível**: sem publicar, a hipótese de que ela carrega a terça não era conferível.
- **Arbitragem estendida a `5`/`sus4`/`sus2`** e **`FRACAO_DA_TERCA` de 0,40 para 0,20**, medido — o limiar antigo só servia para melodia parada.
- **2 testes:** a terça fraca promove `5` e `sus4`, e a prova negativa (sem terça na melodia o motor não inventa qualidade).

### Medido — trecho 39–51 s de `Me Derramar`, com a verdade do dono

| trecho | motor | a melodia tocou | verdade |
|---|---|---|---|
| 0,16–1,95 s | `G5` → **`G`** ✓ | B (terça de G) 21% | `G9` |
| 6,90–9,45 s | `Em` → `Em` ✓ | G (terça de Em) 30% | `Em7` |
| 9,45–11,91 s | `C5` → **`C`** ✓ | E (terça de C) 19% | `C9` |
| 1,95–3,65 s | `Gsus2` ✗ | A 24%, B 20%, D 10% | `G9` |

O quarto não é falha do árbitro: o acorde detectado é `Dsus4` com baixo **G**, então ele procurou a terça **de Ré** e a melodia toca **Si**, a terça **de Sol**. **A raiz estava errada antes da terça** — o conserto é o baixo vir antes do acorde.

## [feat] Terça pela melodia no motor (ADR-0014, bloco 3) (2026-11-22)
### Adicionado
- **`crates/mir_engine/src/melody.rs`** — `extrair(spectro, taxa)` devolve a **classe de altura da voz por quadro**, com `contagem`, `quadros_com_f0` e `dominante`. O método é soma harmônica sobre os picos do espectro (a fundamental da voz vem com 2f e 3f), na banda de 200–1000 Hz, com piso de 20% do maior pico do quadro.
- **`chords::detect_com_melodia(chroma, Option<&Melodia>)`** — o detector guarda a **estrutura** (tônica e qualidade) até o fim e só monta o nome depois que a melodia opina. `com_a_melodia` **só promove `5` → maior/menor**, com mínimo de 3 quadros com F0 e 40% de fração.
- **Fixture `power_chord_com_voz`** em `tests/common` — fundamental + quinta (sem terça) com a voz por cima, e a variante `power_chord_com_voz_em(terca, ganho)` para calibrar.
- **7 testes** em `tests/melodia.rs`.

### Medido — a janela de ganho da voz (2026-11-22)

| Ganho da voz | Sem melodia | Com melodia |
|---|---|---|
| 0,30 | `C` | `C` — o croma decide sozinho; não prova nada |
| **0,16** | **`C5`** | **`C`** — é aqui que o árbitro é a diferença |
| 0,12 (default) | `C5` | `C` |
| 0,09 | `C5` | `C5` — a melodia perde a voz e o motor **abstém** |

A janela não foi escolhida: foi medida. E o caso de 0,09 é a **prova negativa** do árbitro — sem evidência suficiente, ele não inventa qualidade.

### Verificado
- **55 testes** em `mir_engine` (7 novos); `clippy -D warnings` e `fmt --check` limpos.
- **A fixture dourada não mudou** — prova de que o árbitro não toca em acorde já decidido: `git status` da pasta de fixtures veio vazio depois da mudança.

### Ressalvas
- **O F0 erra oitava e isso é irrelevante:** a decisão é por **classe de altura**, e oitava não muda classe. É o que permite um método simples em vez de um rastreador de pitch completo.
- **Mixagem real é mais difícil que a fixture:** separar voz de acompanhamento sem stems continua sendo o ponto frágil. O árbitro é estreito de propósito — se errar, o custo é um `5` que continua `5`.
- **A janela de 0,12–0,16 é de áudio sintetizado.** No repertório real o número muda; medir é a próxima tarefa, não um ajuste de limiar no escuro.

## [feat] O motor lê mp3, m4a e flac — Symphonia no lugar do ffmpeg (ADR-0016) (2026-11-22)
### Adicionado
- **`crates/mir_engine/src/decode.rs`** — `abrir(caminho)` e `decodificar(bytes, extensão)`. O dispatch é por **conteúdo** (`RIFF`/`WAVE`), não por extensão: extensão mente, e um `.wav` renomeado não pode virar erro de formato. WAV segue no leitor próprio (sem dependência e testável em memória); todo o resto vai pelo **Symphonia**.
- **Dependência `symphonia` 0.5.5** (Rust puro, **MPL-2.0** — licença que o `deny.toml` já aceita para dependência sem modificação). As otimizações SIMD trazem o `rustfft`, que já estava no workspace pelo croma.
- **3 testes:** WAV decodificado **pelo Symphonia** bate com o leitor próprio (taxa, canais e amostras, diferença < 1e-3); buffer que não é áudio falha com "formato não reconhecido"; e o reconhecimento de WAV é por conteúdo.

### Decidido
- **O produto não embarca `ffmpeg`** (GPL-3.0-or-later, medido no insight). Formato fora da matriz vira erro explícito, e converter é decisão do usuário.
- **Um caminho para todas as plataformas**: por ser Rust puro, serve Android, iOS, web (wasm) e desktop sem binário por SO.

### Verificado
- **Provado de ponta a ponta na CLI:** MP3, **M4A (AAC)** e **FLAC** gerados na hora (`ffmpeg -f lavfi`) e analisados — 4,0 s, 44 100 Hz, 1 canal, sem erro.
- **48 testes** em `mir_engine`; `clippy -D warnings` e `fmt --check` limpos; **`cargo deny check licenses` → `licenses ok`** (MPL-2.0 aceito).
- A CLI deixou de dizer "o ffmpeg converte antes".

### Ressalvas
- **Opus e WavPack ficaram de fora:** a versão 0.5.5 **não** inclui `symphonia-codec-opus` no guarda-chuva — Ogg/Opus falha com "códec não suportado". Pesa, porque o `yt-dlp` entrega Opus em WebM/Ogg.
- **Vorbis compila mas não foi exercido:** `symphonia-codec-vorbis` e `format-ogg` estão na árvore, mas esta máquina não tem codificador Vorbis para gerar a prova.
- **HE-AAC** continua exigindo adaptador (`libfdk-aac`).
- A matriz de formatos é a do upstream **filtrada** pelo que a versão compila — não é cobertura nossa, e o teste prova o encanamento, não cada códec.

## [feat] Afinação global medida e relatada (ADR-0014, bloco 6, parte) (2026-11-22)
### Adicionado
- **`Spectrogram::desvio_de_afinacao`** — mediana do desvio de cada pico espectral em relação ao semitom mais próximo, em semitons. Entram só picos com pelo menos **30% da magnitude máxima do quadro**, o que descarta lóbulo lateral e ruído sem limiar absoluto (que dependeria de normalização do arquivo). `Report.audio.afinacao_cents` leva o valor em cents, e o `resumo()` só o imprime quando passa de **5 cents** — abaixo disso é ruído de medição.
- **`para_cada_pico`** — a leitura de pico com interpolação parabólica saiu do `chroma` para uma função única, usada pelo croma **e** pela medição. Se cada uma interpolasse do seu jeito, as duas visões do mesmo espectro divergiriam — e o número medido não descreveria o croma que ele deveria descrever.
- **4 testes:** em A440 dá quase zero; **−30 cents** mede −30; **+45 cents** mede +45; e o **limite declarado** — acima de meio semitom a medida é circular (−60 lê como +40), porque "distância ao semitom mais próximo" vive em ±50 cents.

### Decidido
- **O croma não é deslocado pelo desvio.** Dentro de ±50 cents o arredondamento para a classe já dá conta; no limite a correção é ambígua e poderia piorar o resultado. O número é **medido e relatado** — que é o que a ADR-0014 pede ("estimar o desvio de A440 antes de casar contra escalas") —, não aplicado por cima do croma sem prova. Acima de meio semitom o assunto é **transposição**, não afinação, e o motor não finge resolver.

### Verificado
- **45 testes** em `mir_engine`; `clippy --all-targets -D warnings` e `fmt --check` limpos; e a **fixture dourada não mudou** — que é justamente a prova de que o croma seguiu idêntico.

## [feat] Centro tonal por perfil no motor (ADR-0014, bloco 1) (2026-11-22)
### Adicionado
- **`crates/mir_engine/src/tonality.rs`** — `analisar(&Chroma) -> Tonalidade`: croma médio (só quadros com energia) contra os perfis de **Krumhansl–Kessler** nas **24 tônicas** (12 × maior/menor), por **correlação de Pearson**, devolvendo a **regra que produziu** o resultado (correlação e margem sobre a segunda) e `classes_presentes`. Sem energia, ou com menos de **quatro** classes de altura, a resposta é **indeterminada com o motivo** — fail-loud, não palpite.
- **`chords::nome_da_classe`** — tonalidade e acorde passam a nomear a classe de altura pela **mesma** convenção (bemóis nas teclas pretas), para um `Db` não virar `C#` de um lado só.
- **7 testes de unidade:** Dó maior, Lá menor natural, **E maior não vira Si**, **A maior com D natural não vira Mi** (o par de discriminadores do grau 4 que a ADR nomeia), sem energia, duas notas (intervalo não sustenta tonalidade) e quadro silencioso que não dilui a média.

### Mudado
- **`Report`** ganhou `tonalidade: Tonalidade`; ela entra no `resumo()` com correlação e margem, e o `{key:}` da cifra passou a sair dela. **`tom_estimado` foi removido**, junto com o `raiz_e_qualidade` que ele usava — o defeito `{key: B5}` desaparece por construção, porque não há mais parsing de nome de acorde para adivinhar tom.
- **Fixture dourada:** `crates/mir_engine/tests/fixtures/cifra-com-letra.cho` passou de `{key: Am}` para `{key: C}`. É a correção do defeito que a própria ADR-0014 cita ("na fixture em Dó devolveu `Am`"); o diff é **uma linha**.
- **`apps/kentron_app/test/cifra_com_letra_test.dart`:** a linha de acordes passou a ser encontrada por **estilo monoespaçado**, não por texto solto — com `{key: C}` o chip do tom no AppBar também mostra "C" e `find.text` achava dois. O teste ficou mais preciso: agora prova que a linha é monoespaçada, que é o que faz a coluna coincidir com a letra.

### Verificado
- **41 testes** em `mir_engine`; `cargo clippy --workspace --all-targets -- -D warnings` limpo; `cargo fmt --all --check` limpo; suíte de Flutter verde.

### Ressalvas
- **Margem baixa não bloqueia:** a resposta sai com `margem` no relatório, mas o motor não recusa tonalidade duvidosa nem marca `Indeterminado` por margem — só por falta de energia ou de classes. Endurecer isso é **decisão** (qual limiar), não execução.
- **Empréstimo modal:** o perfil é maior/menor; Mixolídio, Dórico e afins ainda não têm perfil próprio (bloco 6 da ADR-0014).
- **O que não mudou:** a acurácia continua **candidata**. O ganho não é acertar sempre — é a resposta vir com a razão.

## [processo] Insight: decodificação de áudio por plataforma — o ffmpeg do brew é GPL (2026-11-22)
### Adicionado
- **`docs/insights/2026-11-22-decodificacao-de-audio-por-plataforma.md`** — mapeamento local do `ffmpeg`, rotas por plataforma e a candidata que dispensa o problema.

### Medido
- **Máquina:** **um** ffmpeg — `/opt/homebrew/bin/ffmpeg` → `Cellar/ffmpeg/9.0.2_1`; `/usr/local/bin/ffmpeg` e `/usr/bin/ffmpeg` **não existem** (sem duplicata, o teste da skill `toolchain-unica` passa). `brew info` declara **License: GPL-3.0-or-later**; `ffmpeg -version` confirma `--enable-gpl --enable-version3 --enable-shared` com `libx264`/`libx265`/`libsvtav1`/`libvmaf`; `otool -L` mostra link **dinâmico** às dylibs do Cellar.
- **`arthenica/ffmpeg-kit`** — repositório **arquivado em 2026-07-02**, *"FFmpegKit has been officially retired"*; a continuação é o **FFmpegKitNext**, **só código-fonte**. Licença **LGPL-3.0** por padrão, **GPL-3.0** com `--enable-gpl`.
- **`pdeljanov/Symphonia`** — **Rust puro**, **MPL-2.0**; lê mp3/mp2, AAC-LC, ALAC, FLAC, Vorbis, Opus, PCM, ADPCM e WavPack; as otimizações SIMD **puxam o `rustfft`**, que o workspace já usa; desempenho declarado "geralmente ±15% do FFmpeg".

### Decidido
- **Dívida fechada:** "ffmpeg é LGPL" não vale para a build padrão do Homebrew — ela é **GPL-3.0-or-later**, por causa de x264/x265/svt-av1/vmaf.
- **Onboarding com gerenciador de pacotes** (detectar → aceite → instalar via brew/winget/apt) é o padrão de desktop **e a saída de licença**: o binário GPL é distribuído pelo gerenciador do usuário, não por nós.
- **Symphonia no `mir_engine` exige ADR** antes de executar: muda o contrato de entrada (hoje `wav.rs` só aceita RIFF/WAVE).

### Aberto
- ADR da decodificação de áudio. Segue **não verificado**: o texto da regra 2.5.2 das Diretrizes da App Store e a build de `ffmpeg` embarcada pelo OVD.

## [processo] Insight: Open Video Downloader e o padrão da ferramenta do usuário (2026-11-22)
### Adicionado
- **`docs/insights/2026-11-22-open-video-downloader.md`** — análise do `jely2002/youtube-dl-gui` (Open Video Downloader): **Tauri (Rust) + Vue 3/TypeScript**, GUI fina sobre o **`yt-dlp`** como subprocesso, com cookies do navegador para autenticação, fila em lote e template de nome. Licença **AGPL-3.0**, distribuído em exe/dmg/AppImage/deb/rpm, Homebrew, WinGet e **Microsoft Store**, com disclaimer de que **a responsabilidade é do usuário**.

### Decidido
- **Nenhum código do OVD entra no projeto:** AGPL-3.0 é incompatível com a Apache-2.0 (copiar tornaria o Kentron AGPL). O `yt-dlp` que ele supervisiona é **Unlicense**, então o impedimento é do *wrapper*, não do extrator.
- A decisão da **ADR-0013 (forma C) fica reforçada, não alterada**: o caso é o precedente de que "ferramenta do usuário" é viável, e o uso de **cookies do usuário** mostra que "ato do usuário, com credencial do usuário" é padrão técnico, não retórica.

### Aberto
- Hipótese registrada no PENDING: aceitar **lote de arquivos** da ferramenta do usuário, casando por nome e duração, para a forma C ser usável no repertório de 313 linhas.
- Permanece **não verificado**: a licença da build de `ffmpeg` que o OVD distribui, e a acurácia do casamento nome/duração num lote real.

## [processo] ADR-0013, ADR-0014 e ADR-0015 aceitas (2026-11-22)
### Decidido
- **ADR-0013 — aceita na forma C:** o link resolve **sempre** a identidade da obra e do fonograma; o áudio vem do usuário. A forma B (o produto baixa) fica **rejeitada**. Playlist entra como **lista de referências**, criada na conta do usuário, sem o produto redistribuir áudio.
- **ADR-0014 — aceita nos blocos 1–6:** centro tonal por perfil no eixo de quintas, afinamento global, terça pela **melodia** (F0), baixo por compasso para a inversão, região harmônica separada da seção timbrística e empréstimos modais como conjunto permitido. A separação **ritmo × baixo** vai por **HPSS + harmonicidade**, sem modelo.
- **ADR-0015 — aceita:** alvos **desktop** (Windows, macOS, Linux) com a tela de palco por **URL** (`?palco=`); segunda tela é janela web, não multi-janela. Execução **depois da release da R1**, ordem Windows → Linux → macOS.

### Verificado
- `python3 .harness/documentation.py check` sem link quebrado; `ADR-INDEX` com os três status e o próximo número livre em ADR-0016; ROADMAP, PENDING e CONCLUSION alinhados ao aceite.

### Aberto
- Implementação: o **centro tonal por perfil** é o primeiro código (aceito em separado, não precisa de modelo nem de áudio novo) e troca "nota mais frequente" por "tonalidade com a razão".

## [processo] Ritmo × baixo sem modelo, playlist nas APIs e alvos desktop (2026-11-22)
### Adicionado
- **ADR-0015** (proposta) — **alvos desktop** (Windows, macOS, Linux) com a **tela de palco por URL** (`?palco=<base64>&tom=`). A segunda tela é **janela web** apontando para a URL — navegador da TV, segunda janela do desktop ou Browser Source —, não multi-janela nativa: um caminho de renderização só, o que já está provado. Ordem recomendada **Windows → Linux → macOS** (notarização do macOS é o custo mais alto e o menor alcance no público-alvo) e execução **fora da R1**.
- **ADR-0014, emenda** — bloco 1 (centro tonal por perfil) **aceito pelo dono**. Separação **ritmo × baixo sem modelo**, em duas etapas: **HPSS** (filtro de mediana no espectrograma — Fitzgerald 2010 —, sem pesos, cabe no `rustfft` que o crate já usa) remove o transiente do bumbo, e um **discriminador de harmonicidade** decide o resíduo (baixo = série harmônica sustentada e estável; corpo do bumbo = decaimento quase senoidal). Modelo de 4 stems vira melhoria, não pré-requisito.
- **ADR-0013, emenda** — busca e playlist medidas: no YouTube, `search.list` tem **balde próprio de 100/dia** e `playlists.insert`/`playlistItems.insert` custam **50 unidades cada** num balde de **10.000/dia**; no Spotify, `Create Playlist` e `Add Items` existem (a rota antiga `/tracks` está **[DEPRECATED]**). O caminho barato **não usa busca**: o export do usuário já traz a coluna `Video`, então a playlist sai dos IDs que ele já tem.

### Verificado
- Demucs v4 e Open-Unmix declaram **MIT no repositório** e separam `drums`/`bass` (Demucs arquivado em 2025-01-01; Open-Unmix SDR `bass 5,23 / drums 5,73` em UMX). **Ressalva registrada:** nenhum dos dois declara a licença dos **pesos** — o `MANIFEST` exige confirmar na fonte dos pesos, e o madmom é **CC BY-NC**, proibido.

### Aberto
- Aceite das ADR-0013 (forma C) e ADR-0015 (alvos desktop), e dos blocos 2–6 da ADR-0014.

## [processo] Link, playlist e análise harmônica: duas ADRs propostas (2026-11-22)
### Adicionado
- **ADR-0013** (proposta) — **link como referência**, **playlist como lista de referências** e a fronteira do áudio. Das três formas, **C** (o produto resolve o link até a *identidade* e pede o arquivo quando a análise precisa de áudio) é escolhida e **B** (o produto baixa) é rejeitada com motivo. Base em fonte primária: YouTube API Developer Policies proíbe *"separate, isolate, or modify the audio or video components"*; Spotify Developer Policy (15/05/2025) só trata áudio como streaming e clipe de prévia, sem endpoint de download, e a quota estendida só aceita organização desde 15/05/2025.
- **ADR-0014** (proposta) — **análise harmônica pós-extração**: centro tonal por perfil no eixo de quintas, afinamento global, terça decidida pela **melodia** (F0), baixo por compasso para a **inversão** e **região harmônica** separada da seção timbrística. Registra a causa mecânica do viés para maior — o **5º harmônico da fundamental coincide com a terça maior** (razão 5/4) — e nomeia **onze lacunas de sinal** no processo ditado pelo dono (compasso/downbeat, bumbo × baixo, modo sem sensível, empréstimo modal, peso dos graus, janela harmônica).
- **`docs/PENDING.md`** — itens novos: link → identidade da obra; playlist como referências; análise pós-extração; compasso e downbeat. Cifra em monitor de palco (TV) avaliada e **não priorizada**, com o caminho barato registrado.
- **`docs/ROADMAP.md`** — R4 ganhou as duas notas (link/áudio e análise harmônica).

### Retificado
- A nota do PENDING que afirmava que a inversão "exige baixo isolado (stems)" era **hipótese, não medição**: linha de baixo e melodia predominante são tarefas de MIR **sem** separação de fontes; stems fica como melhoria de acurácia.

### Verificado
- `python3 .harness/documentation.py check` sem link quebrado após as duas ADRs e o índice.
- **Não verificado** (declarado nas ADRs): a cláusula de consumo do YouTube (a página volta truncada nesse ponto) e o bloqueio de `audio-features` citado no plano v2.5 (changelog devolve 404).

## [processo] Piloto automatizado no OBS e navegação por seção na URL (2026-11-21)
### Adicionado
- **`scripts/piloto-obs.sh`** — piloto da ADR-0010: serve o build web, abre dois túneis SSH (porta 4455 do `obs-websocket` e a do servidor de volta para a VM), cria uma fonte de navegador apontando para a URL de projeção, captura a fonte por `GetSourceScreenshot` e compara o hash do pixel entre seções e entre duas rodadas. Reprova quando a tela fica chapada (1 cor, σ=0.0). `--negativo` roda a prova negativa com URL morta; `--construir` reconstrói o build antes.
- **`?secao=N` na URL de projeção** (`apps/kentron_app/lib/src/stage/projection_link.dart`) — o Browser Source do OBS não recebe clique nem teclado, então a URL é o único canal determinístico para navegar. `secaoDaUrl` lê o índice; `urlDaProjecao` aceita `secao`; `ProjecaoApp`/`ProjectionPage` abrem na seção pedida e limitam ao intervalo válido.
- **Testes novos** — `secaoDaUrl` (ausente, presente, inválida) e bootstrap abrindo na seção pedida; 17 testes no app.
- Skill **`ssh-no-windows`** — a chave que o servidor OpenSSH do Windows recebe e recusa, o `authorized_keys` do usuário contra o `administrators_authorized_keys`, a ACL que o serviço (SYSTEM) precisa, o grupo `Administradores` em pt-BR que não casa com o `administrators` do `sshd_config`, e o `sshd -T` que decide qual arquivo o servidor lê.

### Verificado
- Duas rodadas aprovadas no OBS: seção 0 com 353 cores (σ≈19,5) e seção 1 com 354 cores (σ≈19,2); o hash do pixel mudou entre as seções e repetiu entre as rodadas.
- Prova negativa: URL morta reprovou nas 6 verificações e o roteiro saiu 0 — o verificador não é complacente.
- `flutter analyze` limpo; `bash -n` e `shellcheck` verdes no roteiro.

### Ressalvas
- **Causa raiz de um falso negativo:** servir pelo IP da LAN deixa a fonte em branco — o CEF do OBS ignora COOP/COEP fora de contexto seguro, o fallback do frb reinjeta `pkg/kentron_ffi.js` e o wasm morre com `Identifier 'wasm_bindgen' has already been declared`. O roteiro contorna com túnel reverso (`ssh -R`) servindo por `localhost`; a dupla carga em si continua sem correção no código.
- **O que o piloto não prova:** transposição dentro do OBS (a projeção não mostra acorde; é provada em `scripts/e2e-web.sh` e nos testes Rust) e o app sobreviver a fechar/reabrir (o roteiro não reinicia o app; fica no alpha assistido da R1-C).

## [processo] Vínculo ficha ↔ obra original e o portão de publicação no app (2026-11-21)
### Adicionado
- **Coluna `obra_original` no asset de fichas** — `crates/mir_engine/assets/fichas_hinario.tsv` passou a declarar a obra do corpus de que a versão deriva (o 09 declara `Holy, Holy, Holy`). O vínculo é **declarado, não adivinhado**: vazio = sem vínculo. O cabeçalho `#` documenta que é vínculo de **referência**, não de identidade — a tradução é obra própria e responde pelo veredito dela; o original responde pelo dele.
- **`fichas::obra_original_no_corpus`** — resolve o título declarado no corpus de letras (`hinos_dominio_publico.tsv`) e devolve a `Obra`; `obra_da_ficha` passa a carregar o vínculo na `Obra` que reconstrói. `--ficha-hino` imprime a obra original quando existe.
- **`scripts/verificar_fichas.py`** — o vínculo declarado tem de existir no corpus e o autor do corpus tem de conferir com a `autoria` da ficha. `scripts/prova-verificar-fichas.sh` foi a **9 casos negativos** (novos: obra original inexistente; autor que não confere).
- **`kentron_ffi::pode_publicar`** — a ponte expõe o portão do motor (`mir_engine::sheet::pode_publicar`) em vez de copiar a regra: o app pergunta, o motor decide. Dependência `mir_engine` na ponte e bindings regenerados.
- **`linkDeProjecao` / `PublicacaoBloqueada`** (`apps/kentron_app/lib/src/stage/projection_link.dart`) — o link de projeção só é montado depois do portão; escopo privado do grupo lança e a tela mostra o motivo (`stageProjectionBlocked`, nos três ARBs base). `song_reader_page.dart` usa o portão injetável (`portaoDePublicacao`), com default no núcleo Rust.

### Verificado
- Gate aprovando as 12 fichas e reprovando os **9 casos negativos**; `cargo fmt --check`, `clippy -D warnings` no workspace, **102 testes Rust** (novos: vínculo resolvido no corpus, ficha sem vínculo, `Obra` carregando o vínculo, e o portão da ponte bloqueando/liberando por escopo), **34 testes Flutter** e `flutter analyze` limpo.
- `--ficha-hino "Cantor Cristão|09"` imprime `obra original no corpus: Holy, Holy, Holy — Reginald Heber (1826)`.

### Ressalvas
- O vínculo é **declarado à mão** por título: ficha nova sem `obra_original` não ganha vínculo por heurística, e o gate só confere o que foi declarado — semelhança de título não vira vínculo.
- **A exportação não existe no app ainda** — o portão cobre o link de projeção hoje; quando a exportação nascer, usa o mesmo `pode_publicar`.

## [processo] Fichas do hinário: dado em TSV, gate e prova negativa (2026-11-21)
### Adicionado
- **`crates/mir_engine/assets/fichas_hinario.tsv`** — **12 fichas** chaveadas pelo par **(hinário, número)**: **8 em domínio público, 3 protegidas e 1 GAP**, com **procedência por campo** (`fonte_autoria`, `fonte_traducao`, `fonte_melodia`). Só fatos: letra e cifra da edição não entram. Cabeçalho `#` documenta o formato e por que ele **não se funde** com `hinos_dominio_publico.tsv` (lá é corpus de letras, lido por `include_str!`; aqui é ficha). A linha do 291 registra que a sigla→nome saiu de OCR (`Anlúniv Almeida`) corrigido por fonte secundária.
- **`scripts/verificar_fichas.py`** — reprova veredito inválido, `pd`/`protegido` sem ano, `gap` sem observação, **fato sem a fonte do campo** (`fonte_autoria`/`fonte_traducao`/`fonte_melodia`), **procedência que cita OCR sem `observacao`**, campo acima de 300 caracteres e **qualquer coluna de letra/cifra/tablatura**. O limite legal deixa de ser recomendação: vira mecanismo.
- **`scripts/prova-verificar-fichas.sh`** — prova no sentido negativo com **seis** variações do asset que **têm** de reprovar — incluindo "fato sem procedência" e "procedência cita OCR sem observação"; reprovaram as seis.
- **Gate na bateria:** o par asset+verificador roda no `.githooks/pre-push` (escopado ao `CHANGED_FILES`) e no `scripts/ci-local.sh`, **com a prova negativa junto** — se o verificador enfraquecer, é a prova que acusa. O `.harness.toml` passou a dizer onde ele vive.
- Skills **`hinario-e-dominio-publico`** e **`ocr-em-digitalizacao`**, com o frontmatter das demais.
- **`mir_engine::fichas`** — a ficha deixa de ser dado inerte: lê o asset embutido (procedência por campo), responde por **(hinário, número)**, **recusa** título ambíguo, **deriva a resposta do ano** (`Livre` / `RequerLetraDoUsuario` / `Gap` / `Ambiguo` / `SemFicha`) e `auditar()` prova `morte+71` linha a linha com a regra única (`lyrics::veredito_pd`).
- **CLI:** `--ficha-hino "hinário|número"`, `--listar-fichas`, `--auditar-fichas`; `--hino` sem letra no corpus passa a **explicar pela ficha** (livre / protegida / gap / ambígua) em vez de só dizer "não está no corpus".
- **`uso::veredito_por_titulo`** consulta a **ficha antes do corpus**: é a ficha que carrega a tradução, a camada que manda. Título ambíguo vira `Indeterminado` nomeando as candidatas.

### Verificado
- Gate aprovando as 12 fichas e reprovando os 6 casos negativos; **o próprio hook recusou o push** com uma ficha `pd` sem ano (`PUSH BLOQUEADO — gate(s) reprovado(s): fichas`) e liberou com o asset íntegro; `ruff check`, `ruff format --check` e `shellcheck` verdes nos dois scripts; `documentation.py check` sem link quebrado; bateria do pre-push verde no push `a33598f..ca61cbb`.
- **A auditoria do registro da sessão (22:02) achou uma leitura inventada:** o OCR da p.722 dizia `A.A. - Anlúniv Almeida` e os documentos passaram a afirmar `A.A. → Antônio Almeida` como leitura do índice. Corrigido no asset (com `fonte_traducao` + `observacao`), no `CONCLUSION`, no `PENDING` e nas duas skills — e o gate passou a exigir `observacao` quando a procedência cita OCR.
- **Ficha no motor, provada:** 10 testes novos (asset com 12 fichas e 10 no repertório, identidade por par com número sem zero, ambiguidade do "Louvor", resposta derivada do ano em 2017/2018 e 2039/2040, `gap`, `sem_ficha`, asset malformado, auditoria da fronteira, obra derivada com o tradutor como camada que manda, e o uso recusando escolher qual "Louvor"); **31 testes no total**, `clippy -D warnings` limpo.
- **Gate com a aritmética:** `verificar_fichas.py` passou a reprovar `ano_dominio_publico != morte + 71` (Lei 9.610/98, art. 41), e a prova foi a **7 casos negativos**.

### Ressalvas
- O asset **ainda não é lido por código** — `lyrics.rs` continua com `hinos_dominio_publico.tsv`, intacto. O gate **está na bateria** e o contrato `[validation]` do `.harness.toml` segue só com gitleaks (decisão de 2026-11-21). A relação entre ficha e corpus está documentada nos dois arquivos (não se fundem); falta a **consulta em código** — PENDING.
- **O tradutor do 291 foi identificado a olho:** a sigla `AA.` é leitura do hino e o índice da CPAD (p.722) foi conferido na **imagem legível** — `A.A. - Antônio Almeida`. O OCR tinha lido `Anlúniv Almeida`, e foi essa correção que ficou registrada como se leitura fosse, até a conferência. As **datas 1879-1969** seguem de fonte secundária (hinologia.org — o índice não traz anos), e o ano **2040** é calculado delas.
- O hook chamava `avisa_base "$1"` e abortava fora do `git push` (`set -u`): passou a `"${1:-origin}"`, sem o que nenhum gate poderia ser provado manualmente.
- **Duas reprovações reais só apareceram quando o commit existiu:** o push anterior dizia "nada a commitar" e deixava `CHANGED_FILES` vazio — `lint` e `shell` não tinham arquivo para olhar. `ruff format` e `shellcheck` (SC2164) reprovaram então, e foram corrigidos antes do commit.

## [ADR-0012] Sem catálogo, repertório móvel e obra derivada (2026-11-21)
### Decidido
- **O produto nasce sem catálogo**, com **onboarding** explicando o funcionamento e o usuário o alimentando (palavras do dono). Consequência declarada: o corpus de domínio público embutido **não é catálogo** — é **tabela de referência de ficha**, que só responde quando o usuário traz uma obra com aquele título. Ele não sugere repertório nem aparece como biblioteca.
- **O repertório é móvel.** O dono corrigiu meu enquadramento: o repertório **já teve mais de 1000 obras**; as 313 de hoje vêm de um enxugamento que acontece (na intenção a cada 6 meses, na prática ano a ano), e as **execuções** é que são numerosas. Nada no produto pode assumir catálogo estável: o que se mantém é **histórico de execução**, e o relatório é **foto datada de repertório que anda**. Ficha conferida não expira — obra que sai pode voltar.
- **Obra derivada é obra própria.** Caso do dono: **"Maravilhosa Graça"** e a versão **"Tremenda Graça"** — sentido, versos, poesia e música (cifra, refrões, arranjos) iguais, mas **título e frases mudam**. Não é tradução nem é a mesma obra: é **obra derivada**, julgada **pelo autor dela**. O caso geral que ele descreveu — *4 acordes de uma obra de domínio público com frases novas* — significa que **a obra original pode ser livre e a adaptação, não**.
- **Alinhamento da letra (sem mudança de decisão):** transcrever do áudio é o caminho **principal**; letra fornecida é **opção** para quando a versão muda frases. A capacidade fica no produto; o **ato continua declarado como do usuário**, com escopo privado do grupo. Ganho técnico que só apareceu agora: **ASR é a única fonte da letra adaptada** — nenhuma fonte oficial tem "Tremenda Graça"; o áudio tem.

### Adicionado
- **`Obra.adaptador` / `adaptador_morte`** (o campo que era `tradutor`): **tradução é o primeiro caso de adaptação**, e o julgamento é o mesmo — pelo autor da obra derivada.
- **`Obra.obra_original`**: referência à obra de que deriva, para "mesma música, outro título e outras frases". É vínculo, **não fusão** — o veredito de uma não empresta para a outra.
- `--ficha` aceita a ficha com os novos campos e imprime o vínculo quando há adaptação.

### Verificado
- **`adaptacao_com_frases_novas_e_obra_propria_e_nao_herda_o_veredito`**: "Maravilhosa Graça" (1779) é domínio público; "Tremenda Graça" com adaptador falecido em 2015 é **protegida**, e a regra **nomeia o adaptador**. A ficha guarda `obra_original`.
- Demonstração real: mesma obra → `domínio público (vida de Autor Antigo (1807) + 70 anos)`; adaptação → `protegido (vida de Adaptador da equipe (2015) + 70 anos)` e, nos EUA, `indeterminado — obra derivada: falta o ano de publicação da adaptação`.
- **17 suítes de Rust verdes** (9 testes só de ficha/obra) e 32 do Flutter; `clippy -D warnings` limpo.

### Ressalvas
- **O onboarding não existe** — é item de PENDING; a decisão é que o produto nasça vazio e explique, mas a interface ainda não foi feita.
- **A transcrição por ASR continua não implementada** (exige peso no `models/MANIFEST.toml`). O que existe é o enquadramento completo: origem declarada, escopo privado do grupo e `pode_publicar` reprovando.
- O corpus segue com **4 hinos** e agora carrega a coluna de **obra derivada**; ampliá-lo com hinários brasileiros continua exigindo a ficha do adaptador/tradutor, hino a hino.

## [ADR-0012] Fronteira de produto e o repertório real medido (2026-11-21)
### Medido
- O dono exportou o repertório do app atual (313 linhas, `.xlsx`) e fixou a fronteira: **a versão e o repertório são entrada do usuário**; o produto não decide o que se canta nem qual versão vai ao ar. OBS + Holyrics são o transporte — o Kentron pretende substituí-los medindo e posicionando, não opinando.
- **313 linhas · 299 títulos distintos (295 normalizando "(Ao Vivo)"/"(Live)"/"- Part.") · 156 artistas.**
- **O export do usuário já carrega os dois eixos da ADR-0012:** `Song name`+`Artist` = obra; `Album`/`Version name`/`Version notes` = versão; `Video` = fonograma com link (**299 das 313**); `Key` (**293**) e `BPM` (**224**) = execução. **244 linhas** trazem nuance de execução no próprio texto ("Masculino: A / Feminino: D") — a nuance que o dono pediu para preservar desde o começo, agora num campo nosso (`Execucao`).
- **O corpus de domínio público classifica 0 das 313 obras.** Ele é de hinos ingleses pré-1900; o repertório é português — **10 linhas** de hinário (Cantor Cristão, Harpa) e **303** contemporâneas.
- **`letras.mus.br` / `cifraclub.com.br` aparecem em 54–56 linhas** — é o caminho de letra e cifra que a equipe usa hoje. O produto **não** os busca: deriva a cifra do áudio e recebe a letra do usuário (ADR-0012).
- Qualidade de dado observada: em pelo menos uma linha o campo `Video` contém link de letra — o importador **não pode confiar no rótulo da coluna**, tem de olhar o domínio da URL.

### Decidido
- **Emenda à ADR-0012:** repertório e versão são entrada do usuário; o produto mede, posiciona e declara procedência. Escrito com o número que sustenta: o corpus atual **não** cobre este repertório, e `SEM FICHA` será a resposta da maior parte dele — o que **não é falha do motor**, é a natureza daquelas obras (contemporâneas, protegidas).
- Duas coortes de trabalho, com tratamentos diferentes: **hinário** (ficha do tradutor → veredito de domínio público) e **contemporâneo** (atribuição + decisão de licença do dono; o relatório é o dado da decisão, não um veredito meu).

### Ressalvas
- **Escala corrigida:** o repertório limpo é de **~300 obras distintas**; as "1000+ por ano" que eu havia lido são **execuções**, não obras. Isso muda o esforço das fichas de inviável para factível.
- Nada de código mudou nesta rodada: é medição e decisão. O corpus embutido segue com 4 hinos e **não** foi ampliado com hinários brasileiros — o que exige a ficha do tradutor, hino a hino (PENDING).

## [ADR-0006 emendada] A agenda vira o relatório de uso (2026-11-21)
### Adicionado
- **`Execucao`**: a obra **e a versão em que foi tocada** (versão + tom). A mesma obra volta em outro culto e **nem sempre na mesma versão** — sem isso duas execuções diferentes viram a mesma linha.
- **`Setlist` é o evento**: ganhou `data`, `tipo` (culto, ensaio, evento) e entradas com versão/tom. A agenda do culto é o registro; o relatório é subproduto dela.
- **`relatorio_de_uso`** com `RelatorioUso { eventos, execucoes, por_obra }` — por obra: quantas execuções e **em que versões** — e `texto()` em ordem estável (mais tocada primeiro).
- **`schema_version` 2 com passo de migração real** (`migrar_1_para_2`): agenda v1 (entrada era id solto) vira execução **sem versão inventada** — ausente é ausente, e o relatório diz `(versão não declarada)`.
- **`bundle_store::load_setlists(dir)`** lê `<dir>/setlists/*.json` em ordem de nome, para o relatório ser comparável entre execuções.
- **`mir_engine::uso`** + CLI **`--relatorio-uso <pasta>`**: cruza a agenda (ADR-0006) com a **ficha da obra** (ADR-0012) e separa **domínio público por ficha** de **sem ficha** — a segunda é a lista de trabalho, ditada pelo uso real.

### Decidido
- **Emenda da ADR-0006**, pelo que o dono indicou do funcionamento real: **agenda por culto/evento**, **uma playlist por evento**, músicas que **repetem** e, quando repetem, **nem sempre na mesma versão**. Repertório acima de **1000 músicas por ano** — enumerar é inviável, então **classifica-se o que se canta**.
- **Pressuposto nomeado na ADR:** a agenda do culto é preenchida de qualquer forma; o relatório não pede hábito novo.
- **Não se inventa formato de licença.** O dono não tem um, e o app não arbitra: o relatório diz o que é domínio público por ficha e o que está **sem ficha** — que é a informação para ele decidir sobre licença coletiva.

### Verificado
- **`agenda_guarda_evento_e_versao_da_execucao`**: dois cultos, a mesma obra em versões diferentes → 1 obra, 2 versões, 2 execuções, e o texto traz o nome das versões.
- **`agenda_v1_migra_para_v2_sem_inventar_versao`**: agenda v1 escrita à mão migra, `data`/`tipo`/`versao` ficam **ausentes**, e o relatório imprime `(versão não declarada)`.
- **`relatorio_separa_dominio_publico_de_obra_sem_ficha`**: biblioteca e agendas reais em diretório próprio → 1 obra com ficha (corpus) e 1 **sem ficha**; `diretorio_sem_biblioteca_falha_alto_em_vez_de_mentir` cobre o negativo.
- **`load_refuses_a_newer_schema`** atualizado para `SCHEMA_VERSION + 1`: o teste que usava "2" como versão futura deixou de valer com o bump, e passou a se calibrar pela constante em vez do número solto.
- Demonstração real com bundle em disco: `2 eventos · 5 execuções · 3 obras distintas`, `Essência da Adoração` com **2 versões** (álbum, acústica) e `1 domínio público · 2 sem ficha`.
- **Rust: 17 suítes verdes** (2 novas de uso, 3 novas de agenda/migração, 1 de relatório no domínio) e **32 testes do Flutter** inalterados.

### Ressalvas
- **O app ainda não tem tela de agenda.** O modelo, a persistência, a migração e o relatório existem; preencher agenda pelo app é trabalho de interface.
- **Sem ficha, o relatório não afirma domínio público nem protegido** — diz `sem ficha`. É o limite declarado da ADR-0012.
- O formato do campo de licença continua **decisão do dono**; hoje o que existe é `--licenca` como texto livre na origem da letra.

## [ADR-0012 aceita] Procedência em dois eixos e extração pelo usuário (2026-11-21)
### Adicionado
- **Proveniência em dois eixos.** `lyrics::Obra` (título, autor, ano, **ano de falecimento**, tradutor, base) e `lyrics::Fonograma` (descrição, ano, titular) agora são tipos separados: **gravação de ontem de obra de 1822 continua protegida**, e `Fonograma` **não tem veredito de domínio público** — a categoria não existe do lado da gravação.
- **`lyrics::veredito_pd(obra, jurisdição, ano)`** com três estados — `DominioPublico`, `Protegido`, **`Indeterminado`** — e **a regra que produziu o veredito** no texto: no Brasil, vida do autor + 70 anos (Lei 9.610/98, art. 41, contados de 1º de janeiro do ano seguinte ao falecimento); nos EUA, publicado antes de 1930. Sem ano de falecimento, o resultado é `Indeterminado`: **ficha incompleta nunca vira "domínio público"**.
- **Armadilha da tradução tratada como dado:** a ficha tem `tradutor`/`tradutor_morte`, e a obra traduzida é julgada **pelo tradutor** — hinário de 1922 pode conter tradução de 1980. Nos EUA, tradução sem o ano de publicação dela é `Indeterminado`.
- **`lyrics::ExtraidaPeloUsuario`** — extração **pelo usuário**, com o ato no nome dele e **escopo privado do grupo**; `sheet::pode_publicar` **reprova** publicação de cifra com escopo `grupo`. Não existe variante para extração feita *pelo produto*: o enum não aceita, de propósito.
- **`--licenca`** na letra do usuário (texto livre, formato a definir pelo dono) e **`--gravacao "descrição|ano|titular"`**; verbos novos: **`--auditar-corpus`** e **`--ficha "título|autor|ano|falecimento|tradutor|falecimento tradutor|base"`**, que imprime o veredito por jurisdição.
- A cifra passa a carregar **`{work_status: …}`** (com a regra), **`{recording_source: …}`** (ou "não declarada", sem omitir) e **`{lyrics_scope: …}`**.

### Decidido
- **ADR-0012 aceita pelo dono** e emendada no mesmo dia. O argumento do crédito (crédito ≠ licença) e a resolução — **a licença que o usuário já tem** (licença coletiva de igreja/CCLI) — ficam registrados na ADR, junto com a ressalva de que não é parecer jurídico.
- Extrair letra do áudio entra como **ato do usuário**, não do produto: quem fornece responde, a cifra nasce privada no grupo, e publicar continua dependendo da licença coletiva dele.

### Verificado
- **Rust: 15 testes** (2 unidade + 4 análise + 5 letra/ficha + 4 fixture). Novos: `obra_de_1822_com_gravacao_de_ontem_nao_vira_gravacao_livre` (obra livre nas **duas** jurisdições, gravação com titular próprio, tradução recente **protegida** no Brasil e `Indeterminada` nos EUA), `ficha_incompleta_nao_vira_domínio_público`, `obra_recente_nao_passa_por_domínio_público`, `extracao_pelo_usuario_nasce_privada_e_nao_publica` e `extracao_do_usuario_bloqueia_publicacao` (portão no sentido negativo).
- **Corpus auditado em teste:** cada ficha resolve domínio público nas duas jurisdições — `auditar_corpus` reprova ficha sem base, sem letra ou sem falecimento.
- Fixture dourada regenerada, agora com os dois eixos: `{work_status: domínio público (vida de John Newton (1807) + 70 anos)}` e `{recording_source: áudio sintetizado pelo próprio teste; 2026; titular: kentron (fixture)}`.

### Ressalvas
- **Corpus brasileiro (Harpa Cristã, Cantor Cristão) não foi incluído.** Esses hinários reúnem **traduções** que podem ter autor recente; entrar exige a ficha do tradutor, e a ficha é o que o `--ficha` calcula e o `--auditar-corpus` confere. É trabalho de conferência, não de heurística.
- **Extrair de verdade ainda não existe** (exige peso no `models/MANIFEST.toml`). O que entrou é o **enquadramento**: responsabilidade declarada e escopo privado, que é o que faltava para o caminho ser previsto sem pôr o produto no lugar do titular.
- O app ainda não honra `lyrics_scope` na exportação nem no link de projeção — hoje quem reprova é o motor.

## [ADR-0012] Crédito não é licença: o argumento do dono, registrado (2026-11-21)
### Decidido
- **Emenda proposta à ADR-0012.** O dono perguntou, com razão de fato: se uma live sem crédito é derrubada, por que extrair o texto do áudio seria diferente de obtê-lo de arquivo ou link, **informando na tela quem detém o direito**? O argumento foi registrado com a resposta, em vez de descartado — para nenhuma sessão futura tratá-lo como exceção óbvia.
- **Crédito e licença são direitos diferentes.** Crédito cumpre o **direito moral** de paternidade; **reprodução** e **comunicação ao público** (tela, projeção, transmissão) são **direitos patrimoniais**. Creditar não autoriza. A queda de live vem, tipicamente, da exigência de identificação/licenciamento da plataforma ou do **Content ID casando o fonograma** — é por isso que a banda ao vivo passa e a gravação original cai.
- **A letra é obra autônoma**, protegida separadamente da melodia e do fonograma. Extrair o texto por ouvido, por OCR ou por **ASR** produz uma **cópia** dela: o meio não muda a natureza do ato. "É praticamente transcrição" descreve o ato, não o defende.
- **O que muda entre os caminhos é de quem é o ato:** do usuário que transcreve para uso próprio, do **produto** que gera e distribui (um produto publicado não invoca uso privado), de ninguém no domínio público, do provedor licenciado, ou **do usuário licenciado** quando a letra é dele sob licença própria.
- **A ADR passa a nomear o caminho que resolve — e não é ASR:** é a **licença que o usuário já tem** (licença coletiva de igreja/CCLI e equivalentes, que autoriza reproduzir e projetar letra). Com ela o app deixa de ser fonte concorrente de letra e vira **ferramenta do licenciado**; o motor agrega o que o texto sozinho não tem — alinhamento ao áudio, dinâmica por linha, estrutura e tempo.
- **Ressalva escrita na ADR:** é registro de decisão de produto, **não parecer jurídico**; o caso concreto (igreja, transmissão, país) quem confirma é a entidade de licenciamento.

## [R4] Comparação medida contra uma cifra humana de referência (2026-11-21)
### Medido
- O dono trouxe uma **cifra feita à mão** (CifraClub) da mesma música analisada no incremento 1, como gabarito. A cifra e a letra **não entram no repositório** (conteúdo de terceiro); o que fica registrado são os números e o método.
- **Cobertura de fundamental: 8/8.** Todas as classes de altura que o gabarito usa (`C, C#, D, E, F#, G, A, B`) aparecem na detecção — e o motor não forjou nenhuma fora delas que mudasse o quadro.
- **Modulação encontrada:** o gabarito tem um Solo 2 em `Am C G D/F#`; o motor marca um trecho de área harmônica distinta em **264–294 s** com `Cmaj7, G, D, A, Em7` — três dos quatro acordes do gabarito ali, sem ninguém ter dito que havia modulação.
- **Menores onde a terça aparece:** `F#m` → `Gbm7/Gbm`, `C#m7` → `Dbm7`, `Bm` → `Bm7`. O `E7M` do gabarito sai como `Emaj7` e o `B4` como `Bsus4`.
- **Não entrega, por escopo declarado:** tablatura de solo, inversões (`E/G#`, `D/F#`, `B/D#` — o motor detecta conjunto de classes, não quem está no baixo), e a notação de tensão (`2`, `4`, `7M` sai como `sus4`/`maj7`/`5`).
- **Erro que a comparação achou:** o `Am` do gabarito é lido como **A maior** (`Amaj7`/`A5`) nesse trecho — a terça menor do A não foi resolvida.
- **Tom:** o gabarito declara `E`; o motor disse `B`. A área harmônica é a de E maior, o palpite de tom não (segue heurística de fundamental mais frequente).

### Corrigido
- **Terça descartada onde ela existia.** Medido nos 91 trechos que saíam como potência (`5`): a razão terça/fundamental tem mediana **0,28** e em **80%** deles a terça fica abaixo de 0,4 — ou seja, o `5` é a leitura honesta na maior parte. Nos 20% em que a terça **está** no áudio, o template de duas notas ganhava por diluição do croma. Agora, com terça acima de 0,4 da fundamental, o motor diz o acorde inteiro: **6 trechos** mudaram (potência 45% → 42%), sem inventar terça em nenhum outro.

## [R4] Letra na cifra: fonte declarada, alinhamento ao áudio e desenho no app (2026-11-21)
### Adicionado
- **`mir_engine::lyrics`** — fonte da letra com **procedência obrigatória** (`do usuário`, `domínio público`, `ASR` declarado e não implementado), leitura de texto livre/ChordPro, **frases vocais** por energia na banda 200–3500 Hz e **alinhamento** das linhas ao tempo, com confiança por linha.
- **Corpus de domínio público embutido** (`assets/hinos_dominio_publico.tsv`): quatro hinos pré-1900 com autor e ano. Regra de entrada escrita no próprio arquivo, e os hinos que **não** podem entrar (tradução inglesa de 1949, obra de 1923 na fronteira) estão lá nomeados como recusa.
- **`mir_engine::sheet`** — monta a cifra como **objeto do domínio** (`notation_bundle::chordpro::Song`) e o acorde vai para a **sílaba** que o tempo dele indica; escrita por `chordpro::write`. `notation_bundle` passa a ser dependência do motor: o formato do app é o mesmo por construção.
- **CLI**: `--letra <arquivo>`, `--hino <título>`, `--listar-hinos` e `--frases-vocais` (esqueleto: onde a voz soa, para encaixar letra própria).
- **Testes**: 4 de letra/sílaba/alinhamento, 3 de fixture/negativos, além dos 4 da análise. O app ganhou `test/cifra_com_letra_test.dart`, que lê a **fixture gerada pelo motor** pela ponte real e desenha no Modo Palco.

### Decidido
- **ADR-0012 (proposta ao dono).** Achado que muda o pedido: **não existe fonte oficial de letra gratuita** — os provedores "oficiais" são pagos e **proíbem armazenar** o texto. Portanto "oficial" e "sem custo legal" são incompatíveis, e o caminho barato e legítimo é texto do próprio usuário + obra em **domínio público**. Raspador de site de letra não é escrito nem para teste.
- O "cruzar a letra com o áudio" do pedido é **alinhamento forçado**, que não depende de licença nenhuma: texto conhecido + áudio.
- **Pressuposto de produto nomeado:** a letra que o usuário já possui é fornecida por ele; o app é a régua que posiciona a letra sobre o áudio, não a fonte de letra de terceiro.

### Corrigido
- **Detector de frase vocal, três vezes.** Começou com limiar sobre o pico global → **uma frase de 385 s em 400** na música real (em mixagem densa a banda da voz nunca cai a 12% do pico). Passou para média local (causal) → no áudio sintetizado **não achava o primeiro bloco cantado**: nível constante nunca fica 15% acima da própria média. Ficou em **queda sob o máximo recente** — presença de voz é *pausa*, e a frase é o que fica entre pausas. Medido: **4 frases casando exato** com 4 blocos cantados (sintético) e **63 frases, 302 s de 400 s** na música real, de 1,5 a 5,8 s cada.
- **Síntese do teste estava errada:** eu "tocava" a tríade em **sequência** (C, depois E, depois G) e o detector lia três acordes onde havia um — a fixture saía `[F][A5][Dsus4]` em vez de `C`. Acorde é simultâneo; com a tríade sustentada a fixture traz a progressão real (C, G, Am, F).
- **Formante senoidal virava nota:** 900 Hz ≈ Lá 5 injetava um A no croma e o acorde de Dó saía `Am7`. O sopro da voz de teste agora é ruído de banda larga.
- **`.cho` com a letra interrompida:** a conferência procurava a frase contígua no arquivo, mas `[C]Ama[G]zing` **quebra** o texto — a verificação passou a ser no texto que o parser separa, que é o que o app desenha.
- **`deny` reprovou o push (2026-11-21):** a dependência de caminho para `notation_bundle` entrou sem `version`, e o `cargo-deny` lê dependência de caminho sem versão como **wildcard** (`wildcards = "deny"`). Mesma armadilha já documentada no `kentron_ffi`; corrigido com `version = "0.0.0"` e o motivo escrito no `Cargo.toml`.

### Verificado
- **Rust:** 11 testes verdes — procedência e recusa do corpus (negativo: hino protegido não está lá), sílaba ("San-to" e não "Sa-nto"; "Bra-vo" com o `r` na sílaba certa), alinhamento em quatro blocos cantados, ida e volta pelo parser do app (`write` → `parse` devolve a mesma música) e **fixture dourada**.
- **App:** 32 testes de Flutter verdes, incluindo os dois novos — o núcleo devolve a letra com os acordes por cima e a **tela do Modo Palco desenha letra e acorde**, com a seção carregando o papel inferido do áudio.
- Fixture única para os dois lados (`crates/mir_engine/tests/fixtures/cifra-com-letra.cho`): o teste dourado de Rust confere contra o gerador, e o teste do app desenha esse mesmo arquivo.

### Ressalvas
- **Alinhamento por frase é aproximado**, não fonema a fonema: sem modelo acústico não há como amarrar sílaba a instante. A confiança por linha existe para o músico saber onde conferir.
- **Música protegida cuja letra o dono não tenha:** o motor entrega o **esqueleto** (onde cada linha cai, seção, dinâmica), não as palavras. É limitação declarada, não defeito.
- Detector afinado em **dois** casos (um sintético, uma música real). O repertório inteiro é medição pendente.
- Tom estimado segue sendo a fundamental mais frequente (a fixture devolve `Am` onde a música está em Dó).

## [R4] Motor MIR: cifra, estrutura e dinâmica a partir do áudio (2026-11-21)
### Adicionado
- **`crates/mir_engine`** — primeiro incremento do motor de análise (ADR-0011): lê WAV PCM, calcula **dinâmica** (EBU R128 por pesagem K, blocos de 400 ms, portão absoluto e relativo, LRA, pico de amostra), **andamento** (autocorrelação da novidade espectral) e **acordes** (croma por picos interpolados + casamento por template, com confiança por trecho).
- **Estrutura**: cortes por pico de novidade, agrupamento por auto-similaridade do croma e rótulos por repetição (`A`/`B`/`C`) com papel **inferido** (introdução, refrão candidato, seção repetida, interlúdio candidato, final).
- **CLI `mir`**: resumo legível, `--json` (relatório completo) e `--chordpro` (cifra por seção).
- `Report::tom_estimado` e `Report::para_chordpro` — a cifra sai em ChordPro com `{key}`, `{tempo}` e `{duration}`.

### Decidido
- **ADR-0011 · MIR: primeiro incremento da análise de áudio.** O R4 foi antecipado como instrumento de trabalho (decisão do dono), **fora do portão da R1**: é DSP puro, **sem nenhum modelo** — nada a registrar no `models/MANIFEST.toml`. Letra (ASR) e tablatura (pitch tracking + corda/traste) ficam para incrementos próprios, cada um com o peso declarado antes de entrar.
- R128 escrito à mão em vez do crate `ebur128` que o plano citava: menos dependência externa e a garantia vem de **contraprova independente** (o `ffmpeg` mede o mesmo arquivo), não de confiar no próprio código.
- Saída é **candidata com confiança**, nunca verdade: em mixagem real o acorde detectado erra, e o relatório diz isso.

### Corrigido
- **C3 virava B** na detecção de acorde: o bin da FFT (10,8 Hz em 44,1 kHz/4096) é mais largo que um semitom abaixo de ~180 Hz e a classe de altura era truncada. Agora o croma sai de **picos com interpolação parabólica** e arredondamento de classe — achado pelo teste em áudio sintetizado.
- **Estrutura degenerada no primeiro áudio real**: com mínimo de 4 s o motor cortava a cada troca de acorde e devolvia 30+ "trechos", todos rotulados refrão. Mínimo passou a 15 s, limiar de novidade a 1,5σ e o refrão é o rótulo **mais repetido** (os outros viram "seção repetida").
- **`{key: B5}`** na primeira cifra gerada: o extrator de fundamental cortava na primeira minúscula, então `Db` devirava `D` e `B5` tinha o `5` como parte da fundamental. Corrigido com parse explícito de fundamental/qualidade.
- **Teste de contraprova não era hermético** (apontado pelo radar de skills no push): o processo filho passou a declarar o próprio ambiente (`env_clear`, `PATH` explícito, `LC_ALL=C`) e o arquivo de prova leva o PID — nome fixo em `/tmp` colide se duas sessões rodarem o hook ao mesmo tempo. Conferido que a contraprova **roda** nesse desenho (sem o aviso de `ffmpeg` ausente, que também deixaria o teste verde).

### Verificado
- **Áudio sintetizado** (progressão C–G–Am–F a 120 BPM, com final esmaecendo): 4 testes — leitura/escrita de WAV ida e volta, acordes e andamento reconhecidos, introdução/final e cauda mais baixa, e cifra sem dígito no tom nem acorde repetido em sequência.
- **Contraprova do LUFS contra o `ffmpeg`** (dois medidores independentes): sintético dentro de 1 LU; **no áudio real, −12,6 LUFS (motor) contra −12,0 LUFS (ffmpeg)** — 0,6 LU de diferença.
- Áudio real de 400,4 s analisado em **0,77 s** (build release), com `cargo fmt`, `cargo clippy -D warnings` e `cargo test` verdes.
- Achado musical: a sequência detectada (E, B, F#/Gb, C#m/Dbm, A e, num trecho, D, Cmaj7, G, Em7) é I–V–vi–IV em E maior — o motor acha a região harmônica certa, com a qualidade "5" (power chord) por a terça ficar mascarada na mixagem.

### Ressalvas
- **Letra e tablatura não estão entregues** — são os próximos incrementos; sem eles, "cifra completa com letra" não existe.
- O tom estimado é a **fundamental mais frequente**, não análise de centro tonal: numa música em E maior o motor devolveu `B`.
- O pico reportado é **pico de amostra**, não pico verdadeiro (exigiria sobreamostragem com filtro polifásico).

## [processo] Dicionário de variantes: uma língua, N variantes, sem duplicar texto (2026-11-21)
### Adicionado
- `tools/variantes.tsv`: pares de formas equivalentes entre variantes da mesma língua (`pt`↔`pt_PT`, `es`↔`es_ES`). É dado, não código — acrescentar uma linha muda o gate.
- Duas regras no gate `arb`: **bloqueia** forma da variante no arquivo base e **avisa** quando a variante não sobrescreve uma chave cuja base carrega forma local.
### Corrigido
- O radar achou sozinho um defeito real: `pt_PT` não cobria `stageProjectNext`/`stageProjectPrevious` — em Portugal escreve-se "secção". As duas chaves entraram no `app_pt_PT.arb`.
- `.githooks/pre-push` chama o gate `arb` **direto**, não via `run_gate`: aquele guarda a saída e só a mostra quando o gate falha, e este gate também emite avisos quando passa. Radar invisível é radar nenhum (mesma razão do `skills-radar`).
### Verificado
- Regra 1 no negativo: "Ecrã" injetado em `app_pt.arb` → `❌ PUSH BLOQUEADO — gate(s) reprovado(s): arb`, uma única mensagem apesar de duas linhas do dicionário apontarem para "ecrã".
- Regra 2 no negativo: removendo os dois overrides do `pt_PT`, o push **passa** e imprime os 2 avisos nomeando a chave e a forma esperada.
- Positivo: gate e hook verdes com os cinco ARBs consistentes.

## [processo] `pt` é pt-BR por gate, não por prosa (2026-11-21)
### Adicionado
- `tools/check_arb.py` reprova **grafia de português europeu** no `app_pt.arb` (`ecrã`, `secção`, `ficheiro`, `utilizador`, `telemóvel`, `autocarro`, `connosco`, `receção`). A variante europeia continua livre em `pt_PT`.
### Decidido
- O dono do projeto fixou em 2026-11-21: **sem duplicação** de ARB, `pt` segue a ortografia do português brasileiro e isso não é negociável. A prosa virou mecanismo porque a grafia não se defende com aviso.
### Verificado
- Sentido positivo: `check_arb.py` verde com os cinco ARBs atuais (`pt_PT` pode usar "Ecrã").
- Sentido negativo: com "Ecrã" injetado em `app_pt.arb`, o gate **reprovou** (`gearProjectionScreen usa grafia de português europeu ('Ecrã')`); restaurado o arquivo, voltou a passar.

## [processo] Prova de navegador, locales e piloto automatizado (2026-11-21)
### Adicionado
- **Prova de navegador** (Playwright + Chromium headless) sobre o build web: home, leitor, transposição, projeção, avanço/volta de seção, contraste, bootstrap por URL e cópia do link — com screenshots. Evidência do link: o clipboard passou de vazio para `…/?projecao=<base64 do ChordPro>`.
- **ADR-0010 · Piloto automatizado antes do uso ao vivo:** o portão R1-D passa a ser automatizado (VM Windows + OBS via Browser Source) com duas rodadas consecutivas simuladas; o uso ao vivo vem depois e realimenta a próxima release.
### Corrigido
- `apps/kentron_app/pubspec.yaml` não tinha a seção `flutter:`, então `uses-material-design` estava desligado e a **fonte de ícones não era empacotada** — no web os ícones saíam como "tofu". Achado pela prova de navegador e corrigido (tree-shake de 1,6 MB → 2 KB).
- O **AppBar do Modo Palco** não acompanhava o tema: barra clara sobre corpo escuro. Corrigido e coberto por teste.
- `docs/architecture/README.md` listava `musicxml` como entregue; é **stub** — o contrato existe e recusa com erro explícito.
### Decidido
- Locale de faixa 1 conforme a ADR-0002 (pt-BR, en-US, es-419). O `gen-l10n` **exige arquivo-base por idioma**, então os ARBs seguem `pt`/`en`/`es` e a correspondência (`pt` = pt-BR, `es` = es-419) fica explícita no `AGENTS.md` — um `pt_BR` sem `pt` faz o gerador abortar.
### Verificado
- 30 testes de app; `flutter analyze` limpo; `flutter build web --release` verde com a fonte de ícones empacotada.

## [R1-B] Browser Source do OBS por URL
### Adicionado
- `projection_link.dart`: `chordproDaUrl` (lê `?projecao=<base64url do ChordPro>`) e `urlDaProjecao` (monta a URL preservando host, caminho e fragmento). Base64url evita escapes de `&`, `#` e acentos.
- `projection_bootstrap.dart`: `ProjecaoApp` abre direto na projeção, sem tela inicial, com carregador injetável e falha visível.
- `main.dart` detecta a URL de projeção e sobe o app mínimo; o leitor ganhou "Copiar link de projeção" (web).
- Chaves `stageCopyProjectionLink` e `stageProjectionLinkCopied` nos três ARBs-base.
### Verificado
- 5 testes de URL (round-trip com acentos, URL inválida, preservação de host/caminho/fragmento, troca de query) e 2 de bootstrap (abre na projeção; falha do núcleo visível).
- Suíte do app: **29 testes**; `flutter analyze` limpo; `flutter build web --release` compilou.
### Nota
- O round-trip **dentro do OBS** não foi executado; o build web prova compilação, não o Browser Source em uso. A prova no OBS segue pendente.

## [R1-C] Modo projeção manual em tela cheia
### Adicionado
- `ProjectionPage` (`apps/kentron_app/lib/src/stage/projection_page.dart`): uma seção por vez, sem acorde, com avanço por toque (metade esquerda/direita), setas do teclado e rodapé com contador e botões anterior/próxima. Abre pelo botão **Projetar** no leitor.
- Chaves `stageProject`, `stageProjectNext` e `stageProjectPrevious` nos três ARBs-base.
### Verificado
- 5 testes herméticos do modo projeção (render sem acorde, avanço/volta, limites desabilitados, toque e teclado) + 1 prova de abertura a partir do leitor.
- Suíte do app: **22 testes** passando; `flutter analyze` sem achados.
### Nota
- O modo em tela cheia é o **conteúdo** da projeção; o transporte para tela secundária ou janela do OBS segue aberto e não foi marcado como entregue.

## [R1-C] Leitor de cifras (Modo Palco) entregue e ligado à tela inicial
### Adicionado
- **`SongReaderPage`** (`apps/kentron_app/lib/src/stage/song_reader_page.dart`): desenha a cifra inteira, acorde sobre a letra, com transposição por tom, alternância de contraste e erro do núcleo visível (não engolido). Carregador, transpositor e tons são injetáveis para os testes herméticos.
- **`chordLine`** (`.../stage/chord_line.dart`): alinha os acordes nas colunas que o núcleo devolve; acorde que invadiria o anterior é empurrado para a direita.
- **Botão "Abrir cifra"** na tela inicial, com chave de tradução `stageOpenReader` nos três ARBs-base (`en`, `pt`, `es`); variantes regionais herdam.
- **Chaves novas** `stageContrast`, `stageKey`, `stageLoadFailed`, `stageOpenReader` em `core_l10n`.
### Verificado
- `flutter analyze` sem achados; **16 testes** de app passam (5 novos do leitor, 3 provas reais na ponte, 8 anteriores).
- Provas reais na ponte: `readSong` devolve seção/linha/coluna; `transposeSong` muda o tom e os acordes; `keyChoices` devolve os doze tons.
### Nota
- O `cdylib` estava mais velho que `bridge.rs` (21:09 vs 23:35); reconstruído antes dos testes para não provar contra binário obsoleto.
- `gen-l10n` e `analyze` no mesmo lote disputaram o lock do Flutter: comandos dependentes precisam ser sequenciados.

## [processo] ROADMAP corrigido: portão da R1 em dois, alpha antes do piloto (ADR-0009)
### Adicionado
- **`R1-0 · Fundação técnica (entregue)`** no ROADMAP: ponte FFI, alvos Android/iOS/Web, Native Assets e `HomeScreen` consumindo o núcleo — entregas que existiam só na CONCLUSION e não apareciam na direção.
- **`R1-C · Alpha utilizável`** (portão inicial) e **`R1-D · Piloto e release`** (portão final) no ROADMAP e no plano v2.6.
### Alterado
- O piloto de dois cultos consecutivos, o zero-crash e a revisão por nativos saíram do portão inicial e passaram a ser o **último** marco da R1 (R1-D), depois do R1.1. O portão de release da ADR-0002 não muda.
- "Portão de Entrada" da R1 deixou de ser o spike S5: ele dimensiona MusicXML/ATEX (R1.1) e não bloqueia alpha nem piloto, como a ADR-0005 já dizia.
### Decisão
- [ADR-0009](docs/adr/0009-portao-inicial-da-r1-alpha-antes-do-piloto.md) aceita: o portão caro não pode ser o primeiro, porque depende de calendário de igreja e trava o ciclo de feedback em duas semanas.

## [processo] `origin/HEAD` corrigido para `main` (2026-11-21)
### Alterado
- `git remote set-head origin -a` no clone principal: `origin/HEAD` apontava para `master` e agora aponta para `main`. **O GitHub já estava certo** — `gh repo view` devolve `defaultBranchRef = main` e `git ls-remote --symref origin HEAD` devolve `refs/heads/main`; o desatualizado era o ref **local**, que não acompanha mudança de default no remoto.
### Achado
- `origin/HEAD` é ref local (`refs/remotes/origin/HEAD`): trocar o default no GitHub não o atualiza. `git remote set-head origin -a` relê do remoto e corrige.
- `master` no remoto é branch **órfã**: 1 commit (`4b10a6d Initial commit`), raiz própria, **não** ancestral de `main` (80 commits). Apagar é decisão separada.

## [processo] Contrato de validação declarado: gitleaks, não a bateria inteira (2026-11-21)
### Alterado
- `.harness.toml` `[validation]`: `configured = true` com um gate explícito, `gitleaks` (mesmo argv do `gate_gitleaks` do pre-push). Antes o contrato estava vazio e o `.harness/validation.py` recusava rodar por desenho.
- Decisão do dono do projeto: a bateria completa **não** entra no contrato — ela já é aplicada pelo `.githooks/pre-push` em cada push; repeti-la dobraria o custo sem acrescentar cobertura. Higiene de dispositivos também não vira gate de `pre-push`: derrubar dispositivo dentro de um comando de verificação é destrutivo.
### Verificado
- `python3 .harness/validation.py check` → `gates aprovados: 1`, `EXIT=0` (80 commits, nenhum vazamento).
- Sentido negativo do contrato: gate trocado por `false` → `gates reprovados: gitleaks (exit 1)`, `EXIT=1`.
- Sentido negativo do gitleaks: repositório temporário com uma chave de formato AWS (`AKIA` + 16 caracteres) → `leaks found: 1`, `EXIT=1`.
### Achado
- A chave de exemplo canônica da AWS (a que termina em `EXAMPLE`) **não** é detectada: está na allowlist padrão do gitleaks. Canário com ela passa em falso e não prova nada.
- O literal do canário não entra na documentação: o `semgrep` bloqueia `AKIA` + 16 caracteres em qualquer arquivo, inclusive `.md`. O push foi bloqueado por isso e a entrada foi reescrita sem o valor.

## [processo] Perfil do projeto preenchido e shellcheck vira gate (2026-11-20)
### Adicionado
- `shellcheck` nos dois pontos onde a bateria roda: `scripts/ci-local.sh` (todos os `scripts/*.sh`) e `.githooks/pre-push` (só os `.sh` que o push toca, mesmo recorte dos gates de lint por stack). O workflow `.github/workflows/ci.yml` instala a ferramenta antes de chamar o script.
### Alterado
- `AGENTS.md` deixou de ter placeholders: "Perfil do projeto" declara topologia, estrutura, branch/remote, stack, comandos de validação, idiomas e autoridades de aprovação; "Comunicação" fixa português (Brasil); "Release" diz o estado real (sem release publicada, `0.0.0` nos dois manifestos).
- `.githooks/pre-push`: `cd "$REPO_ROOT"` passou a `cd "$REPO_ROOT" || exit 1` (SC2164) — sem isso, um `cd` que falha deixava os gates rodando no diretório errado, em silêncio.
### Verificado
- `shellcheck scripts/*.sh` → `EXIT=0` (três scripts, zero achados).
- Gate `shell` provado no sentido negativo: com uma linha defeituosa injetada em `scripts/encerrar-dispositivos.sh`, o hook reprovou (`❌ shell`, `PUSH BLOQUEADO`, `HOOK_EXIT=1`) e o log diagnóstico mostrou SC2154/SC2086; com o arquivo restaurado, o mesmo hook passou (`✅ shell`, `HOOK_EXIT=0`).
### Ressalva
- O hook em si continua com 17 achados de `shellcheck` (SC2329 é falso positivo de função chamada por `run_gate`; SC2001 é estilo). Por isso o gate cobre `scripts/*.sh`, não o hook.

## [processo] Lições da sessão viram skills e um mecanismo (2026-11-20)
### Adicionado
- Quatro skills em `.agents/skills/`: `empacotamento-rust-flutter` (Native Assets, predicados técnicos e erros com assinatura), `higiene-de-dispositivos` (encerrar emulador/simulador depois da prova), `lote-nao-e-sequencia` (comando agrupado roda em paralelo) e `toolchain-unica` (duplicata de binário decide o PATH).
- `scripts/encerrar-dispositivos.sh`: detecta, encerra e verifica emulador Android e simulador iOS; `--dry-run` mostra o que faria; sai 1 se algo sobrar. Dispositivo físico é ignorado com aviso.
- `vies-de-plausibilidade` §11 (o diff tem direção e o olho lê ao contrário); `toolchain-unica` ganhou o caso do cask que resolve o SDK root para si mesmo; `quadro-de-trabalho` registra a ordem do commit do quadro.

### Removido
- Branches já mergeadas, com autorização do dono do projeto: `spike/native-assets` (local e `origin`), `feature/alvo-android`, `feature/alvo-ios`, `feature/alvo-web` e `feature/ponte-rust-flutter`.

### Verificado
- `scripts/encerrar-dispositivos.sh` no sentido negativo: com o emulador `kentron` no ar, detectou `emulator-5554`, encerrou e o `adb devices` seguinte ficou vazio; `--dry-run` e o caminho sem `adb` também exercitados.
- `adb devices` e `xcrun simctl list devices booted` vazios ao fim da sessão.

### Achado que custou tempo
- `git branch -d` recusou `spike/native-assets` porque o ref remoto rastreado ficou obsoleto depois de a remota ser apagada. Ordem correta: `git fetch --prune origin` antes do `-d` (ou apagar a local primeiro).

## [R1] Empacotamento do Rust por Native Assets (ADR-0008 aceita) (2026-11-20)
### Alterado
- O Rust deixou de entrar no app como **plugin** (cargokit, CocoaPods-only) e passou a entrar como **code asset** por build hooks: `apps/kentron_app/hook/build.dart` → `native_toolchain_rust`, com `flutter_rust_bridge_hooks: 2.13.0`.
- `apps/kentron_app/rust_builder/` removido; `pubspec` sem a dependência de path e `analysis_options` sem o exclude do cargokit.
- `rust-toolchain.toml` passou de `stable` (flutuante) para **`1.97.1` pinado**, com a lista de alvos; **cópia dentro de `crates/kentron_ffi/`**, porque o `native_toolchain_rust` valida a toolchain no diretório do crate, não subindo a árvore.
- `ios/` regenerado **sem CocoaPods**: sem `Podfile`, sem `Podfile.lock`, sem `Pods/`.

### Verificado
- `flutter build ios --debug --simulator` → `✓ Built Runner.app` (23,9 s) com **zero** ocorrências do aviso de SPM; app aberto no simulador mostrando os dados do núcleo.
- `flutter build apk --debug` → `✓ Built app-debug.apk` (32,7 s); app rodando no emulador com `libkentron_ffi.so` dentro do APK.
- `flutter test` 8/8 e `scripts/ci-local.sh` `EXIT=0` depois da migração.

### Decisão
- **ADR-0008** aceita pelo dono do projeto em 2026-11-20. Reversão é um commit (`git revert`), porque o commit do cargokit continua no histórico.


## [manutenção] Runtime de simulador único e o caso do SPM (2026-11-20)
### Alterado
- Removido o runtime iOS redundante **`24B91016s`** (seed, 7,7 GB): ficou o **`24B8117`**, onde está o iPhone 17 usado nas provas. `xcrun simctl list runtimes` agora lista um só.

### Investigado (com medição — e uma hipótese minha derrubada)
- O aviso *"plugins do not support Swift Package Manager … will become an error"* **não** é silenciado por `flutter config --no-enable-swift-package-manager`: com a flag desligada o build continuou verde **e o aviso continuou aparecendo** — é alerta informativo da resolução de plugins, independente da flag. A configuração foi **restaurada ao default** (`flutter config --clear-features`, que preserva `android-sdk` e `jdk-dir`).
- `flutter_rust_bridge_codegen integrate` do **2.13.0** não tem opção de SPM e não gera `Package.swift`; o cargokit não tem builder SPM (só pod/gradle/cmake). Upstream o suporte existe (issues #3036 e #3202, fechadas) → o caminho durável é subir o gerador quando houver versão **estável** com SPM.

## [R1] iOS: o Rust entra no app e o app roda no simulador (2026-11-20)
### Adicionado
- `apps/kentron_app/ios/`: scaffold (`flutter create --platforms=ios --org com.kentron`), bundle id `com.kentron.kentronApp`; o `pubspec` saiu **idêntico**.
- `crates/kentron_ffi`: **`staticlib`** no `crate-type` (`["lib", "cdylib", "staticlib"]`) — o Android segue no `cdylib` (`.so`) e o iOS usa o `.a`.

### Verificado
- `flutter build ios --debug --simulator` → `✓ Built build/ios/iphonesimulator/Runner.app` (Xcode 27.2, 24,6 s).
- No simulador (iPhone 17): `simctl install` + `launch` (PID 53043); o print mostra `Exemplo`, `Kentron · tom C · 2 seção(ões)`, os acordes `C G Am F` e `núcleo: kentron_ffi 0.0.0` — o núcleo respondendo no iOS.
- **Regressão do Android revalidada** com o novo `crate-type`: `flutter build apk --debug` verde.

### Achado que custou tempo (com a correção)
- `Exception: Unable to find bundle for dynamic library` (`cargokit/build_pod.dart:86`). O cargokit **usa o `.a` se ele existe** (`if (staticLibs.isNotEmpty)`) e só cai no caminho de Framework quando não existe; como o crate declarava apenas `cdylib`, não havia `.a` no alvo iOS e ele exigia `kentron_ffi.framework/…`. Ler o código do cargokit deu a correção exata, em vez de tentativa e erro.

### Risco registrado
- O Flutter avisa que **`kentron_ffi` não suporta Swift Package Manager** ("will become an error in a future version of Flutter"): o cargokit é CocoaPods-only. Quando o Flutter virar a chave para SPM, o plugin precisará de SPM (ou o iOS terá de continuar em CocoaPods explicitamente).

## [R1] Android: o Rust entra no APK e o app roda no emulador (2026-11-20)
### Adicionado
- `apps/kentron_app/android/`: scaffold (`flutter create --platforms=android --org com.kentron`), appId `com.kentron.kentron_app` — sem `com.example`, que o Play rejeita. O `pubspec` saiu **idêntico** e o `analysis_options` só ganhou `android/**` nos excludes.
- `apps/kentron_app/rust_builder/`: plugin **cargokit** criado pelo `flutter_rust_bridge_codegen integrate --rust-crate-name kentron_ffi --rust-crate-dir ../../crates/kentron_ffi --platforms android --no-write-lib --no-integration-test`. O caminho do crate fica nos scripts (`manifestDir = "../../../../crates/kentron_ffi"`).
- Dependência `kentron_ffi: path: rust_builder` no `pubspec`, comentada com o comando que a gera.

### Verificado
- `flutter build apk --debug` → `✓ Built .../app-debug.apk` (137,8 s); `unzip -l` mostra `lib/arm64-v8a/libkentron_ffi.so`, `lib/armeabi-v7a/libkentron_ffi.so` e `lib/x86_64/libkentron_ffi.so`.
- No emulador `kentron`: `adb install` ok; `am start -W` → `Status: ok`, `LaunchState: COLD`, 1754 ms; processo vivo e **nenhum** `UnsatisfiedLink`/`FATAL`. O print mostra `Exemplo`, `Kentron · tom C · 2 seção(ões)`, os acordes `C G Am F` e `núcleo: kentron_ffi 0.0.0`.

### Achados que custaram tempo
- **Java 27 quebra o Gradle 9.5**: `BUG! ... Unsupported class file major version 71`. Corrigido com **JDK 21** (`brew install openjdk@21` — formula, sem sudo) + `flutter config --jdk-dir`. O Flutter já avisava: *"Newer than known valid Java version (27.0.1)"*.
- O `integrate` **exige `flutter` no PATH** e morre no meio sem ele: a primeira rodada criou o `rust_builder` mas deixou o `pubspec` sem a dependência; a segunda, com PATH, fechou.
- O cargokit compila **quatro** ABIs (inclui `i686-linux-android`) e **instala sozinho** o alvo Rust que falta — mesmo padrão de auto-instalação do `cargo-expand`/`wasm-pack`; divulgar sempre.

### Ferramentas que o pipeline agora depende (medidas)
- Flutter do projeto em `~/development/flutter` (o clone que o `.flutter-version` pina), agora **à frente no PATH** do `~/.zshrc`; os casks duplicados (`flutter`, `android-platform-tools`) foram removidos pelo dono.
- JDK 21 via `flutter config --jdk-dir`; Gradle 9.5 e AGP 9.3.1 conforme o template do Flutter 3.50.1.

## [R1] Android local pronto e ponte provada no navegador (2026-11-20)
### Verificado
- **Prova no navegador:** a tela web mostrou dados vindos do núcleo Rust — `Exemplo`, `Kentron · tom C · 2 seção(ões)`, acordes `C G Am F` e `núcleo: kentron_ffi 0.0.0`. Cadeia Rust → wasm → Flutter web → tela provada no browser.

### Toolchain Android (medida)
- `flutter doctor`: **Android toolchain ✓** (SDK 36.0.0), com Java **Temurin 27**; comando de versões tirado do próprio SDK: compileSdk/targetSdk 36, minSdk 24, **NDK 28.2.13676358**, Gradle 9.5.0, AGP 9.3.1, KGP 2.4.20.
- Pacotes instalados em `~/Library/Android/sdk`: build-tools 36.0.0, emulator 37.3.10, NDK 28.2.13676358, platform-tools 37.0.3, platforms;android-36, system-images;android-36;google_apis;arm64-v8a. AVD **`kentron`** criado.
- `flutter config --android-sdk` aponta para esse SDK.

### Armadilhas registradas
- O `avdmanager` **do cask** resolve o SDK root pela própria localização (o cask), onde não há system-images — daí `Package path is not valid ... null`. O SDK root só fica íntegro com uma **cópia real** de `cmdline-tools` dentro dele (symlink não engana a detecção). O SDK em `~/Library/Android/sdk` também fica imune a `brew upgrade`.
- `flutter doctor` avisa de **dois `adb`** (SDK + cask `android-platform-tools`) e existem **dois Flutter** (o do pipeline em `~/development/flutter` e o do cask). Ver PENDING.

## [R1] Tela inicial consumindo o núcleo Rust (2026-11-20)
### Adicionado
- `HomeScreen` deixou de ser estática: chama `parseSong` e mostra título, artista/tom/seções, os acordes em chips e **qual núcleo respondeu** (`núcleo: kentron_ffi …`) — a ponte visível para quem usa o app.
- `cifraDeExemplo`: cifra **só de acordes** (sem letra de terceiros) usada pela tela; o objetivo é provar a ponte, não entregar repertório.
- Carregador injetável (`carregar`) em `KentronApp`/`HomeScreen`: o default é o núcleo real, e os testes herméticos injetam outro.

### Verificado
- `bridge_test.dart`: teste de **widget com o `cdylib` real**, provando a tela renderizar o parse do Rust (título, tom, seções, acordes e o núcleo que respondeu). `flutter test`: 8/8 verdes; `flutter analyze`: sem issues.
- O parse de linhas **só com acordes** (`[C] [G] [Am] [F]`) ficou provado por esse teste.

### Decisão de teste
- O teste com a ponte real **não** usa `pumpAndSettle`: o indicador de carregamento anima indefinidamente e a resposta chega por uma porta do FFI, que o relógio falso do teste não avança. A espera é em tempo real, via `runAsync`, até o indicador sair da árvore.

### Não provado
- A execução **no navegador**: o `build/web` está servível, mas ninguém abriu ainda. Ver a nota no PENDING sobre canvas/DOM antes de investir em E2E automatizado.

## [R1] Alvo web do app (2026-11-20)
### Adicionado
- `apps/kentron_app/web/`: scaffold web (`flutter create --platforms=web`) com `index.html` carregando o glue wasm-bindgen (`pkg/kentron_ffi.js`) e título/descrição do Kentron.
- `scripts/servir-web.py`: servidor estático que envia `Cross-Origin-Opener-Policy: same-origin` e `Cross-Origin-Embedder-Policy: require-corp` (sem eles não há `SharedArrayBuffer` e a ponte wasm não carrega), além de `application/wasm`; falha alto com instrução quando o build não existe.

### Alterado
- `apps/kentron_app/analysis_options.yaml`: `web/**` fora do analyzer (o próprio `flutter create` acrescenta).
- Removido o `README.md` boilerplate que o `flutter create` gerou: documentação vive em `docs/`.

### Verificado
- `flutter build web --release` → `✓ Built build/web`; servido: `/` 200 com COOP, `/pkg/kentron_ffi_bg.wasm` 200 `application/wasm`, `/pkg/kentron_ffi.js` e `/main.dart.js` 200, `index.html` com a tag da ponte. Negativo: `servir-web.py` com dir inexistente → exit=1.

### Pré-requisitos (medidos)
- `build-web` compila com `-Z build-std` para atomics/shared memory: exige toolchain **nightly** com `rust-src` e o alvo `wasm32-unknown-unknown`.
- O próprio gerador **auto-instala** ferramentas quando faltam: `cargo-expand` 1.0.127 e `wasm-pack` 0.15.1 (divulgado).

### Decisão
- O build web **não** entra no `ci-local.sh` por ora: exigiria nightly + `build-std` em toda bateria, custo alto para o benefício atual. Revisar quando o alvo web passar a ser entregue.

## [R1] Ponte Rust ↔ Flutter real (ADR-0007 aceita, 2026-11-20)
### Adicionado
- `crates/kentron_ffi`: crate adaptador — o único `cdylib` do workspace e o único a depender do `flutter_rust_bridge` 2.13.0. Expõe `parse_song(input) -> SongSummary` (título, tom, artista, seções, acordes e qual núcleo respondeu).
- Bindings gerados (`crates/kentron_ffi/src/frb_generated.rs` + `apps/kentron_app/lib/src/rust/`) e `flutter_rust_bridge.yaml` com o comando de regeração.
- `apps/kentron_app/test/bridge_test.dart`: carrega o `cdylib` e chama o núcleo de verdade — prova a ponte, não um dublê.

### Alterado
- `notation_bundle` volta a ser domínio puro: sai o `cdylib` e a dependência de FFI, e `unsafe_code = "forbid"` segue valendo.
- `scripts/ci-local.sh` e `.githooks/pre-push`: constroem o `cdylib` antes dos passos do app, porque o `flutter test` carrega o artefato.

### Emenda à ADR-0007 (o que a medição obrigou)
- A cláusula "`unsafe_code = "forbid"` permanece" era **inalcançável**: a cola gerada tem um bloco `unsafe` e `forbid` não pode ser relaxado por `allow`. O FFI passou para `crates/kentron_ffi` com `unsafe_code = "deny"` e `#[allow(unsafe_code)]` escopado ao módulo gerado — o domínio mantém `forbid`.

### Validação
- 58 testes Rust (3 do adaptador) e 5 de Flutter (2 da ponte) verdes; `cargo clippy --workspace --all-targets -- -D warnings` limpo; `ci-local.sh` verde.

## [R0] core_l10n integrado ao app (2026-11-20)
### Adicionado
- `packages/core_l10n/lib/core_l10n.dart`: caminho único de import das localizações geradas — o app deixa de conhecer o caminho do arquivo gerado.
- ARB `homeWelcome` (en/pt/es) e a tela inicial usando a string do ARB no lugar de texto fixo.
- Testes do app cobrindo en, pt e es na mesma tela, mais o fallback regional `pt_PT` → `pt`.

### Alterado
- `apps/kentron_app` passou a depender de `core_l10n` e monta o `MaterialApp` com `CoreL10n.delegate` e `...GlobalMaterialLocalizations.delegates`; `supportedLocales` vem de `CoreL10n.supportedLocales`.
- `scripts/ci-local.sh`: a geração do `core_l10n` passou a ser **no lugar** (não mais em cópia temporária) e **antes** dos passos do app, que precisam do arquivo gerado.
- `.githooks/pre-push`: o gate de Flutter gera o `core_l10n` antes de analisar e testar — sem isso o push reprovaria por um arquivo que o pipeline encontra pronto.

### Descoberto (e registrado no PENDING)
- O `MaterialLocalizations` do `material_ui` **não** é satisfeito pelas `Global*` do `flutter_localizations` que o `gen-l10n` emite: o app reprovava com "No MaterialLocalizations found". A saída é usar as `Global*` do `material_ui` — e isso revisou o item que pedia "migrar `core_l10n` para `material_ui`", que **não** é resolvível no `pubspec` do pacote.


### Alterado
- `.github/workflows/ci.yml` deixou de duplicar a matriz: instala Rust, Flutter (3.50.1, casando com `.flutter-version`) e `cargo-deny`, e chama `./scripts/ci-local.sh`. Quatro jobs viraram um; a cobertura é a mesma, agora definida num lugar só.
- Gatilhos preservados: `pull_request`, `workflow_dispatch` e `schedule` semanal — segue sem disparar em push para a `main` (ADR-0004).

### Corrigido
- O risco de divergência silenciosa entre script e workflow (item aberto no PENDING e na tabela de riscos do plano v2.6) deixa de existir: não há duas cópias para divergir.

## [Infra] Duas lições do kentron publicadas no sovereign-harness 0.7.2 (2026-11-20)
### Publicado
- Release [v0.7.2](https://github.com/personal-digital-sovereignty/sovereign-harness/releases/tag/v0.7.2): skill `estado-compartilhado-multi-agente` §1c (PR [#21](https://github.com/personal-digital-sovereignty/sovereign-harness/pull/21)) e o gate de lint lendo a edição do manifesto (PR [#22](https://github.com/personal-digital-sovereignty/sovereign-harness/pull/22), tag `v0.7.2`), com teste novo provado no negativo.
- Versões → 0.7.2 nos 7 manifestos do kit; `sincronizar --verificar` 45/45; CI verde em ubuntu e macos.
- O projeto e o kit voltam a concordar sobre a edição do `rustfmt` — a correção do kentron (`a68c85c`) e a do template (0.7.2) são a mesma leitura do `Cargo.toml`.

## [Infra] Gate de lint do pre-push alinhado à edição do crate (2026-11-20)
### Corrigido
- `.githooks/pre-push` formatava com `rustfmt --edition 2021` **fixo**, enquanto o workspace é edition 2024 e o `ci-local.sh` usa `cargo fmt` (que lê a edição real). Os dois gates do projeto passaram a discordar sobre a ordenação de itens de import e o push era bloqueado por arquivo que o próprio pipeline aprovava.
- A edição passa a ser lida do `Cargo.toml`. Medição que fecha o diagnóstico: **mesmo arquivo e mesmo `rustfmt`** — `--edition 2024` limpo, `--edition 2021` com diff. Provado no negativo: um arquivo deliberadamente mal formatado continua reprovando.

## [R1] Song Bundle v1: persistência (ADR-0006 aceita, 2026-11-20)
### Adicionado
- Módulo `bundle_store`: bundle em diretório com `meta.json` versionado (`schema_version`), `song.cho` como fonte de verdade do conteúdo, índice da biblioteca (`library.json`), setlists (`*.json`) e exportação por cópia do diretório.
- Serializador ChordPro (`chordpro::write`): metadados em ordem determinística e cada acorde recolocado na posição original; ida e volta pelo parser preserva o modelo.
- Cadeia de migração por degrau (`MIGRATIONS`), vazia hoje porque v1 é o primeiro esquema: versão sem degrau é recusada em voz alta, nunca lida pela metade.
- Primeira dependência externa do workspace: `serde` + `serde_json`, 11 pacotes com transitivas, aprovados pelo `cargo deny`.

### Validação
- 55 testes no workspace (12 novos: 4 do serializador ChordPro e 8 de persistência, com diretório descartável). `ci-local.sh` verde e `cargo deny check` ok.

### Premissa corrigida
- "Persistir exige embutir o modelo em JSON": o conteúdo vai como `.cho` (texto) e o JSON carrega só identidade — o modelo em `bundle` segue sem nenhum atributo de serialização.

## [R1] Song Bundle v1: modelo em memória (2026-11-20)
### Adicionado
- Módulo `bundle` em `crates/notation_bundle`: `SongId`, `SetlistId`, `SongBundle`, `Library`, `Setlist` e `BundleError` com `message_key()` estável.
- 14 testes: 11 unitários (invariantes de id, título, duplicata, setlist e resolução) e 3 de integração em `tests/bundle_tests.rs` sobre o fixture `worship.cho`.
- Composição provada com parser e harmonia: o bundle carrega o `.cho` e a música transpõe sem perder a identidade bibliográfica.

### Alterado
- `lib.rs` documenta o que está implementado e o que depende da ADR-0006.

### Premissa corrigida
- "Persistência é o primeiro passo do Song Bundle": o dono decidiu o contrário — o modelo em memória vem primeiro e o formato é decidido depois. Nenhuma dependência externa foi adicionada; o workspace segue sem dependências.

## [Docs] ADR-0006 proposta: Song Bundle v1 (2026-11-20)
### Adicionado
- ADR-0006 (proposta): Song Bundle como diretório com `meta.json` versionado, `serde`/`serde_json` como primeira dependência externa, migração por degrau e exportação por cópia do diretório na v1.

### Alterado
- Índice de ADRs: ADR-0006 registrada como proposta; próximo número livre passa a ADR-0007.

## [Docs] ADR-0005 rejeitada e branches limpas (2026-11-20)
### Alterado
- ADR-0005 marcada como **rejeitada**: a redução de escopo da R1 tratava uma ordem de execução como corte de escopo de uma ADR já aceita. A ADR-0002 permanece integral.
- Plano v2.6, ROADMAP, PENDING e CONCLUSION alinhados: cifra primeiro é ordem de execução; MusicXML/ATEX ficam no marco R1.1, depois do S5, sem emenda de escopo.
- Referências da ADR-0005 corrigidas (caminhos relativos quebrados a partir de `docs/adr/`).

### Adicionado
- Skill `estado-compartilhado-multi-agente` §1c: branch de escopo vira segundo trunk; como distinguir trabalho de sombra antes de ler estado.
- `AGENTS.md`: ao terminar, apagar branches substituídas no mesmo passo; nome de branch é unidade de trabalho, não escopo.

### Removido
- Branches `feature/R1-restante`, `feature/R1-transposicao-harmonia` e `wip/harmony-transpose`: nenhuma tinha conteúdo exclusivo em relação à `main`.

### Corrigido
- `AGENTS.md` afirmava que `CLAUDE.md` e `GEMINI.md` eram symlinks; `CLAUDE.md` é arquivo próprio com as instruções do RTK.
- Referências a `DESIGN.md` em `docs/` apontavam para `docs/DESIGN.md` (inexistente); agora resolvem para a raiz e `verificar_refs.py` sai 0.

### Premissa corrigida
- "O piloto público aguarda o S5" era inferência da proposta, não cláusula da ADR-0002. O portão do piloto é o uso real por uma igreja, offline.

## [Infra] Pipeline local-first (ADR-0004, 2026-11-20)
### Adicionado
- `scripts/ci-local.sh` como portão principal: Rust (`fmt --check`, `clippy`, testes), `cargo deny`, paridade de ARBs e Flutter (`analyze`, `test`, `gen-l10n`).
- ADR-0004 e entrada no índice de ADRs.
- Gates transversais no `.githooks/pre-push` (`deny`, `arb`, `flutter`), com escape por `HARNESS_SKIP`.

### Alterado
- `.github/workflows/ci.yml` sem gatilho em `push` para `main`; mantém `pull_request`, `workflow_dispatch` e `schedule` semanal.
- README, ROADMAP, PENDING e CONCLUSÃO alinhados ao pipeline local.

### Corrigido
- Versão do Flutter fixada em `.flutter-version` (3.50.1), verificada pelo script e espelhada no workflow.
- `packages/core_l10n` incluído no pipeline com `gen-l10n` e `analyze --no-fatal-infos`.
- Localizações geradas (`app_localizations*.dart`) no `.gitignore`; deixam de poluir `git status`.

### Premissa corrigida
- O repositório é público, portanto o GitHub Actions é gratuito e ilimitado. A mudança é por soberania, redução de ruído e não duplicação de validação — não por custo.

## [R1] Transposição e harmonia (2026-11-20)
### Adicionado
- `Chord::transpose` em `chord_naming.rs`, com aritmética de letra e acidente.
- Módulo `harmony.rs` com transposição de música inteira e renderização nos quatro sistemas.
- Testes dourados `golden_harmony/worship.cho` e saídas Common/Latin.
- ADR-0005 (proposta): escopo da R1 com ChordPro primeiro e MusicXML/ATEX em R1.1.

### Corrigido
- Lints de formatação no código de harmonia e nos testes.

### Validação
- 29 testes Rust passando, incluindo acordes invertidos, `6/9`, acidentes duplos e acordes desconhecidos.

## [R1] Cifra e Projeção (parcial integrado, 2026-11-20)
### Adicionado
- Parser ChordPro com modelo canônico e testes dourados.
- Placeholder de parser MusicXML.
- Branch `feature/R1-cifra-projecao` criada a partir da `main` e integrada pelo merge `50c7323`.

### Corrigido
- Lints do parser ChordPro e remoção do módulo `harmony` incompleto da árvore ativa.
- Workflow de CI com ações fixadas por SHA e escopo correto do Flutter.
- App Flutter sem dependências inexistentes de `core_l10n` e `flutter_rust_bridge`.
- Exclusão dos scripts vendorizados do Impeccable da análise Semgrep.
- App migrado para `material_ui` 1.11.1 e token explícito "Sovereign Emerald".
- CI do GitHub verde no run `44665120771` (Rust, licenças, i18n e Flutter).
- Validação local com Flutter 3.50.1: `flutter analyze` sem achados, `flutter test` passando e `flutter gen-l10n` de `core_l10n` sem erro.

## [R0] Fundação e Identidade (2026-11-19)
### Adicionado
- Identidade oficial **Kentron Music Hub** e licença Apache-2.0 com aviso de uso.
- Sovereign Harness com guardrails e git gates.
- Impeccable UI documentado em `DESIGN.md`.
- Monorepo Melos, scaffold Flutter e core Rust `notation_bundle`.
- Ponte `flutter_rust_bridge` como placeholder.
- CI para Rust, licenças, i18n e Flutter.
- ARBs da Faixa 1 (pt-BR, en-US, es-419) e validador `check_arb.py`.
- ADR-0001, ADR-0002 e ADR-0003.

### Publicado
- `main` enviada ao GitHub com `[skip ci]` no primeiro push.
- Branch padrão do repositório alterada para `main`.
