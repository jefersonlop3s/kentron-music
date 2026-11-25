#!/usr/bin/env bash
# Processa uma pasta de áudios com separação ONNX de stems.
# Uso:  scripts/separar-lote.sh <pasta-de-audios> [pasta-de-saida]
#
# Para cada áudio, executa `mir --stems onnx` e escreve os stems em
#   <pasta-de-saida>/<nome-do-audio>/ (drums.wav, bass.wav, etc.)
#
# Requer: modelo ONNX baixado (scripts/baixar-pesos.sh)

set -euo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
cd "$HERE"

PASTA_IN="${1:?uso: scripts/separar-lote.sh <pasta-audios> [pasta-saida]}"
PASTA_OUT="${2:-$PASTA_IN/stems}"

if [ ! -d "$PASTA_IN" ]; then
    echo "✗ pasta não encontrada: $PASTA_IN" >&2
    exit 1
fi

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
    mkdir -p "$saida"

    echo "[$((i+1))/$total] $nome → $saida/"
    if cargo run -p mir_engine --release -- "$arquivo" --stems onnx --json > /dev/null 2>&1; then
        # Move os stems do /tmp para o diretório de saída
        src_stem="/tmp/kentron-stems/$nome_sem_ext"
        if [ -d "$src_stem" ]; then
            cp "$src_stem"/*.wav "$saida/" 2>/dev/null || true
        fi
        echo "  ✓"
        ((ok++))
    else
        echo "  ✗ falha"
        ((falha++))
    fi
done

echo ""
echo "concluído: $ok sucesso, $falha falha(s) em $total"
[ "$falha" -eq 0 ]
