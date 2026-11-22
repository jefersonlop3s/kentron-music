# O campo harmônico como régua de avaliação (2026-11-22)

**Origem:** reel de `marcoantero` — *"A Matemática do Campo Harmônico"* — trazido pelo dono.
**Uso aqui:** virou **instrumento de medição** do motor, não conteúdo didático.

## A tese, em três linhas

1. Todo tom tem **7 acordes** que pertencem à música — sistema fechado, sem nota inventada.
2. Acorde é **triângulo de terças** (pula uma nota). A qualidade vem da **ordem**: maior = 4+3, menor = 3+4, diminuto = 3+3.
3. O padrão **M–m–m–M–M–m–dim** não é regra a decorar: é **consequência** dos dois meio-tons da escala (3→4 e 7→1), que "entortam" os triângulos. Em qualquer tom, os meio-tons caem nos mesmos lugares relativos.

## Por que isso vale para o motor

A tese é uma **restrição verificável**: dado o tom, só 7 acordes são legais. Isso permite medir a coerência interna do motor **sem gabarito** — que era exatamente o buraco: a planilha registra o tom da **banda**, não o da **gravação**.

## Medição (121 músicas do repertório, 18.459 trechos de acorde)

**93,1% dos trechos caem no campo harmônico do tom detectado.**

| Grau | Fatia | | Grau | Fatia |
|---|---|---|---|---|
| I | 31,9% | | iii | 5,1% |
| V | 17,5% | | bVII | 2,8% |
| IV | 15,9% | | bVI | 1,5% |
| vi | 11,9% | | vii° | 1,4% |
| ii | 9,4% | | bIII | 1,3% |

**Os 6,9% fora do campo não são ruído.** Eles se concentram na família do **empréstimo modal** (paralela menor): bVII (2,8%), bVI (1,5%), bIII (1,3%), bII (0,7%), bV (0,6%). Se o detector estivesse errando, os erros se espalhariam pelos 12 graus; eles se agrupam exatamente onde a música gospel/pop empresta.

O perfil I > V > IV > vi > ii é a assinatura de música tonal real, não de sorte.

## Duas previsões da tese confirmadas nos dados

- **vii° em 1,4%** — a tese explica: B° (B-D-F) e G7 (G-B-D-F) são as **mesmas notas**. Sem o baixo, o vii° é ouvido como V7 sem fundamental. Isso é a **justificativa teórica do bloco 4** (baixo/inversão), que até aqui era só prática.
- **A confusão com o dominante é estrutural, não acidental:** o campo de D e o de A compartilham 4 dos 7 acordes (D, F#m, A, Bm). Duas tonalidades a uma quinta de distância são as mais fáceis de trocar — o que explica por que o erro de tom, quando aparece, aparece a −5 semitons.

## O que a medição corrigiu

- **O −5 do corpus não é viés do motor, é transposição.** Caso decidido: *Cantai* — o motor detecta **E** com os acordes `E, Bm7, A, F#m7` (campo de Mi maior); a planilha declara **A**. A gravação está em Mi, a banda toca em Lá. Bate com o fluxo do dono: a planilha registra **o tom que a banda toca**.
- **Hipótese anterior refutada:** a de que o 5º harmônico (terça maior, 14 cents acima) viraria terça falsa. Fixtures sintéticas: Dm com 12 harmônicos continua **Dm**; power chord continua **D5**; I‑V‑vi‑IV em Ré sai em **Ré**. O viés não está na conta.
- **Um erro real encontrado:** *Quão Grande é o Meu Deus* — tom **Abm** com correlação **0,51**, num campo de Ab com `G` (dominante secundário). É Ab **maior**; o motor errou e a correlação baixa já avisava.

## Consequências

1. **A tese vira restrição de código:** tom e acordes hoje são decididos de forma independente; a prior diatótica acopla os dois (é a "região harmônica" que a ADR-0014 já previa).
2. **O bloco 4 (baixo/inversão) subiu de prioridade** — agora tem razão teórica (vii° ≡ V7) e empírica (1,4%).
3. **Avaliação de acurácia só é possível com áudio sintético de verdade conhecida.** A planilha nunca foi gabarito: ela mede a distância entre dois tons legítimos, e essa distância é **requisito de produto** (transpor a cifra da gravação para o tom da banda), medida em 43% do repertório.

## Como reproduzir

As fixtures sintéticas e a análise NNS rodaram em `/tmp` nesta sessão. Promover a testes versionados é o próximo passo — as asserções de contrato são: `D5` nunca vira `D`; senoide pura nunca inventa tom; I‑V‑vi‑IV sai na tônica; e a taxa de diatonicidade não cai abaixo de 85% no corpus.