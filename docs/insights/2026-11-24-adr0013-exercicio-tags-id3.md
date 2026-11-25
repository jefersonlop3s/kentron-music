# Insight: Exercício prático de ADR-0013 — Planilha → Tags ID3

**Data:** 2026-11-24
**Origem:** Sessão de trabalho pós-R1.1, antes de atacar ADR-0017.

## O que foi feito

Script Python que lê a planilha `Repertoire_Louvor_IBG_20261121-155610.xlsx` (313
músicas) e tenta casar cada MP3 em `~/Music/MP3/` (40 arquivos) com sua entrada
correta na planilha, para escrever tags ID3/MP4 corretas.

## Resultado

| Métrica | Valor |
|---------|-------|
| Músicas na planilha | 313 |
| MP3s encontrados | 40 |
| Casados por nome | 37 (92%) |
| Não casados | 3 |

## Lições

### 1. Symphonia lê tags existentes corretamente ✅

O decoder extrai `title`, `artist`, `album`, `date` dos arquivos que têm ID3v2.
O MP3 "De Joelhos" já tinha tags corretas — o Symphonia as lê e a CLI passou a
usá-las como título (item 4 do GAP, corrigido neste mesmo ciclo).

### 2. Casamento por nome de arquivo é frágil ⚠️

O algoritmo de overlap de palavras normalizadas casou 37/40, mas com falsos
positivos: múltiplas músicas da Oficina G3 foram todas casadas como "Ele Vive"
porque o nome do artista dominava o overlap. Um arquivo "Oficina G3 | D.A.G" não
deveria casar com uma música chamada "Ele Vive".

**Solução verdadeira:** resolver o **link do YouTube** (coluna K da planilha) →
título, artista, álbum oficiais pela YouTube Data API. O link é a identidade, não
o nome do arquivo.

### 3. Escrever tags com ffmpeg é trivial

```bash
ffmpeg -i original.mp3 -metadata title="D.A.G" -metadata artist="Oficina G3" \
       -metadata album="Elektracustica" -codec copy original.tagged.mp3
```

### 4. Isolamento do MP3 na planilha

A planilha tem **313 links de vídeo** (coluna K) e **266 tons** (coluna F). Para
um exercício completo de ADR-0013, o pipeline seria:

```
link YouTube → YouTube Data API → {title, artist, album} canônico
                                          ↓
                              match com planilha por song_id
                                          ↓
                              escrever tags ID3 no MP3
                                          ↓
                              Symphonia lê tags → CLI usa
```

## Decisão

O exercício confirmou que o **decoder funciona** e que o **gargalo é a resolução
de link** (ADR-0013), não a leitura de tags. Seguimos para ADR-0017 (bateria
separada) — o que fecha o problema do compasso (item 5 do GAP). O link →
identidade fica como o próximo bloco depois da ADR-0017.

## Artefatos

- Script de casamento: `/tmp/tag_repertorio.py` (não versionado)
- Planilha fonte: `~/Downloads/Repertoire_Louvor_IBG_20261121-155610.xlsx`
- MP3s: `~/Music/MP3/` (40 arquivos)