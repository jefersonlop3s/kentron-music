# Insight — Open Video Downloader: o padrão da ferramenta do usuário

**Data:** 2026-11-22 · **Autoria:** sessão `cline/0caf3` · **Escopo:** avaliar `jely2002/youtube-dl-gui` (Open Video Downloader, "OVD") à luz da [ADR-0013](../adr/0013-link-de-musica-referencia-playlist-e-fronteira-do-audio.md) · **Questão:** como ele funciona, que regras quebra e o que o Kentron aprende sem copiar o modelo.

## Evidências e hipóteses

**Observado — repositório do OVD (2026-11-22):**

- **Tecnologia:** **Tauri** (backend **Rust**) + **Vue 3 + TypeScript**. Não é Electron.
- **Como funciona:** o README diz que, ao adicionar vídeo ou playlist, *"the app communicates with yt-dlp to fetch information, process options and start the download"*. É **GUI fina sobre um subprocesso**: a extração é do `yt-dlp`, não do app.
- **Recursos:** resolução/quadro/formato (MP4, MKV), playlists inteiras, template de nome de saída, fila balanceada, **autenticação por arquivo de cookies do navegador**, basic auth, senha de vídeo, **atualização automática do app e do `yt-dlp`**, notificações e atalhos.
- **Distribuição:** exe (Windows), dmg (Intel e Apple Silicon), AppImage/deb/rpm (Linux), Homebrew cask, WinGet e **Microsoft Store**.
- **Licença:** **AGPL-3.0**, com disclaimer explícito no README: *"We do not condone the use of this software to violate local laws or platform terms of service (including the DMCA). **Users are personally responsible** for ensuring they use this software fairly and within legal boundaries."*
- **Estado:** mantido — 1.064 commits; 74 issues e 26 PRs abertos; o repositório tem até `../../AGENTS.md`.

**Observado — `yt-dlp/yt-dlp` (mesmo dia):**

- Licença **Unlicense** (domínio público). 210 mil estrelas, 24.055 commits.
- Não usa a API oficial do YouTube: fala com o site.

**Inferências (não medidas):**

- O `ffmpeg` entra na cadeia para remux/conversão (daí MP4/MKV). A **licença da build distribuída** (LGPL ou GPL, conforme a configuração) **não foi verificada** — dívida de conferência para quem for distribuir.
- A auto-atualização do `yt-dlp` existe porque a extração **quebra com frequência**: é o custo real de depender do site, não da API.

## Alternativas e crítica

| Alternativa | Ganho | Custo — e o que a refutaria |
|---|---|---|
| O produto embarca e auto-atualiza o `yt-dlp`, como o OVD | O usuário não instala nada; um clique resolve | Muda a natureza: o **produto** vira autor da extração, contra a ADR-0012 §108; traz um binário Python de atualização semanal para a matriz de release; `deny.toml`/`MANIFEST` não cobrem isso |
| O produto copia ou forka o OVD | Reaproveita uma UX pronta | **Impossível por licença:** AGPL-3.0 × Apache-2.0. Copiar tornaria o Kentron AGPL |
| O produto **consome o arquivo** que a ferramenta do usuário produziu (ADR-0013, forma C) | Zero acoplamento; a responsabilidade fica onde já estava; a ferramenta evolui sozinha | Exige um passo manual — a refutaria se esse passo inviabilizar o uso real (o repertório tem 313 linhas) |

**O que a evidência sustenta:** o padrão "ferramenta do usuário" é **viável e maduro** — 9,8 mil estrelas com o **mesmo enquadramento** da ADR-0013 (o ato é do usuário; o app não é o autor). E a **autenticação por cookies do usuário** mostra que "ato do usuário, com credencial do usuário" é um padrão **técnico**, não só retórica jurídica.

**Sobre "quebrar regras", com precisão:** o que se quebra é o **contrato de uso do YouTube** (baixar conteúdo sem o link de download), que é um acordo entre usuário e plataforma — a consequência típica é pessoal (conta restringida), não criminal. Como o OVD **não usa a API oficial**, as *Developer Policies* da API (que proíbem isolar o áudio) **não se aplicam a ele**. O que **não** se quebra: a licença do `yt-dlp` (Unlicense permite embarcar) nem a do próprio OVD para quem apenas o usa como aplicativo.

**O que se aproveita de UX:** fila em lote (repertório de 313 linhas), template de nome de saída (casar `Artista - Título` com a ficha) e progresso/erro por item.

## Encaminhamento

- **Nada muda na decisão da ADR-0013** (forma C, aceita): o OVD **reforça**, não contradiz.
- **Restrição registrada:** nenhum código do OVD entra no projeto — **AGPL-3.0 é incompatível com Apache-2.0**. Interoperar por arquivo ou binário externo é permitido; copiar, não.
- **Hipótese para o produto:** aceitar **lote de arquivos** produzidos pela ferramenta do usuário, casando com a ficha por nome e duração, e declarando a origem `ExtraidaPeloUsuario`.
- **Permanece hipótese:** a licença da build de `ffmpeg` distribuída pelo OVD; e a acurácia do casamento nome/duração num lote real. **A build do Homebrew desta máquina foi medida** em [Decodificação de áudio por plataforma](2026-11-22-decodificacao-de-audio-por-plataforma.md): **GPL-3.0-or-later**.