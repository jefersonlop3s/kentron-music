#!/usr/bin/env bash
# Mede o repertório: roda o motor em cada arquivo e emite uma linha por música.
#
# Não julga acerto — não há gabarito. Mede **plausibilidade e consistência** em
# lote: duração, BPM, tonalidade (com correlação e margem), nº de trechos de
# acorde e nº de seções. É o dado que faltava para sair de "provou em uma música".
#
# Uso:
#   scripts/medir-repertorio.sh ~/Downloads/repertorio
#   scripts/medir-repertorio.sh ~/Downloads/repertorio --tsv /tmp/medicao.tsv
#
# Requisitos: cargo e python3. Os arquivos **não** entram no repositório: o
# script só lê a pasta que você apontar e escreve o resultado onde você pedir.
set -euo pipefail

RAIZ="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PASTA_IN="${1:?uso: scripts/medir-repertorio.sh <pasta> [--tsv saida.tsv]}"
shift
SAIDA="/tmp/medicao-repertorio.tsv"
while [ "$#" -gt 0 ]; do
  case "$1" in
    --tsv) SAIDA="${2:?--tsv precisa de caminho}"; shift 2 ;;
    *) echo "argumento desconhecido: $1" >&2; exit 2 ;;
  esac
done

[ -d "$PASTA_IN" ] || { echo "não é pasta: $PASTA_IN" >&2; exit 1; }

extensoes=(-iname '*.mp3' -o -iname '*.m4a' -o -iname '*.flac' -o -iname '*.wav' -o -iname '*.ogg')
ARQUIVOS=()
while IFS= read -r linha; do
  [ -n "$linha" ] && ARQUIVOS+=("$linha")
done < <(find "$PASTA_IN" -maxdepth 2 -type f \( "${extensoes[@]}" \) | sort)
if [ "${#ARQUIVOS[@]}" -eq 0 ]; then
  echo "nenhum áudio em $PASTA_IN (procurei mp3, m4a, flac, wav, ogg)" >&2
  exit 1
fi

echo "==> compilando o motor"
( cd "$RAIZ" && cargo build -q -p mir_engine )
BIN="$RAIZ/target/debug/mir"

echo "==> ${#ARQUIVOS[@]} arquivo(s); resultado em $SAIDA"
printf 'arquivo\tduracao_s\tafinacao_cents\tbpm\ttonalidade\tcorrelacao\tmargem\ttrechos_acorde\tsecoes\tregra\n' > "$SAIDA"

falhas=0
for arquivo in "${ARQUIVOS[@]}"; do
  if ! json="$("$BIN" "$arquivo" --json 2>/dev/null)"; then
    printf '%s\tERRO\tERRO\tERRO\tERRO\tERRO\tERRO\tERRO\tERRO\n' "$(basename "$arquivo")" >> "$SAIDA"
    falhas=$((falhas + 1))
    continue
  fi
  NOME="$(basename "$arquivo")" JSON="$json" python3 -c '
import json, os
d = json.loads(os.environ["JSON"])
t = d["tonalidade"]
print("\t".join(str(v) for v in [
    os.environ["NOME"],
    round(d["audio"]["duracao_s"], 1),
    round(d["audio"]["afinacao_cents"], 1),
    round(d["tempo"]["bpm"], 1),
    t["nome"] or "indet",
    round(t["correlacao"], 2),
    round(t["margem"], 2),
    len(d["chords"]),
    len(d["sections"]),
    t["regra"].replace("\t", " "),
]))' >> "$SAIDA"
done
[ "$falhas" -eq 0 ] || echo "==> $falhas arquivo(s) ilegível(is) — aparecem como ERRO na tabela"

echo "==> resumo"
python3 - "$SAIDA" <<'PY'
import csv, sys, statistics as st
linhas = list(csv.DictReader(open(sys.argv[1]), delimiter="\t"))
bons = [l for l in linhas if l["bpm"] not in ("ERRO", "PARSE")]
if not bons:
    print("nenhuma música legível")
    sys.exit(1)
def nums(campo):
    return [float(l[campo]) for l in bons if l[campo] not in ("ERRO", "PARSE")]
print(f"  lidas        {len(bons)}/{len(linhas)}")
print(f"  duração      mediana {st.median(nums('duracao_s')):.0f} s")
print(f"  BPM          mediana {st.median(nums('bpm')):.1f}")
print(f"  trechos      mediana {st.median(nums('trechos_acorde')):.0f}")
print(f"  seções       mediana {st.median(nums('secoes')):.0f}")
tons = {}
for l in bons:
    tons[l["tonalidade"]] = tons.get(l["tonalidade"], 0) + 1
print("  tonalidades  " + ", ".join(f"{k}={v}" for k, v in sorted(tons.items(), key=lambda x: -x[1])))
motivos = {}
for l in bons:
    if l["tonalidade"] == "indet":
        r = l["regra"].split(":")[0]
        motivos[r] = motivos.get(r, 0) + 1
for r, v in sorted(motivos.items(), key=lambda x: -x[1]):
    print(f"  indet        {v}× {r}")
fracos = [l for l in bons if float(l["margem"] or 0) < 0.05]
print(f"  margem baixa {len(fracos)} música(s) com margem < 0,05 (decisão frágil)")
PY