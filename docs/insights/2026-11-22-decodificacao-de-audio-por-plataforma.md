# Insight — Decodificação de áudio por plataforma: ffmpeg, onboarding e mobile

**Data:** 2026-11-22 · **Autoria:** sessão `cline/0caf3` · **Escopo:** fechar a dívida de licença do `ffmpeg` deixada no [insight do Open Video Downloader](2026-11-22-open-video-downloader.md) e avaliar como o Kentron decodifica áudio em desktop e mobile.

## Evidências e hipóteses

**Observado — a máquina de desenvolvimento (2026-11-22; comandos e saídas):**

- `which -a ffmpeg` devolve **uma** entrada: `/opt/homebrew/bin/ffmpeg`; `type -a` confirma. `/usr/local/bin/ffmpeg` e `/usr/bin/ffmpeg` **não existem** → **sem duplicata** (o teste da skill `toolchain-unica` passa).
- Symlink: `/opt/homebrew/bin/ffmpeg -> ../Cellar/ffmpeg/9.0.2_1/bin/ffmpeg`.
- `brew list --versions ffmpeg` → `ffmpeg 9.0.2_1`. `brew info` → **License: GPL-3.0-or-later**; *bottled*; 290 arquivos; 53,9 MB; 11 dependências (dav1d, lame, libvmaf, libvpx, openssl@4, opus, sdl2-compat, svt-av1, x264, x265, xz).
- `ffmpeg -version` → `--enable-gpl --enable-version3 --enable-shared`, com `--enable-libx264 --enable-libx265 --enable-libsvtav1 --enable-libvmaf`, além de `--enable-videotoolbox --enable-audiotoolbox`.
- `otool -L` → linkado **dinamicamente** às dylibs do próprio Cellar (`libavcodec.63.dylib`, `libavformat.63.dylib`, …) e às frameworks do sistema (AudioToolbox, AVFoundation, CoreAudio).
- **Conclusão:** a build do Homebrew é **GPL-3.0-or-later** — o GPL vem de x264/x265/svt-av1/vmaf. Não é LGPL.

**Observado — `arthenica/ffmpeg-kit` (2026-11-22):**

- Repositório **arquivado em 2026-07-02**. Aviso do README: *"FFmpegKit has been officially retired."* A continuação é o **FFmpegKitNext**, do mesmo autor, **distribuído só como código-fonte**.
- Licença: **LGPL-3.0** por padrão (os scripts usam `--enable-version3` e nunca `--enable-gpl`); com `--enable-gpl`, ou binários com sufixo `-gpl`, o bundle inteiro vira **GPL-3.0**.
- O próprio README avisa sobre patentes: x264, x265, kvazaar e openh264 (MPEG LA).

**Observado — `pdeljanov/Symphonia` (2026-11-22):**

- **Rust puro**, licença **MPL-2.0**. Demuxers: isomp4 (m4a/mp4), mkv, ogg, riff (wav), caf. Decodificadores: mp3/mp2, AAC-LC, ALAC, FLAC, Vorbis, Opus, PCM, ADPCM, WavPack.
- **As otimizações SIMD puxam o `rustfft`** — dependência que o workspace **já tem**.
- Desempenho declarado: "geralmente ±15% do FFmpeg". MSRV 1.85 (o projeto pina 1.97.1).

**Observado — o projeto (mesmo dia):**

- `deny.toml` **permite MPL-2.0** explicitamente, com o comentário "Copyleft por arquivo: só como dependência sem modificação".
- `crates/mir_engine/src/wav.rs` **só aceita RIFF/WAVE** — recusa o resto com "não é RIFF/WAVE". Hoje o motor exige WAV.

**Não verificado nesta sessão:** o texto da regra **2.5.2** das Diretrizes de Revisão da App Store (a página de ~97 KB volta truncada exatamente nesse ponto); e a licença da build de `ffmpeg` que o **OVD distribui** — o medido aqui é só a do Homebrew desta máquina.

## Alternativas e crítica

| Rota | Ganho | Custo — e o que a refutaria |
|---|---|---|
| **Onboarding + gerenciador de pacotes** (detecta, pede aceite, instala via brew/winget/apt) | É o padrão de desktop e **tira o binário GPL da nossa distribuição**: quem instala é o gerenciador do usuário | Exige rede e privilégio; a refutaria um ambiente sem gerenciador (Windows sem winget, máquina offline) |
| **Embarcar o `ffmpeg`** | Um passo só; funciona offline | **GPL-3.0**: cria obrigação de oferta de fonte do binário, e binário GPL dentro de pacote de loja é atrito. No mobile, pior ainda |
| **Decodificadores do sistema** (AVFoundation/AudioToolbox no iOS; MediaCodec/MediaExtractor no Android) | Zero licença extra, zero binário, aceleração de hardware | Cobre só o que a plataforma cobre; formato exótico fica de fora |
| **Symphonia no núcleo (Rust)** | Um caminho só para todas as plataformas, sem subprocesso, sem GPL, **MPL-2.0 já permitido** no `deny.toml`, e puxa `rustfft` que já temos | Muda o contrato de entrada do motor (hoje só WAV) e acrescenta uma dependência de decodificação — **decisão que pede ADR** |

## Encaminhamento

- **Dívida fechada:** o `ffmpeg` do Homebrew é **GPL-3.0-or-later**. "ffmpeg é LGPL" é verdade para *outras* builds — não para a padrão do brew.
- **Desktop:** o padrão que o dono descreveu (detectar → pedir aceite → instalar) é **correto e é também a saída de licença**, porque o binário GPL passa a ser distribuído pelo gerenciador de pacotes do usuário, não por nós. Alinha com o `../../AGENTS.md` — "dependência de sistema ausente: identifique o pacote, nunca contorne em silêncio".
- **Mobile:** não há gerenciador de pacotes nem instalação em tempo de execução. As rotas são **embarcar** (custo de licença) ou **usar o sistema**. A rota histórica — `ffmpeg-kit` — foi **aposentada em julho/2026** e segue como **FFmpegKitNext, só código-fonte**.
- **Hipótese que dispensava a pergunta:** adotar **Symphonia** no `mir_engine` para ler o arquivo do usuário direto. **Aceita e entregue em 2026-11-22** ([ADR-0016](../adr/0016-decodificacao-de-audio-sem-ffmpeg.md)): MP3, M4A (AAC) e FLAC provados de ponta a ponta. **Medido na entrega:** Opus e WavPack **não** vêm no guarda-chuva do 0.5.5, então Ogg/Opus ainda falha — ficou no PENDING com o caminho.
- **Permanece hipótese:** a acurácia do casamento nome/duração em lote real (do insight anterior) e a licença da build de `ffmpeg` embarcada pelo OVD.