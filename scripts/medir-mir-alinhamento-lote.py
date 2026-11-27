#!/usr/bin/env python3
"""As constantes do alinhamento de linha generalizam no repertorio?

Duas perguntas diferentes na mesma saida, porque uma nao substitui a outra:

 1. CONSTANTES (todos os casos, sem letra): o detector de frase vocal. Quantas
    frases, que cobertura, quantas caem na BANDA DE DECISAO do corte de 1,5 s
    (o corte so muda o destino de frase de 1,0 a 2,0 s), quantas passam de 20 s,
    se caiu no fallback de frase unica -- e quanto da voz medida por OUTRO
    detector (os quadros com F0 da propria melodia) cai FORA das frases. E a
    pergunta do piso por mediana: ele corta voz?

 2. ERRO (so onde existe referencia humana marcada): erro do inicio de linha
    contra a marcacao humana, e a CALIBRACAO do indicador barato -- taxa de
    silabas plausivel ou nao -- contra esse erro. Sem a calibracao o indicador
    nao vale para os casos sem verdade: seria so um numero.

Sem letra nao existe alinhamento: mede-se a constante, nao o acerto. O que nao
foi medido e dito assim, e nao vira zero.

Prova negativa: com --prova-negativa as linhas da letra sao embaralhadas de
proposito; a taxa de linhas com silabas implausiveis tem de SUBIR. Se nao subir,
o indicador nao mede alinhamento.

uso:
  medir-mir-alinhamento-lote.py --bin target/release/mir --pasta ~/Music/MP3
  medir-mir-alinhamento-lote.py --bin target/release/mir --pasta ~/Music/MP3 \\
      --gabarito ~/Downloads/letra-marcada.txt --audio-gabarito "caminho.m4a" \\
      [--recursivo] [--limite N] [--prova-negativa]

O gabarito e material de terceiro e vive FORA do repositorio: entra por caminho,
e so os numeros saem.
"""

from __future__ import annotations

import argparse
import json
import os
import random
import re
import statistics
import subprocess
import sys
import tempfile
import unicodedata

# Faixa plausivel de silabas por segundo em canto. Abaixo disso a linha foi
# esticada por um span longo demais; acima, comprimida. Calibrada no gabarito
# humano -- nao escolhida no escuro.
SILABAS_MIN = 1.5
SILABAS_MAX = 6.0

VOGAIS = "aeiouyáéíóúâêôãõàü"


def silabas(texto: str) -> int:
    """Conta nucleos vocais: o mesmo criterio de `lyrics::silabas` (grupo de vogais)."""
    return max(1, len(re.findall(f"[{VOGAIS}]+", texto.lower())))


def norm(s: str) -> str:
    s = unicodedata.normalize("NFD", s.lower())
    s = "".join(c for c in s if unicodedata.category(c) != "Mn")
    return re.sub(r"[^a-z0-9 ]", "", s).strip()


def seg(t: str) -> int:
    m = re.match(r"(\d+)m(\d+)s", t)
    if not m:
        raise SystemExit(f"tempo fora do formato XmYYs: {t!r}")
    return int(m.group(1)) * 60 + int(m.group(2))


def roda(binario: str, args: list[str]) -> str:
    r = subprocess.run([binario, *args], capture_output=True, text=True, check=False)
    if r.returncode != 0:
        raise SystemExit(
            f"mir falhou ({r.returncode}) em {args[0]}: {r.stderr.strip()[-300:]}"
        )
    return r.stdout


def frases(binario: str, audio: str) -> tuple[list[tuple[float, float]], float, float]:
    """Frases vocais e (cobertura, duracao) do caso."""
    saida = roda(binario, [audio, "--frases-vocais"])
    cab = re.search(r"cobertura ([\d.]+) s de ([\d.]+) s", saida)
    if not cab:
        raise SystemExit(f"saida sem cobertura em {audio}: {saida[:200]}")
    achadas = [
        (float(a), float(b))
        for a, b, _ in re.findall(r"([\d.]+)s[–-]\s*([\d.]+)s\s+\(([\d.]+) s\)", saida)
    ]
    return achadas, float(cab.group(1)), float(cab.group(2))


def relatorio_json(binario: str, audio: str) -> dict:
    return json.loads(roda(binario, [audio, "--json"]))


def voz_fora_das_frases(
    dados: dict, achadas: list[tuple[float, float]]
) -> tuple[float, float]:
    """Quadros com F0 dentro e fora das frases detectadas.

    Estimativa declarada: a melodia publica `quadros_com_f0` por trecho de
    acorde, e o peso de cada trecho e a fracao dele que cai dentro de alguma
    frase. A granularidade e a do trecho de acorde -- nao ha F0 por quadro no
    relatorio.
    """
    dentro = 0.0
    total = 0.0
    for trecho in dados.get("melodia", []):
        f0 = float(trecho["quadros_com_f0"])
        total += f0
        if f0 <= 0 or not achadas:
            continue
        dur = trecho["fim_s"] - trecho["inicio_s"]
        if dur <= 0:
            continue
        coberto = sum(
            max(0.0, min(trecho["fim_s"], b) - max(trecho["inicio_s"], a))
            for a, b in achadas
        )
        dentro += f0 * min(1.0, coberto / dur)
    return dentro, total


def constantes(binario: str, audio: str) -> dict:
    achadas, cob, dur = frases(binario, audio)
    dados = relatorio_json(binario, audio)
    dentro, total = voz_fora_das_frases(dados, achadas)
    duracoes = sorted(b - a for a, b in achadas)
    n = len(duracoes)
    fallback = (
        "sim"
        if (n == 1 and cob > 0.85 * dur)
        else ("sim (0 frases)" if n == 0 else "nao")
    )
    return {
        "arquivo": os.path.basename(audio),
        "duracao_s": dur,
        "frases": n,
        "cobertura_pct": 100.0 * cob / dur if dur > 0 else 0.0,
        "mediana_s": statistics.median(duracoes) if duracoes else 0.0,
        "min_s": duracoes[0] if duracoes else 0.0,
        "max_s": duracoes[-1] if duracoes else 0.0,
        # O corte de 1,5 s so decide o destino das frases desta banda: 1,0 s
        # recebe zero linha, 2,0 s recebe uma. Fora dela o corte e inerte.
        "banda_1a2_s": sum(1 for d in duracoes if 1.0 <= d < 2.0),
        "acima_20s": sum(1 for d in duracoes if d > 20.0),
        "fallback": fallback,
        "voz_fora_pct": 100.0 * (total - dentro) / total if total > 0 else 0.0,
        "quadros_f0": int(total),
    }


def alinhamento(
    binario: str, audio: str, letra: str, embaralhar: int = 0
) -> list[tuple[float, float, str]]:
    """Linhas alinhadas pelo motor: (inicio, fim, texto)."""
    with open(letra) as fh:
        linhas = [
            m.group(1).strip()
            for m in (re.match(r"^\d+m\d+s\s+(.+)$", ln.rstrip("\n")) for ln in fh)
            if m
        ]
    if not linhas:
        raise SystemExit(f"letra limpa vazia a partir de {letra}")
    if embaralhar:
        random.Random(embaralhar).shuffle(linhas)
    with tempfile.NamedTemporaryFile("w", suffix=".txt", delete=False) as tmp:
        tmp.write("\n".join(linhas) + "\n")
        caminho = tmp.name
    try:
        saida = roda(binario, [audio, "--letra", caminho])
    finally:
        os.unlink(caminho)
    achadas = []
    for linha in saida.splitlines():
        m = re.match(
            r"\s*([\d.]+)s[–-]\s*([\d.]+)s\s+\S+\s+\S+\s+(.*?)\s+[\d.-]+ LUFS\s+conf ([\d.]+)",
            linha,
        )
        if m:
            achadas.append((float(m.group(1)), float(m.group(2)), m.group(3).strip()))
    return achadas


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


def parear(
    hum: list[tuple[float, str]], eng: list[tuple[float, float, str]]
) -> list[tuple[float, float, float, str]]:
    j, pares = 0, []
    for t, texto in hum:
        alvo = norm(texto)
        while j < len(eng) and norm(eng[j][2]) != alvo:
            j += 1
        if j < len(eng):
            pares.append((t, eng[j][0], eng[j][1], eng[j][2]))
            j += 1
    return pares


def catalogo(pasta: str, recursivo: bool) -> list[str]:
    exts = (".mp3", ".m4a", ".flac", ".wav", ".ogg")
    if recursivo:
        achados = []
        for raiz, _, arquivos in os.walk(pasta):
            achados += [
                os.path.join(raiz, a) for a in arquivos if a.lower().endswith(exts)
            ]
        return sorted(achados)
    return sorted(
        os.path.join(pasta, a) for a in os.listdir(pasta) if a.lower().endswith(exts)
    )


def calibra_indicador(
    pares: list[tuple[float, float, float, str]],
) -> tuple[list[tuple[float, float, str]], list[tuple[float, float, str]]]:
    """Separa as linhas em (taxa plausivel, taxa fora da faixa), com o erro de cada uma."""
    ordenadas = sorted(pares, key=lambda p: p[1])
    taxas = []
    for i, (humano, inicio, fim, texto) in enumerate(ordenadas):
        limite = ordenadas[i + 1][1] if i + 1 < len(ordenadas) else fim
        taxa = silabas(texto) / max(0.1, limite - inicio)
        taxas.append((taxa, abs(inicio - humano), texto))
    return (
        [c for c in taxas if SILABAS_MIN <= c[0] <= SILABAS_MAX],
        [c for c in taxas if not (SILABAS_MIN <= c[0] <= SILABAS_MAX)],
    )


def mediana_erro(linhas) -> float:
    return statistics.median([x[1] for x in linhas]) if linhas else float("nan")


def taxa_fora_da_faixa(alinhadas: list[tuple[float, float, str]]) -> int:
    return sum(
        1
        for inicio, fim, texto in alinhadas
        if not (SILABAS_MIN <= silabas(texto) / max(0.1, fim - inicio) <= SILABAS_MAX)
    )


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("--bin", required=True)
    ap.add_argument("--pasta", required=True)
    ap.add_argument("--limite", type=int, default=0)
    ap.add_argument("--recursivo", action="store_true")
    ap.add_argument("--gabarito", help="letra marcada por humano (fora do repo)")
    ap.add_argument("--audio-gabarito", help="audio do gabarito")
    ap.add_argument("--prova-negativa", action="store_true")
    args = ap.parse_args()

    for p in (args.bin, args.pasta):
        if not os.path.exists(p):
            raise SystemExit(f"precondicao ausente: {p}")

    audios = catalogo(args.pasta, args.recursivo)
    if args.limite:
        audios = audios[: args.limite]
    if not audios:
        raise SystemExit(f"nenhum audio em {args.pasta}")

    print(
        f"# bin={args.bin} pasta={args.pasta} casos={len(audios)} recursivo={args.recursivo}"
    )
    print(
        "arquivo\tfrases\tcob%\tmediana_s\tmin_s\tmax_s\tbanda_1-2s\tacima_20s\tfallback\tvoz_fora%"
    )
    casos = [constantes(args.bin, a) for a in audios]
    for c in casos:
        print(
            f"{c['arquivo'][:38]}\t{c['frases']}\t{c['cobertura_pct']:.0f}\t{c['mediana_s']:.1f}\t"
            f"{c['min_s']:.1f}\t{c['max_s']:.1f}\t{c['banda_1a2_s']}\t{c['acima_20s']}\t"
            f"{c['fallback']}\t{c['voz_fora_pct']:.0f}"
        )
    cobs = sorted(c["cobertura_pct"] for c in casos)
    fora = sorted(c["voz_fora_pct"] for c in casos)
    print(
        f"\n# n={len(casos)}  cobertura mediana={cobs[len(cobs) // 2]:.0f}% min={cobs[0]:.0f}%  "
        f"voz fora das frases mediana={fora[len(fora) // 2]:.0f}% max={fora[-1]:.0f}%"
    )
    print(
        "# frases na banda de decisao do corte (1,0-2,0 s): "
        f"{sum(c['banda_1a2_s'] for c in casos)} frase(s) em "
        f"{sum(1 for c in casos if c['banda_1a2_s'])} musica(s) de {len(casos)}"
    )
    print(
        f"# musica(s) com frase > 20 s: {sum(1 for c in casos if c['acima_20s'])}/{len(casos)}  "
        f"fallback: {sum(1 for c in casos if c['fallback'] != 'nao')}/{len(casos)}"
    )
    print(
        "# voz (F0) fora das frases acima de 5%: "
        f"{sum(1 for c in casos if c['voz_fora_pct'] > 5)}/{len(casos)}"
    )

    if not args.gabarito:
        print(
            "\n## sem --gabarito: o ERRO de inicio de linha NAO foi medido "
            "(nenhuma referencia neste corpus); as constantes acima nao medem acerto"
        )
        return 0
    if not args.audio_gabarito:
        raise SystemExit("--gabarito exige --audio-gabarito")

    hum = marca_humana(args.gabarito)
    eng = alinhamento(args.bin, args.audio_gabarito, args.gabarito)
    pares = parear(hum, eng)
    if not pares:
        raise SystemExit(
            "nenhuma linha casada -- letra do gabarito e a do motor divergem?"
        )
    print(
        f"\n## erro contra a marcacao humana ({os.path.basename(args.audio_gabarito)})"
    )
    print(f"casadas={len(pares)}/{len(hum)}")
    erros = sorted(abs(e - h) for h, e, _, _ in pares)
    n = len(erros)
    print(
        f"erro_inicio  mediana={erros[n // 2]:.1f}s  p90={erros[min(n - 1, int(0.9 * n))]:.1f}s  "
        f"max={erros[-1]:.1f}s"
    )
    dentro, fora_banda = calibra_indicador(pares)
    print(
        f"indicador: taxa de silabas dentro de {SILABAS_MIN}-{SILABAS_MAX} sil/s -> "
        f"{len(dentro)}/{len(pares)} linha(s), erro mediano {mediana_erro(dentro):.1f}s"
    )
    print(
        f"           taxa FORA da faixa -> {len(fora_banda)}/{len(pares)} linha(s), "
        f"erro mediano {mediana_erro(fora_banda):.1f}s"
    )
    for taxa, erro, texto in sorted(fora_banda)[:6]:
        print(f"           {taxa:5.2f} sil/s  erro {erro:5.1f}s  {texto[:40]}")

    if args.prova_negativa:
        print("\n## prova negativa: letra embaralhada de proposito")
        conf = alinhamento(args.bin, args.audio_gabarito, args.gabarito, embaralhar=7)
        antes = taxa_fora_da_faixa(eng)
        depois = taxa_fora_da_faixa(conf)
        print(f"  ordem cantada: {antes}/{len(eng)} linha(s) fora da faixa")
        print(f"  embaralhada:   {depois}/{len(conf)} linha(s) fora da faixa")
        if depois <= antes:
            print(
                "  FALHOU: embaralhar a letra nao piorou o indicador -- indicador morto"
            )
            return 1
        print("  OK: o indicador piora quando a verdade e corrompida")
    return 0


if __name__ == "__main__":
    sys.exit(main())
