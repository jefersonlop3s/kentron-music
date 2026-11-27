#!/usr/bin/env python3
"""Mede o CUSTO de rodar um modelo do whisper.cpp no desktop: tempo de parede e
pico de memoria residente, com um recorte fixo e declarado de um audio declarado.

Pergunta que o numero responde: "onde o `medium` roda?". Para isso o desktop
precisa de dois numeros medidos do MESMO material e do MESMO recorte:
tempo de parede e pico de RSS -- comparaveis com os numeros do aparelho
(`small-q5_1` 557 MB e 220 s para 400 s; `medium-q5_0` 1,11 GB e 3,1x mais
lento, medidos no Android com outro instrumento -- a comparacao e entre dois
numeros medidos, nao entre dois instrumentos iguais).

O medidor e `/usr/bin/time -l` (macOS): `real` e `maximum resident set size`
(bytes). O que ele NAO captura esta dito no relatorio: RSS e do processo; buffer
de GPU em memoria unificada pode nao aparecer -- por isso o braco `cpu` roda o
mesmo modelo com `-ng` (sem Metal): a diferenca de RSS e a medida de quanto do
custo de GPU o medidor enxerga.

Prova negativa (`--prova-negativa`): o medidor tem de acertar dois custos
conhecidos de antemao -- um processo Python que aloca 200 MB (RSS >= 200 MB) e
um `sleep` de 1,5 s (parede entre 1,0 e 4,0 s). Medidor que devolve zero, ou que
nao sente o custo plantado, esta morto.

Uso:
  python3 scripts/medir-asr-custo.py --whisper /tmp/whisper.cpp/build/bin/whisper-cli \\
      --modelo-dir /tmp/asr-models --audio ~/Music/MP3/ancora.mp3 --recorte-s 120 \\
      --modelos small-q5_1,medium-q5_0 --prova-negativa
"""

from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile

SR = 16000


def exige(ferramenta: str, dica: str) -> None:
    if shutil.which(ferramenta) is None:
        raise SystemExit(f"precondicao ausente: {ferramenta} ({dica})")


def mede(cmd: list[str]) -> dict:
    """Roda sob /usr/bin/time -l e devolve parede (s) e pico de RSS (bytes)."""
    r = subprocess.run(
        ["/usr/bin/time", "-l"] + cmd, capture_output=True, text=True, check=False
    )
    if r.returncode != 0:
        raise SystemExit(
            f"comando falhou (rc={r.returncode}): {' '.join(cmd[:3])}...\n"
            f"{(r.stderr or r.stdout)[-800:]}"
        )
    tempo = re.search(r"^\s*([\d.]+) real", r.stderr, re.MULTILINE)
    pico = re.search(r"^\s*(\d+)\s+maximum resident set size", r.stderr, re.MULTILINE)
    if not tempo or not pico:
        raise SystemExit(
            "medidor morto: /usr/bin/time -l nao devolveu real/maximum resident "
            f"set size. Saida:\n{r.stderr[-600:]}"
        )
    return {
        "parede_s": float(tempo.group(1)),
        "pico_bytes": int(pico.group(1)),
        "log": r.stdout,
    }


def prova_negativa_medidor() -> None:
    """Custos conhecidos: o medidor tem de acusa-los."""
    falhas = []
    alvo = 200 * 1024 * 1024
    m = mede(
        [
            sys.executable,
            "-c",
            f"import time;b=bytearray({alvo});b[0]=1;time.sleep(1.0)",
        ]
    )
    ok_rss = m["pico_bytes"] >= alvo
    print(
        f"# prova negativa [alocacao de 200 MB]: pico={m['pico_bytes'] / 2**20:.0f} MB "
        f"(>= 200 MB: {ok_rss}) parede={m['parede_s']:.2f}s -> "
        f"{'OK' if ok_rss else 'FALHOU'}"
    )
    if not ok_rss:
        falhas.append("alocacao")
    m = mede(["sleep", "1.5"])
    ok_tempo = 1.0 <= m["parede_s"] <= 4.0
    print(
        f"# prova negativa [sleep 1.5s]: parede={m['parede_s']:.2f}s "
        f"(1,0 a 4,0 s: {ok_tempo}) pico={m['pico_bytes'] / 2**20:.1f} MB -> "
        f"{'OK' if ok_tempo else 'FALHOU'}"
    )
    if not ok_tempo:
        falhas.append("relogio")
    if falhas:
        raise SystemExit(f"medidor morto nos casos: {falhas}")


def recorta_audio(ffmpeg: str, audio: str, segundos: int, destino: str) -> str:
    """Recorte fixo e declarado, 16 kHz mono s16le, com header WAV do Python."""
    import wave

    raw = destino + ".raw"
    wav = destino + ".wav"
    with open(raw, "wb") as saida:
        subprocess.run(
            [
                ffmpeg,
                "-v",
                "error",
                "-t",
                str(segundos),
                "-i",
                audio,
                "-ar",
                str(SR),
                "-ac",
                "1",
                "-f",
                "s16le",
                "-",
            ],
            stdout=saida,
            check=True,
        )
    with open(raw, "rb") as fh:
        amostras = fh.read()
    with wave.open(wav, "wb") as wf:
        wf.setnchannels(1)
        wf.setsampwidth(2)
        wf.setframerate(SR)
        wf.writeframes(amostras)
    with wave.open(wav, "rb") as wf:
        real = wf.getnframes() / wf.getframerate()
    if not (0.5 * segundos <= real <= 1.5 * segundos):
        raise SystemExit(
            f"recorte implausivel: pedido {segundos} s, medido {real:.1f} s "
            "(instrumento morto ou audio mais curto que o recorte)"
        )
    return wav


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("--whisper", required=True)
    ap.add_argument("--modelo-dir", required=True)
    ap.add_argument("--audio", required=True, help="audio de origem (fora do repo)")
    ap.add_argument("--recorte-s", type=int, default=120)
    ap.add_argument("--modelos", default="small-q5_1,medium-q5_0")
    ap.add_argument("--lingua", default="pt")
    ap.add_argument("--threads", type=int, default=8)
    ap.add_argument(
        "--repeticoes", type=int, default=1, help="mede N vezes, fica com a menor"
    )
    ap.add_argument(
        "--cpu", action="store_true", help="roda tambem com -ng (sem Metal)"
    )
    ap.add_argument("--prova-negativa", action="store_true")
    ap.add_argument("--json", default=None)
    args = ap.parse_args()

    exige("ffmpeg", "recorte do audio")
    if not os.access(args.whisper, os.X_OK):
        raise SystemExit(f"whisper-cli nao executavel: {args.whisper}")
    if not os.path.exists(args.audio):
        raise SystemExit(f"audio de origem ausente: {args.audio}")

    pasta = tempfile.mkdtemp(prefix="spike-custo-")
    if args.prova_negativa:
        prova_negativa_medidor()

    wav = recorta_audio("ffmpeg", args.audio, args.recorte_s, f"{pasta}/recorte")
    print(
        f"# audio de origem: {os.path.basename(args.audio)} | recorte fixo: "
        f"{args.recorte_s} s | {os.path.basename(wav)}"
    )
    print(
        "modelo\tacelerador\tparede_s\tx_tempo_real\tpico_MB\trecorte_s\t"
        "parede_s_por_hora_audio"
    )

    aceleradores = (("metal", []), ("cpu", ["-ng"])) if args.cpu else (("metal", []),)
    resultados = []
    for nome in args.modelos.split(","):
        nome = nome.strip()
        modelo = os.path.join(args.modelo_dir, f"ggml-{nome}.bin")
        if not os.path.exists(modelo):
            raise SystemExit(f"modelo ausente: {modelo}")
        for acelerador, extra in aceleradores:
            cmd = [
                args.whisper,
                "-m",
                modelo,
                "-f",
                wav,
                "-l",
                args.lingua,
                "-t",
                str(args.threads),
                "-np",
            ] + extra
            medidas = [mede(cmd) for _ in range(args.repeticoes)]
            melhor = min(medidas, key=lambda m: m["parede_s"])
            pico_mb = melhor["pico_bytes"] / 2**20
            r = {
                "modelo": nome,
                "acelerador": acelerador,
                "parede_s": round(melhor["parede_s"], 2),
                "pico_mb": round(pico_mb, 1),
                "recorte_s": args.recorte_s,
                "x_tempo_real": round(args.recorte_s / melhor["parede_s"], 2),
                "parede_s_por_hora_audio": round(
                    melhor["parede_s"] * 3600 / args.recorte_s, 1
                ),
                "todos_parede_s": [round(m["parede_s"], 2) for m in medidas],
                "todos_pico_MB": [round(m["pico_bytes"] / 2**20, 1) for m in medidas],
            }
            resultados.append(r)
            print(
                f"{nome}\t{acelerador}\t{r['parede_s']}\t{r['x_tempo_real']}\t"
                f"{r['pico_mb']}\t{args.recorte_s}\t{r['parede_s_por_hora_audio']}"
            )

    print(
        "# extrapolacao DECLARADA do TEMPO por duracao (escala linear admitida, "
        "nao medida acima do recorte):"
    )
    for r in resultados:
        print(
            f"#   {r['modelo']}/{r['acelerador']}: 400 s -> "
            f"{r['parede_s_por_hora_audio'] * 400 / 3600:.1f} s de parede | "
            f"313 musicas de 400 s -> "
            f"{r['parede_s_por_hora_audio'] * 400 / 3600 * 313 / 3600:.1f} h"
        )
    print("# reprodutibilidade (todas as repeticoes):")
    for r in resultados:
        print(
            f"#   {r['modelo']}/{r['acelerador']}: parede {r['todos_parede_s']} s | "
            f"pico {r['todos_pico_MB']} MB"
        )
    print(
        "# limite do medidor: RSS e do processo; buffer de GPU em memoria unificada "
        "pode nao aparecer no pico (a diferenca metal x cpu e a medida disso). "
        "O pico NAO se extrapola por duracao: rode outro --recorte-s e compare."
    )

    if args.json:
        with open(args.json, "w") as fh:
            json.dump({"resultados": resultados}, fh, indent=2, ensure_ascii=False)
            fh.write("\n")
        print(f"# numeros gravados em {args.json}")
    print(f"# residuo em {pasta} (apagar quando nao servir mais)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
