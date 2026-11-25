# Compasso e Ritmo — Como o Kentron detecta o metro da música

## O que é o compasso?

O compasso (time signature) define como os tempos são agrupados na música.
Os mais comuns na música popular são:

| Compasso | Sensação | Exemplos |
|----------|----------|----------|
| **4/4** | Quatro batidas por compasso, caixa no 2 e 4 ("backbeat") | Rock, Pop, Gospel, Samba |
| **3/4** | Três batidas, sensação de "valsa" | Hinos, Valsas, Folk |
| **6/8** | Duas batidas grandes, cada uma dividida em 3 | Baladas, Soul, Blues lento |

## Como o motor detecta

O Kentron analisa dois sinais simultaneamente:

**1. Onde a energia percussiva cai** (bumbo, caixa, pratos)
O motor separa o bumbo (30–110 Hz) da caixa (150–450 Hz) e mede a relação
entre eles. Em 4/4, a caixa cai nos tempos 2 e 4; em 3/4, a caixa divide
os três tempos; em 6/8, a caixa cai na 4ª colcheia.

**2. Onde os acordes mudam** (ritmo harmônico)
A harmonia muda predominantemente no tempo 1 (downbeat). Em 4/4, pode
mudar também no tempo 3 (meio do compasso). Mudanças no tempo 4 geralmente
são acordes de passagem, não estruturais.

### A regra de decisão

Combinando os dois sinais, o motor calcula:

- **Lag ratio**: a frequência das mudanças harmônicas dividida pela
  frequência da percussão. ~4 → 4/4 (harmonia muda a cada 4 batidas),
  ~3 → 3/4, ~2 → 6/8 ou 4/4 com acordes rápidos.
- **Posições harmônicas**: onde os picos de mudança caem no compasso.
  Se o 2º pico está no último tempo, é provavelmente acorde de passagem
  em 4/4, não 6/8.

## Quando o motor pode errar

A detecção automática de compasso é um problema em aberto na literatura
acadêmica (Klapuri 2006, Peeters 2010). O motor acerta na maioria dos
casos, mas pode confundir:

- **4/4 com caixa no contratempo** → pode parecer 3/4
- **6/8 acelerado** → pode parecer 3/4
- **4/4 com acordes de passagem no tempo 4** → pode parecer 6/8

**Se a sugestão do motor parecer errada, você pode corrigir manualmente**
no campo de compasso. A escolha fica salva na cifra como `{time: 4/4}`
e sobrevive a reload do app.

## Como ajudar o motor a acertar

- **Forneça o áudio com bateria bem separada**: o motor funciona melhor
  quando consegue isolar o bumbo da caixa.
- **Se possível, indique o compasso conhecido**: se você já sabe que a
  música é 3/4, informe antes — o motor confirma ou sugere ajuste.

## Referências técnicas

Para detalhes da implementação, veja o código fonte:
- Algoritmo de compasso: `crates/mir_engine/src/compasso.rs`
- Separação de peças de bateria: `crates/mir_engine/src/pecas.rs`
- Separação de stems (ADR-0017): `crates/mir_engine/src/stems.rs`
- Paper de referência: Klapuri et al. (2006), "Analysis of the Meter of
  Acoustic Musical Signals", IEEE TASLP.
