#!/usr/bin/env python3
"""
Exporta o htdemucs_6s (PyTorch) para ONNX.

Uso:
  ./scripts/exportar_onnx.py [--peso /caminho/htdemucs_6s.th] [--saida /tmp/models]

Pre-requisitos:
  - Python 3.12+ com PyTorch, Demucs e onnxruntime (venv em /private/tmp/demucs/venv)
  - Modelo htdemucs_6s.th baixado (models/MANIFEST.toml, sha256 34c22ccb...)

O script:
  1. Carrega os pesos do bag do Demucs (bag["state"], nao o bag)
  2. Instancia a arquitetura descrita no proprio checkpoint (klass + kwargs)
  3. Exporta o modelo INTEIRO para ONNX (opset 17)
  4. Compila o artefato com onnxruntime e imprime tamanho, sha256 e manifesto

O escopo do export e o modelo inteiro, com o STFT/iSTFT dentro da rede.

MEDIDO (2026-11-26), com o checkpoint do MANIFEST (52,4 MB, sha256 34c22ccb):

  1) com `bag["state"]` os pesos carregam — `load_state_dict` estrito passa com
     as 525 chaves (antes o script passava o bag inteiro e reprovava);
  2) o export **nao** passa, e o erro e do torch, nao do script:

       torch.onnx.errors.SymbolicValueError: STFT does not currently
       support complex types

     O `torch.stft` de `HTDemucs._spec` nao tem symbolic function para ONNX
     (opset 17). E exatamente por isso que o `sevagh/demucs.onnx` — a origem
     dos pesos que a ADR-0017 adota — move o STFT/iSTFT para FORA da rede.

Este script nao faz essa cirurgia: ele exporta o modelo inteiro, e para no
ponto em que ela seria necessaria, com o erro real. Exportar de novo so faz
sentido depois de o STFT sair do grafo.
"""

import argparse
import hashlib
import json
import os
import sys
from fractions import Fraction

import torch

MODEL_PATH_PADRAO = "/private/tmp/demucs/htdemucs_6s.th"
OUTPUT_DIR_PADRAO = "/tmp/models"


def carregar_bag(model_path):
    """Le o checkpoint do Demucs e devolve (klass, kwargs, state).

    O checkpoint e um *bag* com as chaves `klass`, `args`, `kwargs`, `state`,
    `training_args` e `metrics`. Os **pesos** estao em `bag["state"]`: passar o
    bag direto para `load_state_dict` reprova com as 525 chaves faltando
    (medido em 2026-11-26 — era o defeito deste script).
    """
    # O checkpoint do Demucs e um pickle (nao ha alternativa: `torch.load` e
    # pickle-based). A regra dispara na chamada, nao no `weights_only`. A
    # excecao e justificada pelo pin: os bytes vem da URL do MANIFEST e o
    # sha256 confere exatamente (34c22ccb, 52,4 MB) — proveniencia fixada.
    bag = torch.load(model_path, map_location="cpu", weights_only=False)  # nosemgrep
    if not isinstance(bag, dict):
        raise SystemExit(f"checkpoint nao e um bag do Demucs: {type(bag).__name__}")
    for chave in ("klass", "state"):
        if chave not in bag:
            raise SystemExit(f"checkpoint sem a chave {chave!r}: {sorted(bag)}")
    return bag["klass"], bag.get("kwargs") or {}, bag["state"]


def verificar_onnx(caminho):
    """Compila o artefato com onnxruntime — export que nao compila nao e export."""
    try:
        import onnxruntime as ort
    except ImportError as erro:
        raise SystemExit(
            f"onnxruntime ausente ({erro}); o artefato NAO foi verificado. "
            "Instale com: pip install onnxruntime"
        ) from erro
    sessao = ort.InferenceSession(caminho, providers=["CPUExecutionProvider"])
    entrada = sessao.get_inputs()[0]
    saida = sessao.get_outputs()[0]
    print(
        f"   compila · entrada {entrada.name} {entrada.shape} "
        f"· saida {saida.name} {saida.shape}"
    )
    if saida.shape != [1, 6, 2, int(entrada.shape[2])]:
        raise SystemExit(
            f"saida inesperada {saida.shape}: o motor espera (1, 6, 2, amostras)"
        )


def sha256_de(caminho):
    digest = hashlib.sha256()
    with open(caminho, "rb") as f:
        for bloco in iter(lambda: f.read(1 << 20), b""):
            digest.update(bloco)
    return digest.hexdigest()


def main():
    parser = argparse.ArgumentParser(description="Exporta o htdemucs_6s para ONNX.")
    parser.add_argument(
        "--peso",
        default=MODEL_PATH_PADRAO,
        help="checkpoint .th do Demucs (ver models/MANIFEST.toml)",
    )
    parser.add_argument(
        "--saida", default=OUTPUT_DIR_PADRAO, help="diretorio do artefato"
    )
    args = parser.parse_args()

    if not os.path.isfile(args.peso):
        raise SystemExit(
            f"peso nao encontrado: {args.peso}\n"
            "veja models/MANIFEST.toml (weights_url + sha256 do htdemucs_6s)"
        )

    print(f"1. Carregando pesos: {args.peso}")
    klass, kwargs, state = carregar_bag(args.peso)
    print(f"   {klass.__name__} · {len(state)} chaves de pesos")

    print("2. Instanciando a arquitetura que o checkpoint declara...")
    model = klass(**kwargs)
    model.load_state_dict(state)
    model.eval()
    print(f"   {sum(p.numel() for p in model.parameters()) / 1e6:.1f}M parametros")

    # A entrada e o segmento do proprio modelo: (1, canais, segmento * taxa) —
    # o mesmo (1, 2, 343980) que o MANIFEST e o `stems_onnx.rs` usam. Um dummy
    # de 1 s exportava um grafo com outro tamanho de janela.
    segmento = Fraction(kwargs.get("segment", Fraction(39, 5)))
    taxa = int(kwargs.get("samplerate", 44_100))
    canais = int(kwargs.get("audio_channels", 2))
    fontes = list(kwargs.get("sources", []))
    if len(fontes) != 6:
        raise SystemExit(f"checkpoint sem 6 fontes: {fontes}")
    amostras = int(segmento * taxa)
    print(f"3. Exportando para ONNX (opset 17): (1, {canais}, {amostras}) ...")

    os.makedirs(args.saida, exist_ok=True)
    caminho = os.path.join(args.saida, "htdemucs_6s.onnx")
    dummy = torch.randn(1, canais, amostras)
    torch.onnx.export(
        model,
        dummy,
        caminho,
        input_names=["mix"],
        output_names=fontes,
        dynamic_axes={"mix": {2: "time"}},
        opset_version=17,
        do_constant_folding=True,
        # `dynamo=False` = exportador TorchScript, o histórico. O caminho
        # `dynamo` (padrão no torch >= 2.9) exige o pacote `onnxscript`, que o
        # ambiente do Demucs não tem — e gerar peso não é motivo para abrir
        # dependência nova. Medido em 2026-11-26: sem isto, o script morre em
        # `ModuleNotFoundError: No module named 'onnxscript'`.
        dynamo=False,
    )

    print("4. Verificando o artefato...")
    verificar_onnx(caminho)

    tamanho_mb = round(os.path.getsize(caminho) / 1024 / 1024, 1)
    digest = sha256_de(caminho)
    print()
    print("ONNX exportado e verificado:")
    print(f"   caminho: {caminho}")
    print(f"   tamanho: {tamanho_mb} MB")
    print(f"   sha256:  {digest}")
    print(f"   fontes:  {', '.join(fontes)}")

    manifesto = {
        "model": "htdemucs_6s",
        "format": "onnx",
        "size_mb": tamanho_mb,
        "sha256": digest,
        "sources": fontes,
        "sample_rate": taxa,
        "segment_samples": amostras,
        "opset": 17,
    }
    caminho_manifesto = os.path.join(args.saida, "MANIFEST.json")
    with open(caminho_manifesto, "w", encoding="utf-8") as f:
        json.dump(manifesto, f, indent=2)
        f.write("\n")
    print(f"   manifesto: {caminho_manifesto}")
    print()
    print("Para o motor Rust usar este artefato, ele vai em:")
    print("  models/weights/htdemucs_6s/htdemucs_6s.onnx")
    print(
        "e o sha256 acima entra em models/MANIFEST.toml ([[model]] htdemucs_6s_onnx)."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
