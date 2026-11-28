# Insight — O STT do Sovereign Pair: `large-v3-turbo` nativo, e o que disso serve ao Kentron

**Data:** 2026-11-27 · **Autoria:** sessão de engenharia · **Escopo:** recon por SSH no CT `workspace` (`/srv/projects/sovereign-pair`), repos `sp-service` e `sp-ui-chat` — leitura de código e de ADR, **sem executar nada lá** · **Questão:** qual modelo transcreve áudio no chat daquele produto, como foi feito, e o que muda na medição de ASR deste projeto.

## Evidências e hipóteses

**Fontes primárias (lidas em 2026-11-27):** `sp-service/src/speech/stt.rs` (508 linhas — o motor); `sp-service/Cargo.toml:183` e `:241`; `docs/adr/ADR-010-voz-e-visao-ocr-e29.md`; `sp-service/src/lib.rs:1804` (a rota); `sp-ui-chat/src/lib/components/ChatPanel.svelte` (`on:transcription`).

**O modelo.** `ggml-large-v3-turbo.bin` — revisão **pinada** `5359861c739e955e79d9a303bcbc70fb988958b1` do `ggerganov/whisper.cpp`, **sha256 conferido** (`1fc70f77…`), **1.624.555.275 bytes**, cacheado em `~/.sovereign/models/whisper`. Substituiu o `ggml-large-v3.bin` (~3 GB) porque *"os ~3GB do large-v3 falhavam consistentemente no meio do download em conexão doméstica"* — decisão do dono em 2026-08-22, com variantes quantizadas (q8_0 ~874 MB, q5_0 ~574 MB) mapeadas como plano B.

**O motor.** Crate **`whisper-cpp-plus` 0.1.5** (bindings pure-Rust para whisper.cpp), escolhido contra o `whisper-rs` — **arquivado** desde jul/2025 — com trade-off explícito: *"ativo-mas-jovem preferível a arquivado-mas-maduro"*. O `-sys` é **vendorizado** por uma incompatibilidade real (bindgen × libclang 23 gera `whisper_full_params` opaco → 40 erros E0609).

**Os parâmetros — e são só estes** (`whisper_params()`): `Greedy { best_of: 1 }`, `language("pt")`, `translate(false)`, `no_timestamps(true)`, `temperature(0.0)`. **Sem VAD, sem `suppress_non_speech`, sem prompt inicial.**

**O pipeline.** Decodificação com **symphonia** (Rust — sem `ffmpeg`), média dos canais para mono, reamostragem para 16 kHz, **chunks de 30 s exatos e sem overlap** (para **progresso** durante reuniões longas; 30 s é a própria janela do encoder, então o corte não custa mais que o whisper.cpp já custaria ali), inferência em `spawn_blocking`, contexto carregado uma vez (`ensure_whisper_context`).

**Duas medições deles, feitas com dado e não suposição:**
1. **Idioma fixo, não `auto`:** com `language("auto")` numa amostra pt-BR, o whisper.cpp autodetectou **italiano com 45,8% de confiança** (`p = 0,457982`) e o texto saiu misturado (`"è un test di…"` por `"é um teste de…"`); forçando `"pt"` o texto ficou correto **e quase 2× mais rápido** (25,8 s contra 47,5 s — pula o passe de detecção).
2. **Áudio nativo do Ollama: testado e descartado.** `capabilities: ["audio"]` descreve o **modelo**, não o runtime; o campo `audio` no payload é **ignorado em silêncio** (mesma contagem de tokens do baseline só-texto) e um WAV em `images` **é ingerido pelo pipeline de visão** — o modelo respondeu *"vendo uma imagem"*. Conclusão deles: **STT separado é obrigatório**.

**O que substituíram:** um pipeline **morto** — `POST /v1/multimodal/audio/transcribe` já existia e a UI já postava, mas o handler chamava um script Python (`faster_whisper`) que só existia em `_legacy/`; **toda transcrição falhava em silêncio em produção**. Trocaram o motor mantendo o **mesmo contrato** (`WhisperResult`), sem tocar no frontend.

## Alternativas e crítica

| item | eles (medido lá) | nós (medido aqui) | ação |
|---|---|---|---|
| modelo | **`large-v3-turbo`**, 1,62 GB | `small-q5_1`, `medium-q5_0` | **testar** — o mesmo `whisper-cli` roda o mesmo `.bin` |
| decodificação | **greedy `best_of 1`, `temp 0`, sem fallback** | default: `-bo 5` + fallback de temperatura (`-tpi 0.20`) | **testar `--no-fallback` / `-bo 1`** — hipótese: os laços de invenção são artefato do fallback |
| idioma | fixo `pt` (auto errou) | fixo `-l pt` ✔ | nada |
| voz cantada | **nenhum caso, nenhum tratamento** | medido, com WER por música | continuamos sozinhos nisso |
| pesos | revisão pinada + sha256 | mesma disciplina (E67) | nada |
| áudio multimodal no chat | Ollama áudio **descartado** | — | nada (o `lfm3-audio:2b` local **descreve**, não transcreve — ver PENDING) |

**Crítica:** a leitura é de **código e ADR**, e o produto deles é **fala de ditado/reunião** — o próprio comentário do módulo diz *"qualidade pt-BR praticamente equivalente pra fala de ditado/reunião"*. **Nenhum caso de voz cantada** existe lá, então isto **não responde** ao nosso problema de ASR em música: dá **duas alavancas novas** para medir e confirma que separar STT do chat multimodal foi a escolha certa (nós medimos o mesmo com o `lfm3-audio:2b`: ele **ouve, mas descreve**).

## Encaminhamento

Duas hipóteses **mensuráveis** entram na fila do ASR, no mesmo caderno de placar (as 17 músicas, a letra do dono como verdade, mesma métrica): **(a) `large-v3-turbo`** no lugar do `medium-q5_0`; **(b) decodificação sem fallback de temperatura / `best_of 1`** — se os laços de invenção desaparecerem, o modo de falha que assusta o produto era **nosso parâmetro**, não o modelo. Insight **não** aceita ADR nem autoriza implementação.
