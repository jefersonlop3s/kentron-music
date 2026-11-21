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


## R1 · Empacotamento do Rust por Native Assets (ADR-0008 aceita, 2026-11-20)

**Status:** 🟢 Aceito e provado nas duas plataformas; o app deixa de depender de CocoaPods e do plugin do Gradle.

### Evidências
- `flutter build ios --debug --simulator` → `✓ Built Runner.app` (23,9 s), com **0** ocorrências do aviso *"plugins do not support Swift Package Manager"* (antes aparecia em todo build de iOS).
- App instalado e aberto no simulador mostrando `Exemplo`, `tom C · 2 seção(ões)`, acordes `C G Am F` e `núcleo: kentron_ffi 0.0.0`.
- `flutter build apk --debug` → `✓ Built app-debug.apk` (32,7 s); app rodando no emulador.
- `flutter test` 8/8 e `scripts/ci-local.sh` `EXIT=0` após a migração.
- `ios/` sem `Podfile`, sem `Podfile.lock` e sem `Pods/` — o `pod install` sai do caminho do Rust.

### O que mudou no projeto
- `hook/build.dart` aponta para `../../crates/kentron_ffi`; `rust_builder/` (cargokit) removido.
- `rust-toolchain.toml` pinado em `1.97.1` com alvos, **duplicado dentro do crate** por exigência do `native_toolchain_rust`.
- `crate-type` mantém `["lib", "cdylib", "staticlib"]` — requisito do backend e o que já satisfazia o cargokit.

### Ressalvas
- O pin da toolchain transforma bump de Rust em decisão deliberada (perde-se o "pega o latest stable").
- O upstream testou este backend **só em Linux desktop**; iOS e Android foram provados aqui.
- O gerador `flutter_rust_bridge_codegen` 2.13.0 continua sendo pré-requisito para **regerar** bindings quando `bridge.rs` mudar; os arquivos gerados ficam versionados.

## Processo · Lições da sessão viram skills e um mecanismo (2026-11-20)

**Status:** 🟢 Integrado à `main`; as lições da sessão de Native Assets deixaram de viver só no relato e passaram a ser recuperáveis.

### O que foi criado
- **Quatro skills** em `.agents/skills/`, com gatilho concreto na tabela do `/AGENTS.md`:
  - `empacotamento-rust-flutter` — como o núcleo entra no app hoje (build hooks, ADR-0008), os predicados técnicos que quebram o build quando ausentes (`crate-type`, `rust-toolchain.toml` dentro do crate, JDK 21) e os erros com assinatura conhecida.
  - `higiene-de-dispositivos` — encerrar emulador/simulador depois da prova, verificando pelo estado do dispositivo em vez de padrão de processo.
  - `lote-nao-e-sequencia` — comandos agrupados executam em paralelo; dependência entre eles vira corrida.
  - `toolchain-unica` — duas instalações do mesmo binário fazem o PATH decidir; duplicata se resolve removendo, não escolhendo.
- **`scripts/encerrar-dispositivos.sh`** — o mecanismo da higiene de dispositivos: detecta, encerra e verifica Android e iOS, com `--dry-run` e saída 1 se algo sobrar.

### O que foi corrigido nas skills existentes
- `vies-de-plausibilidade` §11: o diff tem direção e o olho lê ao contrário (erro de método da sessão).
- `toolchain-unica`: o caso do cask que resolve o SDK root para si mesmo (symlink não engana a detecção).
- `quadro-de-trabalho`: a ordem do commit do quadro (na branch principal, depois do merge da branch de trabalho).

### Evidências
- `scripts/encerrar-dispositivos.sh` provado no sentido negativo: com o emulador `kentron` no ar, detectou `emulator-5554`, encerrou e o `adb devices` seguinte ficou vazio; `--dry-run` e o caminho sem `adb` exercitados.
- `adb devices` e `xcrun simctl list devices booted` vazios ao fim da sessão.
- Branches mergeadas removidas com autorização do dono: `spike/native-assets` (local e `origin`), `feature/alvo-android`, `feature/alvo-ios`, `feature/alvo-web` e `feature/ponte-rust-flutter`.

### Ressalvas
- Skill é julgamento, não gate: nada impede a próxima sessão de não carregá-la. O único item com mecanismo é a higiene de dispositivos, e mesmo ela depende de alguém rodar o script.
- Apagar a branch remota antes da local faz o `git branch -d` recusar (ref rastreado obsoleto): `git fetch --prune origin` antes resolve.

## Processo · Perfil do projeto e shellcheck como gate (2026-11-20)

**Status:** 🟢 Integrado à `main`; `/AGENTS.md` deixou de ter placeholders e o `shellcheck` virou gate nos dois pontos onde a bateria roda.

### O que foi preenchido
- **Perfil do projeto** no `/AGENTS.md`: topologia de repositório único (`.harness.toml`), fronteiras (workspace Rust, app Flutter, pacote de localização, ferramentas Python), `main` protegida com `origin` como único remote, stack com as versões pinadas, comandos reais de validação, locales dos ARBs e autoridades de aprovação.
- **Comunicação**: português (Brasil).
- **Release**: sem release publicada; `0.0.0` em `Cargo.toml` e `apps/kentron_app/pubspec.yaml`; a primeira é o piloto da R1.

### O que virou mecanismo
- `shellcheck` em `scripts/ci-local.sh` (todos os `scripts/*.sh`, com `require` fail-loud) e em `.githooks/pre-push` (só os `.sh` do push, mesmo recorte dos gates de lint por stack). O workflow `.github/workflows/ci.yml` instala a ferramenta antes de chamar o script.
- `.githooks/pre-push` corrigiu SC2164: `cd "$REPO_ROOT" || exit 1`. Sem isso, um `cd` que falhasse deixava os gates rodando no diretório errado, sem aviso.

### Evidências
- `shellcheck scripts/*.sh` → `EXIT=0` nos três scripts.
- Gate `shell` no sentido negativo: linha defeituosa injetada em `scripts/encerrar-dispositivos.sh` → `❌ shell`, `PUSH BLOQUEADO`, `HOOK_EXIT=1`, log com SC2154/SC2086; arquivo restaurado → `✅ shell`, `HOOK_EXIT=0`.

### Ressalvas
- O hook continua com 17 achados de `shellcheck`: SC2329 é falso positivo (função chamada por `run_gate`) e SC2001 é estilo. O gate cobre `scripts/*.sh`, não o hook, porque um erro no hook já aparece como falha de gate durante o push.
- `.harness.toml` seguia com `[validation] configured = false` e `gates = []` enquanto o projeto tem gates reais; `.harness/validation.py` recusava rodar nesse estado. **Resolvido em 2026-11-21** na seção seguinte.

## Processo · Contrato de validação: gitleaks, não a bateria inteira (2026-11-21)

**Status:** 🟢 Integrado à `main`; o `.harness/validation.py` deixou de recusar rodar por contrato vazio.

### Decisão
- O contrato `[validation]` do `.harness.toml` declara **um** gate: `gitleaks`, com o mesmo argv do `gate_gitleaks` do pre-push. A bateria completa (Rust, Flutter, ARBs, licenças) **não** entra no contrato — ela já é aplicada pelo `.githooks/pre-push` em cada push, e repeti-la dobraria o custo sem acrescentar cobertura. Decisão do dono do projeto.
- Higiene de dispositivos **não** vira gate de `pre-push`: derrubar dispositivo dentro de um comando de verificação é destrutivo para quem estiver com `flutter run`. Fica no ritual da skill `higiene-de-dispositivos`.

### Evidências
- `python3 .harness/validation.py check` → `gates aprovados: 1`, `EXIT=0` (80 commits, `no leaks found`).
- Contrato no sentido negativo: gate trocado por `false` → `gates reprovados: gitleaks (exit 1)`, `EXIT=1`; arquivo restaurado depois.
- Gitleaks no sentido negativo: repositório temporário com uma chave de formato AWS (`AKIA` + 16 caracteres) → `leaks found: 1`, `EXIT=1`.

### Achado
- A chave de exemplo da documentação da AWS (a que termina em `EXAMPLE`) **não** é detectada: está na allowlist padrão do gitleaks. Canário com ela devolve `no leaks found` e daria falsa confiança — o teste precisa de chave plausível.
- O literal do canário não pode morar na documentação: o `semgrep` bloqueia `AKIA` + 16 caracteres em qualquer arquivo, inclusive `.md`. O primeiro push desta entrada foi bloqueado por isso; o texto foi reescrito sem o valor.

### Ressalva
- O argv do contrato é estático: se `.gitleaks.toml` ou `.gitleaks-baseline.json` forem criados, o hook os acrescenta e o contrato não. Anotado no próprio `.harness.toml`.

## Processo · `origin/HEAD` apontando para `main` (2026-11-21)

**Status:** 🟢 Corrigido no clone principal; nada mudou no GitHub, que já estava certo.

### O que estava errado
- `git branch -r` mostrava `origin/HEAD -> origin/master`, mas o default do repositório no GitHub **já era** `main` (`gh repo view` → `defaultBranchRef = main`; `git ls-remote --symref origin HEAD` → `refs/heads/main`). O sinal enganoso era o ref **local** `refs/remotes/origin/HEAD`, que não acompanha mudança de default no remoto.

### Correção
- `git remote set-head origin -a` relê o default do remoto e reescreve o ref local: `origin/HEAD -> origin/main`. Sem mudança no GitHub, sem push de conteúdo.

### Pendente de decisão
- `master` no remoto é branch **órfã**: 1 commit (`4b10a6d Initial commit`), raiz própria, não ancestral de `main`. Apagar é ação destrutiva no remoto e fica com o dono do projeto. **Resolvido:** apagada pelo dono em 2026-11-21.

## Processo · Portão da R1 em dois: alpha antes do piloto (ADR-0009, 2026-11-21)

**Status:** 🟢 ROADMAP e plano v2.6 corrigidos; ADR-0009 aceita.

### O que estava errado
- O ROADMAP tinha **um único portão** para a R1: `R1-C · Piloto e release`, com dois cultos consecutivos sem assistência, zero perda de dados e traduções revisadas por nativos. Esse portão depende de calendário de igreja e só devolve resultado depois de duas semanas — e estava a três entregas de distância (não havia leitor nem projeção).
- O ROADMAP também estava **atrás da realidade**: ponte FFI, alvos Android/iOS/Web, Native Assets e `HomeScreen` já estavam entregues e só apareciam na CONCLUSION.
- O "Portão de Entrada" da R1 era o spike S5, contradizendo a ADR-0005 (rejeitada), que diz que o S5 dimensiona MusicXML/ATEX e **não** bloqueia o piloto.

### O que mudou
- **R1-C · Alpha utilizável** (portão inicial): leitor, projeção e transposição exercitados em **um ensaio assistido**, com falhas registradas e dados sobrevivendo a fechar/reabrir. Não depende de calendário de culto.
- **R1-D · Piloto e release** (portão final): os critérios caros da ADR-0002 permanecem intactos e passam a ser o **último** marco da R1, depois do R1.1.
- **R1-0 · Fundação técnica (entregue)** registrada no ROADMAP com as quatro entregas que faltavam na direção.
- "Portão de Entrada" alinhado à ADR-0005: o S5 dimensiona R1.1 e não bloqueia alpha nem piloto.

### Por que dois portões
- O portão caro não pode ser o primeiro: ele transforma cada ajuste de UI numa espera de duas semanas. O alpha devolve feedback em um ensaio e deixa o produto ser corrigido antes de chegar a um culto.
- O piloto não foi eliminado, só **reordenado**: a evidência de produção continua sendo exigida, no fim da linha.

### Evidências
- `adr/0009-portao-inicial-da-r1-alpha-antes-do-piloto.md` — decisão, custos e pressuposto de produto.
- `ROADMAP.md` §R1 — R1-0, R1-C e R1-D.
- `plans/plano-mestre-v2.6.md` §3 e §5 — sequência ajustada.
- `PENDING.md` — alpha (R1-C) e piloto (R1-D) como itens separados.

### Ressalva
- O alpha em ensaio **não substitui** evidência de produção: o risco de falha em culto é adiado, não eliminado. A R1 só fecha com o R1-D.

## R1 · Leitor de cifras (Modo Palco) entregue (2026-11-21)

**Status:** 🟢 Leitor utilizável e alcançável pela tela inicial; projeção e setlists ainda abertas.

### O que foi entregue
- `SongReaderPage` desenha a cifra inteira com acorde sobre a letra, transposição por tom e alternância de contraste. O erro do núcleo aparece na tela — não vira vazio.
- `chordLine` alinha os acordes nas colunas devolvidas pelo núcleo; acorde que invadiria o anterior é empurrado para a direita.
- Botão **Abrir cifra** na tela inicial, com chave `stageOpenReader` nos três ARBs-base; `pt_PT` e `es_ES` herdam.

### Provas
- `flutter analyze` sem achados; **16 testes** de app passam.
- 5 testes herméticos do leitor (alinhamento, desenho, transposição, erro).
- 3 provas reais na ponte: `readSong` (seção/linha/coluna), `transposeSong` (tom e acordes), `keyChoices` (doze tons).
- O `cdylib` foi reconstruído antes dos testes: estava mais velho que `bridge.rs` e teria provado contra binário obsoleto.

### O que falta para o alpha (R1-C)
- Projeção manual (tela secundária ou janela do OBS) a partir do leitor.
- Setlists utilizáveis no Modo Palco.
- Ensaio assistido com dados sobrevivendo a fechar/reabrir.

## R1 · Modo projeção manual em tela cheia (2026-11-21)

**Status:** 🟢 Conteúdo da projeção pronto e aberto pelo leitor; transporte para segunda tela/OBS segue aberto.

### O que foi entregue
- `ProjectionPage`: uma seção por vez, sem acorde, com letra grande. Avanço por toque (metade esquerda/direita), setas do teclado e rodapé com contador e botões anterior/próxima. Limites desabilitam o botão em vez de sair do intervalo.
- Botão **Projetar** no leitor abre a projeção com a cifra já lida — a projeção não relê o texto nem toca no núcleo de novo.
- Chaves `stageProject`, `stageProjectNext` e `stageProjectPrevious` nos três ARBs-base.

### Provas
- 5 testes herméticos do modo projeção: render sem acorde, avanço/volta, limites, toque e teclado.
- 1 prova de abertura a partir do leitor.
- Suíte do app: **22 testes** passando; `flutter analyze` sem achados.

### O que continua aberto
- Transporte para **tela secundária** ou **janela do OBS** (Browser Source). O modo em tela cheia é o conteúdo; falta o canal para a segunda tela — não foi marcado como entregue.
- Setlists no Modo Palco.
- Ensaio assistido do R1-C.

## R1-B · Browser Source do OBS por URL (2026-11-21)

**Status:** 🟢 Bootstrap e cópia de link entregues e compilados no alvo web; prova dentro do OBS pendente.

### O que foi entregue
- `?projecao=<base64url do ChordPro>` abre o app direto na projeção, sem passar pela tela inicial. Base64url evita escapes de `&`, `#` e acentos no caminho do navegador.
- `chordproDaUrl` decodifica a URL e devolve `null` quando o parâmetro falta ou está estragado — URL inválida não vira projeção vazia.
- `urlDaProjecao` monta a URL preservando host, caminho e fragmento; o leitor oferece **Copiar link de projeção** no alvo web.
- `ProjecaoApp` lê a cifra pelo núcleo com carregador injetável; falha do núcleo aparece na tela.
- Sem `obs-websocket`, como o marco exige.

### Provas
- 5 testes de URL: round-trip com acentos, URL inválida, preservação de host/caminho/fragmento e troca de query.
- 2 testes de bootstrap: abre direto na projeção; falha do núcleo visível.
- Suíte do app: **29 testes**; `flutter analyze` limpo; `flutter build web --release` compilou (`✓ Built build/web`).

### O que continua aberto
- O round-trip **dentro do OBS** não foi executado: o build web prova compilação, não o Browser Source em uso. Essa prova continua pendente.
- Tela secundária nativa (fora do OBS) segue sem transporte.

## Processo · Prova de navegador, locales e piloto automatizado (2026-11-21)

**Status:** 🟢 Telas entregues provadas em navegador real; dois defeitos encontrados e corrigidos; portão da R1 passa a ser automatizado.

### O que foi provado (Playwright + Chromium headless, sobre o build web)
- **Home:** saudação localizada, `núcleo: kentron_ffi 0.0.0` (a ponte wasm responde no navegador) e o botão **Abrir cifra**.
- **Leitor:** seções, acordes alinhados em ouro, seletor `Tom: C` e os três ícones de ação.
- **Transposição:** trocar o tom levou `C G Am F` a `Db Ab Bbm Gb` — transpõe de verdade na tela.
- **Projeção:** fundo escuro, seção e letra grandes; **avanço/volta por toque** mudam `1 / 2` ↔ `2 / 2`.
- **Bootstrap por URL:** `?projecao=<base64>` abriu direto na projeção com a cifra da URL.
- **Copiar link:** o clipboard saiu de vazio para `…/?projecao=<base64 do ChordPro>`.

### Defeitos encontrados por olhar a tela (e corrigidos)
- **Ícones como "tofu":** o `pubspec.yaml` não tinha a seção `flutter:`, então `uses-material-design` estava desligado e a fonte de ícones não era empacotada. Corrigido; o build passou a fazer tree-shake de 1,6 MB → 2 KB.
- **AppBar do Modo Palco claro sobre corpo escuro:** corrigido com `backgroundColor`/`foregroundColor` seguindo o modo, e coberto por teste.

### Decisões registradas
- **Locale de faixa 1:** `pt_BR` foi pedido, mas o `gen-l10n` **exige arquivo-base por idioma** (`pt_BR` sem `pt` faz o gerador abortar) e `es_419` usa região UN M49 que o gerador não trata como país. Ficou `pt`/`en`/`es` com a correspondência explícita no `/AGENTS.md` (`pt` = pt-BR, `es` = es-419), sem duplicar texto.
- **Contrato de arquitetura:** `musicxml` é **stub**, não entrega — o `architecture/README.md` foi corrigido.
- **ADR-0010:** o portão R1-D passa a ser **piloto automatizado** (VM Windows + OBS por Browser Source, duas rodadas simuladas); o uso ao vivo vem depois e realimenta a próxima release. ADR-0002 e ADR-0009 ganharam a nota de emenda, e ROADMAP/plano v2.6 foram alinhados — sem contradição entre eles.

### Evidências
- 30 testes de app; `flutter analyze` limpo; `flutter build web --release` verde.
- Screenshots da prova em `/tmp/kentron-e2e/shots2` e `/tmp/kentron-e2e/shots3` (artefatos locais da sessão).

## Processo · `pt` é pt-BR por gate (2026-11-21)

**Status:** 🟢 Regra do dono virou mecanismo; gate provado nos dois sentidos.

### A decisão
- O dono do projeto fixou: **sem duplicação** de ARB, `pt` segue a ortografia do **português brasileiro**, e isso não é negociável. `en` e `es` ficam como estão.

### Por que virou gate
- Grafia não se defende com aviso. Sem verificação, um "Ecrã" ou um "ficheiro" entra no arquivo brasileiro e o app passa a falar português de Portugal com o usuário brasileiro — e ninguém nota até um usuário reclamar.

### O mecanismo
- `tools/check_arb.py` (o gate `arb` do `pre-push`) reprova marcas de português europeu em `app_pt.arb`: `ecrã`, `secção`, `ficheiro`, `utilizador`, `telemóvel`, `autocarro`, `connosco`, `receção`. `pt_PT` continua livre — é onde a variante europeia mora.

### Provas
- Positivo: verde com os cinco ARBs atuais, incluindo o "Ecrã" do `pt_PT`.
- Negativo: com "Ecrã" injetado em `app_pt.arb`, o gate reprovou apontando a chave (`gearProjectionScreen`) e a palavra; restaurado o arquivo, voltou a passar.

### O dicionário de variantes: uma língua, N variantes, sem duplicar texto
- `tools/variantes.tsv` guarda pares de formas equivalentes (`pt`↔`pt_PT`, `es`↔`es_ES`). É **dado**, não código: acrescentar uma linha muda o gate, sem lista escondida no Python.
- Regra 1 (**bloqueia**): forma da variante no arquivo base. É o que sustenta `pt` = pt-BR.
- Regra 2 (**avisa**): a base carrega forma local e a variante não sobrescreve a chave — é o texto que o leitor da variante leria errado.
- A regra 2 é radar, não gate: dicionário conhece palavras, não julga frases. Pelo método do projeto, verificação que não decide sintaticamente não bloqueia.
- **O radar pagou-se na primeira execução:** apontou que `pt_PT` não cobria `stageProjectNext`/`stageProjectPrevious` — em Portugal, "seção" escreve-se "secção". As duas chaves foram para o `app_pt_PT.arb` e o aviso desapareceu.
- `.githooks/pre-push` passou a chamar o gate `arb` **direto**, não via `run_gate`: aquele guarda a saída e só a mostra quando o gate falha, então o aviso de quem passa nunca apareceria. Provado: com o override removido, o push **autoriza** e imprime os 2 avisos; com "Ecrã" no `pt`, bloqueia com `gate(s) reprovado(s): arb`.


## R4 · Primeiro incremento do motor MIR: análise de áudio real (ADR-0011 aceita, 2026-11-21)

**Status:** 🟢 Entregue como **instrumento de trabalho**, fora do portão da R1. DSP puro, sem modelo, provado em áudio sintetizado e em uma música real.

### A decisão
- O dono pediu o caminho curto: apontar uma música e obter cifra, estrutura e dinâmica **a partir do áudio**, sem consultar repositório de cifras. Como isso é o R4, a governança exigiu ADR antes do terminal — a [ADR-0011](adr/0011-mir-primeiro-incremento-de-analise-de-audio.md) separa o que é entregável agora (acordes, dinâmica, estrutura, todas por DSP) do que exige peso e incremento próprio (**letra** por ASR e **tablatura** por pitch tracking + corda/traste).
- Consequência de escopo: "cifra completa com letra" **não** existe nesta entrega. O que existe é cifra por seção, em ChordPro, com confiança por trecho.

### O que ficou de pé
- `crates/mir_engine`: leitor de WAV PCM sem dependência; dinâmica EBU R128 (pesagem K, blocos de 400 ms com 75% de sobreposição, portão absoluto e relativo, LRA, pico de amostra); STFT com `rustfft`; croma por picos interpolados; acordes por casamento de template com suavização temporal; andamento por autocorrelação da novidade; estrutura por picos de novidade + auto-similaridade do croma com rótulos de repetição.
- CLI `mir` com resumo legível, `--json` e `--chordpro`.
- Segunda dependência externa do workspace: `rustfft` (MIT/Apache-2.0), coberta pelo `deny.toml`. Nenhum modelo, portanto nenhuma linha nova no `models/MANIFEST.toml`.

### Provas
- **Áudio sintetizado** (C–G–Am–F a 120 BPM, com final esmaecendo): 4 testes verdes — ida e volta de WAV, progressão e andamento reconhecidos, introdução/final e cauda mais baixa, cifra sem dígito no tom nem acorde repetido.
- **Contraprova do LUFS contra o `ffmpeg`** — dois medidores independentes no mesmo arquivo: sintético dentro de 1 LU e, no áudio real, **−12,6 LUFS (motor) contra −12,0 LUFS (ffmpeg)**.
- **Áudio real de 400,4 s analisado em 0,77 s** (release). `cargo fmt`, `cargo clippy --all-targets -D warnings` e `cargo test` verdes.
- Sequência detectada na música real: E, B, F#/Gb, C#m/Dbm, A, com um trecho em D/C/G/Em — **I–V–vi–IV em E maior**, a região harmônica certa.

### O que os testes acharam (e por que existem)
- **C3 saía como B**: o bin da FFT (10,8 Hz em 44,1 kHz com janela de 4096) é mais largo que um semitom abaixo de ~180 Hz, e a classe de altura era **truncada** em vez de arredondada. O croma passou a sair de picos com interpolação parabólica.
- **Estrutura degenerada no primeiro áudio real**: mínimo de 4 s cortava a cada troca de acorde (30+ "trechos", todos "refrão"). Mínimo para 15 s, limiar de novidade para 1,5σ e refrão passou a ser o rótulo mais repetido.
- **`{key: B5}` na primeira cifra**: o extrator de fundamental cortava na primeira minúscula, então `Db` virava `D` e o `5` de `B5` entrava como fundamental. Cada um desses virou teste.

### Ressalvas honestas
- **Letra e tablatura não estão entregues.** São incrementos próprios, com peso declarado antes de entrar.
- **Acurácia de acorde em mixagem é moderada e não medida em escala:** a prova foi em **uma** música. A qualidade "5" (power chord) pode ser a disposição real da guitarra ou a terça mascarada na mixagem — o motor não distingue, e o relatório diz "candidato" por isso.
- **Tom estimado é heurística:** devolveu `B` numa música em E maior (é a fundamental mais frequente, não o centro tonal).
- **Pico de amostra, não pico verdadeiro** — nomear diferente seria mentir no relatório.
- O áudio usado na prova foi obtido **fora do repositório** (nenhum downloader foi versionado); o motor consome WAV, e a etapa de aquisição é decisão do dono.


## R4 · Letra na cifra: fonte declarada, alinhamento ao áudio e desenho no app (ADR-0012 proposta, 2026-11-21)

**Status:** 🟢 Entregue e provado de ponta a ponta. A ADR-0012 está **proposta** — o aceite é do dono, porque um ponto dela contraria o pedido original.

### O achado que mudou o desenho
- O pedido era buscar a letra em "fontes oficiais" e cruzar com o áudio, pagando pouco e sem compromisso legal. Medido: **não existe fonte oficial de letra gratuita.** Os provedores oficiais (Musixmatch, LyricFind, Genius, letras do Spotify) são licenciados e pagos, e seus termos **proíbem armazenar ou reproduzir** a letra — exatamente o compromisso que se queria evitar. "Oficial" e "sem custo legal" não convivem hoje.
- O que é barato **e** legítimo: texto do próprio usuário, obra em **domínio público** (corpus embutido, só pré-1900, com autor e ano por entrada) e, no futuro, ASR local com peso declarado.
- O "cruzar com o que o áudio apresenta" é **alinhamento forçado** — técnica que não depende de licença nenhuma: texto conhecido + áudio.

### O que ficou de pé
- `lyrics`: procedência obrigatória, leitura de letra livre/ChordPro, corpus de domínio público com regra de entrada escrita no arquivo, **frases vocais** por energia 200–3500 Hz e alinhamento das linhas com **confiança por linha**.
- `sheet`: a cifra vira `notation_bundle::chordpro::Song` — o acorde é colocado na **sílaba** que o tempo dele indica, e o texto é escrito por `chordpro::write`. O formato do app é o mesmo por construção; **nenhum binding novo** foi preciso.
- CLI: `--letra`, `--hino`, `--listar-hinos`, `--frases-vocais`.
- Fixture única (`crates/mir_engine/tests/fixtures/cifra-com-letra.cho`): o teste dourado do motor fixa o arquivo e o teste de Flutter desenha **esse mesmo arquivo** pela ponte real. Se o formato mudar, os dois lados quebram juntos.

### Provas
- **Rust: 11 testes.** Procedência e recusa do corpus no sentido negativo (o hino protegido **não** está lá), sílaba ("San-to", não "Sa-nto"; "Bra-vo" com o `r` na sílaba certa), alinhamento casando com quatro blocos cantados, ida e volta pelo parser do app (`write` → `parse` devolve a mesma música) e fixture dourada.
- **App: 32 testes de Flutter**, incluindo o núcleo devolvendo a letra com acordes por cima e a **tela do Modo Palco desenhando letra e acorde** sobre a cifra do motor.
- **Detector de frase vocal medido em três desenhos.** Pico global: **1 frase de 385 s em 400** na música real (inútil). Média local causal: no sintético **não achava o primeiro bloco** (nível constante nunca fica 15% acima da própria média) e truncava as frases em 2,8 s. Máximo recente: **4 frases casando exato** com os 4 blocos cantados e **63 frases cobrindo 302 s de 400 s** na música real, de 1,5 a 5,8 s cada — escala de linha cantada.

### O que o processo de teste pegou (e valeu a sessão)
- **Minha síntese estava errada:** a tríade era tocada em **sequência**, e o detector lia três acordes onde havia um (fixture saía `[F][A5][Dsus4]`). Corrigido para tríade sustentada — a fixture passou a trazer a progressão real, C, G, Am e F.
- **Formante senoidal é nota:** 900 Hz ≈ Lá 5 injetava um A no croma e o acorde de Dó saía `Am7`. Virou ruído de banda larga.
- **A conferência procurava texto contíguo** num arquivo onde os colchetes interrompem a letra (`[C]Ama[G]zing`): passou a conferir o texto que o parser separa — o que o app desenha.

### Ressalvas honestas
- **Alinhamento por frase é aproximado**, não fonema a fonema: sem modelo acústico não há como amarrar sílaba a instante. Por isso a confiança por linha existe.
- **Música protegida cuja letra o dono não tenha:** sai o **esqueleto** (onde cada linha cai, seção e dinâmica), **não** as palavras. É limitação declarada, não defeito.
- Detector calibrado em **dois** casos; o repertório é medição pendente (PENDING).
- O tom estimado continua heurístico (devolveu `Am` na fixture em Dó).

