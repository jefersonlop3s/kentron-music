#!/usr/bin/env bash
# Processa uma pasta de áudios com separação ONNX de stems.
# Uso:  scripts/separar-lote.sh <pasta-de-audios> [pasta-de-saida]
#
# Para cada áudio, executa `mir --stems onnx` e escreve os stems em
#   <pasta-de-saida>/<nome-do-audio>/ (drums.wav, bass.wav, etc.)
#
# Requer: modelo ONNX baixado (scripts/baixar-pesos.sh)
#
# Nenhuma falha é engolida: o motor sai com código ≠ 0 quando não escreve os
# stems, e se ele sair com 0 sem arquivo em /tmp/kentron-stems/<nome>/ o
# áudio conta como FALHA — antes daqui o `|| true` fazia o script imprimir
# "✓" sem nada ter sido escrito.

set -euo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
cd "$HERE"

PASTA_IN="${1:?uso: scripts/separar-lote.sh <pasta-audios> [pasta-saida]}"
PASTA_OUT="${2:-$PASTA_IN/stems}"

if [ ! -d "$PASTA_IN" ]; then
    echo "✗ pasta não encontrada: $PASTA_IN" >&2
    exit 1
fi

# Onde o motor (`mir --stems onnx`) escreve os stems — ver bin/mir.rs.
STEMS_TMP="/tmp/kentron-stems"

# Verifica se o modelo ONNX existe
MODELO="models/weights/htdemucs_6s/htdemucs_6s.onnx"
if [ ! -f "$MODELO" ]; then
    echo "✗ modelo ONNX não encontrado em $MODELO" >&2
    echo "  baixe com: scripts/baixar-pesos.sh" >&2
    exit 1
fi

# Lista os arquivos de áudio
echo "procurando áudios em $PASTA_IN..."
ARQUIVOS=()
while IFS= read -r f; do
    ARQUIVOS+=("$f")
done < <(find "$PASTA_IN" -type f \( -name '*.wav' -o -name '*.mp3' -o -name '*.m4a' -o -name '*.flac' \) | sort)

if [ "${#ARQUIVOS[@]}" -eq 0 ]; then
    echo "✗ nenhum áudio encontrado" >&2
    exit 1
fi

echo ""
echo "processando ${#ARQUIVOS[@]} áudio(s)..."
echo ""

total="${#ARQUIVOS[@]}"
ok=0
falha=0

for i in "${!ARQUIVOS[@]}"; do
    arquivo="${ARQUIVOS[$i]}"
    nome="$(basename "$arquivo")"
    nome_sem_ext="${nome%.*}"
    saida="$PASTA_OUT/$nome_sem_ext"

    echo "[$((i + 1))/$total] $nome → $saida/"

    if ! mkdir -p "$saida"; then
        echo "  ✗ não foi possível criar $saida" >&2
        falha=$((falha + 1))
        continue
    fi

    # O stderr do motor é preservado: é ele que diz por que a separação falhou.
    erro_log="$(mktemp)"
    if cargo run -p mir_engine --release --quiet -- "$arquivo" --stems onnx --json \
        > /dev/null 2>"$erro_log"; then
        rm -f "$erro_log"
    else
        echo "  ✗ falha na separação" >&2
        sed 's/^/    /' "$erro_log" >&2
        rm -f "$erro_log"
        falha=$((falha + 1))
        continue
    fi

    src_stem="$STEMS_TMP/$nome_sem_ext"
    if [ ! -d "$src_stem" ]; then
        echo "  ✗ o motor saiu com sucesso mas não escreveu nada em $src_stem" >&2
        falha=$((falha + 1))
        continue
    fi

    copiados=0
    for wav in "$src_stem"/*.wav; do
        [ -e "$wav" ] || continue
        if ! cp "$wav" "$saida/"; then
            echo "  ✗ falha ao copiar $wav para $saida/" >&2
            falha=$((falha + 1))
            continue 2
        fi
        copiados=$((copiados + 1))
    done

    if [ "$copiados" -eq 0 ]; then
        echo "  ✗ nenhum .wav em $src_stem" >&2
        falha=$((falha + 1))
        continue
    fi

    echo "  ✓ ($copiados stem(s))"
    ok=$((ok + 1))
done

echo ""
echo "concluído: $ok sucesso, $falha falha(s) em $total"
[ "$falha" -eq 0 ]
