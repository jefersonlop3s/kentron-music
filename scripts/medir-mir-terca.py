#!/usr/bin/env python3
"""A terca no repertorio: quantos trechos GANHAM terca, e quantos ganham uma que
nao tem.

Contagem em classes, antes de contar qualquer acerto:

  ausente   o nome final nao tem terca (`5`, `sus`) — a leitura honesta quando a
            terca nao esta no audio;
  croma     a regra `chords::com_terca` devolveu o acorde inteiro porque a razao
            terca/fundamental passou de 0,40. E aqui que mora o risco;
  melodia   o croma perdeu a terca e a melodia (F0) a trouxe;
  template  o template de acorde ja tinha terca: nada foi decidido.

Sobre os que a regra do croma promoveu, dois juizes INDEPENDENTES do croma:

  melodia   a propria melodia mostra a classe da terca na fracao que
            `com_a_melodia` exige (0,20 dos quadros com F0). Sem ela, a terca
            pode ser o 5o harmonico da fundamental (razao 5/4, documentado no
            modulo `melody`) — e nao a terca;
  tom       a qualidade promovida e a diatonicamente esperada no tom que o motor
            detectou? (maior onde o tom pede menor e suspeita). O tom sai do
            MESMO croma: e juiz fraco, e esta dito.

O que NAO da para medir aqui: "ganhou terca que nao tem" so tem verdade por
trecho com cifra rotulada — o unico conjunto rotulado do projeto tem 91 trechos
de UMA musica (medido em 2026-11-22). O numero que sai daqui e SUSPEITA
quantificada, com o denominador ao lado; sem o denominador ele nao diz nada.

uso:
  medir-mir-terca.py --bin target/release/mir --pasta ~/Music/MP3 \\
      [--pasta ~/Music/GabaritoBennett] [--pasta ~/Music/EstudoMetro] \\
      [--cache /tmp/json-terca] [--recursivo] [--limite N]
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
import sys

# Classes de altura na convencao do motor (bemois nas teclas pretas), na mesma
# ordem de `chords::TONICAS`.
TONICAS = ["C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B"]

# Escala maior em semitons, para julgar a diatonia da qualidade promovida.
MAIOR = {0, 2, 4, 5, 7, 9, 11}


def roda(binario: str, audio: str, cache: str | None) -> dict:
    caminho = None
    if cache:
        os.makedirs(cache, exist_ok=True)
        chave = hashlib.sha1(audio.encode()).hexdigest()[:16]
        caminho = os.path.join(cache, f"{chave}.json")
        if os.path.exists(caminho):
            with open(caminho) as fh:
                return json.load(fh)
    r = subprocess.run(
        [binario, audio, "--json"], capture_output=True, text=True, check=False
    )
    if r.returncode != 0:
        raise SystemExit(
            f"mir falhou ({r.returncode}) em {audio}: {r.stderr.strip()[-300:]}"
        )
    dados = json.loads(r.stdout)
    if caminho:
        with open(caminho, "w") as fh:
            json.dump(dados, fh)
    return dados


def classe(nome: str) -> int:
    """Classe de altura a partir do inicio do nome do acorde (`Bb`, `C`, `F`…).

    O motor nomeia sempre com bemol nas teclas pretas (`chords::TONICAS`); um
    sustenido aqui seria nome de outra fonte, e falha alto em vez de adivinhar.
    """
    if nome[:2] in TONICAS:
        return TONICAS.index(nome[:2])
    if nome[:1] in TONICAS:
        return TONICAS.index(nome[:1])
    raise SystemExit(f"nome de acorde sem classe reconhecivel: {nome!r}")


def analisa(dados: dict) -> dict:
    """Conta as classes de um caso e o que os dois juizes dizem."""
    tonica_tom = None
    if dados["tonalidade"]["nome"] in TONICAS:
        tonica_tom = TONICAS.index(dados["tonalidade"]["nome"])
    contagem = {"ausente": 0, "croma": 0, "melodia": 0, "template": 0}
    sem_melodia = 0
    fora_do_tom = 0
    razoes = []
    melodia = dados.get("melodia", [])
    for i, trecho in enumerate(dados["chords"]):
        origem = trecho["origem_terca"].lower()
        contagem[origem] = contagem.get(origem, 0) + 1
        if origem != "croma":
            continue
        razoes.append(trecho["terca_razao"])
        # Juiz 1: a melodia confirma? O campo é calculado pelo motor com o mesmo
        # critério de `com_a_melodia`; aqui só se lê.
        if i < len(melodia) and melodia[i].get("terca_confirmada") is False:
            sem_melodia += 1
        # Juiz 2: a qualidade promovida casa com a escala maior do tom detectado?
        if tonica_tom is not None:
            grau = (classe(trecho["acorde"]) - tonica_tom) % 12
            # `m` (e só `m`) promete menor; o resto promete maior.
            promete_maior = "m" not in trecho["acorde"].replace("maj", "")
            if promete_maior != (grau in MAIOR):
                fora_do_tom += 1
    razoes.sort()
    return {
        "n": len(dados["chords"]),
        **contagem,
        "sem_melodia": sem_melodia,
        "fora_do_tom": fora_do_tom,
        "razao_min": razoes[0] if razoes else 0.0,
        "razao_mediana": razoes[len(razoes) // 2] if razoes else 0.0,
        # Quantos ficam a menos de 0,05 do limiar: é onde a decisão é mais frágil.
        "colados_ao_limiar": sum(1 for r in razoes if r < 0.45),
    }


def catalogo(pastas: list[str], recursivo: bool, limite: int) -> list[str]:
    exts = (".mp3", ".m4a", ".flac", ".wav", ".ogg")
    audios: list[str] = []
    for pasta in pastas:
        if recursivo:
            for raiz, _, arquivos in os.walk(pasta):
                audios += [
                    os.path.join(raiz, a) for a in arquivos if a.lower().endswith(exts)
                ]
        else:
            audios += [
                os.path.join(pasta, a)
                for a in os.listdir(pasta)
                if a.lower().endswith(exts)
            ]
    audios.sort()
    return audios[:limite] if limite else audios


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("--bin", required=True)
    ap.add_argument("--pasta", action="append", required=True)
    ap.add_argument("--cache")
    ap.add_argument("--recursivo", action="store_true")
    ap.add_argument("--limite", type=int, default=0)
    args = ap.parse_args()

    if not os.path.exists(args.bin):
        raise SystemExit(f"precondicao ausente: {args.bin}")
    for pasta in args.pasta:
        if not os.path.isdir(pasta):
            raise SystemExit(f"precondicao ausente (pasta): {pasta}")

    audios = catalogo(args.pasta, args.recursivo, args.limite)
    if not audios:
        raise SystemExit("nenhum audio nas pastas dadas")

    print("# classes ANTES dos acertos: 'ausente' e a taxa-base do risco")
    print(
        "arquivo\tacordes\tausente\tcroma\tmelodia\ttemplate\tsem_melodia\tfora_tom\t"
        "razao_min\trazao_med\tcolados"
    )
    linhas = []
    for audio in audios:
        r = analisa(roda(args.bin, audio, args.cache))
        linhas.append(r)
        print(
            f"{os.path.basename(audio)[:34]}\t{r['n']}\t{r['ausente']}\t{r['croma']}\t"
            f"{r['melodia']}\t{r['template']}\t{r['sem_melodia']}\t{r['fora_do_tom']}\t"
            f"{r['razao_min']:.2f}\t{r['razao_mediana']:.2f}\t{r['colados_ao_limiar']}"
        )
    total = sum(r["n"] for r in linhas)

    def soma(campo: str) -> int:
        return sum(r[campo] for r in linhas)

    print(f"\n# n={len(linhas)} musica(s)  trechos={total}")
    print(
        f"# ausente={soma('ausente')} ({100 * soma('ausente') / total:.0f}%)  "
        f"croma={soma('croma')} ({100 * soma('croma') / total:.0f}%)  "
        f"melodia={soma('melodia')}  template={soma('template')}"
    )
    print(
        f"# dos que o CROMA promoveu: {soma('sem_melodia')}/{soma('croma')} sem a melodia "
        f"confirmar, {soma('fora_do_tom')}/{soma('croma')} fora do tom detectado, "
        f"{soma('colados_ao_limiar')} com razao < 0,45 (colados ao limiar)"
    )
    print(
        "# 'sem a melodia confirmar' NAO e prova de terca inexistente: e a suspeita, "
        "com o denominador ao lado"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
