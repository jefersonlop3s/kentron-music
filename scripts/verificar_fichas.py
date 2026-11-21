#!/usr/bin/env python3
"""Verifica o asset de fichas do hinário (skill `hinario-e-dominio-publico`).

0. `#` comenta e é ignorado — mesma convenção do corpus `hinos_dominio_publico.tsv`;
   a primeira linha não-comentada é o cabeçalho;
1. toda ficha tem par (hinário, número) não vazio;
2. veredito é um de: pd, protegido, gap, sem_ficha;
3. pd e protegido exigem ano de morte relevante E ano de domínio público;
4. gap exige observação explicando o dado que falta;
5. procedência (fonte) não pode ser vazia;
6. o asset não pode ter coluna de letra/cifra, nem campo longo (limite legal).
Uso: verificar_fichas.py [caminho do tsv]
"""

import pathlib
import sys

VEREDITOS = {"pd", "protegido", "gap", "sem_ficha"}
LIMITE_CARACTERES = 300
PROIBIDOS = ("letra", "cifra", "tablatura")

caminho = pathlib.Path(
    sys.argv[1] if len(sys.argv) > 1 else "crates/mir_engine/assets/fichas_hinario.tsv"
)
if not caminho.exists():
    sys.exit(f"REPROVADO: asset não encontrado em {caminho}")
linhas = [
    texto
    for texto in caminho.read_text(encoding="utf-8").splitlines()
    if texto.strip() and not texto.lstrip().startswith("#")
]
if len(linhas) < 2:
    sys.exit("REPROVADO: asset vazio ou só com cabeçalho")

colunas = linhas[0].split("\t")
for proibido in PROIBIDOS:
    if any(proibido in c.lower() for c in colunas):
        sys.exit(
            f'REPROVADO: coluna proibida "{proibido}" — letra e cifra da edição não entram'
        )
obrigatorias = ("hinario", "numero", "veredito", "fonte")
faltando = [c for c in obrigatorias if c not in colunas]
if faltando:
    sys.exit(f"REPROVADO: colunas obrigatórias ausentes: {faltando}")

idx = {c: i for i, c in enumerate(colunas)}
falhas, contagem = [], {}
for n, linha in enumerate(linhas[1:], start=2):
    campos = linha.split("\t")
    if len(campos) != len(colunas):
        falhas.append(f"linha {n}: {len(campos)} campos contra {len(colunas)} colunas")
        continue
    reg = {c: campos[i] for c, i in idx.items()}
    veredito = reg["veredito"].strip()
    contagem[veredito] = contagem.get(veredito, 0) + 1
    if not reg["hinario"].strip() or not reg["numero"].strip():
        falhas.append(f"linha {n}: sem par (hinário, número)")
    if veredito not in VEREDITOS:
        falhas.append(f'linha {n}: veredito inválido "{veredito}"')
    if veredito in ("pd", "protegido"):
        if not reg.get("ano_relevante", "").strip().isdigit():
            falhas.append(
                f'linha {n}: veredito "{veredito}" sem ano de morte relevante'
            )
        if not reg.get("ano_dominio_publico", "").strip().isdigit():
            falhas.append(
                f'linha {n}: veredito "{veredito}" sem ano de domínio público'
            )
    if veredito == "gap" and not reg.get("observacao", "").strip():
        falhas.append(f'linha {n}: veredito "gap" sem observação do dado que falta')
    if not reg["fonte"].strip():
        falhas.append(f"linha {n}: procedência (fonte) vazia")
    for c, v in reg.items():
        if len(v) > LIMITE_CARACTERES:
            falhas.append(
                f'linha {n}: campo "{c}" com {len(v)} caracteres '
                f"(limite {LIMITE_CARACTERES}) — texto longo sugere letra colada"
            )

if falhas:
    print(f"REPROVADO — {len(falhas)} falha(s):")
    for f in falhas:
        print("  ·", f)
    sys.exit(1)
print(f"APROVADO — {len(linhas) - 1} fichas")
for v in sorted(contagem):
    print(f"  {v}: {contagem[v]}")
