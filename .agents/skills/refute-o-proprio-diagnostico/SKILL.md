---
name: refute-o-proprio-diagnostico
description: O ataque adversarial ao próprio diagnóstico antes de fechá-lo — as quatro classes de contra-argumento que encontram erro real (a métrica vazia, o escopo maior que a medição, o ajuste em poucos pontos, a premissa não medida) e o que fazer quando uma vence. Carregar SEMPRE antes de declarar um marco fechado, antes de escrever "está resolvido" e quando o dono pedir para você tentar se contradizer. Caso de origem: seis contra-argumentos, dois venceram contra o próprio autor, e os dois viraram correção publicada.
---

# Tente derrubar o seu diagnóstico antes de fechá-lo

Um diagnóstico é uma afirmação, e afirmação não verificada é palpite com boa
redação. **Quem o formulou é a pior pessoa para julgar se ele resiste** — e é
exatamente por isso que o ataque tem de ser **estruturado**, não sincero.

Caso de origem: o dono pediu, antes de fechar um marco: *"tente provar que meu
diagnóstico está errado — se não provar, fechamos; se provar, fazemos mais uma
rodada"*. **Seis contra-argumentos foram testados. Dois venceram — contra o
próprio autor — e os dois viraram correção publicada.**

## As quatro classes de ataque, e o que cada uma já encontrou

1. **A métrica é vazia — ela não consegue falhar.** *"A resposta certa está dentro
   da oferta em 10 de 10"* parecia um resultado e não media nada: a oferta era
   sempre `[6/8, 4/4]`, então qualquer músico `4/4` a satisfazia. **Teste: descreva
   um caso em que a métrica daria zero. Se não existir, ela é decorativa.**
   Encontrado e **retirado de três documentos**.

2. **O escopo da afirmação é maior que a medição.** *"A escala está fechada"* —
   era verdade na mediana e falsa em **6 das 40** faixas, que ficavam abaixo do
   piso fisiológico e sumiam no agregado. **Teste: escreva o escopo exato (quantos
   de quantos) e veja se a frase sobrevive.** Encontrado e o escopo **estreitado**.

3. **O ajuste é fino demais para os pontos que o sustentam.** Um limiar calibrado
   em **dois** casos, com margem de **0,05 s** até o caso seguinte. **Teste: meça a
   distância entre o limiar e o vizinho mais próximo que deveria passar.** Fica
   registrado como risco latente, mesmo quando não é exercido.

4. **A premissa não foi medida — só assumida.** O diagnóstico dizia *"a caixa nos
   terços"*; medido, **13 das 15 faixas tinham a massa na fatia 1, não nos
   terços** — a premissa era falsa e a conclusão seguia de pé **por outro motivo**.
   **Teste: liste as premissas e meça cada uma. Premissa falsa com conclusão certa
   é a mais perigosa — ela contamina o próximo raciocínio.**

## O que fazer quando uma vence

**Corrigir e publicar, com o contra-argumento no registro.** O valor não está em
vencer: está em que o número publicado deixe de mentir. As duas correções do caso
de origem foram para o CHANGELOG e para o `PENDING` **com o contra-argumento que
as produziu** — quem ler depois não repete o caminho.

**E quando nenhuma vence, o marco ganha valor:** *"sobreviveu a seis ataques"* é
uma afirmação diferente de *"está pronto"*. **Diga qual dos dois você tem.**

## Regras

1. **Ataque a conclusão, não a redação.** Reescrever melhor não é refutar.
2. **Cada contra-argumento tem de ser testável.** *"Talvez não seja assim"* não é
   ataque; *"se o alvo harmônico for 1,5 barra, a escolha vai para 2,82 s"* é — e
   foi medido.
3. **Teste também os que parecem fortes.** O ataque mais promissor do dia (a razão
   barra/batida decide o compasso) foi o que falhou mais limpo: as faixas se
   sobrepõem inteiramente.
4. **Registre os que falharam, com o número.** *"Falha: mede a taxa de troca
   (gabarito ~6,0; corpus ~3,2), não o metro"* poupa a próxima tentativa.
5. **Peça o ataque quando o outro lado conhece o domínio.** O dono do repertório
   viu em um ouvido o que sete medidas não separaram.

## Ritual

1. Escreva o diagnóstico em uma frase, com o escopo exato.
2. Liste as premissas — e meça cada uma.
3. Para cada métrica citada, descreva o caso em que ela daria zero.
4. Meça a margem dos limiares contra o vizinho mais próximo.
5. Rode os contra-argumentos **antes** de escrever "fechado".
6. Publique as correções com o contra-argumento que as produziu.
