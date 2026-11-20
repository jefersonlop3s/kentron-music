# Conclusões · Kentron Music Hub

## R0 · Fundação publicada (2026-11-19)

**Status:** 🟢 Publicado na `main`; validação local e pipeline local concluídos.

### Evidências
- Commits: `7f68e2b` (R0), `b149bb3` (endurecimento de CI/app), `d21d1fe` (registros vivos alinhados ao CI verde e ao `material_ui`).
- `main` publicada em `origin`; branch padrão do GitHub alterada para `main`.
- 15 testes Rust passando; `cargo clippy`, `cargo fmt`, `cargo deny` e `check_arb.py` verdes.
- Semgrep sem achados no código do projeto após exclusão dos scripts vendorizados do Impeccable.
- Pipeline não disparado no primeiro push por `[skip ci]`.
- CI do GitHub verde no run `44665120771`: Rust, licenças, i18n e Flutter passaram.
- App migrado para `material_ui` 1.11.1, eliminando a depreciação de `package:flutter/material.dart`.
- Validação local com Flutter 3.50.1 (stable): `flutter analyze` sem achados, `flutter test` com o teste de fumaça passando e `flutter gen-l10n` de `core_l10n` gerando os arquivos sem erro.

### Entregáveis
- Identidade **Kentron Music Hub** e licença Apache-2.0 com aviso de uso.
- Sovereign Harness e Impeccable UI documentados.
- Monorepo Melos, scaffold Flutter e CI com ações fixadas por SHA.
- `notation_bundle` com `chord_naming` e parser ChordPro.

### Ressalvas
- Flutter 3.50.1 instalado em `~/development/flutter`; a validação local passou, mas o SDK ainda não está no `PATH` padrão do ambiente.
- `core_l10n` ainda não está integrado ao app.
- A ponte Rust ↔ Flutter é um placeholder.
- Pipeline local como portão principal (ADR-0004): `scripts/ci-local.sh` verde de ponta a ponta.

## R1 · Parser ChordPro (parcial integrado, 2026-11-20)

**Status:** 🟢 Parser integrado à `main`; o restante da R1 segue em branch própria.

- Parser ChordPro e testes dourados implementados (`b42421a`).
- Correções de lints e remoção do módulo `harmony` incompleto (`4beb085`).
- Integração na `main` pelo merge `50c7323`, com `analysis_options.yaml` ajustado no mesmo ciclo.
- Validação do resultado integrado: 15 testes Rust, Clippy, formatação, `check_arb.py`, `cargo deny`, `flutter analyze` e `flutter test` verdes.
- MusicXML permanece como placeholder.
- Persistência, UI de palco e projeção ainda não existem.

## R1 · Transposição e harmonia (2026-11-20)

**Status:** 🟢 Integrada à `main`; o restante da R1 segue em branch própria.

- `Chord::transpose` implementado em `chord_naming.rs`, com aritmética de letra e acidente.
- Módulo `harmony.rs` reintroduzido: transposição de música inteira e renderização nos quatro sistemas.
- Testes: 29 no workspace, incluindo acordes invertidos, `6/9`, acidentes duplos e acordes desconhecidos.
- Testes dourados novos: `golden_harmony/worship.cho` e saídas em Common e Latin.
- Escopo da R1 formalizado como proposta na ADR-0005 (ChordPro primeiro; MusicXML/ATEX em R1.1) — **rejeitada** em 2026-11-20; ver seção abaixo.

**Próxima Fase:** concluir R1-A, R1-B e R1-C conforme o plano v2.6, em nova branch a partir da `main`.

## R1 · Escopo da R1: ADR-0005 rejeitada (2026-11-20)

**Status:** 🟢 Decisão do dono registrada; ADR-0002 integral.

- A ADR-0005 foi **rejeitada**: a redução de escopo tratava uma ordem de execução como corte de escopo de uma ADR já aceita.
- A cláusula "o piloto público aguarda o S5" era inferência da proposta, não da ADR-0002: o portão do piloto continua sendo o uso real por uma igreja, offline, em pt-BR ou es-419.
- O escopo de formatos da ADR-0002 permanece integral (`.cho`, `.musicxml`, `.atex`); a sequência cifra → MusicXML/ATEX continua como ordem de execução no marco R1.1.
- Limpeza de branches: `feature/R1-restante`, `feature/R1-transposicao-harmonia` e `wip/harmony-transpose` apagadas — as duas primeiras eram ancestrais da `main` (0 commits exclusivos) e a terceira não tinha arquivo exclusivo. Worktree `de6b0` re-apontado para `feature/song-bundle-v1` a partir da `main`.
- Próximo passo: Song Bundle v1 (persistência local, migrações e exportação/restauração).

## R1 · Song Bundle v1: modelo em memória (2026-11-20)

**Status:** 🟢 Modelo em memória entregue; persistência aguarda a ADR-0006.

- Novo módulo `crates/notation_bundle/src/bundle.rs`: `SongId`, `SetlistId`, `SongBundle`, `Library`, `Setlist` e `BundleError` com `message_key()` estável para a UI.
- `SongBundle` não duplica bibliografia: título, tom e artista vêm do `metadata` do `.cho`, que segue como fonte única.
- Invariantes provadas: id vazio recusado, título ausente recusado, duplicata de id recusada na biblioteca, setlist só aceita música da biblioteca, entrada duplicada recusada, resolução de setlist falha alto quando a música sumiu.
- 14 testes novos (11 unitários no módulo, 3 de integração em `tests/bundle_tests.rs` com o fixture `worship.cho`); workspace passa de 29 para 43 testes.
- Decisão do dono: o formato de persistência é decidido **depois** do modelo — a ADR-0006 fica proposta e adiada, sem dependência externa adicionada.

## R1 · Song Bundle v1: persistência (ADR-0006 aceita, 2026-11-20)

**Status:** 🟢 Persistência entregue na `main`; resta empacotar `score.musicxml`/`tabs/*.atex` e o contêiner único (v1.1).

- ADR-0006 **aceita** pelo dono, com o modelo em memória já entregue na ordem que ele pediu.
- `bundle_store`: bundle é diretório com `meta.json` (`schema_version`, `id`, `updated_at`, inventário de arquivos) e `song.cho`; `library.json` indexa a biblioteca e cada setlist tem o seu `*.json`.
- `chordpro::write` fecha o ciclo do parser: metadados em ordem determinística, acordes recolocados na posição, ida e volta preservando o modelo.
- Migração por degrau com a cadeia vazia (v1 é o primeiro esquema) — versão sem degrau falha alto, provado nos dois sentidos (0 e 2 recusados, 1 aceito).
- `serde`/`serde_json` entram como primeira dependência externa: **11 pacotes** medidos no `Cargo.lock`, todos aprovados por `cargo deny`.
- Todo valor volta ao modelo pelos construtores validados: um setlist com entrada órfã é recusado na leitura, não carregado com buraco.
- 55 testes no workspace (12 novos); `ci-local.sh` verde.

## R0 · Gate de lint alinhado à edição do crate (2026-11-20)

**Status:** 🟢 Corrigido e provado nos dois sentidos.

- O pre-push formatava com `rustfmt --edition 2021` fixo; o workspace é edition 2024 e o `ci-local.sh` usa `cargo fmt`. Medição: **mesmo arquivo e mesmo `rustfmt`** — 2024 limpo, 2021 com diff.
- A edição passa a vir do `Cargo.toml`; a prova negativa (arquivo mal formatado) continua reprovando, então o gate não foi afrouxado.
- Limite conhecido e registrado: membro do workspace que declare edição própria seria verificado com a do workspace (hoje nenhum declara).

## R0 · Lições do kentron publicadas no sovereign-harness 0.7.2 (2026-11-20)

**Status:** 🟢 Publicado — release [v0.7.2](https://github.com/personal-digital-sovereignty/sovereign-harness/releases/tag/v0.7.2).

- **PR [#21](https://github.com/personal-digital-sovereignty/sovereign-harness/pull/21)** (mergeado em `cb67018`): skill `estado-compartilhado-multi-agente` §1c — branch de escopo vira segundo trunk, com o caso medido aqui e os três comandos que separam trabalho de sombra.
- **PR [#22](https://github.com/personal-digital-sovereignty/sovereign-harness/pull/22)** (mergeado em `4066f5a`, que é a tag `v0.7.2`): o gate `lint` do `templates/git/pre-push` passa a ler a edição do `Cargo.toml` em vez de fixar `2021`; teste novo provado no negativo.
- Versões → 0.7.2 nos 7 manifestos do kit; `sincronizar --verificar` 45/45; CI verde em ubuntu e macos.
- O gate do kentron já tinha a mesma correção (`a68c85c`), então projeto e kit voltaram a concordar.
- Fechamento: os dois itens saíram do PENDING; restam no kit apenas as ressalvas já documentadas (canário Claude live pendente de login do dono).

## Manutenção · Runtime único e o caso do SPM (2026-11-20)

**Status:** 🟢 Um runtime de simulador só; o aviso de SPM fica como risco datado, com caminho definido e evidência registrada.

- Removido o runtime `24B91016s` (seed, 7,7 GB); ficou o `24B8117`, com o iPhone 17 das provas.
- Hipótese minha **derrubada por medição**: desligar SPM (`flutter config --no-enable-swift-package-manager`) **não** silencia o aviso — build verde e aviso presente. Config restaurada ao default, sem perder `android-sdk`/`jdk-dir`.
- O suporte a SPM **não existe** no frb 2.13.0 estável (sem opção no `integrate`, sem `Package.swift` gerado) e o cargokit é CocoaPods-only; existe **upstream** (issues #3036 e #3202 fechadas). Quando houver estável com SPM, subir o gerador resolve de vez.

## R1 · iOS: o Rust no simulador (2026-11-20)

**Status:** 🟢 O app roda no simulador iOS com o núcleo Rust. Três plataformas provadas: teste (cdylib), navegador (wasm) e Android/iOS.

- `flutter create --platforms=ios --org com.kentron` → bundle id `com.kentron.kentronApp`, `pubspec` **idêntico**.
- **A correção estava no código do cargokit**, não em tentativa: ele usa o `.a` quando existe e só exige Framework quando não existe. Como o crate declarava apenas `cdylib`, o iOS não gerava `.a` → `Unable to find bundle for dynamic library`. Basta **`staticlib` no `crate-type`** (Android segue no `cdylib`).
- **Prova:** build do simulador verde (Xcode 27.2), app instalado e aberto (PID 53043) e o print — que eu li e conferi — mostra `Exemplo`, `tom C`, os acordes e `núcleo: kentron_ffi 0.0.0`. Regressão do Android revalidada depois da mudança de `crate-type`.
- **Risco anotado:** o Flutter avisa que o plugin `kentron_ffi` não suporta **Swift Package Manager** e que isso *vai virar erro* numa versão futura (o cargokit é CocoaPods-only).
- Higiene de disco: ficaram **dois** runtimes iOS 27.2 instalados (~7 GB cada); remover um quando não precisar.

## R1 · Android: o Rust no APK (2026-11-20)

**Status:** 🟢 O app roda no emulador Android com o núcleo Rust; falta iOS (depende do Xcode, que só o dono instala).

- `flutter create --platforms=android --org com.kentron` gerou o `android/` com appId de verdade (`com.kentron.kentron_app`) e **sem tocar no `pubspec`**.
- O `integrate` criou o `rust_builder` (cargokit) apontando para o nosso crate fora do app: `manifestDir = "../../../../crates/kentron_ffi"` — e o `--no-write-lib` protegeu o `main.dart`, porque por padrão ele escreve código de exemplo.
- **Prova:** `flutter build apk --debug` verde, com `libkentron_ffi.so` dentro do APK em três ABIs; no emulador o app abre (`COLD`, 1,7 s), o processo vive e o print mostra `Exemplo`, `tom C`, os acordes e `núcleo: kentron_ffi 0.0.0`.
- **Java 27 não serve**: o Gradle 9.5 quebra com `Unsupported class file major version 71`. Ficou JDK 21 (`openjdk@21`, formula) + `flutter config --jdk-dir` — o Flutter já avisava que 27 era mais novo do que ele conhece.
- Armadilha de sequência: o `integrate` precisa de `flutter` no PATH; sem ele, cria metade e falha no `pub add`, deixando o `rust_builder` sem dependência no `pubspec`.
- Auto-instalação de ferramenta de novo: o cargokit baixou o alvo Rust `i686-linux-android` sozinho (quatro ABIs, não três).

## R1 · Android local pronto e ponte provada no navegador (2026-11-20)

**Status:** 🟢 Toolchain Android completa e o app web provado no navegador; falta o app abrir no emulador (depende dos diretórios de plataforma) e o Xcode para iOS.

- **O navegador mostrou o núcleo Rust**: `Exemplo`, `Kentron · tom C · 2 seção(ões)`, acordes `C G Am F`, `núcleo: kentron_ffi 0.0.0`. Era a última prova pendente do caminho web.
- **Android medido e funcional**: SDK 36 completo (build-tools 36.0.0, emulator 37.3.10, **NDK 28.2.13676358**, platforms 36, imagem arm64-v8a), Java Temurin 27, AVD `kentron`, `flutter doctor` ✓.
- Armadilha que custou investigação: o `avdmanager` do Homebrew resolve o SDK root para o cask e não enxerga os system-images; symlink não basta — o SDK root precisa de uma **cópia real** de `cmdline-tools`, que também o deixa imune a `brew upgrade`.
- Higiene em aberto: dois Flutter e dois `adb` na máquina (ver PENDING) — mesma versão hoje, mas o PATH decide; escolher um.

## R1 · Tela inicial consumindo o núcleo Rust (2026-11-20)

**Status:** 🟢 O caminho Dart está provado com o `cdylib` real; falta ver no navegador.

- A `HomeScreen` chama `parseSong` e mostra o que voltou do Rust — inclusive `núcleo: kentron_ffi …`, que identifica quem respondeu.
- O carregador é injetável: os testes de widget ficam **herméticos** (sem FFI) e um teste no `bridge_test.dart` prova o caminho completo com o cdylib de verdade. 8/8 verdes, analyze limpo.
- Armadilha registrada: com a ponte real, `pumpAndSettle` estoura — o indicador anima para sempre e a resposta chega por porta do FFI; a espera é em tempo real (`runAsync`).
- Achado: o parser extrai acordes de linhas só com acordes (sem letra), provado por esse teste.
- Aberto: a renderização no navegador (o Flutter web desenha em canvas — asserção de texto exigiria semântica habilitada).

## R1 · Alvo web (2026-11-20)

**Status:** 🟢 Build e serviço verificados; a tela ainda não consome a ponte, então a execução no navegador **não** está provada.

- `flutter create --platforms=web` deixou o `pubspec` intacto (comentários preservados) e só acrescentou `web/**` aos excludes do analyzer.
- A ponte compila para wasm: `kentron_ffi_bg.wasm` (104 KB) + `kentron_ffi.js` (34 KB, alvo `no-modules`), com nightly + `rust-src`, porque o frb usa `-Z build-std` para atomics/shared memory.
- `scripts/servir-web.py` serve o `build/web` com COOP/COEP e `application/wasm` — provado nos dois sentidos (200 nos artefatos; exit=1 quando o build não existe).
- Achado de caminho a manter divulgado: o gerador **auto-instala** ferramentas quando faltam (`cargo-expand`, `wasm-pack`).
- O que falta para "ver o núcleo rodando": ligar a tela a `parseSong` e provar num navegador — a CT `playwright` tem Playwright 1.63 e alcança o app servido aqui via Tailscale.

## R1 · Ponte Rust ↔ Flutter entregue (ADR-0007 aceita e emendada, 2026-11-20)

**Status:** 🟢 A chamada real existe e está provada; falta o app abrir em dispositivo (diretórios de plataforma).

- `crates/kentron_ffi` é o **adaptador**: único `cdylib` e único a depender do `flutter_rust_bridge` 2.13.0. Expõe `parse_song(input)`, devolvendo título, tom, artista, número de seções, acordes distintos e qual núcleo respondeu.
- O domínio `notation_bundle` volta a ser puro e mantém `unsafe_code = "forbid"`.
- **Emenda medida à ADR:** a cláusula 4 ("`forbid` permanece") não sobrevivia — a cola gerada tem um bloco `unsafe`, e `forbid` não aceita `allow`. O adaptador usa `deny` com o allow escopado ao módulo gerado; o invariante que o dono quis preservar segue de pé onde o código é nosso, e a exceção é auditável.
- **Prova de ponta a ponta:** `apps/kentron_app/test/bridge_test.dart` carrega o `cdylib` e chama o Rust — `parse_song` devolve o resumo real e cifra vazia vira exceção. 58 testes Rust (3 do adaptador) + 5 de Flutter.
- Pipeline e pre-push constroem o `cdylib` antes dos passos do app. Regerar bindings exige `flutter_rust_bridge_codegen` 2.13.0 **e** um `cargo fmt --all` depois (o gerador não segue a edição 2024 do crate) — documentado no `flutter_rust_bridge.yaml`.
- Ferramenta instalada nesta máquina por `cargo install`; os arquivos gerados vão **versionados**, então o dia a dia não precisa dela.
- Ficou aberto: diretórios de plataforma (`flutter create --platforms=...`) para o app rodar fora do teste.

## R0 · core_l10n integrado ao app (2026-11-20)

**Status:** 🟢 Entregue na `main`.

- `packages/core_l10n/lib/core_l10n.dart` é o caminho único de import das localizações: o app não conhece o caminho do arquivo gerado.
- O app monta `MaterialApp(locale:, localizationsDelegates: [CoreL10n.delegate, ...GlobalMaterialLocalizations.delegates], supportedLocales: CoreL10n.supportedLocales)`, e a saudação da tela inicial vem do ARB `homeWelcome` (en/pt/es) — nenhuma string de interface mora no app.
- **Achado que mudou a leitura de um item do PENDING:** as `Global*` do `flutter_localizations` que o `gen-l10n` emite **não** satisfazem o `MaterialLocalizations` do `material_ui` — o app reprovava com "No MaterialLocalizations found". A migração pedida não é resolvível editando o `pubspec` do pacote: depende do gerador.
- Pipeline: a geração passou a ser no lugar e **antes** dos passos do app (que precisam do arquivo gerado); o gate de Flutter do pre-push também gera, senão o push reprovaria por arquivo que o pipeline encontra pronto.
- Provas: `flutter analyze` limpo no app e no pacote (pacote com `--no-fatal-infos`); 3 testes de widget verdes (en, pt, es e o fallback `pt_PT` → `pt`); paridade de ARBs ok com a chave nova; `ci-local.sh` verde.

## R0 · Matriz do CI com fonte única (2026-11-20)

**Status:** 🟢 Entregue na `main`.

- `.github/workflows/ci.yml` passou a instalar as ferramentas e chamar `./scripts/ci-local.sh`: a matriz existe num lugar só. Antes eram quatro jobs repetindo o script, com risco de divergir em silêncio — ressalva que a própria entrada da ADR-0004 registrava.
- Cobertura conferida antes de trocar: Rust (`fmt --check`, `clippy -D warnings`, testes), `cargo deny`, paridade de ARBs, Flutter do app (`analyze`, `test`) e `core_l10n` (`gen-l10n` + `analyze`) — todas dentro do script, nenhuma a menos.
- Gatilhos e pinagem por SHA preservados; nenhuma ação nova foi introduzida (as quatro usadas já estavam fixadas por SHA no arquivo).
- Provas: YAML validado (1 job, 6 steps, três gatilhos) e `./scripts/ci-local.sh` verde de ponta a ponta (`exit 0`).

## R0 · Pipeline local-first (ADR-0004, 2026-11-20)

**Status:** 🟢 Decisão aceita e implementada.

### Evidências
- ADR-0004 aceita: pipeline local como portão principal; Actions como rede de segurança.
- `scripts/ci-local.sh` criado e executado com sucesso: `cargo fmt --check`, `cargo clippy`, `cargo test`, `cargo deny`, `check_arb.py`, `flutter analyze`, `flutter test` e `gen-l10n` de `core_l10n`.
- `.githooks/pre-push` ampliado com gates transversais (`deny`, `arb`, `flutter`), garantindo que o push execute a matriz mesmo sem CI remoto.
- `.github/workflows/ci.yml` sem gatilho em `push` para `main`; mantém `pull_request`, `workflow_dispatch` e `schedule` semanal.

### Premissa corrigida
- O repositório é **público**, portanto o Actions é gratuito e ilimitado. A decisão **não** foi motivada por custo, e sim por soberania, redução de ruído e não duplicação de validação.

### Ressalvas
- A matriz está duplicada entre `scripts/ci-local.sh` e `ci.yml`; pode divergir.
- A versão do Flutter não está fixada em nenhum dos dois.
- `packages/core_l10n` ainda não tem `analyze`/`test` no pipeline local.

