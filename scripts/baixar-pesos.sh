#!/usr/bin/env bash
# Baixa o modelo ONNX htdemucs_6s e verifica o hash.
# Uso:  scripts/baixar-pesos.sh
#
# O modelo é colocado em models/weights/htdemucs_6s/htdemucs_6s.onnx.
# O hash SHA-256 está no MANIFEST.toml.

set -euo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
cd "$HERE"

MODEL_DIR="models/weights/htdemucs_6s"
MODEL_PATH="$MODEL_DIR/htdemucs_6s.onnx"

# Hash esperado (do MANIFEST.toml)
EXPECTED="7ce55792e2231c93fbf92de95f5fd5b3a5e6c89f7db690dfd693e8f1dce56869"

# URL do modelo (fp16, 136 MB)
URL="https://huggingface.co/StemSplitio/htdemucs-6s-onnx/resolve/main/htdemucs_6s_fp16weights.onnx"

if [ -f "$MODEL_PATH" ]; then
    echo "✓ modelo já existe em $MODEL_PATH"
    echo "verificando hash…"
    HASH=$(shasum -a 256 "$MODEL_PATH" | cut -d' ' -f1)
    if [ "$HASH" = "$EXPECTED" ]; then
        echo "✓ hash ok"
        exit 0
    else
        echo "✗ hash não confere (esperado $EXPECTED, obtido $HASH)"
        echo "  baixando novamente…"
        rm "$MODEL_PATH"
    fi
fi

mkdir -p "$MODEL_DIR"

echo "baixando modelo ONNX (htdemucs_6s, fp16, ~136 MB)…"
echo "  $URL"
curl -L -o "$MODEL_PATH" "$URL"

echo "verificando hash…"
HASH=$(shasum -a 256 "$MODEL_PATH" | cut -d' ' -f1)
if [ "$HASH" != "$EXPECTED" ]; then
    echo "✗ hash não confere"
    echo "  esperado: $EXPECTED"
    echo "  obtido:   $HASH"
    rm "$MODEL_PATH"
    exit 1
fi

echo "✓ modelo baixado e verificado: $MODEL_PATH ($(du -h "$MODEL_PATH" | cut -f1))"
