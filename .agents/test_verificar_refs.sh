#!/usr/bin/env bash
# shellcheck disable=SC2016  # os conteúdos abaixo são Markdown literal, não shell
# =============================================================================
# Prova do verificador de referências entre `.md` (`.harness/verificar_refs.py`)
# =============================================================================
# Caso de origem (2026-11-26): o verificador **reprovava a `main`** porque
# `docs/CONCLUSION.md` cita `` `AGENTS.md` `` em prosa e ele resolvia o nome
# relativo a `docs/` — o arquivo mora na **raiz**. O conserto aceita a raiz como
# segundo candidato, e este teste prova os **dois lados**: nome que só existe na
# raiz passa, alvo que não existe em lugar nenhum continua reprovando (é o que
# impede o conserto de virar "verificador que nunca reprova").
#
# Uso: .agents/test_verificar_refs.sh
# =============================================================================
set -uo pipefail

AQUI=$(cd "$(dirname "$0")" && pwd)
VER="$AQUI/../.harness/verificar_refs.py"
[ -f "$VER" ] || { echo "falta $VER"; exit 2; }

T=$(mktemp -d) || exit 1
trap 'rm -rf "$T"' EXIT
mkdir -p "$T/docs"
printf '# Raiz\n' > "$T/raiz.md"

falhas=0
ok()  { echo "✅ $1"; }
nok() { echo "❌ $1"; falhas=$((falhas + 1)); }

# tenta <0|1> <trecho esperado, ou -> <descrição> <conteúdo do docs/a.md>
tenta() {
    local quer=$1 trecho=$2 desc=$3 conteudo=$4
    printf '%s\n' "$conteudo" > "$T/docs/a.md"
    local out rc
    out=$(python3 "$VER" --docs "$T/docs" --raiz "$T" 2>&1); rc=$?
    [ "$rc" -ne 0 ] && rc=1
    if [ "$rc" -eq "$quer" ] && { [ "$trecho" = "-" ] || grep -qF -- "$trecho" <<<"$out"; }; then
        ok "$desc"
    else
        nok "$desc (exit=$rc, esperado=$quer, trecho='$trecho'): $out"
    fi
}

tenta 0 - "nome de arquivo que so existe na raiz passa (o caso do AGENTS.md)" \
    'Registrada na tabela do `raiz.md`.'
tenta 0 - "link relativo de verdade continua passando" \
    'Veja [a raiz](../raiz.md).'
tenta 0 - "code span que é TEXTO de link não conta como alvo" \
    'Veja [`raiz.md`](../raiz.md).'
tenta 1 'naoexiste.md' "alvo ausente em qualquer lugar continua reprovando" \
    'Isto nao existe: `naoexiste.md`.'
tenta 1 '../naoexiste2.md' "link relativo ausente continua reprovando" \
    'Isto tambem nao: [x](../naoexiste2.md).'

if [ "$falhas" -eq 0 ]; then
    echo "APROVADO: o verificador aceita a raiz e continua reprovando o que falta"
    exit 0
fi
echo "$falhas caso(s) reprovado(s)"
exit 1
