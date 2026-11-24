#!/bin/bash
# cifrar-10.sh — Seleciona 10 músicas aleatórias de ~/Music/MP3/ e gera a cifra
#
# Uso: ./scripts/cifrar-10.sh
# O motor MIR analisa cada música e produz a saída em ChordPro.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MUSIC_DIR="$HOME/Music/MP3"
SAIDA_DIR="/tmp/cifras-10"
mkdir -p "$SAIDA_DIR"

echo "==> Coletando arquivos de áudio em $MUSIC_DIR …"
ARQUIVOS=()
while IFS= read -r f; do
    ARQUIVOS+=("$f")
done < <(find "$MUSIC_DIR" -type f \( -name '*.mp3' -o -name '*.m4a' -o -name '*.flac' -o -name '*.wav' \) 2>/dev/null | sort)

TOTAL="${#ARQUIVOS[@]}"
echo "    $TOTAL arquivo(s) encontrado(s)"

if [ "$TOTAL" -eq 0 ]; then
    echo "❌ Nenhum arquivo de áudio encontrado em $MUSIC_DIR"
    exit 1
fi

# Selecionar 10 aleatoriamente com perl
SEED="$RANDOM"
printf "%s\n" "${ARQUIVOS[@]}" > /tmp/.cifrar-lista.txt
ESCOLHIDOS=()
while IFS= read -r f; do
    ESCOLHIDOS+=("$f")
done < <(perl -MList::Util=shuffle -e '
    srand($ARGV[0]);
    chomp(my @a = <STDIN>);
    @a = shuffle(@a);
    splice(@a, $ARGV[1]) if @a > $ARGV[1];
    print "$_\n" for @a;
' "$SEED" 10 < /tmp/.cifrar-lista.txt)

echo ""
echo "============================================"
echo " 10 MÚSICAS SORTEADAS"
echo "============================================"
for f in "${ESCOLHIDOS[@]}"; do
    nome="$(basename "$f")"
    echo "  • $nome"
done
echo ""

cd "$REPO_ROOT"

# Compilar o motor primeiro (uma vez só)
echo "==> Compilando o motor MIR …"
cargo build -p mir_engine --quiet 2>&1 || cargo build -p mir_engine

echo ""
echo "============================================"
echo " ANÁLISE"
echo "============================================"

SUCESSO=0
FALHA=0

for f in "${ESCOLHIDOS[@]}"; do
    nome="$(basename "$f")"
    echo ""
    echo "────────────────────────────────────────"
    echo "🎵 $nome"
    echo "   $f"
    echo "────────────────────────────────────────"

    # Nome sanitizado para o arquivo de saída
    sanitizado="$(echo "$nome" | sed 's/[^a-zA-Z0-9._-]/_/g' | sed 's/\.mp3\|\.m4a\|\.flac\|\.wav$//')"
    saida="$SAIDA_DIR/${sanitizado}.cho"

    # Executar o motor com --chordpro
    if cargo run -p mir_engine -- "$f" --chordpro > "$saida" 2>&1; then
        echo "✅ OK → $saida"
        wc -l "$saida" | awk '{print "   " $1 " linha(s) na cifra"}'
        SUCESSO=$((SUCESSO + 1))
    else
        echo "❌ FALHA"
        head -5 "$saida" 2>/dev/null || true
        FALHA=$((FALHA + 1))
    fi
done

echo ""
echo "============================================"
echo " RESUMO"
echo "============================================"
echo "   ✅ $SUCESSO cifra(s) gerada(s) em $SAIDA_DIR/"
echo "   ❌ $FALHA falha(s)"
echo ""
echo "Arquivos gerados:"
ls -1 "$SAIDA_DIR/"*.cho 2>/dev/null || echo "(nenhum)"

exit $((FALHA > 0))