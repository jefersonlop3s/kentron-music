# GAP: Motor MIR não cifra corretamente

**Data:** 2026-11-24
**Origem:** Teste de demonstração com `scripts/cifrar-10.sh` (worktree `fe672`)
**Severidade:** Média — não bloqueia R1 (MIR é etapa separada), mas inviabiliza o uso do motor para cifrar repertório real.

---

## Metodologia

O script `scripts/cifrar-10.sh` sorteou 10 músicas de `~/Music/MP3/` (263 arquivos disponíveis) e rodou `cargo run -p mir_engine -- <arquivo> --chordpro` para cada uma. Nenhuma falhou (rc=0 em todas). Resultados em `/tmp/cifras-10/`.

## Achados

### 1. Nenhum acorde detectado — 0 acordes em 10/10 músicas

O motor gerou seções (`[A-0s introducao]`, `[A-15s refrao_candidato]`, etc.) com acordes listados dentro, mas o parser não os reconhece como acordes válidos. Os acordes aparecem soltos na linha (ex: `[C] [C5] [Gm] [Gbmaj7]`), mas o `grep` de contagem não os capturou — formato de saída divergente do esperado.

**Possíveis causas:**
- O motor escreve acordes com espaços extras (`[C] [C5]`) em vez do formato ChordPro padrão `[C]` colado ao texto
- O separador de acordes pode estar diferente do que o `notation_bundle` espera

### 2. Muitas seções — super-segmentação

| Música | Seções |
|---|---|
| dag | 6 |
| depois-da-guerra | 28 |
| redencao | 42 |
| essencia | 46 |
| rompendo | 38 |
| eu-sou-livre | 80 |
| te-adorar | 46 |
| te-agradeco | 32 |
| ousado-amor | 72 |
| cantai | 34 |

Uma música de 3–5 minutos com 80 seções significa que o motor está criando uma seção a cada ~3 segundos. O limiar de novidade timbrística está muito sensível.

### 3. Tonalidades inconsistentes

| Música | Tom detectado | Tom esperado (approximado) |
|---|---|---|
| D.A.G (Oficina G3) | Gm | Provável Gm (OK) |
| Depois da Guerra | Abm | Dúvida |
| Redenção | D | Possível D (OK) |
| Essência da Adoração | B | Provável C ou G |
| Rompendo em Fé | A | Possível A (OK) |
| Eu Sou Livre | C | Possível C (OK) |
| Te Adorar | D | Possível D (OK) |
| Te Agradeço | E | Possível E (OK) |
| Ousado Amor | Dbm | Provável Db (aceitável) |
| Cantai | E | Possível E (OK) |

`B` para "Essência da Adoração" e `Abm` para "Depois da Guerra" são suspeitos — a margem de correlação pode estar baixa.

### 4. Título poluído com metadados do arquivo

Exemplos:
```
Oficina G3 ｜ D.A.G-(129k)
Oficina G3 ｜ Depois da Guerra (Animação)-(132k)
Te Adorar ｜ DVD Viver Por Ti ｜ CTMDT 1 ｜ Diante do Trono-(133k)
```

O motor está incluindo qualidade do áudio (`129k`), descrição visual (`HD`), e metadados de playlist no título. O título deveria ser extraído dos tags de metadados do arquivo, não do nome do arquivo.

### 5. Compasso não decidido

Todas as 10 músicas trazem:
```
{comment: compasso: 3/4 ou 6/8 ou 4/4? (relação bumbo×caixa) · barra N.NN s — confirme com {time: …}}
```

O motor não está discriminando `3/4` × `6/8` × `4/4` — é o problema conhecido da R4 (item aberto no PENDING).

---

## Recomendações para próxima sessão

1. **Verificar o formato de saída do `--chordpro`** — comparar a saída gerada contra o que `ChordProParser::parse` aceita.
2. **Ajustar a segmentação** — o limiar de novidade timbrística em `chords.rs` ou `estrutura.rs`.
3. **Limpar o título** — extrair metadados das tags ID3/MP4 em vez do nome do arquivo.
4. **Validar tonalidade** contra gabarito humano nas 10 músicas.
5. **Separar `4/4` × `6/8` × `3/4`** — problema conhecido, falta sinal discriminador.

## Artefatos

- Resultados brutos: `/tmp/cifras-10/` (10 `.cho` + 1 erro vazio)
- Script de teste: `scripts/cifrar-10.sh` (versionado na `main` desde `f7d8808`)