#!/usr/bin/env python3
"""Repetição SEM a declaração: um sinal acessível separa repetição real de
coincidência?

A pergunta do GAP aberto em 2026-11-26: a letra declara a ordem cantada
(`{bloco}`/`{canta}`/`(Nx)`) e o motor expande; detectar a repetição **sem** a
declaração é outra coisa. Aqui se mede se dá para fazer isso com o que o
relatório já publica:

  sinal 1  histograma de classes de altura da melodia (F0) no trecho da linha —
           similaridade de cosseno entre as duas ocorrências;
  sinal 2  forma da curva de loudness de curto prazo (EBU R128) — correlação de
           Pearson entre as duas ocorrências.

Classes, contadas ANTES de qualquer acerto (e com o Δt pareado, senão a duração
decide em vez do sinal):

  repetição real   dois trechos da MESMA linha marcada em tempos diferentes
                   (verdade humana: o dono copiou a linha onde ela é cantada de
                   novo);
  coincidência     dois trechos de linhas de texto DIFERENTE, no mesmo Δt.

Se as distribuições se sobrepõem, o sinal não separa — e a hipótese "o áudio acha
a repetição sozinho" fica refutada com o número, em vez de virar código.

uso:
  medir-mir-repeticao.py --bin target/release/mir --audio <audio> \\
      --gabarito <letra marcada por humano, fora do repo>
"""

from __future__ import annotations

import argparse
import json
import math
import os
import re
import statistics
import subprocess
import sys

# Mesmo Δt mínimo para as duas classes: sem isso a classe negativa fica com pares
# curtos e a duração passa a decidir no lugar do sinal.
DT_MIN_S = 8.0
DT_MAX_S = 90.0


def roda(binario: str, audio: str) -> dict:
    r = subprocess.run(
        [binario, audio, "--json"], capture_output=True, text=True, check=False
    )
    if r.returncode != 0:
        raise SystemExit(f"mir falhou ({r.returncode}): {r.stderr.strip()[-300:]}")
    return json.loads(r.stdout)


def seg(t: str) -> float:
    m = re.match(r"(\d+)m(\d+)s", t)
    if not m:
        raise SystemExit(f"tempo fora do formato XmYYs: {t!r}")
    return int(m.group(1)) * 60 + int(m.group(2))


def marca_humana(caminho: str) -> list[tuple[float, str]]:
    with open(caminho) as fh:
        bruto = fh.read()
    marcas = [
        (seg(m.group(1)), m.group(2).strip())
        for m in re.finditer(r"^(\d+m\d+s)\s+(.+)$", bruto, re.MULTILINE)
    ]
    if len(marcas) < 5:
        raise SystemExit(f"gabarito com {len(marcas)} linhas -- suspeito: {caminho}")
    return marcas


def norm(s: str) -> str:
    import unicodedata

    s = unicodedata.normalize("NFD", s.lower())
    s = "".join(c for c in s if unicodedata.category(c) != "Mn")
    return re.sub(r"[^a-z0-9 ]", "", s).strip()


def histograma(dados: dict, inicio: float, fim: float) -> list[float]:
    """Classes de altura da melodia no intervalo, somando os trechos de acorde."""
    soma = [0.0] * 12
    for trecho in dados.get("melodia", []):
        a, b = trecho["inicio_s"], trecho["fim_s"]
        if b <= inicio or a >= fim:
            continue
        peso = min(b, fim) - max(a, inicio)
        for i, n in enumerate(trecho["contagem"]):
            soma[i] += n * peso / max(1e-6, b - a)
    return soma


def cosseno(a: list[float], b: list[float]) -> float:
    na = sum(x * x for x in a) ** 0.5
    nb = sum(x * x for x in b) ** 0.5
    if na <= 0 or nb <= 0:
        return 0.0
    return sum(x * y for x, y in zip(a, b)) / (na * nb)


def curva(dados: dict, inicio: float, fim: float) -> list[float]:
    return [l for t, l in dados["dynamics"]["curva_curto_prazo"] if inicio <= t <= fim]


def pearson(a: list[float], b: list[float]) -> float:
    n = min(len(a), len(b))
    if n < 3:
        return float("nan")
    a, b = a[:n], b[:n]
    ma, mb = statistics.mean(a), statistics.mean(b)
    va = sum((x - ma) ** 2 for x in a)
    vb = sum((y - mb) ** 2 for y in b)
    if va <= 0 or vb <= 0:
        return float("nan")
    return sum((x - ma) * (y - mb) for x, y in zip(a, b)) / (va * vb) ** 0.5


def sem_nan(valores: list[float]) -> list[float]:
    """NaN não é valor: `pearson` devolve NaN quando falta bloco na janela."""
    return [v for v in valores if not math.isnan(v)]


def separa(positivos: list[float], negativos: list[float]) -> tuple[float, float, int]:
    """Melhor acurácia de um único limiar, e quantas negativas cruzam o menor positivo."""
    positivos = sem_nan(positivos)
    negativos = sem_nan(negativos)
    if not positivos or not negativos:
        return (float("nan"), float("nan"), 0)
    melhor, limite = 0.0, float("nan")
    for v in sorted(set(positivos + negativos)):
        acertos = sum(1 for x in positivos if x >= v) + sum(
            1 for y in negativos if y < v
        )
        taxa = acertos / (len(positivos) + len(negativos))
        if taxa > melhor:
            melhor, limite = taxa, v
    # Sobreposição: negativas que passam do MENOR positivo — nenhum limiar separa.
    colados = sum(1 for y in negativos if y >= min(positivos))
    return (melhor, limite, colados)


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("--bin", required=True)
    ap.add_argument("--audio", required=True)
    ap.add_argument("--gabarito", required=True)
    args = ap.parse_args()
    for p in (args.bin, args.audio, args.gabarito):
        if not os.path.exists(p):
            raise SystemExit(f"precondicao ausente: {p}")

    marcas = marca_humana(args.gabarito)
    dados = roda(args.bin, args.audio)
    # A janela de cada linha vai até o início da seguinte — é o que o humano deu.
    janelas = []
    for i, (t, texto) in enumerate(marcas):
        fim = marcas[i + 1][0] if i + 1 < len(marcas) else dados["audio"]["duracao_s"]
        janelas.append((t, fim, texto))

    positivos_cos, negativos_cos = [], []
    positivos_pea, negativos_pea = [], []
    for i, (ti, fi, texto_i) in enumerate(janelas):
        for tj, fj, texto_j in janelas[i + 1 :]:
            dt = abs(tj - ti)
            if not DT_MIN_S <= dt <= DT_MAX_S:
                continue
            mesmo_texto = norm(texto_i) == norm(texto_j)
            cos = cosseno(histograma(dados, ti, fi), histograma(dados, tj, fj))
            pea = pearson(curva(dados, ti, fi), curva(dados, tj, fj))
            if mesmo_texto:
                positivos_cos.append(cos)
                positivos_pea.append(pea)
            else:
                negativos_cos.append(cos)
                negativos_pea.append(pea)

    rotulos = [s["rotulo"] for s in dados["sections"]]
    # O instrumento está vivo? Uma janela comparada consigo mesma tem de dar
    # cosseno 1,0 — se não der, o número de similaridade não significa nada.
    for t, f, _ in janelas[:3]:
        proprio = cosseno(histograma(dados, t, f), histograma(dados, t, f))
        if abs(proprio - 1.0) > 1e-9:
            raise SystemExit(
                f"cosseno de uma janela com ela mesma = {proprio:.6f} (esperado 1.0)"
            )

    print(
        f"# audio={os.path.basename(args.audio)} linhas={len(janelas)} "
        f"pares com Δt entre {DT_MIN_S:g} s e {DT_MAX_S:g} s"
    )
    print(
        f"# classes: repetição real (mesmo texto) = {len(positivos_cos)} par(es) · "
        f"coincidência (texto diferente) = {len(negativos_cos)} par(es)"
    )
    print(
        f"# rótulos de seção do motor: {len(dados['sections'])} seção(ões), "
        f"{len(set(rotulos))} rótulo(s) distinto(s) -> {sorted(set(rotulos))}"
    )
    if not positivos_cos:
        raise SystemExit(
            "sem pares de repetição real na faixa de Δt -- sem classe positiva"
        )
    for nome, p, n in (
        ("cosseno do histograma de F0", positivos_cos, negativos_cos),
        ("Pearson da curva de LUFS", positivos_pea, negativos_pea),
    ):
        pp = sem_nan(p)
        nn = sem_nan(n)
        print(f"\n## {nome}")
        print(
            f"  repetição real  n={len(pp)} mediana={statistics.median(pp):.3f} "
            f"min={min(pp):.3f} max={max(pp):.3f}"
        )
        print(
            f"  coincidência    n={len(nn)} mediana={statistics.median(nn):.3f} "
            f"min={min(nn):.3f} max={max(nn):.3f}"
        )
        melhor, limite, colados = separa(pp, nn)
        # A taxa-base manda: 90% de acurácia num conjunto com 90% de negativas não
        # é informação nenhuma. Sem esta linha, "91%" passaria por sinal.
        taxa_base = max(len(pp), len(nn)) / (len(pp) + len(nn))
        print(
            f"  melhor limiar único: acurácia {100 * melhor:.0f}% em {limite:.3f} · "
            f"{colados}/{len(nn)} coincidência(s) passam do MENOR positivo"
        )
        print(
            f"  taxa-base (responder sempre a classe maior): {100 * taxa_base:.0f}% · "
            f"ganho do sinal: {100 * (melhor - taxa_base):+.0f} ponto(s)"
        )
        # Conjunto BALANCEADO: as negativas reduzidas ao número de positivas, com
        # semente fixa. É o número que separa de verdade (skill `conjunto-de-teste-de-fora`).
        import random

        amostra = random.Random(7).sample(nn, min(len(nn), len(pp)))
        balanceada, _, _ = separa(pp, amostra)
        print(
            f"  balanceado ({len(pp)}×{len(amostra)}): acurácia {100 * balanceada:.0f}% "
            f"(acaso = 50%)"
        )
        if not balanceada > 0.65:
            print(
                "  -> NÃO separa: hipótese 'o áudio acha a repetição sozinho' REFUTADA"
            )
    gaps = [
        abs(tj - ti)
        for i, (ti, _, texto_i) in enumerate(janelas)
        for tj, _, texto_j in janelas[i + 1 :]
        if norm(texto_i) == norm(texto_j)
    ]
    print(
        f"\n# taxa-base: {len(gaps)} repetição(ões) real(is) de texto, distância mediana "
        f"entre ocorrências {statistics.median(gaps):.0f} s (min {min(gaps):.0f} · "
        f"max {max(gaps):.0f})"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
