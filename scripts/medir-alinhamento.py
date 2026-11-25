#!/usr/bin/env python3
"""Verificador de regressao do alinhamento de linha.

Roda o motor com uma letra MARCADA por humano (ordem cantada + tempo por linha)
e imprime o erro do inicio de linha. O gabarito e conteudo de terceiro e vive
FORA do repositorio: este script so o consome por caminho.

  uso: medir-alinhamento.py --letra <marcada.txt> --audio <audio> --mir <bin>

Falha alto se o gabarito nao existir (precondicao medida no material, nao suposta).
"""

from __future__ import annotations

import argparse
import os
import re
import statistics
import subprocess
import sys
import unicodedata


def norm(s: str) -> str:
    s = unicodedata.normalize("NFD", s.lower())
    s = "".join(c for c in s if unicodedata.category(c) != "Mn")
    return re.sub(r"[^a-z0-9 ]", "", s).strip()


def seg(t: str) -> int:
    m = re.match(r"(\d+)m(\d+)s", t)
    if not m:
        raise SystemExit(f"tempo fora do formato XmYYs: {t!r}")
    return int(m.group(1)) * 60 + int(m.group(2))


def gabarito(caminho: str) -> list[tuple[int, str]]:
    with open(caminho) as fh:
        raw = fh.read()
    marcas = [
        (seg(m.group(1)), m.group(2).strip())
        for m in re.finditer(r"^(\d+m\d+s)\s+(.+)$", raw, re.MULTILINE)
    ]
    if len(marcas) < 5:
        raise SystemExit(f"gabarito com {len(marcas)} linhas — suspeito: {caminho}")
    return marcas


def alinhamento(mir: str, audio: str, letra: str) -> list[tuple[float, str, float]]:
    # O motor recebe a letra LIMPA (sem o prefixo de tempo) — o prefixo e do
    # gabarito, nao da letra. Derivar um do outro evita duas fontes divergentes.
    with open(letra) as fh:
        limpa = [
            m.group(1).strip()
            for m in (re.match(r"^\d+m\d+s\s+(.+)$", ln.rstrip("\n")) for ln in fh)
            if m
        ]
    if not limpa:
        raise SystemExit(f"letra limpa vazia a partir de {letra}")
    import tempfile

    with tempfile.NamedTemporaryFile("w", suffix=".txt", delete=False) as tmp:
        tmp.write("\n".join(limpa) + "\n")
        caminho = tmp.name
    r = subprocess.run(
        [mir, audio, "--letra", caminho], capture_output=True, text=True, check=False
    )
    if r.returncode != 0:
        raise SystemExit(f"mir falhou ({r.returncode}): {r.stderr[-300:]}")
    saida = []
    for linha in r.stdout.splitlines():
        m = re.match(
            r"\s*([\d.]+)s[–-]\s*([\d.]+)s\s+\S+\s+\S+\s+(.*?)\s+[\d.-]+ LUFS\s+conf ([\d.]+)",
            linha,
        )
        if m:
            saida.append((float(m.group(1)), m.group(3).strip(), float(m.group(4))))
    return saida


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--letra", required=True, help="letra marcada por humano (fora do repo)"
    )
    ap.add_argument("--audio", required=True)
    ap.add_argument("--mir", required=True)
    ap.add_argument("--detalhe", action="store_true")
    a = ap.parse_args()
    for p in (a.letra, a.audio, a.mir):
        if not os.path.exists(p):
            raise SystemExit(f"precondicao ausente: {p}")

    hum = gabarito(a.letra)
    eng = alinhamento(a.mir, a.audio, a.letra)
    j, pares = 0, []
    for t, txt in hum:
        n = norm(txt)
        while j < len(eng) and norm(eng[j][1]) != n:
            j += 1
        if j < len(eng):
            pares.append((t, eng[j][0], eng[j][1], eng[j][2]))
            j += 1
    if not pares:
        raise SystemExit(
            "nenhuma linha casada — letra do gabarito e a do motor divergem?"
        )
    erros = sorted(abs(e - h) for h, e, _, _ in pares)
    n = len(erros)
    media_vies = statistics.mean(e - h for h, e, _, _ in pares)
    conf = statistics.mean(c for _, _, _, c in pares)
    print(f"casadas={len(pares)}/{len(hum)}")
    print(
        f"erro_inicio  mediana={erros[n // 2]:.1f}s  p90={erros[min(n - 1, int(0.9 * n))]:.1f}s  max={erros[-1]:.1f}s"
    )
    print(f"vies={media_vies:+.1f}s  conf_media={conf:.2f}")
    piores = sorted(pares, key=lambda p: -abs(p[1] - p[0]))[:5]
    print(
        "piores:", " | ".join(f"{h}s->{e:.0f}s({e - h:+.0f})" for h, e, _, _ in piores)
    )
    if a.detalhe:
        for h, e, txt, c in pares:
            print(f"  {h:4d}s -> {e:7.1f}s ({e - h:+6.1f}s) conf={c:.2f}  {txt[:42]}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
