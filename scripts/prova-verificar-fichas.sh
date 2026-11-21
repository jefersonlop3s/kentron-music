#!/usr/bin/env bash
# Prova o gate no sentido negativo: um gate que nunca reprova não verifica nada.
set -u
cd "$(dirname "$0")/.." || exit 1
BASE=crates/mir_engine/assets/fichas_hinario.tsv
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
falhas=0

prova() {
  local nome="$1" arquivo="$2"
  if python3 scripts/verificar_fichas.py "$arquivo" >/dev/null 2>&1; then
    echo "PROVA FALHOU: $nome — o gate APROVOU o que devia reprovar"
    falhas=$((falhas + 1))
  else
    echo "ok: $nome reprovou como esperado"
  fi
}

python3 - "$BASE" "$TMP" <<'PY'
import pathlib, sys
base, tmp = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
todas = base.read_text(encoding='utf-8').splitlines()
# `#` comenta no asset (mesma convencao do corpus): a prova tambem ignora,
# senao o cabecalho lido seria a primeira linha de comentario.
dados = [l for l in todas if l.strip() and not l.lstrip().startswith('#')]
cab = dados[0].split('\t')
i_morte, i_ver, i_pd, i_fonte = cab.index('ano_relevante'), cab.index('veredito'), \
    cab.index('ano_dominio_publico'), cab.index('fonte')

def variar(nome, muda):
    saida = [dados[0]]
    for l in dados[1:]:
        c = l.split('\t')
        muda(c)
        saida.append('\t'.join(c))
    (tmp / nome).write_text('\n'.join(saida) + '\n', encoding='utf-8')

variar('sem-ano.tsv', lambda c: c.__setitem__(i_pd, '') if c[i_ver] == 'pd' else None)
variar('protegido-sem-morte.tsv', lambda c: c.__setitem__(i_morte, '') if c[i_ver] == 'protegido' else None)
variar('sem-fonte.tsv', lambda c: c.__setitem__(i_fonte, ''))
(tmp / 'com-letra.tsv').write_text(
    '\n'.join([dados[0] + '\tletra'] + [l + '\ttexto qualquer' for l in dados[1:]]) + '\n',
    encoding='utf-8')
# Caso novo (2026-11-21): ignorar `#` nao pode transformar um arquivo so de
# comentarios e cabecalho em asset aprovado.
(tmp / 'so-cabecalho.tsv').write_text('# so comentario\n' + dados[0] + '\n', encoding='utf-8')
PY

prova 'registro pd sem ano de domínio público' "$TMP/sem-ano.tsv"
prova 'registro protegido sem ano de morte'      "$TMP/protegido-sem-morte.tsv"
prova 'procedência (fonte) vazia'                "$TMP/sem-fonte.tsv"
prova 'coluna proibida de letra'                 "$TMP/com-letra.tsv"
prova 'asset só com comentários e cabeçalho'     "$TMP/so-cabecalho.tsv"

if [ "$falhas" -ne 0 ]; then
  echo "REPROVADO: $falhas caso(s) não reprovaram — o gate é fraco"
  exit 1
fi
echo "APROVADO: o gate reprovou os 5 casos negativos"
