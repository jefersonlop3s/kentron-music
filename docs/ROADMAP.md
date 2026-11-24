# Roadmap · Kentron Music Hub

Este documento detalha a trajetória de evolução do Kentron, dividida em seis releases verticais. Cada release é liberada por um **Spike de Validação (S)** e validada por uma **Matriz de Testes** rigorosa.

## 🗺️ Visão Geral do Fluxo
`R0 (Fundação) ➔ R1 (Cifra/Projeção) ➔ R2 (Banda Conectada) ➔ R3 (Palco ao Vivo) ➔ R4 (Motor MIR) ➔ R5 (Rede Global)`

---

## 🏁 R0 · Fundação e Guardas (publicado na `main`)
**Objetivo:** Estabelecer a infraestrutura de engenharia e a identidade do produto.
- [x] Nome definido: **Kentron Music Hub**.
- [x] Licença: Apache-2.0, com aviso explícito de uso "como está".
- [x] Sovereign Harness: guardrails e git hooks ativos.
- [x] Impeccable UI: [DESIGN.md](../DESIGN.md) e vocabulário visual definidos.
- [x] Infra: Cargo Workspace, Melos, scaffold Flutter e CI com ações fixadas por SHA.
- [x] Pipeline local `scripts/ci-local.sh` como portão principal (ADR-0004); Actions como rede de segurança em PRs e semanal.
- [x] Core: `notation_bundle` (`chord_naming`) com testes dourados.
- [x] Validação Flutter local (`flutter analyze`/`flutter test`) com Flutter 3.50.1.
- [x] Integração real do `core_l10n` no app: delegates, locales suportados e a saudação da tela inicial vindos do ARB. Provado em teste com en/pt/es e com o fallback regional `pt_PT` → `pt`.
- [x] Ponte Rust ↔ Flutter: o crate adaptador `kentron_ffi` (único `cdylib`) expõe `parse_song` ao Dart, com bindings gerados e provados por teste de Flutter. Falta o app abrir em dispositivo — depende dos diretórios de plataforma.
- [x] Alvo **web** do app: scaffold, wasm da ponte compilado e `scripts/servir-web.py` com COOP/COEP — build e serviço verificados. Falta a tela consumir a ponte (prova no navegador) e os diretórios `android/`/`ios/`.
- [x] Pipeline local validado de ponta a ponta (Rust, licenças, ARBs e Flutter).
- [x] CI remoto mantido como rede de segurança: sem gatilho em `push` para `main`; `pull_request`, `workflow_dispatch` e `schedule` semanal.
- [x] App migrado para `material_ui` 1.11.1.

---

## 🎸 R1 · Cifra, Palco Local e Projeção (parcialmente integrado à `main`)
**Objetivo:** Resolver a dor imediata de leitura e projeção de cifras offline.
**Portão de Entrada:** o spike S5 dimensiona MusicXML/ATEX (R1.1) e **não** bloqueia o alpha nem o piloto (ADR-0005 rejeitada).

### R1-0 · Fundação técnica (entregue)
- [x] Ponte Rust ↔ Flutter por FFI (ADR-0007): `crates/kentron_ffi` expõe `parse_song` ao Dart, provada por teste de Flutter que carrega o cdylib.
- [x] Alvos Android, iOS e Web gerados; iOS e Android buildam e rodam com o núcleo respondendo; o alvo web foi provado no navegador.
- [x] Empacotamento do Rust por Native Assets (ADR-0008): build hooks no lugar do cargokit, sem CocoaPods e sem o aviso de SPM.
- [x] Tela inicial consumindo o núcleo: `HomeScreen` chama `parse_song` e mostra o tom e os acordes.

### R1-A · Cifra local utilizável
- [x] Parser ChordPro e modelo canônico com testes dourados.
- [ ] Fixtures de parser para acordes invertidos, `6/9` e acidentes duplos (a nomeação já os cobre).
- [x] Biblioteca local, setlists e Song Bundle versionado com exportação/restauração. Modelo em memória (`bundle`) e persistência (`bundle_store`, ADR-0006) entregues: diretório com `meta.json` + `song.cho`, índice da biblioteca, setlists, `schema_version` com migração e exportação por cópia. Falta empacotar `score.musicxml`/`tabs/*.atex` e o contêiner único (v1.1).
- [x] Transposição e renderização nos quatro sistemas (29 testes, incluindo acordes invertidos, `6/9` e acidentes duplos).
- [ ] Testes de propriedade e regressão ampliada.

### R1-B · Projeção manual
- [x] Exportação OpenLyrics com validação XML: módulo `openlyrics::export` no `notation_bundle` gera XML a partir do modelo `Song` (8 testes).
- [ ] Matriz de compatibilidade com versões fixadas de Holyrics e outros consumidores.
- [x] Bootstrap de projeção por URL (`?projecao=<base64url do ChordPro>`) e botão "Copiar link de projeção" no leitor, sem `obs-websocket`.
- [x] Provar o Browser Source dentro do OBS: `scripts/piloto-obs.sh` abre a projeção no Browser Source por `obs-websocket`, captura a fonte e reprova quando a tela fica chapada. Aprovado em duas rodadas (353/354 cores, σ≈19,5) e a prova negativa reprovou a URL morta.
- [x] Paginação intra-seção: seções longas são divididas automaticamente em sub-páginas (6 linhas/página). Acentos e pseudolocalização testados (12 testes Flutter, 8 Rust).

### R1-C · Alpha utilizável (portão inicial, ADR-0009)
**Objetivo:** provar o fluxo completo em uso real de ensaio, sem depender de calendário de culto.
- [x] Leitor de cifras (Modo Palco) com transposição por tom, acorde sobre a letra e alternância de contraste.
- [x] Setlist utilizável no Modo Palco (navegação entre músicas via LibraryPage).
- [x] Modo projeção manual em tela cheia a partir do leitor: uma seção por vez, letra grande, avanço por toque, teclado ou rodapé.
- [x] Projeção em segunda tela via URL (`?projecao=`). Provado com testes de widget (duas instâncias independentes) e com `piloto-obs.sh` (Browser Source do OBS).
- [ ] Alpha assistido: um ensaio com o fluxo completo, desenvolvedor presente, falhas registradas com diagnóstico.
- [ ] Dados sobrevivem a fechar e reabrir o app (bundle restaurado). **Não é bug de persistência: a tela de restauração não existe.** O `bundle_store` (ADR-0006) grava e lê o diretório, e o app não o chama na inicialização — falta a tela/fluxo que restaura o bundle, não a capacidade de armazenar.

### R1-D · Piloto automatizado e release (portão final, ADR-0010)
**Objetivo:** provar a release em ambiente controlado, sem depender de calendário de igreja.
- [x] Piloto automatizado na VM Windows + OBS: projeção carregada por Browser Source, cifra aberta, projetada e navegada por seção — `scripts/piloto-obs.sh` prova o render dentro do OBS e a troca de seção mudando o pixel. **Aberto:** transposição dentro do OBS (a projeção não mostra acorde, então transpor não muda o pixel; é provada em `scripts/e2e-web.sh` e nos testes Rust) e o app sobreviver a fechar/reabrir.
- [x] Duas rodadas consecutivas simuladas (substituem os "dois cultos" como espera) — o roteiro roda as duas e compara o hash do pixel entre elas.
- [ ] Nenhuma perda de dados, crash bloqueante ou falha de projeção.
- [ ] pt-BR, en-US e es-419 revisados por falantes nativos — em paralelo, não como portão de calendário.
- [ ] MusicXML e ATEX ficam para R1.1, depois do S5 — sequência de execução; o escopo de formatos da ADR-0002 permanece integral (ADR-0005 rejeitada).

**Depois da release:** o uso ao vivo em culto acontece e realimenta a próxima release; **não** bloqueia a publicação.

**Depois da release · alvos desktop (2026-11-22, [ADR-0015](adr/0015-alvos-desktop-e-tela-de-palco-por-url.md) aceita):** Windows, macOS e Linux, porque tecladista usa notebook. A tela de palco é **view por URL** (`?palco=`) — o mesmo endereço serve navegador da TV, segunda janela do desktop e Browser Source —, então a segunda tela não exige multi-janela. Ordem: Windows → Linux → macOS (notarização do macOS é o custo mais alto e o menor alcance). Não entra na R1.

**Validação R1:** unitária, regressiva, exploratória e acessibilidade em todo push; E2E por alpha assistido (R1-C) e, no fim, piloto automatizado (R1-D).

---

## 🌐 R2 · Banda Conectada
**Objetivo:** Sincronização de estado e comunicação entre músicos.
**Portão de Entrada:** Spike S4 (Matrix e Push no iOS).

### Etapas:
1. **Identidade:** Perfis, chaves de dispositivo e convites por QR Code.
2. **Sincronização:** Agenda, escalas e setlists via Matrix/Iroh.
3. **Comunicação:** Chat mínimo (texto e salas) integrado ao fluxo de ensaio.
4. **Trust & Safety:** Sistema de denúncia e lista de bloqueio assinada.

---

## ⏱️ R3 · Palco ao Vivo
**Objetivo:** Regência em tempo real e sincronia de palco.
**Portão de Entrada:** Spike S2 (Sincronia de Tempo/Bateria).

### Etapas:
1. **Stage Clock:** Maestro manual, Sync de Start e auto-scroll por compasso.
2. **Regência:** Transições litúrgicas e avisos (cues) via rede LAN efêmera.
3. **Integração OBS:** Controle remoto via `obs-websocket`.
4. **Painel FOH:** Interface para o técnico de som.

---

## 🧠 R4 · Motor MIR (Music Information Retrieval)
**Objetivo:** Automação de cifras e tablaturas via análise de áudio.
**Portão de Entrada:** Spikes S1 (MIR on-device), S2 e S3 (Identidade da Obra).

> **Primeiro incremento entregue (2026-11-21, ADR-0011):** dinâmica (EBU R128, contraprovada contra o `ffmpeg`), andamento, acordes e estrutura — **DSP puro, sem modelo**, aberto por decisão do dono como instrumento de trabalho e **fora do portão da R1**. O portão de entrada acima segue valendo para a parte com ML (stems, letra, tablatura).
>
> **Segundo incremento (2026-11-21, ADR-0012 em proposta):** a **letra** entra na cifra por fonte declarada (texto do usuário ou corpus de domínio público) e é **alinhada ao áudio** por frases vocais; os acordes caem sobre a sílaba. O provedor "oficial" de letra é pago e proíbe armazenar o texto — decisão do dono pendente.
>
> **Camada de dados do corpus (2026-11-21, `scripts/verificar_fichas.py`):** as fichas do hinário carregam o **`lyrics_scope`** (se a letra pode ser exibida: `public_domain`, `licensed`, `none`) e o **vínculo `obra_original`** (ficha cujo texto é a fonte da tradução brasileira). O verificador valida as duas colunas contra o corpus de domínio público e o dicionário de variantes; o vínculo impede que uma tradução seja lida como obra independente. É o que sustenta a decisão de letra no app — sem `lyrics_scope` declarado, a ficha não pode exibir letra.
>
> **Link, playlist e áudio (2026-11-22, [ADR-0013](adr/0013-link-de-musica-referencia-playlist-e-fronteira-do-audio.md) aceita):** o link entra como **referência** (identidade da obra e do fonograma) e a playlist como **lista de referências**; o áudio continua vindo do usuário. Spotify não tem caminho de áudio — sem endpoint, e a quota estendida só aceita organização desde 15/05/2025 — e o YouTube proíbe isolar o áudio pela via oficial ("separate, isolate, or modify the audio or video components", Developer Policies). O produto **não baixa de provedor nenhum**. A etapa 4 fica dividida: **link → identidade da obra** é implementável agora; **→ cifra** exige o áudio do usuário.
>
> **Análise harmônica (2026-11-22, [ADR-0014](adr/0014-analise-harmonica-pos-extracao.md) aceita):** centro tonal por perfil no eixo de quintas, afinamento global, terça decidida pela **melodia** (F0), baixo por compasso para a **inversão** e região harmônica separada da seção timbrística. Resolve como causa comum a inversão, a terça fraca e a tensão — e **não** depende de stems: a nota anterior, que exigia baixo isolado, era hipótese, não medição.

> **Separação de bateria (2026-11-24, [ADR-0017](adr/0017-separacao-de-bateria-por-modelo.md) aceita):** a medida de metro falha porque **a bateria não está separada** — medido: quatro variantes do offset caixa×bumbo, nenhuma separou `3/4` de `6/8`, porque a banda de 30–110 Hz carrega o **baixo** e porque bumbo e caixa batem juntos no tempo 1. Proposta: **`htdemucs_6s` em ONNX por `ort`**, **MIT**, **52,4 MB medidos**, com o STFT no motor. É o **primeiro modelo** do motor — o aceite depende da decisão do dono sobre o portão e da confirmação da licença **dos pesos**.

### Etapas:
1. **Análise Local:** Áudio $\rightarrow$ Acordes, Tom e Compasso (Desktop primeiro). *Acordes e andamento entregues; **tom por perfil de tonalidade** desde 2026-11-22 (ADR-0014, bloco 1). Compasso: a **barra em segundos** e o **acento por instrumento** (bumbo/caixa/chimbau, com `backbeat()`) entregues em 2026-11-22; **a discriminação `3/4` × `6/8` continua aberta** — medido em nove faixas de banda que energia de banda no mix não separa (margem 0,00–0,04 em oito).*
2. **Geração Automática:** Tablaturas e partituras via inferência ONNX. *Pendente, com peso a declarar. **Base medida em 2026-11-22:** o mapa de Hz por peça — bumbo 53% abaixo de 300 Hz, caixa 65% em 1,5–5 kHz, chimbau 60% em 6,5–13 kHz — separa **bumbo de caixa por banda** e **não** separa caixa de chimbau (25% de energia cruzada, simétrico), que exige o **decaimento**. É o vocabulário da partitura de bateria; ver [insight](insights/2026-11-22-faixas-de-hz-da-bateria.md).*
3. **Auto-Maestro:** Sugestão de dinâmica e formação baseada no áudio. *Curva de LUFS do áudio inteiro entregue; por instrumento depende dos stems.*
4. **Resolução de Links:** Link de streaming $\rightarrow$ Identidade da Obra $\rightarrow$ Cifra. *Fora do repositório: o áudio é obtido fora do app, por decisão do dono.*

---

## 🌍 R5 · Rede Global e Lojas
**Objetivo:** Distribuição pública, indexação global e monetização.
**Portão de Entrada:** Spike S6 (Backend do Índice Global).

### Etapas:
1. **Índice Harmônico:** Busca global de mapas harmônicos via ID canônico.
2. **Lojas:** Publicação oficial na App Store, Google Play e F-Droid.
3. **Backup:** Sincronização opcional com Drive/OneDrive.
4. **L10n Expandida:** Faixas 2 (regionais) e 3 (comunidade).

---

## 🛠️ Matriz de Validação Global
Para cada etapa, aplica-se o seguinte rigor:
- **Funcional:** Testes unitários e de integração cobrindo 100% dos casos de uso.
- **Regressivo:** Snapshots de UI (Golden Tests) e tabelas de referência musical.
- **Exploratório:** Testes de estresse, limites de memória e casos de borda.
- **Acessibilidade:** Contraste WCAG, alvos de toque $\ge 48\text{px}$ e Screen Readers.
- **E2E:** Fluxos completos do usuário em dispositivos de referência.

**Execução:** o pipeline local (`./scripts/ci-local.sh`) roda a matriz antes de cada push; o Actions cobre PRs em ambiente limpo e o agendamento semanal detecta *drift* de dependências (ADR-0004).

