# Insight — Como os serviços de transcrição de letra fazem (Lynote, Musely)

**Data:** 2026-11-27 · **Autoria:** sessão de engenharia · **Escopo:** leitura das páginas públicas de dois serviços que transcrevem letra de música; **não testamos as ferramentas** · **Questão:** o que eles usam, o que prometem, e o que disso muda a nossa medição.

## Evidências e hipóteses

**Fontes:** `lynote.ai/pt/ai-music-transcription` e `musely.ai/pt/tools/song-lyrics-transcriber` (ambas consultadas em 2026-11-27). São **páginas de marketing** — o que está abaixo é o que elas **afirmam**, não o que verificamos.

### Lynote

- Produto de "transcrição por IA" **genérico** (transcrição de YouTube, notas, resumo) com uma página para música. Entrada: arquivo, **link do YouTube** ou URL.
- Afirma **"precisão de 99,9%"**, 130+ idiomas, lote, tradução embutida, "privado e seguro".
- **Não nomeia modelo, método, nem separação de voz.** Nenhum detalhe técnico.
- O FAQ admite o que importa: *"instrumentação de fundo intensa, ruídos complexos ou baixa qualidade de gravação podem afetar os resultados"* e recomenda *"faixas vocais nítidas"*.

### Musely

- É o mais concreto: motor **Seed-ASR 2.0** (modelo de ASR real), **"97,3% de precisão em 51 idiomas"**, **10 minutos de duração máxima**.
- **Truque declarado: `dica de gênero`** (8 presets — Pop, Rock, Hip-Hop, R&B, Funk, Clássico, Eletrônico, Folk) "para otimizar como lida com entrega vocal, quebras de linha e ad-libs". Ou seja: **condicionamento do ASR**, não separação de fonte.
- Saídas: 4 presets — letra padrão, **karaokê com marcadores de tempo**, **Caderno do Compositor com sugestões de acordes + esquema de rima**, multilíngue com romanização. Estrutura detectada automaticamente (`[Verso 1]`, `[Refrão]`, `[Ponte]`). Toggle de backing vocals. Exporta Markdown/texto.
- **Também não menciona separação de voz.**

## Alternativas e crítica

| Eixo | Eles | Nós (medido) |
|---|---|---|
| Nunca separa a voz | afirmam precisão alta com "vocais claros" | **separamos** (Demucs) e medimos o ganho: 0,995 → 0,151 no pior caso |
| `dica de gênero` | 8 presets, sem número por gênero | nosso equivalente é o **`--prompt` de léxico**; medido: ajuda 8/13 dos casos ruins e **atrapalha** os bons |
| Karaokê com marcadores de tempo | linha (provável) | **alinhamento forçado sílaba a sílaba** (ADR-0012) — mais fino |
| Sugestões de acordes | "sugestões", sem precisão | medido: **raiz 93,4%**, rótulo exato 29,2% |
| Detecção de estrutura | verso/refrão automático | mecanismo `{bloco}`/`{canta}` + expansão de repetição (PENDING) |
| Precisão anunciada | 97,3%–99,9% | **1,136 de WER no mix**; 0,957 com voz+léxico — **números piores e verificáveis** |

**Leitura:** os dois são **ASR + formatação por LLM**, com pipeline **não divulgado**. A diferença entre os 97,3% deles e o nosso 0,957 é, com alta probabilidade, **o material medido**: eles medem (ou anunciam) voz limpa; nós medimos um repertório de hinos com instrumentação densa contra a letra do dono. O FAQ do Lynote confirma o pressuposto ("faixas vocais nítidas") e o do Musely tem teto de 10 min.

**Crítica que se aplica a nós também:** nenhum dos dois publica erro por música nem declara o conjunto de teste — e nós só temos 17 músicas (11% do repertório), num único gênero. Nossa vantagem não é o número: é **publicar o número**.

## Encaminhamento

- **Decisão do dono (2026-11-27), registrada:** recurso **experimental**, *"melhor erro que silêncio"*, **sem garantia de sucesso** e **sujeito a revisão** — ver §7.6 do [termo de uso](../legal/termo-de-uso.md) e a [nota técnica](../runbooks/transcricao-de-letra-experimental.md).
- **O que adotamos:** separar a voz, `-sns` obrigatório, léxico como contexto **condicional**, e o interruptor por sinal da própria saída (§5 da nota técnica).
- **O que não copiamos:** anunciar precisão sem conjunto de teste declarado; o `dica de gênero` só entra se medido contra o léxico.
- **Hipótese a medir:** o proxy (`[música]`/loop) como preditor do interruptor — sem isso o interruptor é hipótese, não recurso.
- **Vínculos:** [PENDING](../PENDING.md); [ADR-0012](../adr/0012-letra-fonte-alinhamento-e-limites.md); [ADR-0013](../adr/0013-link-de-musica-referencia-playlist-e-fronteira-do-audio.md).
