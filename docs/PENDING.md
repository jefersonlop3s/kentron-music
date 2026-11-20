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
- [ ] **Higiene de toolchains duplicadas:** há **dois Flutter** (`~/development/flutter` — o que o pipeline clona e o `.flutter-version` pina — e `/opt/homebrew/share/flutter`, do cask) e **dois `adb`/platform-tools** (SDK e cask `android-platform-tools`). Nenhuma divergência de versão hoje (ambos 3.50.1), mas o PATH decide qual roda; escolher um de cada e remover o outro.
- [x] **Android no app:** `flutter create --platforms=android --org com.kentron` (appId `com.kentron.kentron_app`; `pubspec` saiu idêntico) + `rust_builder` do cargokit via `flutter_rust_bridge_codegen integrate --rust-crate-dir ../../crates/kentron_ffi --no-write-lib`. O APK sai com `libkentron_ffi.so` em três ABIs e o app **roda no emulador** mostrando os dados do núcleo.
- [x] **iOS no app:** `flutter create --platforms=ios --org com.kentron` (bundle id `com.kentron.kentronApp`; `pubspec` idêntico) + `staticlib` no `crate-type` do crate (era o que faltava para o cargokit usar o `.a` em vez de exigir um Framework). Provado: build do simulador verde e o app rodando com os dados do núcleo.
- [ ] **Swift Package Manager:** o Flutter avisa que o plugin `kentron_ffi` não suporta SPM e que isso *vai virar erro* numa versão futura (o cargokit é CocoaPods-only). Decidir quando acontecer: adotar SPM no plugin ou fixar o iOS em CocoaPods.
- [ ] **Higiene de disco:** há **dois** runtimes iOS 27.2 instalados (`24B8117` e `24B91016s`, ~7 GB cada). Remover um: `xcrun simctl runtime delete <id>`.
- [x] **JDK para Android: 21, não 27.** O Gradle 9.5 reprova com Java 27 (`Unsupported class file major version 71`); o projeto aponta `flutter config --jdk-dir /opt/homebrew/opt/openjdk@21/...` (formula, instala sem sudo). Anotado para quem reinstalar a máquina.
- [ ] Adicionar fixtures de parser para acordes invertidos, `6/9`, acidentes duplos e letras multilíngues (a nomeação já os cobre; falta provar a preservação na leitura do `.cho`).
- [x] Transposição no `chord_naming` e módulo `harmony` integrados à `main` (29 testes; acordes invertidos, `6/9` e acidentes duplos cobertos).
- [x] **Persistência local do Song Bundle (ADR-0006 aceita):** bundle em diretório (`meta.json` + `song.cho`), índice da biblioteca, setlists, `schema_version` com cadeia de migração e exportação por cópia do diretório. `serde`/`serde_json` é a primeira dependência externa do workspace e passa no `cargo deny`.
- [ ] Empacotar `score.musicxml` e `tabs/*.atex` no bundle: a ADR-0006 já os prevê, mas o v1 escreve só o `.cho`.
- [ ] Contêiner de arquivo único (zip) para exportação: v1.1 da ADR-0006, se o piloto exigir compartilhar por arquivo.
- [ ] Criar UI de leitor de cifras (Modo Palco) e setlists.
- [ ] Implementar exportação OpenLyrics/Holyrics e validar XML.
- [ ] Validar Browser Source para OBS sem `obs-websocket`.
- [ ] Executar piloto de dois cultos e fechar a R1.

## 🌐 R2+ (backlog)
- [ ] Spike S4 (Matrix/Push iOS).
- [ ] Spike S2 (sincronia de tempo).
- [ ] Spike S1 (MIR on-device).
- [ ] Spike S6 (índice global).

