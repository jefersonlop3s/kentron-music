#!/usr/bin/env bash
# =============================================================================
# Verificador de SINTAXE Dart para o movimento da `main`
# =============================================================================
# Caso de origem (2026-11-26): `apps/kentron_app/lib/src/stage/song_reader_page.dart`
# ficou com sintaxe quebrada na `main` (commit `b6a9864`, dois `),` a mais) e
# NENHUM gate reprovou — `cargo test --workspace` estava verde, porque **nenhum
# gate Rust enxerga Dart**. O gate `flutter` do pre-push **morde** (há log
# preservado em que ele reprova por dois `info` de `analyze`), mas ele só roda no
# PUSH e só quando a faixa enviada toca `apps/`: um `merge`/`update-ref` — ou um
# push de clone sem `core.hooksPath` — leva o arquivo quebrado para a `main` em
# silêncio, e a guarda da ref só verificava documentação.
#
# O que este verificador faz, e o que ele NÃO faz:
#   - ele lê o **blob do commit** (`git show <rev>:<arquivo>`), não o disco: a
#     guarda da ref roda em `update-ref`, que não toca a árvore de trabalho;
#   - ele reprova **apenas erro de parse** (rc do `dart format` >= 2, medido: 65).
#     `rc=1` significa "o formatador mudaria o arquivo" — formatação, não sintaxe,
#     e **12 de 25 arquivos do app já estão fora do padrão hoje**: reprovar isso
#     bloquearia todo movimento da `main` por dívida pré-existente.
#   - ele **não** substitui `flutter analyze`/`flutter test`: não vê tipo, import
#     quebrado nem teste vermelho. É o instrumento barato do buraco "sintaxe
#     Dart chega à main sem gate nenhum".
#
# Custo medido (2026-11-26, macOS): `dart format` em 25 arquivos = **0,04 s**;
# nenhum export da árvore (nada de `git archive | tar`) — só `git show`.
#
# Uso:  tools/verificar_dart.sh --novo <rev> [--atual <rev>] [--repo <dir>]
#       (sem `--atual`, verifica todos os .dart da árvore de `--novo`)
# Variáveis: DART_BIN (caminho do executável, opcional)
# Saída: 0 = nada quebrado; 1 = erro de parse (lista arquivo e motivo) ou dart
#        ausente (sem verificador, sem atualização da main — `precisa_de` do
#        pre-push segue o mesmo princípio); 2 = uso incorreto.
# =============================================================================
set -uo pipefail

REPO="$PWD"; NOVO=''; ATUAL=''
while [ "$#" -gt 0 ]; do
    case "$1" in
        --repo)  REPO="${2:?--repo exige um diretório}"; shift 2 ;;
        --novo)  NOVO="${2:?--novo exige uma revisão}"; shift 2 ;;
        --atual) ATUAL="${2:?--atual exige uma revisão}"; shift 2 ;;
        -h|--help) sed -n '2,40p' "$0"; exit 0 ;;
        *) echo "argumento desconhecido: $1 (use --novo/--atual/--repo)" >&2; exit 2 ;;
    esac
done
[ -n "$NOVO" ] || { echo "uso: tools/verificar_dart.sh --novo <rev> [--atual <rev>] [--repo <dir>]" >&2; exit 2; }

dart_bin="${DART_BIN:-}"
if [ -z "$dart_bin" ]; then
    if command -v dart >/dev/null 2>&1; then
        dart_bin=$(command -v dart)
    elif [ -x "$HOME/development/flutter/bin/dart" ]; then
        dart_bin="$HOME/development/flutter/bin/dart"
    else
        echo "\`dart\` não encontrado (PATH, DART_BIN ou \$HOME/development/flutter/bin/dart)." >&2
        echo "Sem dart não há verificação de sintaxe Dart. Instale o SDK do .flutter-version" >&2
        echo "ou use a exceção consciente HARNESS_SKIP=dart." >&2
        exit 1
    fi
fi

if [ -n "$ATUAL" ]; then
    arquivos=$(git -C "$REPO" diff --name-only --diff-filter=ACMR "$ATUAL" "$NOVO" 2>/dev/null | grep -E '\.dart$' || true)
else
    arquivos=$(git -C "$REPO" ls-tree -r --name-only "$NOVO" 2>/dev/null | grep -E '\.dart$' || true)
fi
[ -n "$arquivos" ] || exit 0

tmp=$(mktemp -d) || { echo "mktemp falhou — sem verificação, sem atualização da main" >&2; exit 1; }
trap 'rm -rf "$tmp"' EXIT

ruins=0
while IFS= read -r f; do
    [ -n "$f" ] || continue
    git -C "$REPO" show "$NOVO:$f" > "$tmp/arquivo.dart" 2>/dev/null || continue
    saida=$("$dart_bin" format --output=none --set-exit-if-changed "$tmp/arquivo.dart" 2>&1)
    rc=$?
    if [ "$rc" -ge 2 ]; then
        # A mensagem útil vem depois do cabeçalho "Could not format because the
        # source could not be parsed:" — a primeira linha de posição é a que serve.
        motivo=$(printf '%s\n' "$saida" | grep -m1 -E '^line [0-9]+' || true)
        [ -n "$motivo" ] || motivo=$(printf '%s\n' "$saida" | head -1)
        printf '%s: %s\n' "$f" "$motivo"
        ruins=$((ruins + 1))
    fi
done <<< "$arquivos"

if [ "$ruins" -gt 0 ]; then
    echo "" >&2
    echo "$ruins arquivo(s) .dart com ERRO DE PARSE no commit ${NOVO:0:7} — o arquivo não compila." >&2
    exit 1
fi
exit 0
