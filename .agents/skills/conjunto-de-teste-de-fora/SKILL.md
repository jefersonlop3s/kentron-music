---
name: conjunto-de-teste-de-fora
description: Contramedidas para os TRÊS vieses que fazem um placar alto mentir — o de SELEÇÃO (o conjunto foi escolhido por quem construiu), o de COBERTURA (o conjunto não contém a classe difícil) e o de TAXA-BASE (o conjunto tem a distribuição que torna a resposta trivial certa). Cobre 100% em nove músicas virando 1 de 8 no conjunto de outra pessoa, 6 de 6 virando 70% de erro num corpus real, e 13 de 15 virando 1 de 9 num conjunto balanceado. Carregar antes de declarar qualquer acerto, antes de comemorar placar perfeito, e sempre que o número parecer bom demais.
---

# Conjunto escolhido por quem construiu não é teste

Um placar é uma afirmação sobre o mundo feita a partir de uma amostra. Se a
amostra foi escolhida por quem construiu a coisa medida, ela responde à pergunta
"o que eu imaginei?", não "o que existe?". Dois vieses distintos produzem o mesmo
sintoma — placar alto — e têm consertos diferentes.

Caso de origem: duas vezes no mesmo dia, no mesmo motor. O `5 de 6` caiu para
**1 de 8** no conjunto de outra pessoa; depois, o `6 de 6` virou **70% de erro**
num corpus real de 40 músicas.

## Os três vieses

1. **Viés de seleção — o conjunto foi escolhido.** Nove músicas que *eu* escolhi
   para testar `3/4` × `6/8` continham só `3/4` e `6/8`. O placar mediu a minha
   escolha. Medição de origem: `5 de 6` no conjunto próprio, `1 de 8` no conjunto
   do gabarito externo — **mesmo código, mesmo material de áudio**.

2. **Viés de cobertura — o conjunto não tem a classe difícil.** O gabarito externo
   tinha nove músicas e **nenhum `4/4`**. A medida separava "ternário" de "binário"
   e eu li o binário como `6/8`. No corpus de 40 músicas de gospel — quase todo
   `4/4` — **28 foram lidas como `6/8`**. O conjunto cobria os casos que o autor
   pensou em testar; o mundo tinha outro.

3. **Viés de taxa-base — o conjunto tem a distribuição que torna a resposta
   trivial certa.** O terceiro, e o mais traiçoeiro porque **parece acerto**. Uma
   regra que diz `4/4` por padrão acertou **13 de 15** no corpus gospel (quase todo
   `4/4`) — e **1 de 9** num conjunto **balanceado** (três de cada) e **0 de 4**
   num conjunto de **taxa-base oposta** (hinos ternários). O número do corpus não
   media a regra: media a frequência da classe. Medição de origem: `13 de 15` com
   distribuição implausível é a denúncia — **`4/4` era a resposta certa na maioria
   do conjunto, então dizer `4/4` sempre parecia funcionar.**

**Os três dão placar alto e têm consertos diferentes:** o de seleção pede um
conjunto de fora; o de cobertura pede a classe ausente; o de **taxa-base pede o
balanceamento** — ou um segundo conjunto com a distribuição **oposta**.

**O tamanho importa menos que a variedade.** Nove músicas que cobrem os casos que
existem valem mais que nove que cobrem os que eu imaginei. `n=6` não separa `5/6`
de `4/6` — e `n=9` sem a classe difícil não separa o certo do plausível. **E um
conjunto desbalanceado não separa o certo do frequente.**

## Regras

1. **Antes de comemorar, pergunte quem escolheu o conjunto.** Se foi você: o
   número mede a sua imaginação. Busque um conjunto rotulado por outra pessoa —
   e, se não existir, diga que o número é interno.
2. **Liste as classes do conjunto e confira o que está ausente.** Não o que está
   presente: **o que não está**. Uma classe em `0/N` e uma classe **ausente** são
   falhas diferentes — a primeira se mede, a segunda não.
3. **Rode no corpus que você não escolheu.** Áudio do usuário, arquivo de outra
   pessoa, produção. É onde a distribuição real aparece — e ela é sempre mais
   larga que a do conjunto de teste.
4. **Desconfie da distribuição do resultado, não só do acerto.** `70% de 6/8` num
   repertório que é quase todo `4/4` é a denúncia. **O formato da distribuição é
   evidência** — um acerto de 100% com distribuição implausível está errado.
5. **Conte as classes do conjunto antes de contar os acertos.** Se uma classe é a
   maioria, **a resposta trivial acerta por acidente** — e o placar mede a
   frequência, não a regra. O conserto é **balancear** (três de cada) ou medir num
   segundo conjunto com a **distribuição oposta**. Medição de origem: `13 de 15`
   num corpus desbalanceado virou `1 de 9` num balanceado e `0 de 4` no de
   taxa-base oposta.
6. **Placar perfeito pede explicação, não comemoração.** `6 de 6` em nove músicas
   é um resultado sobre nove músicas. Escreva o número **com o conjunto ao lado**,
   sempre — `6 de 6 (gabarito X, sem 4/4)`.
7. **Refutação não é fracasso, é mapa.** Cada conjunto de fora que corrige um
   número estreita o que a coisa é: "não é o detector, não é a banda, não é a
   janela, não é a âncora" é resultado.

## Ritual

1. Medir no conjunto próprio — sabendo que é interno.
2. Buscar/construir um conjunto rotulado por fora, com a classe difícil dentro.
3. Medir nele **antes** de publicar o número.
4. **Contar as classes.** Se a distribuição não for balanceada, construir um
   segundo conjunto **balanceado** ou de **taxa-base oposta** — e medir nos dois.
5. Medir no corpus real, e olhar a **distribuição** do resultado.
6. Escrever o placar sempre com o conjunto e a cobertura declarados.
7. Registrar os vieses encontrados, com o número de cada lado.
