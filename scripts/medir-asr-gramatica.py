#!/usr/bin/env python3
"""Mede WER e o novo modo de falha (ABSTENCAO) do whisper.cpp em fala conectada.

Pergunta: restringir a decodificacao ao vocabulario da musica (--grammar, GBNF)
resolve o que o --prompt resolve? E o que ela CUSTA?

Braços medidos no MESMO material (um audio por voz/lingua, sintetizado uma vez) e
com o MESMO placar:

  baseline           sem ajuda nenhuma
  prompt             --prompt com o vocabulario da musica
  pos_hoc            correcao do texto do baseline por lexico (token a token,
                     sem desfazer fusao -- e o limite declarado do metodo)
  grammar            --grammar GBNF com o vocabulario (decodificacao restrita)
  grammar_suave      --grammar com --grammar-penalty 1 (penaliza, nao proibe)
  grammar_sem_rule   --grammar SEM --grammar-rule. NAO e um braco de qualidade:
                     e a medicao do no-op silencioso (ver abaixo).
  grammar_controle   --grammar com vocabulario DISJUNTO do audio, tambem prova
                     de que a flag e obedecida: se a saida do controle for igual a
                     do baseline, o --grammar foi ignorado em silencio (mesma
                     classe do `-dtw` sob flash-attn) e o instrumento esta morto.

GOTCHA MEDIDO: `--grammar` sozinho NAO restringe nada. O default de
`--grammar-rule` vem VAZIO no help, a gramatica e impressa ("main: grammar:")
e a decodificacao segue livre -- sem aviso nenhum. So com
`--grammar-rule root` (ou o nome da regra) a restricao entra; ai o sampling
fica ~10x mais caro e a saida muda. O braco `grammar_sem_rule` mede esse no-op
e a prova negativa confere que ele repete o baseline.

Placar por braço: WER (S+D+I)/N, contagem de substituicoes/omissoes/insercoes, a
ABSTENCAO (D/N: quantas palavras de referencia o modelo deixou de emitir) e a
INVENCAO (I/N: palavras que ele emitiu sem estar na referencia). A lista das
palavras omitidas sai no relatorio -- e a forma bruta da falha.

Prova negativa (`--prova-negativa`):
  1. invariantes do placar -- identidade da WER 0 com D=I=0; hipotese girada da
     WER >= 1; hipotese sem a ultima palavra da D=1. Qualquer um que falhe ->
     instrumento morto;
  2. atividade da gramatica -- o braço de controle tem de produzir so tokens do
     vocabulario de controle (ou nada).

Precondicoes fail-loud: `say`, `ffmpeg`, binario do whisper e binario com
`--grammar` no help (a flag tem de existir de fato nesta build).
"""

from __future__ import annotations

import argparse
import difflib
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import time
import wave

SR = 16000
LIMIAR_POS_HOC = 0.6
# A letra como ela e cantada: uma frase conectada, sem silencio inserido entre
# as palavras. E o regime em que o baseline FUNDE palavras ("sobre nos" ->
# "obranos") -- a fusao e o erro dominante, e e contra ela que a gramatica joga.
FRASES = {
    "pt": "graça sobre nós eterno amor desce tua mão santo canto paz rei",
    "en": "grace over night eternal love descend your hand holy song peace king",
    "es": "gracia sobre noche eterno amor desciende tu mano santo canto paz rey",
}
VOZES = {
    "pt": "Eddy",
    "en": "Eddy (English (US))",
    "es": "Eddy (Spanish (Spain))",
}
# Vocabulario de controle: disjunto do audio de proposito (prova de atividade).
LEXICO_CONTROLE = ["alfa", "beta", "gama", "delta"]


def exige(ferramenta: str, dica: str) -> None:
    if shutil.which(ferramenta) is None:
        raise SystemExit(f"precondicao ausente: {ferramenta} ({dica})")


def sintetiza_conexo(
    texto: str, voz: str, destino: str, taxa: int | None = None
) -> str:
    """Fala conectada: uma emissao so, sem recorte e sem colar. Sem verdade de tempo."""
    aiff = destino + ".aiff"
    raw = destino + ".raw"
    wav = destino + ".wav"
    cmd_say = ["say"]
    if taxa:
        # Velocidade declarada: na velocidade padrao do `say` o TTS fala rapido
        # demais e o baseline afunda (WER 0,83), o que confunde os bracos.
        cmd_say += ["-r", str(taxa)]
    cmd_say += ["-v", voz, "-o", aiff, texto]
    subprocess.run(cmd_say, check=True)
    # s16le por stdout e o Python escreve o WAV: o header que o ffmpeg gera em
    # stdout nao e buscavel e sai com tamanho de dados errado (medido: o
    # instrumento reportou 134217 s de audio num wav de 5 s).
    with open(raw, "wb") as saida:
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
    with open(raw, "rb") as fh:
        amostras = fh.read()
    with wave.open(wav, "wb") as wf:
        wf.setnchannels(1)
        wf.setsampwidth(2)
        wf.setframerate(SR)
        wf.writeframes(amostras)
    with wave.open(wav, "rb") as wf:
        if wf.getnframes() == 0:
            raise SystemExit(f"audio vazio: {wav}")
        segundos = wf.getnframes() / wf.getframerate()
    if not (1.0 < segundos < 120.0):
        raise SystemExit(
            f"duracao implausivel ({segundos:.1f} s) em {wav}: instrumento morto"
        )
    print(f"# audio {os.path.basename(wav)}: {segundos:.1f} s", file=sys.stderr)
    return wav


def normaliza(texto: str) -> list[str]:
    """Mantem letras (com acento), digitos e apostrofo; o resto vira espaco."""
    t = texto.lower().replace("’", "'")
    t = re.sub(r"[^0-9a-záàâãäéèêëíìîïóòôõöúùûüçñ' ]+", " ", t)
    return [p for p in t.split() if p]


def alinha(ref: list[str], hyp: list[str]) -> dict:
    """Alinhamento minimo: sub/omissoes/insercoes/n_ref e a LISTA das omitidas."""

    n, m = len(ref), len(hyp)
    d = [[0] * (m + 1) for _ in range(n + 1)]
    for i in range(1, n + 1):
        d[i][0] = i
    for j in range(1, m + 1):
        d[0][j] = j
    for i in range(1, n + 1):
        for j in range(1, m + 1):
            if ref[i - 1] == hyp[j - 1]:
                d[i][j] = d[i - 1][j - 1]
            else:
                d[i][j] = 1 + min(d[i - 1][j - 1], d[i - 1][j], d[i][j - 1])
    i, j = n, m
    sub = omi = ins = 0
    omitidas: list[str] = []
    while i > 0 or j > 0:
        if i > 0 and j > 0 and ref[i - 1] == hyp[j - 1] and d[i][j] == d[i - 1][j - 1]:
            i, j = i - 1, j - 1
        elif i > 0 and j > 0 and d[i][j] == d[i - 1][j - 1] + 1:
            sub += 1
            i, j = i - 1, j - 1
        elif i > 0 and d[i][j] == d[i - 1][j] + 1:
            omi += 1
            omitidas.append(ref[i - 1])
            i -= 1
        else:
            ins += 1
            j -= 1
    return {
        "n_ref": n,
        "n_hyp": m,
        "sub": sub,
        "omi": omi,
        "ins": ins,
        "wer": (sub + omi + ins) / n if n else 0.0,
        "abstencao": omi / n if n else 0.0,
        "invencao": ins / n if n else 0.0,
        "omitidas": list(reversed(omitidas)),
    }


def corrige_pos_hoc(palavras: list[str], lexico: list[str], limiar: float) -> list[str]:
    """Correcao por lexico, token a token; NAO desfaz fusao (limite declarado)."""
    saida = []
    for p in palavras:
        if p in lexico:
            saida.append(p)
            continue
        perto = difflib.get_close_matches(p, lexico, n=1, cutoff=limiar)
        saida.append(perto[0] if perto else p)
    return saida


def transcreve(
    whisper: str,
    modelo: str,
    wav: str,
    lingua: str,
    tag: str,
    prompt: str | None = None,
    grammar: str | None = None,
    regra: str | None = None,
    penalty: float | None = None,
    threads: int = 8,
) -> tuple[str, str, float]:
    """Roda o whisper-cli e devolve (texto, log, segundos). Fail-loud em rc != 0."""
    inicio = time.perf_counter()
    cmd = [
        whisper,
        "-m",
        modelo,
        "-f",
        wav,
        "-l",
        lingua,
        "-t",
        str(threads),
        "-of",
        tag,
        "-otxt",
        "-np",
    ]
    if prompt:
        cmd += ["--prompt", prompt]
    if grammar:
        cmd += ["--grammar", grammar]
    if regra:
        # Sem `--grammar-rule` o whisper-cli IMPRIME a gramatica e NAO a aplica
        # (o default do help vem vazio): no-op silencioso, mesma classe do
        # `-dtw` sob flash-attn. Medido: mesma saida do baseline e 10x menos
        # tempo de sampling.
        cmd += ["--grammar-rule", regra]
    if penalty is not None:
        cmd += ["--grammar-penalty", str(penalty)]
    r = subprocess.run(cmd, capture_output=True, text=True, check=False)
    if r.returncode != 0:
        raise SystemExit(
            f"whisper falhou (rc={r.returncode}) em {tag}:\n{(r.stderr or r.stdout)[-800:]}"
        )
    with open(tag + ".txt") as fh:
        return fh.read().strip(), r.stdout + r.stderr, time.perf_counter() - inicio


def roda_braco(
    whisper: str,
    modelo: str,
    wav: str,
    lingua: str,
    pasta_base: str,
    tag: str,
    **kw,
) -> tuple[str, float]:
    """Um braco: roda, imprime qualquer aviso de gramatica do log e cronometra."""
    texto, log, seg = transcreve(
        whisper, modelo, wav, lingua, f"{pasta_base}_{tag}", **kw
    )
    for linha in log.splitlines():
        if "grammar" in linha.lower():
            print(f"# log whisper ({lingua}/{tag}): {linha.strip()[:200]}")
    return texto, seg


def prova_negativa_placar() -> list[str]:
    """Invariantes do placar. Verdade corrompida tem de REPROVAR.

    Os valores esperados sao derivaveis a mao, nao "o que o codigo deu":
      - identidade: 0 operacoes;
      - sem a ultima palavra: 1 omissao em 4 (WER 0,25);
      - girada de 1: LCS = 3 (b,c,d) -> 2*(4-3) = 2 operacoes -> WER 0,50.
        (Foi aqui que a primeira versao do teste mentiu: eu esperava 1,50 por
        "3 substituicoes" e o placar reprovou com o valor certo. A expectativa
        errada faz o aparelho parecer morto.)
      - vocabulario disjunto: 4 substituicoes em 4 (WER 1,00);
      - hipotese vazia: 4 omissoes (WER 1,00, abstencao total).
    """
    ref = ["graça", "sobre", "nós", "eterno"]
    casos = [
        ("identidade", ref, 0.0, 0, 0, 0),
        ("faltando a ultima", ref[:-1], 0.25, 0, 1, 0),
        ("girada de 1", ref[1:] + ref[:1], 0.50, 0, 1, 1),
        ("vocabulario disjunto", ["alfa", "beta", "gama", "delta"], 1.0, 4, 0, 0),
        ("vazia (abstencao total)", [], 1.0, 0, 4, 0),
    ]
    falhas = []
    for nome, hyp, wer_esperada, sub_esp, omi_esp, ins_esp in casos:
        st = alinha(ref, hyp)
        ok = (
            abs(st["wer"] - wer_esperada) < 1e-9
            and st["sub"] == sub_esp
            and st["omi"] == omi_esp
            and st["ins"] == ins_esp
        )
        print(
            f"# prova negativa do placar [{nome}]: WER={st['wer']:.2f} "
            f"(esperado {wer_esperada:.2f}) sub={st['sub']} (esperado {sub_esp}) "
            f"omi={st['omi']} (esperado {omi_esp}) ins={st['ins']} "
            f"(esperado {ins_esp}) -> {'OK' if ok else 'FALHOU'}"
        )
        if not ok:
            falhas.append(nome)
    if falhas:
        raise SystemExit(f"placar morto nos casos: {falhas}")
    return casos[0][1]


def gbnf_de(lexico: list[str]) -> str:
    """GBNF com o vocabulario em minuscula e capitalizado (o token vem com espaco)."""
    alternativas = set()
    for palavra in lexico:
        alternativas.add(json.dumps(palavra, ensure_ascii=False))
        alternativas.add(json.dumps(palavra.capitalize(), ensure_ascii=False))
    return (
        "root ::= ws? palavra (ws palavra)* ws?\n"
        'ws ::= " "+\n'
        f"palavra ::= {' | '.join(sorted(alternativas))}\n"
    )


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("--whisper", required=True, help="caminho do whisper-cli")
    ap.add_argument("--modelo-dir", required=True, help="diretorio com ggml-*.bin")
    ap.add_argument(
        "--modelos", default="medium-q5_0", help="nomes separados por virgula"
    )
    ap.add_argument("--linguas", default="pt", help="idiomas separados por virgula")
    ap.add_argument("--voz", default=None, help="padrao depende da lingua")
    ap.add_argument("--frase", default=None, help="texto conectado (padrao por lingua)")
    ap.add_argument(
        "--taxa",
        type=int,
        default=150,
        help="palavras por minuto do `say` (velocidade declarada do material)",
    )
    ap.add_argument("--prova-negativa", action="store_true")
    ap.add_argument("--json", default=None, help="grava os numeros em JSON")
    args = ap.parse_args()

    exige("say", "vozes do macOS")
    exige("ffmpeg", "conversao para 16 kHz")
    if not os.access(args.whisper, os.X_OK):
        raise SystemExit(f"whisper-cli nao executavel: {args.whisper}")
    ajuda = subprocess.run(
        [args.whisper, "--help"], capture_output=True, text=True, check=False
    )
    # O `--help` do whisper-cli sai em STDERR: olhar so o stdout faz o aparelho
    # declarar "nao medivel" com a flag existindo na build.
    texto_ajuda = (ajuda.stdout or "") + (ajuda.stderr or "")
    if "--grammar" not in texto_ajuda:
        raise SystemExit(
            "esta build do whisper-cli NAO tem --grammar no help: item nao medivel aqui"
        )

    modelos = []
    for nome in args.modelos.split(","):
        nome = nome.strip()
        caminho = os.path.join(args.modelo_dir, f"ggml-{nome}.bin")
        if not os.path.exists(caminho):
            raise SystemExit(f"modelo ausente: {caminho}")
        modelos.append((nome, caminho))

    linguas = [x.strip() for x in args.linguas.split(",")]
    for lingua in linguas:
        if lingua not in FRASES:
            raise SystemExit(f"lingua sem frase conectada padrao: {lingua}")

    pasta = tempfile.mkdtemp(prefix="spike-gramatica-")
    print(f"# instrumento em {pasta}")
    if args.prova_negativa:
        prova_negativa_placar()

    resultados: list[dict] = []
    print(
        "lingua\tmodelo\tbraco\tn_ref\tn_hyp\tWER\tsub\tomi\tins\t"
        "abstencao_%\tinvencao_%\tsegundos\tomitidas"
    )
    for lingua in linguas:
        frase = args.frase if args.frase else FRASES[lingua]
        voz = args.voz if args.voz else VOZES[lingua]
        # Referencia = a frase NA ORDEM (o placar mede ordem); o lexico e o
        # conjunto de palavras distintas (o que o --prompt e o GBNF recebem).
        # Comparar contra o conjunto ORDENADO mede "saiu em ordem alfabetica":
        # defeito do instrumento, encontrado pela forma bruta do braco grammar.
        referencia = normaliza(frase)
        lexico = sorted(set(referencia))
        if not lexico:
            raise SystemExit(f"lexico vazio na lingua {lingua}")
        wav = sintetiza_conexo(frase, voz, f"{pasta}/conexo_{lingua}", taxa=args.taxa)
        print(
            f"# material ({lingua}): voz='{voz}' taxa={args.taxa} wpm frase='{frase}'"
        )
        gbnf = f"{pasta}/vocab_{lingua}.gbnf"
        with open(gbnf, "w") as fh:
            fh.write(gbnf_de(lexico))
        gbnf_controle = f"{pasta}/controle.gbnf"
        with open(gbnf_controle, "w") as fh:
            fh.write(gbnf_de(LEXICO_CONTROLE))
        prompt = ", ".join(lexico)
        print(f"# lexico ({lingua}): {lexico}")
        with open(gbnf) as fh:
            print(f"# gbnf ({gbnf}): {fh.read().strip()}")

        for nome, caminho in modelos:
            base = f"{pasta}/{lingua}_{nome}"
            args_roda = (args.whisper, caminho, wav, lingua, base)
            texto_base, seg_base = roda_braco(*args_roda, "baseline")
            palavras_base = normaliza(texto_base)
            t_prompt, seg_prompt = roda_braco(*args_roda, "prompt", prompt=prompt)
            t_gram, seg_gram = roda_braco(
                *args_roda, "grammar", grammar=gbnf, regra="root"
            )
            t_suave, seg_suave = roda_braco(
                *args_roda, "grammar_suave", grammar=gbnf, regra="root", penalty=1.0
            )
            t_sem, seg_sem = roda_braco(*args_roda, "sem_rule", grammar=gbnf)
            t_ctrl, seg_ctrl = roda_braco(
                *args_roda, "controle", grammar=gbnf_controle, regra="root"
            )
            bracos = [
                ("baseline", texto_base, seg_base, None),
                ("prompt", t_prompt, seg_prompt, None),
                (
                    "pos_hoc",
                    " ".join(corrige_pos_hoc(palavras_base, lexico, LIMIAR_POS_HOC)),
                    0.0,  # nao roda o modelo: e correcao do texto do baseline
                    None,
                ),
                ("grammar", t_gram, seg_gram, None),
                ("grammar_suave", t_suave, seg_suave, None),
                ("grammar_sem_rule", t_sem, seg_sem, "no-op"),
                ("grammar_controle", t_ctrl, seg_ctrl, "controle"),
            ]
            for braco, texto, seg, tipo in bracos:
                st = alinha(referencia, normaliza(texto))
                st.update(
                    {
                        "lingua": lingua,
                        "modelo": nome,
                        "braco": braco,
                        "texto": texto,
                        "segundos": round(seg, 2),
                        "tipo": tipo,
                    }
                )
                resultados.append(st)
                print(
                    f"{lingua}\t{nome}\t{braco}\t{st['n_ref']}\t{st['n_hyp']}\t"
                    f"{st['wer']:.2f}\t{st['sub']}\t{st['omi']}\t{st['ins']}\t"
                    f"{100 * st['abstencao']:.0f}\t{100 * st['invencao']:.0f}\t"
                    f"{st['segundos']:.2f}\t{','.join(st['omitidas']) or '-'}"
                )

        if args.prova_negativa:
            controle = [r for r in resultados if r["tipo"] == "controle"]
            if not controle:
                raise SystemExit("sem braco de controle: prova de atividade nao rodou")
            for r in controle:
                palavras = normaliza(r["texto"])
                # O modelo pode parar no meio da palavra: prefixo de palavra do
                # vocabulario de controle e compatibilidade, palavra inteira fora
                # dele e prova de que a gramatica nao foi obedecida.
                fora = [
                    p
                    for p in palavras
                    if not any(lex.startswith(p) for lex in LEXICO_CONTROLE)
                ]
                base = next(
                    x
                    for x in resultados
                    if x["braco"] == "baseline"
                    and x["lingua"] == r["lingua"]
                    and x["modelo"] == r["modelo"]
                )
                igual = r["texto"] == base["texto"]
                ok = not fora and not igual
                print(
                    f"# atividade da gramatica ({r['lingua']}/{r['modelo']}): saida do "
                    f"controle='{r['texto'][:60]}' | fora do vocabulario de controle: "
                    f"{fora or 'nenhuma'} | igual ao baseline: {igual} -> "
                    f"{'OK' if ok else 'FALHOU'}"
                )
                if not ok:
                    raise SystemExit(
                        "gramatica IGNORADA em silencio: a saida do controle nao "
                        "respeita o vocabulario de controle (ou repete o baseline)"
                    )
            for r in [x for x in resultados if x["tipo"] == "no-op"]:
                base = next(
                    x
                    for x in resultados
                    if x["braco"] == "baseline"
                    and x["lingua"] == r["lingua"]
                    and x["modelo"] == r["modelo"]
                )
                print(
                    f"# --grammar SEM --grammar-rule ({r['lingua']}/{r['modelo']}): "
                    f"saida igual ao baseline = {r['texto'] == base['texto']} "
                    f"(se True, a flag foi carregada e NAO aplicada)"
                )

    print("# bruto por braco (texto exatamente como o whisper emitiu):")
    for r in resultados:
        print(f"#   {r['lingua']}/{r['modelo']}/{r['braco']}: {r['texto']}")

    if args.json:
        with open(args.json, "w") as fh:
            json.dump(
                {"lexico_controle": LEXICO_CONTROLE, "resultados": resultados},
                fh,
                indent=2,
                ensure_ascii=False,
            )
            fh.write("\n")
        print(f"# numeros gravados em {args.json}")
    print(f"# residuo em {pasta} (apagar quando nao servir mais)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
