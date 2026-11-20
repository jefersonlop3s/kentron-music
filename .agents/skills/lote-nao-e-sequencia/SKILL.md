---
name: lote-nao-e-sequencia
description: Comandos enviados juntos executam em paralelo, então dependência entre eles vira corrida e a verificação lê antes do trabalho existir. Cobre copiar+usar no mesmo lote, scaffold+verificar, e download pesado concorrendo com gate. Carregar antes de agrupar comandos dependentes, ao ver "não encontrado" logo depois de criar algo, e ao disparar builds ou downloads em segundo plano.
---

# Lote não é sequência

Agrupar comandos numa chamada economiza turnos — e transforma dependência em corrida. Caso de origem (2026-11-20, duas vezes na mesma sessão):

1. Copiei `cmdline-tools` e rodei `avdmanager` no mesmo lote; o `avdmanager` leu o diretório antes da cópia terminar e falhou com `Package path is not valid`. Passei a acusar a ferramenta até medir em sequência.
2. `flutter create --platforms=ios` e a verificação do `pubspec` no mesmo lote; a verificação olhou antes do scaffold existir e o `diff` falhou.

## Regra

Se B depende de A, os dois não podem estar no mesmo lote. Escolha:

- **Encadeie na mesma string**: `cmd1 && cmd2 && verificação` — uma chamada, execução sequencial.
- **Separe em chamadas distintas** quando o resultado de A decide o que fazer em B.
- **Só use `sleep`/polling** quando o produtor é assíncrono de verdade (build em segundo plano) — e verifique o log, não o relógio.

## Segundo caso: paralelismo que custa CPU

Download de vários GB em segundo plano + gate na frente = contenção. O test host morre com "Connection closed before test suite loaded" e parece defeito de ferramenta. Não rode gate enquanto baixa SDK ou imagem; espere o download terminar (skill `gate-reprovou`).

## Sintoma típico

Erro "não encontrado" ou "caminho inválido" imediatamente depois de criar o arquivo/diretório: antes de acusar a ferramenta, confira se o comando anterior do mesmo lote já terminou. Um `ls` no caminho resolve em um segundo.
