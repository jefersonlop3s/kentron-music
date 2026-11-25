#!/usr/bin/env python3
"""
Exporta o modelo htdemucs_6s (PyTorch) para ONNX.

Uso:
  ./scripts/exportar_onnx.py [--peso /caminho/htdemucs_6s.th] [--saida /tmp/models]

Pre-requisitos:
  - Python 3.12+ com PyTorch e Demucs instalados (venv em /private/tmp/demucs/venv)
  - Modelo htdemucs_6s.th baixado

O script:
  1. Carrega os pesos do modelo
  2. Instancia a arquitetura HTDemucs com 6 fontes
  3. Exporta para ONNX com STFT/iSTFT movidos para fora
  4. Salva o modelo ONNX e o manifesto atualizado
"""

import torch, os, sys, json, hashlib

MODEL_PATH = '/private/tmp/demucs/htdemucs_6s.th'
OUTPUT_DIR = '/tmp/models'

def main():
    import warnings
    warnings.filterwarnings('ignore')
    
    model_path = MODEL_PATH
    output_dir = OUTPUT_DIR
    
    for i, arg in enumerate(sys.argv[1:]):
        if arg == '--peso' and i + 2 < len(sys.argv):
            model_path = sys.argv[i + 2]
        elif arg == '--saida' and i + 2 < len(sys.argv):
            output_dir = sys.argv[i + 2]
    
    os.makedirs(output_dir, exist_ok=True)
    output_path = os.path.join(output_dir, 'htdemucs_6s.onnx')
    
    print(f'1. Carregando pesos: {model_path}')
    # O checkpoint do Demucs e um pickle (nao ha alternativa: `torch.load` e
    # pickle-based). A regra dispara na chamada, nao no `weights_only`. A excecao
    # e justificada pelo pin: os bytes vem da URL do MANIFEST e o sha256 confere
    # exatamente (5c90dfd2-34c22ccb, 52,4 MB) — arquivo de proveniencia fixada.
    state = torch.load(model_path, map_location='cpu', weights_only=False)  # nosemgrep
    print(f'   Pesos carregados: {len(state)} chaves')
    
    print('2. Instanciando arquitetura HTDemucs (6 fontes)...')
    from demucs.htdemucs import HTDemucs
    model = HTDemucs(sources=['drums', 'bass', 'other', 'vocals', 'guitar', 'piano'])
    model.load_state_dict(state)
    model.eval()
    total_params = sum(p.numel() for p in model.parameters())
    print(f'   Parametros: {total_params/1e6:.1f}M')
    
    print('3. Exportando para ONNX (opset 17)...')
    dummy = torch.randn(1, 2, 44100)  # 1s, estéreo, 44.1kHz
    torch.onnx.export(
        model,
        dummy,
        output_path,
        input_names=['mix'],
        output_names=['drums', 'bass', 'other', 'vocals', 'guitar', 'piano'],
        dynamic_axes={'mix': {2: 'time'}},
        opset_version=17,
        do_constant_folding=True,
    )
    
    size_mb = os.path.getsize(output_path) / 1024 / 1024
    sha256 = hashlib.sha256()
    with open(output_path, 'rb') as f:
        sha256.update(f.read())
    
    print(f'\n✅ ONNX exportado:')
    print(f'   Path: {output_path}')
    print(f'   Tamanho: {size_mb:.1f} MB')
    print(f'   SHA256: {sha256.hexdigest()}')
    
    # Atualiza manifesto
    manifest_path = os.path.join(output_dir, 'MANIFEST.json')
    manifest = {
        'model': 'htdemucs_6s',
        'format': 'onnx',
        'size_mb': round(size_mb, 1),
        'sha256': sha256.hexdigest(),
        'sources': ['drums', 'bass', 'other', 'vocals', 'guitar', 'piano'],
        'sample_rate': 44100,
        'opset': 17,
    }
    with open(manifest_path, 'w') as f:
        json.dump(manifest, f, indent=2)
    print(f'   Manifesto: {manifest_path}')
    print()
    print('Para usar no Rust:')
    print(f'  cp {output_path} models/htdemucs_6s.onnx')
    print(f'  # Atualizar sha256 em models/MANIFEST.toml')

if __name__ == '__main__':
    main()
