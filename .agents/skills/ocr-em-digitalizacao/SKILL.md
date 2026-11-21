---
name: ocr-em-digitalizacao
description: Como extrair dado de hinário ou documento que só existe em PDF escaneado — diagnóstico de camada de texto, OCR nativo do macOS (Vision) por pyobjc num venv de /tmp, localização da página por calibração e leitura do crédito impresso. Carregar quando o dado não vem em PDF pesquisável.
---

# OCR em digitalização (hinário, atrás de crédito)

Caso de origem (2026-11-21): a **Harpa Cristã Cifrada** (521 páginas) e a **Harpa Cristã tradicional
CPAD** (724 páginas) vieram em PDF **sem camada de texto** (`get_text()` devolvia 0 caracteres em 6
páginas). Com OCR nativo saíram, do próprio livro, o crédito do hino 291 (`AA.`) e o **Índice de
Autores e Tradutores** (CPAD p.722), cujo OCR trouxe **`A.A. - Anlúniv Almeida`**: o índice
**localiza** a entrada, mas **não fecha o nome** — "Antônio Almeida" foi correção de leitura apoiada
em fonte secundária, e chamar isso de leitura do impresso foi o erro de 2026-11-21.

## Mecânica

1. **Diagnostique antes de tudo** — pesquisável ou imagem:
   `pymupdf.open(cam)[i].get_text()`; menos de ~200 caracteres em 6 páginas = digitalização.
2. **Ambiente isolado** (nada instalado no sistema):
   `python3 -m venv /tmp/pdfvenv && /tmp/pdfvenv/bin/pip install -q pymupdf pyobjc-framework-Vision pyobjc-framework-Quartz`
3. **OCR nativo (Vision)**: `VNRecognizeTextRequest`, `RecognitionLevel=1` (accurate),
   `RecognitionLanguages=["pt-BR","en-US"]`, `UsesLanguageCorrection=True`. Renderize com
   `get_pixmap(dpi=...)`: **180–200 dpi** para corpo de texto; **60 dpi basta** para número, título e
   iniciais (imagem leve também aceita em leitura visual).
4. **Localize por calibração, nunca por varredura cega**: o número do hino vem impresso no topo.
   Leia 2 páginas conhecidas, calcule a razão hinos/página e confirme na candidata (caso real:
   p.273 = 313 e p.300 = 345 ⇒ 1,19 hino/página ⇒ p.254 = 291).
5. **Leia o crédito impresso** — nos hinários da CPAD ele é uma **sigla** ao pé do hino (`AA.`,
   `P.L.M.`, `D.L.`, `S.N.`); o nome sai do **índice do próprio livro**, quase sempre nas últimas
   páginas (`Índice de Autores e Tradutores`).
6. **Valide no sentido negativo**: confirme o número pelo **índice das primeiras linhas** ("... 291").
7. **OCR erra nome próprio** (`Anlúniv Almeida` para Antônio Almeida): use o OCR para **localizar** e
   confirme o nome no impresso (a olho) ou em fonte limpa. **Registre a procedência com o cru e a
   correção** — achar que a correção "obvia" é a leitura foi o que fez `Anlúniv Almeida` virar fato
   nos documentos em 2026-11-21.

## O que NÃO fazer

- Não OCR-izar o livro inteiro (lento e desnecessário): calibre e vá direto às páginas.
- Não aceitar nome próprio saído de OCR sem conferência.
- Não versionar o PDF nem a imagem no repositório — a obra é protegida; o que entra é **fato**.
- Não inferir crédito pelo título: o hino pode ter crédito só de tradução (a sigla) e o original vir
  de fora do livro.
