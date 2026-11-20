---
name: higiene-de-dispositivos
description: Como encerrar emulador Android e simulador iOS depois das provas, sem depender de padrão de processo enganoso e sem deixar a máquina ocupada. Cobre a verificação pelo estado real (adb devices / simctl list devices booted), o desligamento limpo (adb emu kill / simctl shutdown), a distinção entre encerrar o dispositivo e apagar o AVD, e o registro no quadro. Carregar ao terminar uma prova em dispositivo, depois de bateria de testes automatizados, ao liberar RAM/disco e ao preparar um gate que disputa CPU.
---

# Higiene de dispositivos: encerrar depois da prova

Caso de origem (2026-11-20): a sessão deixou o emulador Android (`emulator-5554`) e o simulador iOS (iPhone 17) ligados depois das provas, e "Máquinas de pé" virou pendência no quadro. No mesmo dia um gate reprovou por contenção enquanto downloads pesados rodavam — o mesmo tipo de disputa que um dispositivo esquecido cria.

## Por que encerrar

- Emulador Android medido: ~330 MB de RSS no `qemu/darwin-*`, mais `netsimd` e `crashpad_handler`.
- Simulador iOS: `launchd_sim` + CoreSimulator, com o app instalado.
- Gate que disputa CPU morre com "Connection closed before test suite loaded" — sintoma que parece defeito de toolchain e não é (skill `gate-reprovou`).
- Bateria e RAM num laptop que viaja entre redes.

## Verificação pelo estado real, não por padrão de processo

Medido: `pgrep -f 'emulator -avd'` **não acha** o emulador em execução — o processo é `qemu/darwin-*`. Padrão de processo é frágil; o estado do dispositivo é a fonte:

```bash
export ANDROID_HOME="$HOME/Library/Android/sdk"
"$ANDROID_HOME/platform-tools/adb" devices        # lista vazia = nada rodando
xcrun simctl list devices booted                  # seção vazia = nada rodando
```

## Mecanismo: `scripts/encerrar-dispositivos.sh`

O comando manual abaixo fica para consulta; o caminho normal é o script, que faz
detecção, desligamento e verificação no mesmo passo e sai 1 se algo sobrar:

```bash
scripts/encerrar-dispositivos.sh             # encerra e verifica
scripts/encerrar-dispositivos.sh --dry-run   # só mostra o que faria
```

Provado no sentido negativo em 2026-11-20: com o emulador `kentron` no ar, o
script detectou `emulator-5554`, encerrou e o `adb devices` seguinte ficou
vazio. Dispositivo físico conectado é ignorado com aviso — não é emulador.

## Desligar

Android, pelo próprio emulador (limpo):

```bash
"$ANDROID_HOME/platform-tools/adb" -s emulator-5554 emu kill
```

iOS, todos ou um UDID:

```bash
xcrun simctl shutdown all
# ou: xcrun simctl shutdown <UDID>
```

A janela do Simulator pode ficar aberta sem dispositivo: `osascript -e 'quit app "Simulator"'`. Nunca `pkill -f` (skill `gate-reprovou`).

## O que NÃO fazer

- **Não apagar o AVD para "limpar".** `emulator -avd kentron` volta em segundos; apagar exige recriar a imagem. Encerrar ≠ apagar.
- **Não confiar em `pgrep` para provar que encerrou.** Use `adb devices` e `simctl list devices booted`.
- **Não encerrar durante prova em andamento** (instalação, `flutter run`, captura de tela).

## Ritual

1. Terminou a prova (print/log lido e registrado)? Encerre.
2. Android: `adb -s <serial> emu kill`; iOS: `xcrun simctl shutdown all`.
3. Confirme com `adb devices` e `xcrun simctl list devices booted` (vazios).
4. Registre no quadro: "Máquinas de pé" deixa de existir; se o AVD ou o simulador continuar necessário, diga onde ele está.
