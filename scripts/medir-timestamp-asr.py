#!/usr/bin/env python3
"""Mede o erro do timestamp de palavra do whisper.cpp contra verdade sintetica.

Instrumento do spike de ASR (timestamps). Responde a uma unica pergunta: com que
erro o whisper.cpp posiciona o inicio de cada palavra? A verdade e construida --
cada palavra e sintetizada isolada, o silencio de borda e recortado, e a palavra e
recolocada num mix com silencio conhecido -- e depois CONFERIDA por rota
independente (silencio exato antes do onset, energia depois). Sem essa conferencia
o numero nao vale.

Gotcha que custou medicao: `-dtw` e DESLIGADO EM SILENCIO quando o flash-attn
esta ligado (padrao). O erro so aparece com verbosidade alta. Por isso aqui
sempre se passa `-nfa` no modo DTW.

Unidade do campo `t_dtw` do JSON: CENTESIMOS de segundo (validado contra a
verdade conhecida; usar como ms erra por um fator de 10).

Calibracao do vies (viés tardio e estavel => calibravel):
  --calibrar-vies ARQ   mede o vies (media do erro com sinal) por modelo no
                        material desta execucao e grava em ARQ, junto do nome e
                        do sha256 de cada modelo. A constante e MEDIDA, nao
                        digitada a mao.
  --vies ARQ            aplica as constantes de ARQ e imprime ANTES e DEPOIS
                        (mediana e p90) na mesma tabela e no mesmo material.
  --vies-invertido      aplica a constante com o sinal trocado -- prova
                        negativa: se a calibracao vale algo, o erro PIORA.

Regra de metodo: calibrar num material e reportar em OUTRO (a voz que nao
calibrou). Constante medida no mesmo material que ela reporta mede a si mesma.

Precondicoes (todas fail-loud): binario do whisper.cpp, modelos ggml, `say` com
voz pt-BR e `ffmpeg`.

Uso:
  python3 scripts/medir-timestamp-asr.py \
      --whisper /tmp/whisper.cpp/build/bin/whisper-cli --modelo-dir /tmp/asr-models

Prova negativa embutida: com --prova-negativa, a referencia e deslocada em 500 ms
e o erro medido tem de acompanhar; se nao acompanhar, o instrumento esta morto.
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import statistics
import struct
import subprocess
import sys
import tempfile
import wave

SR = 16000
LIMIAR_BORDA = 0.06
# Uma frase por idioma. As palavras sao curtas e variadas (fricativa, plosiva,
# nasal, ditongo) de proposito: o onset medido depende do ataque da palavra.
FRASES_PADRAO = {
    "pt": [
        "graça",
        "sobre",
        "nós",
        "eterno",
        "amor",
        "desce",
        "tua",
        "mão",
        "santo",
        "canto",
        "paz",
        "rei",
    ],
    "en": [
        "grace",
        "over",
        "night",
        "eternal",
        "love",
        "descend",
        "your",
        "hand",
        "holy",
        "song",
        "peace",
        "king",
    ],
    "es": [
        "gracia",
        "sobre",
        "noche",
        "eterno",
        "amor",
        "desciende",
        "tu",
        "mano",
        "santo",
        "canto",
        "paz",
        "rey",
    ],
}
# Vozes pt-BR/en-US/es-ES instaladas no macOS. Nomes com espaco e parentese
# entram entre aspas na chamada do `say`; aqui vao separados por virgula.
VOZES_PADRAO = {
    "pt": ["Eddy", "Flo"],
    "en": ["Eddy (English (US))", "Flo (English (US))"],
    "es": ["Eddy (Spanish (Spain))", "Flo (Spanish (Spain))"],
}
GAP_MS_PADRAO = 600
LEAD_MS = 1000
DESLOC_PROVA_MS = 500


def exige(ferramenta: str, dica: str) -> None:
    if shutil.which(ferramenta) is None:
        raise SystemExit(f"precondicao ausente: {ferramenta} ({dica})")


def sintetiza(texto: str, voz: str, destino: str) -> list[int]:
    aiff = destino + ".aiff"
    subprocess.run(["say", "-v", voz, "-o", aiff, texto], check=True)
    with open(destino + ".raw", "wb") as saida:
        subprocess.run(
            [
                "ffmpeg",
                "-v",
                "error",
                "-i",
                aiff,
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
    with open(destino + ".raw", "rb") as fh:
        dados = fh.read()
    return list(struct.unpack(f"<{len(dados) // 2}h", dados))


def recorta(s: list[int]) -> list[int]:
    if not s:
        return []
    mx = max(abs(v) for v in s) or 1
    limiar = LIMIAR_BORDA * mx
    i0 = next((i for i, v in enumerate(s) if abs(v) > limiar), 0)
    s2 = s[i0:]
    j = len(s2)
    while j > 0 and abs(s2[j - 1]) <= limiar:
        j -= 1
    return s2[:j]


def constroi(
    voz: str, frase: list[str], pasta: str, gap_ms: int
) -> tuple[str, list[dict]]:
    pcm = [0] * (SR * LEAD_MS // 1000)
    pos = len(pcm)
    verdade = []
    for i, palavra in enumerate(frase):
        s2 = recorta(sintetiza(palavra, voz, f"{pasta}/w{i}"))
        verdade.append({"palavra": palavra, "onset_ms": pos * 1000.0 / SR})
        pcm.extend(s2)
        pos += len(s2)
        gap = SR * gap_ms // 1000
        pcm.extend([0] * gap)
        pos += gap
    wav = f"{pasta}/sint_{voz}.wav"
    with wave.open(wav, "wb") as wf:
        wf.setnchannels(1)
        wf.setsampwidth(2)
        wf.setframerate(SR)
        wf.writeframes(struct.pack(f"<{len(pcm)}h", *pcm))
    return wav, verdade


def valida_verdade(wav: str, verdade: list[dict]) -> None:
    """Confere a verdade por energia: silencio antes, voz depois. Fail-loud."""
    with wave.open(wav, "rb") as wf:
        pcm = struct.unpack(f"<{wf.getnframes()}h", wf.readframes(wf.getnframes()))

    def rms(ms0: float, ms1: float) -> float:
        seg = pcm[int(ms0 * SR / 1000) : int(ms1 * SR / 1000)]
        return (sum(x * x for x in seg) / max(1, len(seg))) ** 0.5

    falhas = []
    for x in verdade:
        t = x["onset_ms"]
        antes = rms(t - 160, t - 60)
        depois = rms(t, t + 120)
        if depois <= 5 * max(antes, 1):
            falhas.append(
                f"{x['palavra']}: energia depois={depois:.0f} nao supera antes={antes:.0f}"
            )
    if falhas:
        raise SystemExit(
            "verdade sintetica nao conferiu (instrumento morto):\n  "
            + "\n  ".join(falhas)
        )


def roda(
    whisper: str, modelo: str, wav: str, tag: str, dtw: str | None, lingua: str
) -> str | None:
    cmd = [
        whisper,
        "-m",
        modelo,
        "-f",
        wav,
        "-l",
        lingua,
        "-ojf",
        "-t",
        "8",
        "-of",
        tag,
        "-np",
    ]
    if dtw:
        cmd += ["-nfa", "-dtw", dtw]  # sem -nfa o flash-attn desliga o DTW em silencio
    r = subprocess.run(cmd, capture_output=True, text=True, check=False)
    if r.returncode != 0:
        print(
            f"  aviso: whisper falhou em {tag}: {r.stderr.strip()[-200:]}",
            file=sys.stderr,
        )
        return None
    return tag + ".json"


def parse(jpath: str) -> list[dict]:
    """Palavras na ordem, com onset em ms. t_dtw esta em centesimos."""
    with open(jpath) as fh:
        d = json.load(fh)
    out, cur = [], None
    for seg in d["transcription"]:
        for t in seg.get("tokens", []):
            if t["id"] >= 50257:  # tokens especiais
                continue
            txt = t["text"]
            if not txt.strip():
                continue
            onset = (
                t["t_dtw"] * 10.0 if t.get("t_dtw", -1) >= 0 else t["offsets"]["from"]
            )
            if cur is None or txt.startswith(" "):
                cur = {"palavra": txt.strip(), "onset_ms": onset}
                out.append(cur)
            else:
                cur["palavra"] += txt
    return [w for w in out if any(c.isalnum() for c in w["palavra"])]


CABECALHO_SIMPLES = (
    "voz\tmodelo\tmodo\tn\tmediana_ms\tp90_ms\tmax_ms\tvies_ms\tdeterministico"
)
CABECALHO_CALIBRADO = (
    "voz\tmodelo\tmodo\tn\tantes_mediana\tantes_p90\tdepois_mediana\tdepois_p90\t"
    "vies_residual\tdeterministico"
)


def sha256(caminho: str) -> str:
    import hashlib

    h = hashlib.sha256()
    with open(caminho, "rb") as fh:
        for bloco in iter(lambda: fh.read(1 << 20), b""):
            h.update(bloco)
    return h.hexdigest()


def estat(erros: list[float]) -> dict:
    a = sorted(abs(e) for e in erros)
    return {
        "n": len(a),
        "mediana": a[len(a) // 2],
        "p90": a[min(len(a) - 1, int(0.9 * len(a)))],
        "max": a[-1],
        "vies": statistics.mean(erros),
    }


def erro(verdade: list[dict], parsed: list[dict]) -> list[float]:
    n = min(len(verdade), len(parsed))
    return [parsed[i]["onset_ms"] - verdade[i]["onset_ms"] for i in range(n)]


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("--whisper", required=True, help="caminho do whisper-cli")
    ap.add_argument("--modelo-dir", required=True, help="diretorio com ggml-*.bin")
    ap.add_argument("--lingua", default="pt", choices=sorted(FRASES_PADRAO))
    ap.add_argument("--vozes", default=None, help="padrao depende da lingua")
    ap.add_argument("--frase", default=None, help="palavras separadas por virgula")
    ap.add_argument("--prova-negativa", action="store_true")
    ap.add_argument(
        "--calibrar-vies",
        metavar="ARQ",
        default=None,
        help="mede o vies por modelo (modo DTW) no material desta execucao e grava em ARQ",
    )
    ap.add_argument(
        "--vies",
        metavar="ARQ",
        default=None,
        help="aplica as constantes de ARQ: imprime antes/depois na mesma tabela",
    )
    ap.add_argument(
        "--vies-invertido",
        action="store_true",
        help="aplica a calibracao com o sinal trocado (prova negativa)",
    )
    args = ap.parse_args()

    if args.vies and args.calibrar_vies:
        raise SystemExit(
            "--vies e --calibrar-vies sao exclusivos: calibre num material, aplique noutro"
        )
    if args.vies_invertido and not args.vies:
        raise SystemExit("--vies-invertido exige --vies (nao ha o que inverter)")

    exige("say", "vozes do macOS")
    exige("ffmpeg", "conversao para 16 kHz")
    if not os.access(args.whisper, os.X_OK):
        raise SystemExit(f"whisper-cli nao executavel: {args.whisper}")

    frase = args.frase.split(",") if args.frase else FRASES_PADRAO[args.lingua]
    vozes = args.vozes.split(",") if args.vozes else VOZES_PADRAO[args.lingua]

    modelos = []
    for nome, preset in (("small-q5_1", "small"), ("medium-q5_0", "medium")):
        caminho = os.path.join(args.modelo_dir, f"ggml-{nome}.bin")
        if os.path.exists(caminho):
            modelos.append((nome, preset, caminho))
    if not modelos:
        raise SystemExit(f"nenhum modelo ggml-* em {args.modelo_dir}")

    vies = None
    if args.vies:
        with open(args.vies) as fh:
            vies = json.load(fh)
        for _, preset, _ in modelos:
            if preset not in vies.get("modelos", {}):
                raise SystemExit(
                    f"{args.vies}: sem constante medida para o modelo {preset}"
                )

    pasta = tempfile.mkdtemp(prefix="spike-asr-")
    sinal = " (SINAL INVERTIDO -- prova negativa)" if args.vies_invertido else ""
    print(
        f"# lingua={args.lingua} instrumento em {pasta} | "
        f"modelos: {', '.join(m[0] for m in modelos)}"
    )
    if vies:
        orig = ", ".join(
            f"{p}={vies['modelos'][p]['vies_ms']:+.1f}ms"
            for p in sorted(vies["modelos"])
        )
        print(f"# calibracao aplicada de {args.vies} ({orig}){sinal}")
    print(CABECALHO_CALIBRADO if vies else CABECALHO_SIMPLES)

    acumulado: dict[str, list[float]] = {}
    for voz in vozes:
        voz = voz.strip()
        wav, verdade = constroi(voz, frase, pasta, GAP_MS_PADRAO)
        valida_verdade(wav, verdade)
        for nome, preset, caminho in modelos:
            for rot, dtw in (("dtw", preset), ("heur", None)):
                j1 = roda(
                    args.whisper,
                    caminho,
                    wav,
                    f"{pasta}/a_{voz}_{nome}_{rot}",
                    dtw,
                    args.lingua,
                )
                j2 = roda(
                    args.whisper,
                    caminho,
                    wav,
                    f"{pasta}/b_{voz}_{nome}_{rot}",
                    dtw,
                    args.lingua,
                )
                if not j1 or not j2:
                    continue
                p1, p2 = parse(j1), parse(j2)
                idem = [round(w["onset_ms"]) for w in p1] == [
                    round(w["onset_ms"]) for w in p2
                ]
                erros = erro(verdade, p1)
                st = estat(erros)
                extra = len(p1) - len(verdade)
                if vies:
                    c = float(vies["modelos"][preset]["vies_ms"])
                    if args.vies_invertido:
                        c = -c
                    stc = estat([e - c for e in erros])
                    print(
                        f"{voz}\t{nome}\t{rot}\t{st['n']}({extra:+d})\t"
                        f"{st['mediana']:.0f}\t{st['p90']:.0f}\t{stc['mediana']:.0f}\t"
                        f"{stc['p90']:.0f}\t{stc['vies']:+.0f}\t{'sim' if idem else 'NAO'}"
                    )
                else:
                    print(
                        f"{voz}\t{nome}\t{rot}\t{st['n']}({extra:+d})\t{st['mediana']:.0f}\t"
                        f"{st['p90']:.0f}\t{st['max']:.0f}\t{st['vies']:+.0f}\t"
                        f"{'sim' if idem else 'NAO'}"
                    )
                # So o modo DTW calibra e so ele e calibrado: o heuristico tem
                # outra escala de erro e outra distribuicao.
                if args.calibrar_vies and rot == "dtw":
                    acumulado.setdefault(preset, []).extend(erros)

        if args.prova_negativa:
            # Prova negativa do VERIFICADOR: verdade corrompida tem de reprovar.
            corrompida = [
                {
                    "palavra": x["palavra"],
                    "onset_ms": x["onset_ms"] + DESLOC_PROVA_MS / 2,
                }
                for x in verdade
            ]
            try:
                valida_verdade(wav, corrompida)
            except SystemExit:
                print(
                    f"# prova negativa ({voz}): verdade +{DESLOC_PROVA_MS / 2:.0f}ms "
                    "REPROVOU o verificador -> OK"
                )
            else:
                print(
                    f"# prova negativa ({voz}): verdade corrompida PASSOU "
                    "-> instrumento morto"
                )
                return 1
    if args.calibrar_vies:
        faltando = [p for _, p, _ in modelos if not acumulado.get(p)]
        if faltando:
            raise SystemExit(
                f"sem erro DTW medido para {faltando}: nada a gravar (instrumento morto?)"
            )
        dados = {
            "unidade": "ms",
            "significado": (
                "vies_ms = media(onset_reportado - onset_verdadeiro) no modo DTW; "
                "aplicar SUBTRAINDO do onsets reportados"
            ),
            "material": {
                "vozes": [v.strip() for v in vozes],
                "lingua": args.lingua,
                "frase": frase,
                "gap_ms": GAP_MS_PADRAO,
                "escopo": "fala sintetizada (TTS), modo DTW com -nfa",
            },
            "modelos": {},
        }
        for nome, preset, caminho in modelos:
            erros = acumulado[preset]
            dados["modelos"][preset] = {
                "vies_ms": round(statistics.mean(erros), 1),
                "n": len(erros),
                "arquivo": os.path.basename(caminho),
                "sha256": sha256(caminho),
                "desvio_padrao_ms": round(statistics.pstdev(erros), 1),
            }
        with open(args.calibrar_vies, "w") as fh:
            json.dump(dados, fh, indent=2, ensure_ascii=False)
            fh.write("\n")
        for preset in sorted(dados["modelos"]):
            m = dados["modelos"][preset]
            print(
                f"# vies medido {preset}: {m['vies_ms']:+.1f} ms (n={m['n']}, "
                f"desvio {m['desvio_padrao_ms']:.0f}, {m['arquivo']}, "
                f"sha256 {m['sha256'][:12]})"
            )
        print(f"# constantes gravadas em {args.calibrar_vies}")

    print(f"# resíduo em {pasta} (apagar quando não servir mais)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
