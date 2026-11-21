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
col = {c: i for i, c in enumerate(dados[0].split('\t'))}

def variar(nome, muda):
    saida = [dados[0]]
    for l in dados[1:]:
        c = l.split('\t')
        muda(c)
        saida.append('\t'.join(c))
    (tmp / nome).write_text('\n'.join(saida) + '\n', encoding='utf-8')

# 1) 'pd' sem ano de dominio publico
def sem_ano(c):
    if c[col['veredito']] == 'pd':
        c[col['ano_dominio_publico']] = ''
variar('sem-ano.tsv', sem_ano)

# 2) 'protegido' sem ano de morte relevante
def prot_sem_morte(c):
    if c[col['veredito']] == 'protegido':
        c[col['ano_relevante']] = ''
variar('protegido-sem-morte.tsv', prot_sem_morte)

# 3) fato sem procedencia: autoria preenchida, fonte_autoria vazia
def fato_sem_fonte(c):
    if c[col['autoria']].strip():
        c[col['fonte_autoria']] = ''
variar('fato-sem-fonte.tsv', fato_sem_fonte)

# 4) procedencia que cita OCR sem observacao (o caso de 2026-11-21)
def ocr_sem_observacao(c):
    if c[col['traducao']].strip():
        c[col['fonte_traducao']] = 'indice lido por OCR (teste)'
        c[col['observacao']] = ''
variar('ocr-sem-observacao.tsv', ocr_sem_observacao)

# 5) coluna proibida: letra
(tmp / 'com-letra.tsv').write_text(
    '\n'.join([dados[0] + '\tletra'] + [l + '\ttexto qualquer' for l in dados[1:]]) + '\n',
    encoding='utf-8')

# 6) so comentarios e cabecalho
(tmp / 'so-cabecalho.tsv').write_text('# so comentario\n' + dados[0] + '\n', encoding='utf-8')

# 7) ano de dominio publico fora da aritmetica (Brasil: morte + 71)
def ano_fora_da_aritmetica(c):
    if c[col['veredito']] in ('pd', 'protegido') and c[col['ano_relevante']].strip():
        c[col['ano_dominio_publico']] = str(int(c[col['ano_relevante']]) + 70)
variar('ano-errado.tsv', ano_fora_da_aritmetica)
PY

prova 'registro pd sem ano de domínio público'     "$TMP/sem-ano.tsv"
prova 'registro protegido sem ano de morte'        "$TMP/protegido-sem-morte.tsv"
prova 'fato sem procedência (fonte_autoria vazia)' "$TMP/fato-sem-fonte.tsv"
prova 'procedência que cita OCR sem observação'    "$TMP/ocr-sem-observacao.tsv"
prova 'coluna proibida de letra'                   "$TMP/com-letra.tsv"
prova 'asset só com comentários e cabeçalho'       "$TMP/so-cabecalho.tsv"
prova 'ano de domínio público fora de morte+71'    "$TMP/ano-errado.tsv"

if [ "$falhas" -ne 0 ]; then
  echo "REPROVADO: $falhas caso(s) não reprovaram — o gate é fraco"
  exit 1
fi
echo "APROVADO: o gate reprovou os 7 casos negativos"
