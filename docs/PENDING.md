# Pendências · Kentron Music Hub

## 🚩 R0 (publicado, com ressalvas)
- [x] `flutter analyze` de `packages/core_l10n` no pipeline local (cópia temporária, `--no-fatal-infos`).
- [x] Versão do Flutter fixada em `.flutter-version` (3.50.1), verificada pelo script e espelhada no workflow.
- [x] `scripts/ci-local.sh` e `.github/workflows/ci.yml` sincronizados: o workflow deixou de repetir a matriz e passou a chamar o script (fonte única). Cobertura preservada — Rust (fmt, clippy, test), `cargo-deny`, ARBs, Flutter do app e `core_l10n`.
- [ ] Migrar `core_l10n` de `flutter_localizations` para `material_ui`: **revisado em 2026-11-20** — a depreciação remanescente está no arquivo **gerado** (`gen-l10n` emite as `Global*` do `flutter_localizations`), e o próprio `material_ui` importa `GlobalWidgetsLocalizations` de lá. Não é resolvível editando o `pubspec` do pacote; depende do gerador. O app já usa as `Global*` do `material_ui`, então a depreciação é cosmética hoje.
- [x] Integrar `core_l10n` ao app e criar `lib/core_l10n.dart`: o app monta o `MaterialApp` com `CoreL10n.delegate` + `...GlobalMaterialLocalizations.delegates`, resolve locale por `CoreL10n.supportedLocales` e a saudação da tela inicial vem do ARB (`homeWelcome`).
- [x] Substituir o placeholder da ponte Rust ↔ Flutter por chamada real: entregue na R1 (crate adaptador `kentron_ffi`, provado por teste de Flutter que carrega o cdylib).
- [ ] Configurar proteção da branch `main` no GitHub.
- [ ] Registro final na verificação de desenvolvedor do Android.
- [ ] Adicionar verificação de licenças para dependências Dart/Flutter no pipeline local (`cargo-deny` cobre apenas Rust).
- [ ] Teste de fumaça do app Flutter em dispositivo real (Android/iOS).
- [ ] Revisar traduções de es-419, pt-BR e en-US com falantes nativos.
- [x] Publicar as duas lições no `sovereign-harness`: a skill `estado-compartilhado-multi-agente` §1c (PR #21) e a correção do gate de lint (`rustfmt --edition` lido do manifesto, PR #22) saíram na **release 0.7.2**.

## 🎸 R1 (prioridades imediatas)

Parser ChordPro, transposição e testes dourados já integrados à `main`; o restante da R1 segue em branch por unidade de trabalho — a branch de escopo `feature/song-bundle-v1` foi apagada em 2026-11-20 por ser idêntica à `main`.
- [ ] **Spike S5:** validar renderização de partituras/tabs via alphaTab/OSMD. Dimensiona o esforço de MusicXML/ATEX (R1.1); não bloqueia o piloto da R1.
- [x] **Ponte Rust ↔ Flutter (ADR-0007 aceita e emendada):** crate adaptador `crates/kentron_ffi` (único `cdylib`) expõe `parse_song` ao Dart via `flutter_rust_bridge` 2.13.0; bindings gerados e **provados** por `apps/kentron_app/test/bridge_test.dart`, que carrega o cdylib e chama o núcleo.
- [ ] Documentar no README o pré-requisito de **regerar** os bindings quando `bridge.rs` mudar (`cargo install flutter_rust_bridge_codegen --version 2.13.0` e o comando do `flutter_rust_bridge.yaml`). Os arquivos gerados vão versionados, então o dia a dia não precisa da ferramenta.
- [x] **Alvo web:** `flutter create --platforms=web` (o `pubspec` saiu intacto), wasm da ponte via `flutter_rust_bridge_codegen build-web` e `scripts/servir-web.py` servindo com COOP/COEP + `application/wasm`. Build e artefatos verificados nos dois sentidos; `web/pkg/` é artefato (ignorado).
- [x] **Tela inicial ligada à ponte:** `HomeScreen` chama `parseSong` e mostra título, artista/tom/seções, os acordes e **qual núcleo respondeu** (`núcleo: kentron_ffi …`); o carregador é injetável para os testes herméticos. Provado com o **cdylib real** em `bridge_test.dart` — era a única prova que faltava do caminho Dart.
- [x] **Prova no navegador, feita:** a tela carregou no browser com os dados do núcleo — `Exemplo`, `Kentron · tom C · 2 seção(ões)`, acordes `C G Am F` e **`núcleo: kentron_ffi 0.0.0`**. O wasm do Rust responde no navegador; a cadeia Rust → wasm → Flutter web → tela está provada (verificado visualmente pelo dono em 2026-11-20).
- [x] **Higiene de toolchains duplicadas:** resolvido por remoção, não por escolha de PATH. O cask `flutter` saiu da máquina (`/opt/homebrew/bin/flutter` e `/opt/homebrew/share/flutter` não existem) e ficou **um** Flutter: `~/development/flutter` (3.50.1, o que o `.flutter-version` pina e o pipeline usa). O cask `android-platform-tools` também saiu: existe **um** `adb`, em `~/Library/Android/sdk/platform-tools/adb`. Permanece o cask `android-commandlinetools` apenas como fornecedor do `sdkmanager` (link em `/opt/homebrew/bin/sdkmanager`), o que não conflita com o `adb`.
- [x] **Android no app:** `flutter create --platforms=android --org com.kentron` (appId `com.kentron.kentron_app`; `pubspec` saiu idêntico) + `rust_builder` do cargokit via `flutter_rust_bridge_codegen integrate --rust-crate-dir ../../crates/kentron_ffi --no-write-lib`. O APK sai com `libkentron_ffi.so` em três ABIs e o app **roda no emulador** mostrando os dados do núcleo.
- [x] **iOS no app:** `flutter create --platforms=ios --org com.kentron` (bundle id `com.kentron.kentronApp`; `pubspec` idêntico) + `staticlib` no `crate-type` do crate (era o que faltava para o cargokit usar o `.a` em vez de exigir um Framework). Provado: build do simulador verde e o app rodando com os dados do núcleo.
- [x] **Empacotamento do Rust — SPM/CocoaPods (ADR-0008 aceita):** o Rust deixou de entrar como **plugin** (cargokit, CocoaPods-only) e passou a entrar como **code asset** por build hooks. Sem plugin não existe incompatibilidade de SPM e o plugin do Gradle também sai do caminho. **Medido:** iOS builda e **roda** (`núcleo: kentron_ffi 0.0.0` no print), Android builda e roda, `flutter test` 8/8, **sem CocoaPods e sem o aviso de SPM** (`grep -c` = 0). O `ios/` regenerado ficou **sem `Podfile` e sem `Pods/`**. Predicado técnico: `rust-toolchain.toml` **pinado e dentro do crate** (o `native_toolchain_rust` não sobe a árvore de diretórios).
- [x] **Higiene de disco — runtime único:** removido o `24B91016s` (seed, 7,7 GB), mantido o `24B8117`, onde está o iPhone 17 das provas; `xcrun simctl list runtimes` lista um só. Restam **25 GB livres** (o Xcode e os builds do iOS comem bastante): `~/Library/Developer/Xcode/DerivedData` e `apps/kentron_app/build/` são os candidatos naturais de limpeza.
- [x] **JDK para Android: 21, não 27.** O Gradle 9.5 reprova com Java 27 (`Unsupported class file major version 71`); o projeto aponta `flutter config --jdk-dir /opt/homebrew/opt/openjdk@21/...` (formula, instala sem sudo). Anotado para quem reinstalar a máquina.
- [ ] Adicionar fixtures de parser para acordes invertidos, `6/9`, acidentes duplos e letras multilíngues (a nomeação já os cobre; falta provar a preservação na leitura do `.cho`).
- [x] Transposição no `chord_naming` e módulo `harmony` integrados à `main` (29 testes; acordes invertidos, `6/9` e acidentes duplos cobertos).
- [x] **Persistência local do Song Bundle (ADR-0006 aceita):** bundle em diretório (`meta.json` + `song.cho`), índice da biblioteca, setlists, `schema_version` com cadeia de migração e exportação por cópia do diretório. `serde`/`serde_json` é a primeira dependência externa do workspace e passa no `cargo deny`.
- [ ] Empacotar `score.musicxml` e `tabs/*.atex` no bundle: a ADR-0006 já os prevê, mas o v1 escreve só o `.cho`.
- [ ] Contêiner de arquivo único (zip) para exportação: v1.1 da ADR-0006, se o piloto exigir compartilhar por arquivo.
- [x] **UI de leitor de cifras (Modo Palco):** `SongReaderPage` desenha a cifra inteira com acorde sobre a letra, transposta pelo tom, e alterna contraste; alcançável pela tela inicial. Provada por 5 testes herméticos e 3 provas reais na ponte (`readSong`, `transposeSong`, `keyChoices`).
- [x] **Modo projeção manual em tela cheia:** `ProjectionPage` projeta uma seção por vez, sem acorde, com avanço por toque, setas do teclado ou rodapé; aberta pelo botão "Projetar" do leitor. Provada por 5 testes herméticos.
- [ ] Projeção em tela secundária (ou janela do OBS): o modo em tela cheia é o conteúdo; falta o transporte para a segunda tela.
- [ ] Setlists utilizáveis no Modo Palco (navegar de uma música para a próxima sem voltar à biblioteca).
- [ ] Implementar exportação OpenLyrics/Holyrics e validar XML.
- [x] **Bootstrap de projeção por URL:** `?projecao=<base64url do ChordPro>` abre o app direto na projeção; o leitor oferece "Copiar link de projeção" no alvo web. Sem `obs-websocket`. Provado por 5 testes de URL + 2 de bootstrap e pela compilação do build web.
- [x] **Prova de navegador dos entregues (2026-11-21):** Playwright + Chromium headless sobre o build web — home, leitor, transposição, projeção, avanço/volta de seção, contraste, bootstrap por URL e cópia do link, com screenshots.
- [ ] **Piloto automatizado (R1-D, ADR-0010):** roteiro na VM Windows + OBS carregando a projeção por Browser Source — abrir, transpor, projetar, navegar, fechar/reabrir — e duas rodadas consecutivas simuladas. Cobre a prova no OBS.
- [x] **`pt` é pt-BR por gate (2026-11-21):** `tools/check_arb.py` reprova grafia europeia em `app_pt.arb` (`ecrã`, `secção`, `ficheiro`, `utilizador`, `telemóvel`, `autocarro`, `connosco`, `receção`); a variante europeia vive em `pt_PT`. Provado nos dois sentidos.
- [x] **Dicionário de variantes (2026-11-21):** `tools/variantes.tsv` + duas regras no gate `arb` — bloqueia forma da variante no arquivo base e **avisa** quando a variante não sobrescreve uma chave. Achou sozinho que `pt_PT` não cobria `stageProjectNext`/`stageProjectPrevious` (corrigido: "secção"). O `pre-push` chama o gate direto para o aviso aparecer quando passa.
- [ ] **Erro de console `wasm_bindgen` declarado duas vezes:** o `index.html` carrega `pkg/kentron_ffi.js` e o carregador do frb injeta o mesmo glue outra vez. O app funciona (a ponte respondeu na prova de navegador), mas a dupla carga precisa ser resolvida — registrado no `scripts/e2e-web.sh` como aviso, não como falha.
- [ ] **Alpha assistido (R1-C, ADR-0009):** um ensaio com o fluxo completo — leitor, projeção e transposição —, falhas registradas com diagnóstico e dados sobrevivendo a fechar/reabrir o app. É o portão inicial e não depende de calendário de culto.
- [ ] Executar o piloto de dois cultos e fechar a R1 (R1-D, portão final, depois do R1.1 — ADR-0009).

- [x] **Lições da sessão viram skills (2026-11-20):** `empacotamento-rust-flutter`, `higiene-de-dispositivos`, `lote-nao-e-sequencia` e `toolchain-unica` criadas em `.agents/skills/` e registradas na tabela do `/AGENTS.md`; `scripts/encerrar-dispositivos.sh` é o mecanismo da higiene de dispositivos. Branches mergeadas removidas com autorização do dono do projeto.
- [x] **Perfil do projeto e shellcheck (2026-11-20):** `/AGENTS.md` sem placeholders (topologia, fronteiras, branch/remote, stack, comandos de validação, idiomas, aprovação e release) e `shellcheck` como gate em `scripts/ci-local.sh` e `.githooks/pre-push`; provado no sentido negativo com um `.sh` defeituoso.
- [x] **Contrato de validação no `.harness.toml` (2026-11-21):** `[validation]` passou a `configured = true` com um gate explícito, o **gitleaks** (mesmo argv do `gate_gitleaks` do pre-push). Decisão do dono do projeto: a bateria completa **não** entra no contrato — ela já é aplicada pelo `.githooks/pre-push` em cada push, e repeti-la dobraria o custo sem acrescentar cobertura. `python3 .harness/validation.py check` roda e aprova.
- [x] **`origin/HEAD` para `main` (2026-11-21):** o GitHub já tinha `main` como default (`defaultBranchRef = main`); o desatualizado era o ref **local** `refs/remotes/origin/HEAD`, que apontava para `master`. Corrigido com `git remote set-head origin -a`. `master` no remoto é branch órfã (1 commit, raiz própria) e fica como decisão separada.

## 🧠 R4 · Primeiro incremento entregue (instrumento de trabalho, ADR-0011)

DSP puro, sem modelo: dinâmica (EBU R128), andamento, acordes e estrutura. O que falta:
- [ ] **Letra por ASR** (transcrição de canto): exige peso declarado no `models/MANIFEST.toml` (licença, fonte, hash, ADR) e correção humana no fluxo — o resultado de ASR de canto não é confiável sozinho. Declarado e **não implementado** na ADR-0012.
- [ ] **Fichas de hinários brasileiros (Harpa Cristã, Cantor Cristão, folclóricas):** a ferramenta está pronta — `--ficha` calcula o veredito por jurisdição e `--auditar-corpus` confere —, mas **entrar exige a ficha do tradutor**, porque hinário antigo reúne traduções que podem ter autor recente. Trabalho de conferência, hino a hino, não de heurística.
- [ ] **Extração de letra do áudio (peso):** o enquadramento está decidido e implementado (`ExtraidaPeloUsuario`, escopo privado do grupo, `pode_publicar` reprovando). Implementar a extração exige modelo no `models/MANIFEST.toml` com licença, fonte e hash.
- [ ] **Tela de agenda no app:** o modelo (`Setlist` = evento, com data, tipo, versão e tom), a persistência v2 com migração e o relatório de uso existem e são provados. Falta a interface para montar a agenda do culto — inclusive o atalho de **repetir a agenda anterior** com versões novas.
- [ ] **O app honrar `lyrics_scope`:** hoje quem reprova publicação é o motor (`pode_publicar`). O app precisa consultar isso na exportação e no **link de projeção** — senão o portão existe só no CLI.
- [ ] **Formato do campo de licença:** o dono não tem formato próprio ainda, e o projeto **não inventa** um (ADR-0012/ADR-0006). O que já funciona: `--licenca` como texto livre na origem, e o relatório dizendo o que é domínio público por ficha e o que está **sem ficha**.
- [ ] **Fichas ditadas pelo relatório:** rodar `--relatorio-uso` no repertório real e conferir as obras **sem ficha**, em ordem de uso, criando as fichas das que mais tocam. É a lista de trabalho finita que substitui a enumeração dos hinários.
- [ ] **Afinar o detector de frase vocal no repertório:** hoje calibrado em **dois** casos (um áudio sintetizado e uma música real: 4 frases exatas e 63 frases cobrindo 302 s de 400 s). Falta medir em 10–20 músicas e registrar a taxa de linha no lugar certo.
- [ ] **Gabarito humano como referência de acurácia (decisão do dono):** existe uma cifra feita à mão do mesmo áudio, que serve de gabarito. Automatizar a comparação exige guardar a **harmonia de referência** no repositório — conteúdo de terceiro, e por isso é decisão do dono. Hoje a comparação foi medida uma vez, à mão, com os números no CHANGELOG/CONCLUSION.
- [ ] **Terça: limiar de 0,4 medido em UMA música.** Vale reconferir no repertório; a regra está isolada em `chords::com_terca` para isso.
- [ ] **`Am` lido como A maior** no trecho modulado (a terça menor do A não foi resolvida). O mesmo vale para qualquer menor cuja terça fique mascarada — sem separar instrumentos, não há como recuperá-la.
- [ ] **Inversões (`E/G#`, `D/F#`, `B/D#`) não existem no motor:** ele detecta conjunto de classes de altura, não a nota do baixo. Estrutural; exige baixo isolado (stems).
- [ ] **Notação de tensão (`2`, `4`, `7M`)**: o motor lê `add` como `sus4`/`maj7`/`5`. Sem stems, distinguir a nota acrescentada da terça é adivinhação.
- [ ] **Entrada de letra na interface:** a CLI tem `--letra`/`--hino`/`--listar-hinos`; o app ainda não oferece colar/escolher a letra, nem exportar a cifra com letra pelo bundle.
- [ ] **Centro tonal de verdade**: hoje `tom_estimado` devolve a fundamental mais frequente — numa música em E maior devolveu `B`, e na fixture em Dó devolveu `Am`. Falta perfil de tonalidade (Krumhansl) ou HMM de tom.
- [ ] **Tablatura de solo**: pitch tracking + casamento de corda/traste. Ambíguo por natureza (a mesma altura tem várias posições); exige decisão de afinação e de digitação.
- [ ] **Separação em stems** (htdemucs 6s, MIT): melhora acorde e dinâmica por instrumento e é pré-requisito realista da tablatura. Portão: spike S1 (MIR on-device) + entrada no `MANIFEST`.
- [ ] **Dinâmica por instrumento**: a curva de pressão sonora (LUFS por stem) que o plano v2.x previa — depende dos stems.
- [ ] **Integração com o app**: o relatório e o ChordPro do `mir_engine` ainda não entram no bundle nem no leitor/projeção; hoje só a CLI consome.
- [ ] **Pico verdadeiro** (true peak) em vez de pico de amostra: exige sobreamostragem com filtro polifásico.
- [ ] Decidir entre R128 à mão (atual, contraprovado contra o `ffmpeg`) e o crate `ebur128` citado no plano.
- [ ] Prova em lote: rodar o motor em N músicas do repertório e medir onde acerta e onde erra (o incremento provou em **uma** música).
- [ ] Aquisição de áudio a partir de link não faz parte do repositório: não há `yt-dlp` nem downloader versionado — a decisão de como obter o áudio é do dono.

## 🌐 R2+ (backlog)
- [ ] Spike S4 (Matrix/Push iOS).
- [ ] Spike S2 (sincronia de tempo).
- [ ] Spike S1 (MIR on-device).
- [ ] Spike S6 (índice global).

