---
name: toolchain-unica
description: Duas instalações do mesmo binário fazem o PATH decidir qual roda, e o gate pode medir uma enquanto o pipeline usa outra. Cobre dois Flutter, dois adb, versões que divergem, e a acusação falsa que nasce da ambiguidade. Carregar ao ver aviso de duplicata no doctor, ao investigar gate que reprova só num contexto, e antes de instalar mais uma cópia de ferramenta que já existe.
---

# Uma ferramenta, uma instalação

Caso de origem (2026-11-20): a máquina tinha **dois Flutter** — o clone `~/development/flutter`, que o `.flutter-version` pina e o pipeline usa, e o cask do Homebrew — e **dois `adb`** (o do SDK e o do cask `android-platform-tools`). O `flutter doctor` avisava dos dois `adb`. Numa reprovação de gate, acusei o Flutter do brew de estar quebrado; a causa real era contenção de CPU. A ambiguidade foi o que tornou a acusação plausível.

## Por que importa

- **O PATH decide, não você.** `flutter` no PATH pode ser o cask; o pipeline usa o clone. Dois comportamentos, um sintoma.
- **Versões divergem com o tempo.** Hoje iguais; depois de um `brew upgrade`, não.
- **Gate mede o que o PATH entregar.** Um gate "verde" pode estar testando outro binário.

## Medir

```bash
which -a flutter; which -a adb; which -a dart
for f in $(which -a flutter); do "$f" --version 2>&1 | head -1; done
"$HOME/Library/Android/sdk/platform-tools/adb" version | head -1
flutter doctor -v 2>&1 | head -30     # avisa quando há dois adb
```

## Resolver por remoção, não por PATH

- Escolha a instalação canônica: a que o projeto pina (aqui, `~/development/flutter` via `.flutter-version`).
- Remova a outra (`brew uninstall --cask flutter`; `brew uninstall --cask android-platform-tools`), em vez de reordenar o PATH. PATH é frágil; remoção é definitiva.
- Confirme que os scripts de gate apontam para a canônica: `FLUTTER_BIN` no `ci-local.sh` e no `pre-push`, com fallback para `~/development/flutter/bin`.
- Registre no CHANGELOG/PENDING o que ficou e por quê.

## Duplicata que se esconde: o cask resolve o SDK root para si mesmo

Caso medido (2026-11-20): o `avdmanager` do cask `android-commandlinetools`
resolvia o SDK root para o próprio cask (sem `system-images`) e falhava com
`Package path is not valid ... null`; symlink **não** engana essa detecção. A
correção foi uma **cópia real** de `cmdline-tools` dentro de
`~/Library/Android/sdk` — que também deixa o SDK imune a `brew upgrade`.

Sintoma da família: a ferramenta existe, o pacote existe, e o caminho que ela
resolve não é o que você está olhando. Meça com `which -a` **e** com o caminho
que a ferramenta imprime; se divergirem, é duplicata.

## Regra de ouro

Antes de instalar mais uma cópia de algo que já existe, pergunte por que a existente não serve. Duplicata é dívida com juros: no dia em que as versões divergirem, o sintoma aparece longe da causa.
