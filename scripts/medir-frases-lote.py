#!/usr/bin/env python3
"""As constantes do detector de frase vocal generalizam? Mede PLAUSIBILIDADE em lote.

Sem gabarito por música, isto NAO mede acuracia — mede se a regra e plausivel:
quantas frases, quanto do audio elas cobrem, que duracao tem, e se a musica cai
no fallback de frase unica.
"""

import glob
import os
import re
import statistics
import subprocess
import sys

MIR = sys.argv[1]
LIMITE = int(sys.argv[2]) if len(sys.argv) > 2 else 25
fontes = sorted(glob.glob(os.path.expanduser("~/Music/Repertorio/*.m4a")))[:LIMITE]
fontes += sorted(glob.glob(os.path.expanduser("~/Music/MP3/*.mp3")))[:LIMITE]
linhas = []
for f in fontes[:LIMITE]:
    r = subprocess.run(
        [MIR, f, "--frases-vocais"], capture_output=True, text=True, check=False
    )
    if r.returncode != 0:
        linhas.append((os.path.basename(f), None, None, None, None, "erro"))
        continue
    fr = re.findall(r"([\d.]+)s[–-]\s*([\d.]+)s\s+\(([\d.]+) s\)", r.stdout)
    dur = re.search(r"de ([\d.]+) s", r.stdout)
    if not fr:
        linhas.append((os.path.basename(f)[:32], 0, 0.0, 0.0, 0.0, "sem frase"))
        continue
    fr = [(float(a), float(b), float(c)) for a, b, c in fr]
    total = float(dur.group(1)) if dur else max(b for _, b, _ in fr)
    cob = sum(c for _, _, c in fr)
    linhas.append(
        (
            os.path.basename(f)[:32],
            len(fr),
            100 * cob / total,
            max(c for _, _, c in fr),
            fr[0][2] if fr else 0,
            "",
        )
    )
print(f"{'arquivo':34s} {'n':>3s} {'cob%':>6s} {'max_s':>6s} {'1a_s':>6s} nota")
for nome, n, cob, mx, p, nota in linhas:
    if n is None:
        print(f"{nome:34s}   -      -      -      - {nota}")
    else:
        print(f"{nome:34s} {n:3d} {cob:6.1f} {mx:6.1f} {p:6.1f} {nota}")
bons = [l for l in linhas if l[1]]
if bons:
    cobs = sorted(l[2] for l in bons)
    maxs = sorted(l[3] for l in bons)
    print(
        f"\n# n={len(bons)}  cobertura mediana={cobs[len(cobs) // 2]:.0f}%  min={cobs[0]:.0f}%"
    )
    print(
        f"# frases mediana={statistics.median([l[1] for l in bons]):.0f}  "
        f"musicas com frase >20s: {sum(1 for l in bons if l[3] > 20)}/{len(bons)}"
    )
    print("# fallback (1 frase cobrindo >85%): pendente de checar")
    print(
        f"# cobertura < 30% (o piso pode estar cortando voz): "
        f"{sum(1 for c in cobs if c < 30)}/{len(bons)}"
    )
