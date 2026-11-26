#!/usr/bin/env python3
"""Recusa a perda silenciosa de seção num documento append-ordered.

Caso de origem (2026-11-26): o merge `71a0b1b` apagou **três** entradas do
`CHANGELOG.md` — duas seções e uma subseção que existiam em `54d218a` e não
existiam no outro lado do merge. O `git` não acusa: a deleção de um lado, sem
mudança do outro, é aplicada como deleção; e `git log A..B` continua dizendo que
tudo está lá, porque o que se perde é **conteúdo dentro de arquivo**, não commit.

A regra: entre dois revs, o número de títulos `## ` do arquivo **não pode
diminuir**. Renomear e mover preserva a contagem; consolidar duas seções numa
também baixa — e aí a exceção é consciente (`HARNESS_SKIP=changelog`).

  uso: check_changelog.py --base <rev> --novo <rev> [--arquivo CHANGELOG.md]

Sai 0 quando não há perda (ou não há base), 1 quando há, e 2 quando não
consegue medir — falha alto, nunca silenciosa.
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys

TITULO = re.compile(r"^## .*$", re.MULTILINE)


def titulos(rev: str, caminho: str) -> list[str] | None:
    """Títulos `## ` do arquivo naquele rev. `None` se o arquivo não existe lá."""
    if rev == "INDEX":
        spec = f":{caminho}"
    else:
        spec = f"{rev}:{caminho}"
    r = subprocess.run(["git", "show", spec], capture_output=True, text=True, check=False)
    if r.returncode != 0:
        return None
    return [t.strip() for t in TITULO.findall(r.stdout)]


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--base", default="")
    ap.add_argument("--novo", required=True, help="rev, ou INDEX para o index")
    ap.add_argument("--arquivo", default="CHANGELOG.md")
    a = ap.parse_args()

    if not a.base:
        return 0  # sem base (ref nova) não há perda a medir

    antes = titulos(a.base, a.arquivo)
    depois = titulos(a.novo, a.arquivo)
    if antes is None or depois is None:
        return 0  # arquivo criado ou removido: não é este gate

    # Conjunto, NÃO contagem. Medido no caso de origem: o merge `71a0b1b`
    # apagou três entradas minhas e trouxe oito de outra sessão — o total
    # AUMENTOU, e uma regra por contagem teria deixado passar.
    perdidos = []
    restantes = list(depois)
    for t in antes:
        if t in restantes:
            restantes.remove(t)
        else:
            perdidos.append(t)
    if not perdidos:
        return 0

    print(
        f"{a.arquivo}: {len(perdidos)} de {len(antes)} títulos `## ` da base "
        f"não estão no novo ({len(depois)} títulos)",
        file=sys.stderr,
    )
    for t in perdidos:
        print(f"  sumiu: {t[:110]}", file=sys.stderr)
    print(
        "\nO que se perde aqui não aparece em `git log A..B` — é conteúdo dentro\n"
        "de arquivo. Readicione o título; renomear é mudar o texto do título, e\n"
        "para isso a exceção é consciente: HARNESS_SKIP=changelog",
        file=sys.stderr,
    )
    return 1


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as erro:  # noqa: BLE001 — falha alta, nunca silenciosa
        print(f"check_changelog: não consegui medir: {erro}", file=sys.stderr)
        sys.exit(2)
