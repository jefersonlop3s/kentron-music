#!/usr/bin/env bash
# Pipeline local do Kentron Music Hub (ADR-0004).
#
# Executa a mesma matriz do workflow remoto sem depender de runners:
# Rust (formatação, Clippy, testes), licenças, paridade de ARBs e Flutter.
# O GitHub Actions permanece como rede de segurança para pull requests.
#
# Uso: ./scripts/ci-local.sh
# Variáveis: FLUTTER_BIN (caminho do executável do Flutter, opcional).
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

step() { printf '\n==> %s\n' "$1"; }

require() {
  command -v "$1" >/dev/null 2>&1 || {
    printf 'ERRO: %s não encontrado no PATH. Instale-o antes de rodar o pipeline local.\n' "$1" >&2
    exit 1
  }
}

# Detecção explícita do Flutter: sem fallback silencioso.
resolve_flutter() {
  if [[ -n "${FLUTTER_BIN:-}" ]]; then
    [[ -x "$FLUTTER_BIN" ]] || {
      printf 'ERRO: FLUTTER_BIN aponta para %s, que não é executável.\n' "$FLUTTER_BIN" >&2
      exit 1
    }
    printf '%s\n' "$FLUTTER_BIN"
    return 0
  fi
  if command -v flutter >/dev/null 2>&1; then
    command -v flutter
    return 0
  fi
  if [[ -x "$HOME/development/flutter/bin/flutter" ]]; then
    printf '%s\n' "$HOME/development/flutter/bin/flutter"
    return 0
  fi
  cat >&2 <<'EOF'
ERRO: Flutter não encontrado.
Instale o SDK 3.50.x (canal stable) ou defina FLUTTER_BIN com o caminho do executável.
Sugestão:
  git clone --filter=blob:none -b stable https://github.com/flutter/flutter.git ~/development/flutter
  export PATH="$HOME/development/flutter/bin:$PATH"
EOF
  exit 1
}

require cargo
require python3
require cargo-deny

FLUTTER="$(resolve_flutter)"

# Versão fixada em .flutter-version (fonte única, espelhada no workflow).
if [[ -f .flutter-version ]]; then
  expected="$(tr -d '[:space:]' < .flutter-version)"
  installed="$("$FLUTTER" --version 2>/dev/null | head -1 | awk '{print $2}')"
  if [[ "$installed" != "$expected" ]]; then
    printf 'ERRO: Flutter %s instalado; .flutter-version exige %s.\n' "$installed" "$expected" >&2
    printf 'Atualize o SDK ou ajuste .flutter-version e o workflow.\n' >&2
    exit 1
  fi
fi

step "Rust: formatação (cargo fmt --check)"
cargo fmt --all -- --check

step "Rust: Clippy (-D warnings)"
cargo clippy --workspace --all-targets -- -D warnings

step "Rust: testes"
cargo test --workspace

step "Licenças e fontes (cargo deny)"
cargo deny check

step "i18n: paridade de ARBs"
python3 tools/check_arb.py

step "Rust: cdylib da ponte Rust ↔ Flutter (ADR-0007)"
# O teste de ponte do app carrega `target/debug/libkentron_ffi.<ext>`; sem este
# build o `flutter test` falha com "cdylib ausente".
cargo build -p kentron_ffi

step "i18n: geração de localizações (packages/core_l10n)"
# Geração NO LUGAR, não mais numa cópia temporária (2026-11-20): o app depende
# do pacote e precisa de `lib/l10n/app_localizations.dart` para analisar e
# testar. Os arquivos gerados estão no .gitignore, então a árvore segue limpa
# para o git — e a ordem importa: gerar antes dos passos do app.
(cd packages/core_l10n && "$FLUTTER" pub get >/dev/null && "$FLUTTER" gen-l10n)
test -f packages/core_l10n/lib/l10n/app_localizations.dart || {
  echo "ERRO: gen-l10n não gerou app_localizations.dart." >&2
  exit 1
}

step "i18n: analyze do pacote core_l10n"
# --no-fatal-infos: o código gerado por gen-l10n emite `info` de depreciação
# (Global*Localizations). Erros e warnings continuam bloqueando.
(cd packages/core_l10n && "$FLUTTER" analyze --no-fatal-infos)

step "Flutter: dependências (apps/kentron_app)"
(cd apps/kentron_app && "$FLUTTER" pub get)

step "Flutter: analyze (apps/kentron_app)"
(cd apps/kentron_app && "$FLUTTER" analyze)

step "Flutter: testes (apps/kentron_app)"
(cd apps/kentron_app && "$FLUTTER" test)

step "Pipeline local concluído: todos os portões passaram."
