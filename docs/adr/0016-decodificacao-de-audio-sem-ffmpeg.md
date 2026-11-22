# ADR-0016 · Decodificação de áudio: o motor lê o arquivo do usuário, sem ffmpeg

- **Status:** aceito (2026-11-22) — decisão do dono
- **Relaciona-se:** [ADR-0001](0001-licenca-apache-2-0.md) (licenças), [ADR-0008](0008-native-assets-em-vez-de-cargokit.md) (empacotamento), [ADR-0011](0011-mir-primeiro-incremento-de-analise-de-audio.md) (DSP), [ADR-0014](0014-analise-harmonica-pos-extracao.md) (análise)
- **Origem:** [Insight — decodificação de áudio por plataforma](../insights/2026-11-22-decodificacao-de-audio-por-plataforma.md)

## Contexto

O motor só lê **RIFF/WAVE** — `wav::ler` recusa o resto com `"não é RIFF/WAVE"` —, então o usuário precisa converter o arquivo antes de analisar. Medido em 2026-11-22:

- O `ffmpeg` do Homebrew é **GPL-3.0-or-later** (`--enable-gpl` com x264/x265/svt-av1/vmaf). Embarcá-lo traz obrigação de GPL e atrito de loja.
- **Mobile não tem gerenciador de pacotes** e o iOS exige app autocontido; a rota histórica, **`ffmpeg-kit`, foi arquivada em 2026-07-02** e segue como FFmpegKitNext, só código-fonte.
- **Symphonia** é Rust puro, **MPL-2.0**. O projeto upstream cobre mp3/mp2, AAC-LC, ALAC, FLAC, Vorbis, Opus, PCM, ADPCM e WavPack, nos contêineres isomp4, mkv, ogg, riff e caf. As otimizações SIMD **puxam o `rustfft`**, que o workspace já usa, e o desempenho declarado é "geralmente ±15% do FFmpeg".
- **Medido na árvore compilada (0.5.5, `cargo tree`)**: entram `codec-aac`, `codec-adpcm`, `codec-alac`, `codec-pcm`, `codec-vorbis`, `bundle-flac`, `bundle-mp3`, `format-isomp4`, `format-mkv`, `format-ogg`, `format-riff` e `format-caf`. **Não** entram `symphonia-codec-opus` nem `-wavpack`: a lista de features do 0.5.5 (`all-codecs`) não os inclui.
- O `deny.toml` **já permite MPL-2.0** como dependência sem modificação.

## Decisão

1. **Symphonia é o decodificador** de tudo que não for WAV PCM.
2. **`wav::ler` fica**: leitor próprio de RIFF/WAVE, sem dependência, que serve o caminho em memória (testes e quem já tem bytes). O dispatch é por **conteúdo** (`RIFF`/`WAVE`), não por extensão — extensão mente.
3. **`decode::abrir(caminho)`** decide entre os dois, e é o que a CLI usa. O caminho novo decodifica o buffer já lido, sem reabrir o arquivo.
4. **O produto não embarca `ffmpeg`.** Formato que o Symphonia não lê vira **erro explícito com o motivo** — fail-loud, e o usuário decide como converter.
5. **Um caminho para todas as plataformas:** por ser Rust puro, funciona em Android, iOS, web (wasm) e desktop, sem binário por SO e sem gerenciador de pacotes.

## Consequências

- **Ganho:** o usuário entrega o mp3/ogg/flac que já tem; cai a dependência de sistema; cai a pergunta de licença GPL; o mesmo código serve mobile, web e desktop; e a análise deixa de depender de conversão prévia.
- **Custo:** dependência nova em tempo e tamanho de build; **HE-AAC fica de fora** sem adaptador (`libfdk-aac`); **Opus e WavPack também ficam de fora** — a versão 0.5.5 não os traz no guarda-chuva, e cobri-los exige o crate `symphonia-codec-opus` e registro manual de códec (trabalho próprio, registrado no PENDING). Isso pesa: o `yt-dlp` costuma entregar **Opus** em WebM/Ogg.
- **Cobertura declarada:** a matriz de formatos é a do projeto upstream **filtrada** pelo que a versão compila — não a nossa. O teste prova o **encanamento** decodificando um WAV pelo mesmo caminho do Symphonia; não re-testamos cada códec, e dizemos isso em vez de fingir cobertura.
- **Provado de ponta a ponta nesta sessão:** MP3, **M4A (AAC)** e **FLAC** gerados na hora e analisados pela CLI (`/tmp/prova.*`). **Vorbis** compila (`symphonia-codec-vorbis` e `format-ogg` estão na árvore) mas **não foi exercido**: esta máquina não tem codificador Vorbis. **Ogg/Opus falha** com `códec não suportado`, como esperado.
- **Risco de licença:** MPL-2.0 vale como dependência **sem modificação**; patchear o código dela obrigaria a publicar o patch. É o mesmo enquadramento já registrado no `deny.toml`.
- **Pressuposto declarado:** precisamos de **PCM**, não de vídeo. Quem precisa de vídeo usa outra ferramenta — o produto não vira conversor multimídia.

## Referências

- [Insight — decodificação de áudio por plataforma](../insights/2026-11-22-decodificacao-de-audio-por-plataforma.md) — medições, licenças e o limite de ±50 cents da afinação.
- [ADR-0011](0011-mir-primeiro-incremento-de-analise-de-audio.md) — o DSP que consome o PCM.
- `deny.toml` — MPL-2.0 permitido como dependência sem modificação.
- `crates/mir_engine/src/wav.rs` — o leitor RIFF/WAVE que permanece.