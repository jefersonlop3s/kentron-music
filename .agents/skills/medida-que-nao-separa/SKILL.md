---
name: medida-que-nao-separa
description: O que fazer quando uma medida não separa as classes — o roteiro de diagnóstico em ordem, do front-end à decisão, com as quatro causas ordenadas por frequência (somar funções distintas, banda errada, comparação sem normalizar, limiar onde cabia evidência) e as duas armadilhas de método (a regra de exceção como remédio de sintoma, e o gate que é no-op por matemática). Carregar quando um número fica no teto, quando o acerto empata, e antes de tentar mais um parâmetro.
---

# A medida não separa — onde está o defeito, em ordem

Quando uma medida acerta pouco e **não melhora com ajuste de parâmetro**, o
defeito quase nunca é o limiar. Este roteiro é a ordem que a evidência de um dia
inteiro de tentativas refutadas impõe — oito hipóteses de front-end falharam sem
regredir o placar, e o salto veio de duas correções de **forma**, não de sinal.

## As quatro causas, em ordem de frequência

1. **Somar funções distintas num sinal só apaga a informação.** Duas peças com
   papéis métricos diferentes — uma marca o tempo 1, outra o contratempo —
   somadas dão o mesmo envelope, e a medida empata. Medição de origem: contraste
   **0,42 nos dois compassos**, empate exato. **Conserto: separar as fontes antes
   de medir** e comparar a **relação** entre elas.
2. **A banda (ou a faixa) está errada — e não é timbre.** Um transiente forte é
   **banda-larga** e dispara em todas as bandas ao mesmo tempo: medido, pele
   `52% 21% 13%`, fios `32% 29% 17%`, prato `25% 35% 18%` — **a mesma forma nas
   três**. Escolha a faixa onde o corpo do evento vive e o resto não tem energia,
   e **descarte o evento que coincide com o mais forte** (ele mascara o outro).
3. **A comparação não está normalizada.** Limiar local sobre duas posições soltas
   confunde ruído com contraste. **Contraste normalizado sobre o ciclo inteiro** é
   o que separa: `4 de 6` → `5 de 6` na medição de origem, sem tocar em uma linha
   de extração.
4. **Cabia evidência onde você pôs limiar.** Um `if` sobre uma grandeza bruta vira
   número mágico que não transfere. **Avalie os candidatos e deixe o de maior
   contraste vencer** — a mesma moeda que decide o resto. Medição de origem: o
   ciclo certo para cada metro saiu sozinho, `0,93 → 1,39 s` e `1,26 → 2,52 s`.

## As duas armadilhas de método

- **A regra de exceção é remédio de sintoma.** Um limiar que "só dispara onde
  precisa" parece cirúrgico e é frágil: quando a causa raiz foi corrigida (a
  banda certa), o sintoma desapareceu e a exceção ficou **obsoleta e nociva** —
  ela passou a disparar onde não devia. **Corrija a causa; a exceção cai.**
- **Gate escalar é no-op em métrica normalizada — verifique antes de codar.** Um
  peso por quadro `W(t)` cancela na distância cosseno:
  `W(t)C(t)·W(t-τ)C(t-τ) / (|W C|·|W C|) = C(t)·C(t-τ)/(|C(t)||C(t-τ)|)`.
  Diferença medida: **1,18e-10**. Um `if` de meia hora teria "funcionado" sem
  fazer nada — e o placar idêntico seria lido como confirmação.

## Regras

1. **Teste o instrumento no caso ideal antes de medir o mundo.** Se a medida não
   separa dois sintéticos limpos, não é a música que está difícil.
2. **Número implausível é denúncia do aparelho.** `74–99%` dos ataques numa
   classe, `75%` das notas no mínimo de duração: é o instrumento gritando, não a
   música falando.
3. **Um parâmetro que não move o número condena o instrumento, não o valor.** Dois
   parâmetros testados sem efeito: pare de ajustar e mude a forma.
4. **Registre a hipótese refutada com o número.** "Banda mais larga: piorou (5 → 4
   de 6)" poupa a próxima pessoa de refazer o caminho — e é o que transforma oito
   tentativas perdidas em mapa.
5. **A causa pode mudar de lado.** Corrigir o front-end pode revelar que o
   gargalo seguinte é a decisão (ou o contrário). Rediagnostique a cada salto.
6. **`A(T/2)` contra `A(T/3)` e afins: normalizar antes de comparar** — a mesma
   correção de forma vale em qualquer escala, do envelope ao ciclo.

## Ritual

1. Validar o instrumento no sintético onde a resposta é conhecida.
2. Se o acerto empata: separar as fontes que você somou.
3. Se ainda empata: conferir a banda e descartar o evento coincidente.
4. Se ainda empata: normalizar a comparação sobre o ciclo inteiro.
5. Se ainda empata: trocar o limiar por competição de hipóteses.
6. Medir no conjunto de fora (ver `conjunto-de-teste-de-fora`) e registrar as
   hipóteses refutadas com o número de cada uma.
