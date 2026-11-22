#!/usr/bin/env bash
# Prova o gate no sentido negativo: um gate que nunca reprova não verifica nada.
set -u
cd "$(dirname "$0")/.." || exit 1
BASE=crates/mir_engine/assets/fichas_hinario.tsv
CORPUS=crates/mir_engine/assets/hinos_dominio_publico.tsv
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
falhas=0
# O verificador resolve o corpus ao lado do asset. Sem esta cópia, as variações
# em $TMP reprovariam por "corpus ausente" e não pelo motivo que dizem provar.
cp "$CORPUS" "$TMP/"

# prova <nome> <arquivo> [motivo esperado na saída]
# O terceiro argumento é opcional: quando dado, o caso só passa se a reprovação
# citar o motivo — senão a prova estaria aprovando pelo motivo errado.
prova() {
  local nome="$1" arquivo="$2" esperado="${3:-}" saida
  if saida=$(python3 scripts/verificar_fichas.py "$arquivo" 2>&1); then
    echo "PROVA FALHOU: $nome — o gate APROVOU o que devia reprovar"
    falhas=$((falhas + 1))
  elif [ -n "$esperado" ] && ! grep -qF -- "$esperado" <<<"$saida"; then
    echo "PROVA FALHOU: $nome — reprovou, mas não pelo motivo esperado"
    echo "  esperado: $esperado"
    echo "  saída:    $(head -1 <<<"$saida")"
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

# 8) vinculo declarado para obra que nao existe no corpus
def vinculo_inexistente(c):
    if c[col['numero']] == '09':
        c[col['obra_original']] = 'Obra Que Nao Existe'
variar('vinculo-inexistente.tsv', vinculo_inexistente)

# 9) vinculo cujo autor nao confere com o do corpus
def vinculo_autor_errado(c):
    if c[col['numero']] == '03':
        c[col['obra_original']] = 'Holy, Holy, Holy'
variar('vinculo-autor-errado.tsv', vinculo_autor_errado)
PY

prova 'registro pd sem ano de domínio público'     "$TMP/sem-ano.tsv"
prova 'registro protegido sem ano de morte'        "$TMP/protegido-sem-morte.tsv"
prova 'fato sem procedência (fonte_autoria vazia)' "$TMP/fato-sem-fonte.tsv"
prova 'procedência que cita OCR sem observação'    "$TMP/ocr-sem-observacao.tsv"
prova 'coluna proibida de letra'                   "$TMP/com-letra.tsv"
prova 'asset só com comentários e cabeçalho'       "$TMP/so-cabecalho.tsv"
prova 'ano de domínio público fora de morte+71'    "$TMP/ano-errado.tsv"
prova 'obra original que não está no corpus'       "$TMP/vinculo-inexistente.tsv" 'não está no corpus de letras'
prova 'obra original com autor que não confere'    "$TMP/vinculo-autor-errado.tsv" 'mas a ficha declara'

if [ "$falhas" -ne 0 ]; then
  echo "REPROVADO: $falhas caso(s) não reprovaram — o gate é fraco"
  exit 1
fi
echo "APROVADO: o gate reprovou os 9 casos negativos"
