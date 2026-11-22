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

## R1-D · Piloto automatizado no OBS (2026-11-21)

**Status:** 🟢 Projeção renderizada e navegada dentro do Browser Source do OBS, em duas rodadas; prova negativa reprovou a URL morta. Faltam transposição dentro do OBS e sobrevivência a fechar/reabrir.

### O que foi entregue
- `scripts/piloto-obs.sh`: serve o build web, abre dois túneis SSH (4455 do `obs-websocket` e a porta do servidor de volta para a VM), cria uma fonte de navegador, aponta para a URL de projeção e captura a fonte por `GetSourceScreenshot`.
- `?secao=N` na URL de projeção: o Browser Source não recebe clique nem teclado, então a URL é o único canal determinístico para navegar. `secaoDaUrl` lê o índice e `ProjectionPage` limita ao intervalo válido — índice fora da faixa não quebra.
- O roteiro compara o hash do pixel entre seções e entre as duas rodadas, e reprova quando a tela fica chapada (1 cor, σ=0.0).

### Provas
- Duas rodadas aprovadas: seção 0 com 353 cores (σ≈19,5) e seção 1 com 354 cores (σ≈19,2); o hash do pixel mudou entre as seções e repetiu entre as rodadas.
- Prova negativa: URL morta (`http://127.0.0.1:9`) reprovou nas 6 verificações e o roteiro saiu 0 — o verificador não é complacente.
- 17 testes de app (URL e bootstrap) e `flutter analyze` limpo.

### Causa raiz de um falso negativo
- Servir pelo IP da LAN (`http://192.168.6.129:8765`) deixava a fonte em branco: o CEF do OBS ignora COOP/COEP fora de contexto seguro, o fallback do frb reinjeta `pkg/kentron_ffi.js` e o wasm morre com `Identifier 'wasm_bindgen' has already been declared`. A correção é servir por `localhost` com túnel reverso (`ssh -R`), que põe o servidor do Mac em `127.0.0.1` dentro da VM.

### O que continua aberto
- **Transposição dentro do OBS:** a projeção não mostra acorde, então transpor não muda o pixel; a transposição é provada em `scripts/e2e-web.sh` e nos testes Rust.
- **Sobreviver a fechar/reabrir:** o roteiro não reinicia o app; a restauração do bundle é provada no alpha assistido (R1-C).

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

## R4 · Comparação medida contra uma cifra humana (2026-11-21)

**Status:** 🟢 Medição feita e registrada. Um erro real encontrado e corrigido por causa dela.

### O que foi comparado
O dono trouxe uma **cifra feita à mão** da mesma gravação analisada no incremento 1 — trabalho humano de audição e notação, com harmonia, estrutura e tablatura. A cifra e a letra **não entram no repositório** (conteúdo de terceiro); ficam o método e os números.

A comparação precisa ser lida com o cuidado certo: a cifra humana descreve **o que o músico toca** (com nome de acorde, inversão e tensão); o motor descreve **o que está soando**. São verdades diferentes, e é por isso que as duas se completam em vez de uma corrigir a outra.

### O que o motor achou e o gabarito confirma
- **Cobertura de fundamental 8/8:** todos os centros que o gabarito usa (`C, C#, D, E, F#, G, A, B`) aparecem na detecção.
- **A modulação:** o gabarito traz um Solo 2 em `Am C G D/F#`; o motor marca um trecho de área harmônica distinta em **264–294 s** com `Cmaj7, G, D, A, Em7`. Ninguém disse ao motor que havia modulação ali.
- **Menores onde há evidência:** `F#m` → `Gbm7/Gbm`, `C#m7` → `Dbm7`, `Bm` → `Bm7`. E o `E7M` do gabarito aparece como `Emaj7`, o `B4` como `Bsus4`.

### O que o motor não entrega — e por quê
- **Tablatura de solo:** fora do escopo declarado (ADR-0011). Exige pitch tracking e casamento de corda/traste.
- **Inversões** (`E/G#`, `D/F#`, `B/D#`): o motor detecta **conjunto de classes de altura**, não a nota do baixo. Estrutural — exige o baixo isolado (stems).
- **Notação de tensão** (`2`, `4`, `7M`): sai como `sus4`/`maj7`/`5`. Os conjuntos se sobrepõem; o *nome* não.
- **Tom:** o gabarito declara `E`, o motor disse `B`. A área harmônica é de E maior; o palpite de tom é heurística de fundamental mais frequente.
- **`Am` lido como A maior** no trecho modulado: a terça menor do A não foi resolvida ali.

### O erro que a comparação pagou
O gabarito mostra que os acordes desse repertório **têm terça** (`E7M`, `F#m7`, `C#m7`, `Bm`). O motor estava entregando 45% dos trechos como potência (`5`). A pergunta certa não era "melhorar o detector", era **medir** se a terça está no áudio ou se está sendo descartada:

- razão terça/fundamental nos 91 trechos rotulados `5`: **mediana 0,28**, e em **80%** deles abaixo de 0,4.

Conclusão medida: na maior parte **a terça não está audível** — o `5` é a leitura honesta, e forçar maior/menor ali seria inventar. Nos 20% em que a terça existe, o template de duas notas ganhava por diluição do croma. A regra `chords::com_terca` passou a exigir **terça acima de 0,4 da fundamental** para dizer o acorde inteiro: **6 trechos** mudaram de potência para acorde completo, e nenhum outro ganhou terça que não tem.

### Leitura honesta do conjunto
Não é mágica: é DSP sem modelo, **0,77 s para 400 s**, acertando o esqueleto harmônico inteiro, a estrutura e uma modulação — e falhando exatamente onde a informação não está no áudio (terça mascarada, baixo, digitação). O que falta é o que exige mais que DSP: **separar instrumentos** (stems) para recuperar terça e baixo, e **pitch tracking** para tablatura. Isso é o portão da R4 que segue de pé (spike S1 + peso no `MANIFEST`).

## ADR-0012 · Crédito não é licença: o argumento do dono, com resposta (2026-11-21)

**Status:** 🟡 Argumento registrado; a ADR segue **proposta** (o aceite é do dono).

### O argumento, do jeito que veio
O dono observou que a live da banda é derrubada quando executa música sem os créditos dos artistas originais — e que isso é justo. Daí: se eu informo na tela (ou na transmissão) quem detém o direito, o que diferencia extrair o texto **do áudio** de obtê-lo de **arquivo ou link**? Não seria a mesma coisa?

### Por que não é a mesma coisa
- **Crédito e licença são direitos distintos.** Crédito cumpre o **direito moral** (paternidade). **Reprodução** e **comunicação ao público** — tela, projeção, transmissão — são **direitos patrimoniais**, e é isso que a autorização tem de cobrir. Quem credita e não licenciou continua sem licença.
- **A derrubada tem duas origens diferentes, e isso explica a assimetria que o dono notou:** a plataforma exigindo identificação/lincenciamento da obra, e o **Content ID** casando o **fonograma**. O que cai, no segundo caso, é a **gravação do titular** — por isso a banda ao vivo costuma passar e a gravação original não.
- **A letra é obra autônoma**, protegida à parte da melodia e do fonograma. Extrair o texto cantado — por ouvido, por OCR ou por ASR — produz uma **cópia** dela. O meio não muda a natureza do ato; "é praticamente transcrição" descreve o ato, não o defende.

### O que o argumento acertou
O dono está certo em um ponto que importa: **a diferença entre os caminhos não é a "fonte", é de quem é o ato.** E há um caminho que ele **já tem** e resolve o caso da igreja: a **licença coletiva** (CCLI e equivalentes), que autoriza reproduzir e projetar letra. Com ela declarada, o app deixa de ser fonte concorrente de letra e passa a ser **ferramenta do licenciado** — e o motor agrega o que o texto sozinho não tem: alinhamento ao áudio, dinâmica por linha, estrutura e tempo.

### Consequências registradas
- ADR-0012 emendada: o **argumento do crédito fica escrito com a resposta** (para não voltar como "melhoria óbvia"), o passo 1 da decisão passa a nomear **letra sob licença declarada do usuário**, e a ressalva é explícita: **não é parecer jurídico**.
- PENDING ganhou a proposta que operacionaliza a emenda: a cifra carregar `{lyrics_license: ...}` e um relatório de uso por música — que é justamente o que a licença coletiva exige de quem projeta e transmite. Formato é decisão do dono; o motor não arbitra licença.
- Nada disso muda o que o ASR faria: ele continua atrás de decisão de peso **e** de licença, porque é o caminho em que o **produto** passa a ser quem reproduz.

## ADR-0012 aceita e emendada: dois eixos de procedência e extração pelo usuário (2026-11-21)

**Status:** 🟢 **Aceita pelo dono**, emendada no mesmo dia e implementada. 15 testes de Rust verdes e o app segue verde.

### O que o dono decidiu
1. **Aprovar a ADR-0012.**
2. **Obra e gravação são eixos distintos:** louvor de domínio público (hinário, Cantor Cristão, Harpa Cristã, clássico, folclórico) tem a **obra** informada como tal — mesmo quando a gravação é de ontem e vem de link. "A música/letra é de 1822 e a gravação é de ontem."
3. **Extrair letra do áudio é necessidade real**, e o que sustenta o uso é ficar claro que **quem fornece é o usuário** — a extração é ato dele —, com a cifra resultante **privada no grupo da banda**.

### O que foi implementado
- **`Obra` e `Fonograma` como tipos separados**, com a regra explícita de que **gravação não tem veredito de domínio público**: a categoria não existe do lado do fonograma. Uma gravação de 2024 de obra de 1822 é registrada com **titular** — é o que a cifra passa a dizer.
- **`veredito_pd` com três estados e a regra no texto**, por jurisdição: Brasil por **vida do autor + 70 anos** (Lei 9.610/98, art. 41, contados de 1º de janeiro do ano seguinte ao falecimento); EUA por **publicado antes de 1930**. **Ficha incompleta devolve `Indeterminado`** — e indeterminado não é "pode".
- **A armadilha da tradução virou dado:** a ficha carrega `tradutor`/`tradutor_morte`, e a obra traduzida é julgada **pelo tradutor**. É o que impede um hinário de 1922 de passar como se a tradução de 1980 fosse de 1922.
- **Extração pelo usuário como origem própria** (`ExtraidaPeloUsuario`), com **escopo `grupo`** e **`pode_publicar` reprovando** a publicação. Não existe variante de origem para extração feita *pelo produto*: o enum não aceita — decisão de tipo, não de prosa.
- **Como identificar**: `--ficha` calcula o veredito por jurisdição a partir dos campos que o usuário informa; `--auditar-corpus` confere cada ficha do corpus; `--listar-hinos` mostra as fichas com o veredito. O caminho é **ficha conferida**, nunca adivinhação.

### Provas
- `obra_de_1822_com_gravacao_de_ontem_nao_vira_gravacao_livre` — obra de domínio público **nas duas jurisdições** (vida de 1868 + 70; publicação anterior a 1930), gravação com ano e titular próprios, e **tradução recente protegida** no Brasil e `Indeterminada` nos EUA por falta do ano de publicação da tradução.
- `ficha_incompleta_nao_vira_dominio_publico`, `obra_recente_nao_passa_por_dominio_publico` — o negativo do verificador.
- `extracao_do_usuario_bloqueia_publicacao` — o portão reprova, e a cifra nomeia de quem é o ato.
- Fixture regenerada com os dois eixos visíveis: `{work_status: domínio público (vida de John Newton (1807) + 70 anos)}`, `{recording_source: …; 2026; titular: …}`, `{lyrics_scope: público}`.
- Corpus auditado em teste: todas as fichas resolvem domínio público nas duas jurisdições.

### Ressalvas
- **Hinários brasileiros não entraram**, e é deliberado: cada hino exige a **ficha do tradutor**. A ferramenta está pronta; o trabalho é de conferência.
- **A extração em si ainda não existe** — falta o peso no `models/MANIFEST.toml`. O que foi entregue é o enquadramento (responsabilidade + escopo), que era o que faltava para o caminho ser previsto sem pôr o produto no lugar do titular.
- **O app ainda não honra `lyrics_scope`** na exportação e no link de projeção: hoje o portão mora no motor.

## R1 · A agenda do culto vira o relatório de uso (ADR-0006 emendada, 2026-11-21)

**Status:** 🟢 Emenda aceita, implementada e provada. 17 suítes de Rust verdes.

### O que o dono respondeu — e por que isso redesenhou o pedido
Eu pedi três formatos; ele respondeu com a realidade:

- **Não existe relatório de uso.** O que existe é **agenda por culto/evento**, **uma playlist por evento**, músicas que **repetem** e, quando repetem, **nem sempre na mesma versão**.
- **Não existe formato de licença** — e o repertório passa de **1000 músicas diferentes por ano** (Cantor Cristão e alguns da Harpa Cristã somados a Fernandinho, Projeto Sola, Aline Barros, Renascer Praise). **Enumerar é inviável.**

Duas conclusões mudaram o desenho: o relatório **não pode pedir hábito novo** — tem de sair do que a equipe já mantém —, e o problema **não é listar o repertório**, é **classificar o que se canta**.

### O que foi implementado
- **A agenda é o registro**: `Setlist` deixou de ser uma lista e virou o **evento** — data, tipo (culto/ensaio/evento) e entradas com **versão** e **tom**.
- **Versão como campo de primeira classe**: a obra se repete, a versão muda. Sem isso, duas execuções diferentes viravam a mesma linha — e o relatório mentiria sobre o que a banda tocou.
- **`relatorio_de_uso`** por obra (execuções + versões) e **`--relatorio-uso <pasta>`**, que cruza a agenda com a **ficha** da obra e separa **domínio público por ficha** de **sem ficha**.
- **`schema_version` 2 com passo de migração de verdade** — o primeiro passo real da cadeia que a ADR-0006 previu: agenda v1 (id solto) vira execução **sem versão inventada**.

### A saída, medida em bundle real
```
eventos: 2 · execuções: 5 · obras distintas: 3

  2×  Amazing Grace              2× (versão não declarada)
  2×  Essência da Adoração       1× álbum · 1× acústica
  1×  Santo, Santo, Santo        1× (versão não declarada)

classificação (ADR-0012) — domínio público por ficha: 1 · sem ficha: 2
```

### O que isso resolve, concretamente
- **A lista de fichas vira finita e priorizada pelo uso**: em vez de conferir os ~400 hinos do hinário, confere-se o que a igreja **de fato canta**, na ordem em que canta.
- **A dúvida de licença ganha dado**: "das músicas deste culto, estas são de domínio público (com a regra) e estas estão sem ficha". Sem ficha, o motor **não afirma** nada — nem público nem protegido.
- **A versão distingue o que se confundia**: a mesma obra em dois cultos, em arranjos diferentes, aparece como duas execuções — que é o que a licença coletiva exige registrar.

### Ressalvas
- **O app não tem tela de agenda**; o modelo, a persistência, a migração e o relatório existem e são provados.
- O **formato de licença continua decisão do dono**, e o projeto não inventa um. O que ele recebeu é a informação para decidir, não uma decisão tomada por mim.
- `load_refuses_a_newer_schema` precisou acompanhar o bump do schema — teste calibrado por constante, não por número solto.

## ADR-0012 · Fronteira de produto e o repertório real medido (2026-11-21)

**Status:** 🟢 Medição feita, fronteira escrita. Nada de código mudou — e isso é o correto: aqui era hora de medir antes de construir.

### A fronteira que o dono fixou
> "É **entrada de usuário**, não que o produto decida sobre isso ou repertório."

Repertório e versão **são do usuário**. O produto mede, posiciona e declara procedência — não escolhe o que se canta, não diz qual versão vai ao ar, não substitui a equipe. Escrito na ADR com o número que sustenta a decisão.

### O que o repertório real mostrou (313 linhas exportadas)
- **299 títulos distintos · 295 obras distintas** normalizando `(Ao Vivo)`/`(Live)`/`- Part. X` · **156 artistas**.
- **O export já carrega os dois eixos da ADR-0012**: obra (`Song name`+`Artist`), versão (`Album`/`Version name`/`Version notes`), **fonograma com link em 299 das 313 linhas**, tom (293) e BPM (224). O modelo do produto **casou com o que a equipe já preenche** — sem campo inventado.
- **244 linhas trazem a nuance de execução no próprio texto** ("Masculino: A / Feminino: D"): é exatamente o que o dono pediu para preservar na primeira mensagem, e agora tem lugar formal (`Execucao.versao` + `Execucao.tom`).
- **O corpus de domínio público classifica 0 das 313.** Ele é de hinos ingleses pré-1900; o repertório é português.

### As duas coortes — e por que a distinção importa
| Coorte | Linhas | O que resolve |
|---|---|---|
| Hinário (Cantor Cristão, Harpa) | **10** | **ficha do tradutor** → veredito de domínio público, com a regra |
| Contemporâneo | **303** | **atribuição + decisão de licença do dono**; nenhuma ficha torna essas obras livres |

Dizer isso é o oposto de vender magia: `SEM FICHA` será a resposta da maior parte do repertório, e **não é falha do motor** — é a natureza daquelas obras.

### Correção de escala (minha, não dele)
Eu havia lido "mais de 1000 músicas" como tamanho de repertório; são **execuções por ano**. O repertório limpo é de **~300 obras**. Consequência prática: o esforço de ficha saiu de inviável para **factível**, e a lista priorizada pelo uso (`--relatorio-uso`) diz por onde começar.

### Artefato para a próxima rodada
Planilha de campanha em `/tmp/kentron-repertorio-campanha.tsv` (dado do dono, **fora do repositório**): título, artista, versão, notas de versão, tom, vídeo, coorte e colunas vazias de ficha (autor, ano, falecimento, base) para preencher. As 10 linhas de hinário vêm primeiro.

## ADR-0012 · Sem catálogo, repertório móvel e obra derivada (2026-11-21)

**Status:** 🟢 Decisões registradas e modeladas; 17 suítes de Rust verdes.

### O que o dono corrigiu
1. **Meu enquadramento de escala estava incompleto, não errado:** o repertório **já teve mais de 1000 obras**; as 313 de hoje vêm de um enxugamento que acontece (na intenção a cada 6 meses, na prática ano a ano), e as **execuções** é que são numerosas. Ou seja: **o repertório anda** — e nada no produto pode assumir catálogo estável.
2. **"A ideia é o produto nascer sem catálogo"**, com onboarding explicando o funcionamento e o usuário o alimentando.
3. **Transcrever a letra do áudio é o caminho principal**; letra fornecida é **opção** para quando a versão muda frases.
4. **O caso "Maravilhosa Graça" → "Tremenda Graça"**: mesmo sentido, versos, poesia e música; **título e frases mudam**. Não é o mesmo caso de tradução nem é a mesma obra.

### O que isso produziu
- **O corpus não é catálogo**: é **tabela de referência de ficha**, que só responde quando o usuário traz a obra. Escrito na ADR — evita que alguém (inclusive uma sessão futura) o trate como repertório sugerido.
- **O relatório é foto datada de repertório móvel**; o que se acumula é **histórico de execução**, e **ficha conferida não expira** (obra que sai pode voltar).
- **Obra derivada ganhou modelo:** `adaptador`/`adaptador_morte` (o campo que era `tradutor`, porque **tradução é o primeiro caso de adaptação**) e **`obra_original`** — vínculo, não fusão. O caso geral que o dono descreveu (4 acordes de obra livre com frases novas) fica explícito: **a obra original pode ser livre e a adaptação, não**.
- **O ASR ganhou um argumento técnico que eu não tinha visto:** para obra derivada, **é a única fonte**. Nenhuma base oficial tem "Tremenda Graça"; o áudio tem. Isso reforça a prioridade que o dono deu.

### Provas
- `adaptacao_com_frases_novas_e_obra_propria_e_nao_herda_o_veredito`: original de 1779 → domínio público; adaptação com adaptador falecido em 2015 → **protegida**, com a regra nomeando o **adaptador**.
- Demonstração no CLI: mesma obra dá `domínio público (vida de Autor Antigo (1807) + 70 anos)`; a adaptação dá `protegido (vida de Adaptador da equipe (2015) + 70 anos)` e, nos EUA, `indeterminado — obra derivada: falta o ano de publicação da adaptação`.

### Ressalvas
- **Onboarding não existe** (PENDING): a decisão é o produto nascer vazio e explicar; a interface não foi feita.
- **ASR não implementado** — falta o peso no `MANIFEST`. O enquadramento (ato do usuário, escopo privado, `pode_publicar`) está pronto e provado.
- A **titularidade da adaptação feita pela própria equipe** tem modelo, mas ainda não tem fluxo: quando a equipe adapta, ela é a titular, e isso precisa aparecer na ficha sem eu presumir por ela.

## ADR-0012 · Como o Letras.mus.br / Cifra Club licencia — investigado nos Termos de Uso (2026-11-21)

**Status:** 🟢 Investigação concluída com evidência primária (os próprios Termos de Uso e a Política de Privacidade).

### Como a leitura foi feita (e o que ela custou)
A borda **Akamai** (`errors.edgesuite.net`) recusa clientes automatizados: **HTTP 403** no `fetch`, no `curl` local e **até no Chromium headless**, e também das máquinas da LAN (`playwright` 192.168.6.221, `lab`). Com **navegador visível** e **sem stealth, sem user-agent falso, sem trocar de rota**, as páginas abriram (HTTP 200). Li **uma vez cada** documento; da página de letra extraí **apenas o rodapé de atribuição** — nenhum verso. Os scripts ficaram em `/tmp` (fora do repositório) e **não serão versionados**: manter no repo uma ferramenta que lê site que proíbe extração é convite a mau uso.

### O que os termos dizem, literalmente
- **"somos meros 'provedores de aplicações', nos termos do Marco Civil da Internet… Não nos cabe lançar ou postar qualquer conteúdo nos nossos sites… sendo a postagem de conteúdo, portanto, uma faculdade exclusiva dos nossos usuários. Por essa razão, como meros provedores, não nos responsabilizamos pelos conteúdos postados pelos usuários. Isso fica a cargo deles."**
- **"Em relação aos direitos autorais, nós pressupomos que você lançará em nossos sites unicamente conteúdos livres e/ou licenciados… Nossa responsabilidade se limita a retirar conteúdos do ar sempre que viermos a receber uma ordem judicial nesse sentido, nos termos do Marco Civil da Internet (art. 19)."**
- **"Não nos responsabilizamos por eventual violação de direitos da personalidade, direitos autorais, direitos conexos e direitos de marca… decorrente de qualquer conteúdo postado por usuários."**
- Procedimento de retirada: notificação por e-mail com as URLs e prova de titularidade; **"Retirar o conteúdo do ar antes de receber uma ordem judicial é uma prerrogativa, um direito nosso, não propriamente uma obrigação."**
- Compromisso do usuário: **nunca** "usar quaisquer métodos de prospecção ou métodos semelhantes de **extração de dados**".
- **Acervo:** não há cláusula de licenciamento de obra de terceiro **nem de cessão de direito ao assinante**; a propriedade intelectual citada é a **da plataforma** (softwares, cursos, marcas), expressamente **não cedida** ao usuário.

### A resposta à pergunta do dono
**Não existe licença de letra nesse modelo.** Existe: **(1)** presunção de que quem envia tinha o direito; **(2)** limitação da responsabilidade da plataforma à retirada após ordem judicial (Marco Civil, art. 19); **(3)** isenção expressa; **(4)** canal de notificação. Um acervo de **3,9 milhões** de letras não é resultado de 3,9 milhões de acordos — é resultado disso. E o rodapé que o dono encontrou (**"Enviada por {usuário}… Revisões por 10 pessoas… Essa informação está errada? Nos avise"**) é a operação desse modelo em campo.

**Assinatura:** vende **serviço** — sem anúncios, offline, cursos, treino de idioma. Nada cede direito sobre música. Detalhe revelador: ao cancelar, **até o conteúdo já baixado é bloqueado**.

### O que isso confirma no projeto
- O **modo teatro / tela cheia** que o próprio Letras oferece mostra o desenho regulatório aceito no mercado: **a ferramenta entrega a exibição; a responsabilidade é de quem a usa.** É exatamente a escolha da ADR-0012.
- O caminho lícito para acervo de terceiro é **permissão pedida, concedida e creditada** — como no meio hinológico ("© Bill H. Ichter — **permissão concedida** à página Hinologia Cristã"; "dados **cedidos gentilmente** por D. Betty Antunes de Oliveira").
- Raspar segue **proibido pelo próprio contrato** deles e desnecessário para nós: derivamos a cifra do áudio e recebemos a letra do usuário sob a licença dele.

## ADR-0012 · Letras.mus.br é da Vignoli, e a assinatura proíbe exibição pública (2026-11-21)

**Status:** 🟢 Investigação fechada com os dois documentos em mãos. Correção importante ao registro anterior.

### O que o dono notou, e estava certo
Ele não achou nada sobre o Letras nos termos que colou — porque **aqueles termos são do Cifra Club**. Fui ao rodapé do Letras e existe documento **próprio**: `/aviso-legal.html`, "Termos de uso e política de privacidade do LETRAS.MUS.BR", assinado por **Vignoli Comunicação Ltda.** — **empresa diferente da Studio Sol**.

### A cláusula que fecha a discussão da projeção
> *"O conteúdo da assinatura é protegido por direitos autorais e disponibilizado mediante **licença limitada, individual, intransferível e exclusivamente para uso pessoal e não comercial**. É vedada sua reprodução, distribuição, **exibição pública**, modificação ou uso comercial sem autorização expressa da Vignoli ou dos respectivos titulares de direitos."*

E a blindagem de conteúdo de usuário, na mesma linha do Cifra Club: *"somos meros 'provedores de aplicativos'… A postagem de conteúdo é, portanto, uma faculdade exclusiva dos nossos usuários"*; *"Postagens devem conter apenas material próprio ou devidamente licenciado"*; *"Nossa responsabilidade se limita a retirar conteúdos do ar… ordem judicial… Marco Civil (art. 19)"*; e ainda removem sem ordem judicial em "violação grave de direitos".

**Consequência dura e clara:** a assinatura do Letras **não cobre** — e **proíbe** — projetar a letra em culto ou transmissão. Não é zona cinzenta: está escrito. O modo teatro que eles oferecem serve para ouvir/ler no dispositivo, sob licença pessoal; a exibição pública é do usuário, com a licença dele.

### Modelo de ASR: o que existe, e o que o `MANIFEST` barra
O dono escolheu **`medium` por padrão com `large`/`turbo` opcional por dispositivo** — registrado — e perguntou por alternativas, citando **`lfm3-audio:2b`**.

- **LFM3-Audio-2B (LiquidAI) existe** e é bem interessante: **2B** parâmetros, **any-to-any** (ASR + compreensão de áudio + **TTS**/voz), **português entre os idiomas**, feito para **rodar local em laptop/edge**, com builds **GGUF (incl. mmproj), ONNX, MLX, CoreML e LiteRT**, ~3,5 GB em BF16, e métricas de ASR publicadas (ESB/Open ASR em inglês; FLEURS multilíngue incluindo português).
- **Mas o portão de licença é decisivo:** os pesos estão sob **"LFM Open License v1.0"** — licença **customizada** (`license: other`), com *Commercial Use* condicionado a **receita anual abaixo de US$ 10 milhões** e **exceção para organização sem fins lucrativos qualificada** em uso não comercial. O `models/MANIFEST.toml` hoje aceita **só** as licenças do `deny.toml` — logo o LFM3 **não entra sem decisão explícita do dono** (aceitar como provedor opcional do usuário, excluir, ou emendar a política de pesos por ADR). Isso é exatamente para o que o MANIFEST existe.
- **Sinergia de desenho que reduz o requisito:** o transcritor **não precisa** devolver timestamps — o motor já acha as **frases vocais** por DSP e alinha as linhas nelas. Um modelo que só devolve texto serve; o tempo vem de nós.

### Decisões e caminhos registrados
- **Spike antes de adotar peso**: `medium` e `large/turbo` numa música com letra conhecida, medindo **WER em português cantado, tempo e memória**. Sem isso, é chute — e a alucinação (linha repetida, verso inventado, refrão em loop) só se revela na escuta.
- **Longo prazo:** as correções do usuário **viram dataset** — e é o argumento mais forte para capturá-las desde o primeiro dia. Um ajuste fino sobre o repertório e as adaptações dele tende a superar qualquer modelo de prateleira, e os dados são **dele**.
- **Hinário impresso confirmado** pelo dono (letra, cifra, partitura, tablatura): o **índice de autores/tradutores está nele**, e é a `base` da ficha. Regra de bolso: **tradutor falecido até 1955 → domínio público no Brasil hoje**.

## R4 · Spike de ASR medido e minuta do termo de uso (2026-11-21)

**Status:** 🟢 Custo medido com número; **acurácia pendente** (falta áudio com letra conhecida) e **medição em smartphone pendente** — exigência do dono.

### Moises: o concorrente mais próximo, e o que ele ensina
- Ferramentas anunciadas: **Stem Separation** (até 27 tipos de stem), **Chord Finder**, **Lyric Transcription**, Speed Changer, Moises Studio. Ou seja: **ele monta cifra e transcreve letra a partir do áudio** — é o concorrente direto do nosso caminho.
- **Postura de licenciamento, nas palavras dele:** *"we only train our models on licensed materials. We never train on any music without the owner's full approval. It's important to us that musicians and other rights holders are paid for their work"*, e no FAQ: *"Moises will never train on your data."*
- **Declarações exigidas do usuário (§11.7):** *"you must not Submit User Content if you are not the owner of or are not fully authorized to grant rights in all of the elements… Moises disclaims any and all liability in connection with User Content. You are solely responsible for your User Content and the consequences of providing [it]"*.
- **Propriedade:** o usuário **retém** direitos sobre o conteúdo **e sobre o "Output"** (incluindo stems, transcrições, análises), com a ressalva de que a lei varia para conteúdo assistido por IA.
- **Cláusula §12.3 (Real-Time Stem Separation)** é o retrato mais claro do setor: uso **pessoal**, *"you may process only audio that you own or have all rights, licenses…"*, e **"Moises does not grant you any rights in third-party content nor authorize you to separate, reproduce, perform, route, record, distribute, or otherwise use any third-party content"** — e o processamento é **local no dispositivo**.
- **Rede de segurança deles:** mantêm uma **coleção curada e licenciada** ("Moises Collection") para demonstração, proibindo usá-la para treinar modelos ou exportar.
- **Leitura honesta:** eles resolvem isso sendo **grandes e pagando licença de treino**, e empurrando responsabilidade para o usuário no conteúdo. Nós resolvemos sendo **locais, sem catálogo e sem treino com dado de usuário** — mais barato e mais defensável, e a nossa ADR-0012 já era exatamente isso.

### Spike de ASR: custo medido (música de 400 s, português cantado, Mac/Metal, `whisper.cpp`)

| Modelo | Disco | Tempo | Memória máx |
|---|---|---|---|
| `tiny` | 78 MB | 11,5 s | 341 MB |
| `base` | 148 MB | **2,6 s** | 485 MB |
| `small` | 488 MB | 10,8 s | 935 MB |
| `medium` | 1.463 MB | 17,1 s | **2,3 GB** |

**Dado que importa mais que a tabela: o modo de falha, medido.** Duas métricas cruas por modelo — quantos segmentos foram rotulados como música (`[música]`, ou seja, **não transcritos**) e a **maior sequência de linhas idênticas seguidas** (loop de invenção):

| Modelo | Segmentos | `[música]` | Loop máx (linhas) | Palavras |
|---|---|---|---|---|
| `tiny` | 95 | 3 | **82** | 555 |
| `base` | 26 | **12** | 6 | 234 |
| `small` | 90 | 0 | 13 | 515 |
| `medium` | 95 | 0 | **4** | 322 |

Leitura honesta: o `tiny` repetiu a **mesma linha inventada 82 vezes seguidas** (os 4,5 minutos medidos antes); o `base` **deixou 12 dos 26 segmentos como "[música]"** (não transcreveu metade); o `small` cobre o canto mas ainda **encadeia 13 repetições**; o **`medium` é o primeiro que não joga trecho em "[música]" nem entra em loop longo** — o que sustenta com dado a escolha dele como padrão. Ressalva: **isto é métrica de modo de falha, não de acurácia** — menos palavras pode significar verso pulado. A acurácia exige letra conhecida (WER), ainda pendente.

**Falha do meu harness, registrada:** a primeira execução do `medium` abortou porque o gatilho de espera disparou **32 MB antes do download terminar** e o whisper carregou arquivo truncado. Corrigido e remedido — número acima é do arquivo íntegro (1.533.763.059 bytes).

Cuidado metodológico registrado: no mesmo modelo, **quanto mais ele alucina, mais tempo leva** (o `tiny` gastou mais que o `base` porque inventou minutos de texto).

### Minuta do termo de uso
Criada em [`docs/legal/termo-de-uso.md`](legal/termo-de-uso.md) com 14 cláusulas e **fundamento rastreável cláusula a cláusula** (quadro "Fundamentos"), incluindo: declarações e garantias do usuário, escopo privado por padrão, sem garantia de exatidão (§7.2 — ancorada na medição acima), isenção e indenidade, canal de notificação com prova de titularidade, registro do ato e a cláusula que não se omite: **"este termo não substitui licença de titular"**. Marcada como **minuta para revisão jurídica**, com `<...>` onde falta decisão do dono. Categoria nova declarada no índice documental (`README.md` deste diretório, seção `legal/`).











## R4 · Spike de ASR em três músicas reais, e as primeiras fichas do hinário (2026-11-21)

**Status:** 🟢 Custo **e** qualidade medidos em material real. O dono mandou os MP3 e as fotos do hinário impresso.

### As três músicas escolhidas, e por quê
| Papel no teste | Música | Duração |
|---|---|---|
| **Âncora (WER)** | *Essência da Adoração* — David Quinlan e Mari Rocha | 400 s |
| **Repertório real** | *Galileu* — Fernandinho (está na planilha do repertório) | 315 s |
| **Outro estilo** | *Incondicional* — Oficina G3 (banda, mixagem densa) | 258 s |

A âncora foi escolhida porque **a letra dela já estava em contexto** — o dono colou a cifra completa do CifraClub numa mensagem anterior —, o que dá verdade de referência legítima sem material novo. **Descoberta no caminho:** este MP3 é **a mesma gravação** que o link do YouTube do primeiro incremento: LUFS integrado idêntico (**−12,6**), duração idêntica (400,4 s) e mesma curva de dinâmica. A música que eu analisei às cegas no começo tem nome: *Essência da Adoração*, David Quinlan e Mari Rocha.

### Custo (Mac/Metal, `whisper.cpp`)
| Música | `small` | `medium` |
|---|---|---|
| âncora (400 s) | 9,1 s / 974 MB | 16,2 s / 2.307 MB |
| repertório (315 s) | 4,6 s / 964 MB | 18,7 s / 2.336 MB |
| banda (258 s) | 3,9 s / 984 MB | 9,9 s / 2.272 MB |

**`medium` roda a ~25× o tempo real** → o repertório inteiro (313 músicas × 5 min) sai em **~63 min de CPU neste Mac**. O gargalo não é tempo: é **memória — 2,3 GB de pico**, que é o que decide no smartphone.

### Qualidade: o que o número mostra (e o que ele não mostra)
- **Na âncora**, WER do bloco conhecido (pré-refrão, 31 palavras): **3% no `small` e 10% no `medium`** — os dois transcrevem bem o canto; a diferença nesse tamanho de bloco não é sinal.
- **`repertorio-small` classificou a música inteira como "[música]"** — **25 de 25 segmentos**, apenas 27 palavras transcritas —, enquanto o **`repertorio-medium` produziu 263 palavras com 1 único "[música]"**. É o achado que **sustenta a escolha do dono com dado**: em material de repertório real, o `small` pode simplesmente não transcrever.
- Loops caíram muito em relação ao `tiny`/`base` (1 a 4 linhas idênticas, contra **82** do `tiny`): a alucinação em loop é fenômeno dos modelos pequenos.
- **Ressalva metodológica registrada:** o WER de bloco só vale para a música cuja letra se conhece. Os 90–94% medidos nas outras duas **não são qualidade** — são a ausência daquele bloco nelas. Reportar isso como nota seria mentir.
- **Cuidado de pipeline:** o WAV de 16 kHz serve para a ASR, **não** para a análise musical — o mesmo áudio deu 133,9 BPM em 16 kHz contra **129,2 BPM** em 44,1 kHz. Análise roda na taxa original.

### As primeiras fichas lidas do hinário impresso (fatos, sem letra)
O **Cantor Cristão Cifrado (2011)** imprime a ficha **em cada hino**: `Autoria:`, `Tradução:`, `Melodia:` — **com anos de nascimento e falecimento**. Três páginas fotografadas:

| Hino | Título | Autoria | Tradução | Melodia | Veredito |
|---|---|---|---|---|---|
| 01 | Antífona | Maxwell Wright (1849-1931) | — | William H. Monk (1823-1889) | **domínio público** (BR: 1931+71 = 2002) |
| 03 | Louvor ao Senhor | William E. Entzminger (1859-1930) | — | Robert Lowry (1826-1899) | **domínio público** (BR: 1930+71 = 2001) |
| 09 | **Santo** | Reginald Heber (1783-1826) | **João Gomes da Rocha (1861-1947)** | J. B. Dykes (1823-1876) | **domínio público** (BR: 1947+71 = **2018**) |
| 15 | **Exultação** | Fanny Jane Crosby (1820-1915) | Joseph Jones (1848-1927) | William Howard Doane (1832-1915) | **domínio público** (BR: 1927+71 = **1998**) |
| 46 | **Jesus me transformou** | James Rowe (1865-1933) | Salomão Luiz Ginsburg (1867-1927) | Howard E. Smith (1863-1918) | **domínio público** (BR: 1933+71 = **2004**) |
| 92 | **Substituição** ("Morri na cruz por ti") | Frances Ridley Havergal (1836-1879) | **D. M. Hazlett (1852-?)** | Philip Paul Bliss (1838-1876) | autoria e melodia PD; **tradutor sem data** → **GAP** |
| 112 | **Vencendo vem Jesus** | Julia Ward Howe (1819-1910) | **Ricardo Pitrowsky (1891-1965)** | John William Steffe (sem datas na ficha) | ⚠️ **NÃO é domínio público** — só em **2036** (BR: 1965+71) |
| 135 | **Louvor** ("Louvamos-te, ó Deus") | William Paton MacKay (1839-1885) | James Theodore Houston (1847-1929) | John Jenkins Husband (1760-1825) | **domínio público** (BR: 1929+71 = **2000**) |
| 295 | **Tudo Entregarei** | Judson W. Van de Venter (1855-1939) | Salomão Luiz Ginsburg (1867-1927) | Winfield Scott Weeden (1847-1908) | **domínio público** (BR: 1939+71 = **2010**) |
| 377 | **Não Sei Por Que** | Daniel Webster Whittle (1840-1901) | Justus Henry Nelson (1849-1931) | James McGranahan (1840-1907) | **domínio público** (BR: 1931+71 = **2002**) |
| 385 | **Louvor** | **Manuel Avelino de Souza (1886-1962)** | — | Charles Hutchison Gabriel (1856-1932) | ⚠️ **NÃO é domínio público** — só em **2033** (BR: 1962+71) |

O hino **09** foi a primeira ficha do repertório resolvida, e veio do livro do dono. Consequência de método: a campanha de fichas deixou de ser pesquisa e passou a ser **fotografar a página do hino**.

### Nove dos dez do repertório, e os três vereditos que o material exigiu

As fotos vieram do celular do dono via `adb` (`DCIM/Camera`), em dois lotes — e a ficha traz **número, título, autoria, tradução e melodia com datas**. Somadas as páginas de **377**, **112**, **295** e **135**, o repertório fica com **nove dos dez hinos fichados**:

| veredito | hinos | por quê |
|---|---|---|
| **domínio público** (6) | 09, 15, 46, 135, 295, 377 | a morte mais recente entre autor, tradutor e melodia + 71 anos já passou |
| **protegido** (2) | **385 Louvor** (até **2033**) e **112 Vencendo vem Jesus** (até **2036**) | 385: Manuel Avelino de Souza (1962). 112: Howe (1910) e Steffe são livres, mas **a tradução de Ricardo Pitrowsky (1965) não é** — a obra brasileira é dele |
| **GAP** (1) | 92 Substituição | tradutor **D. M. Hazlett (1852-?)**: o livro não publica o falecimento, então não se afirma a queda |

Falta **um**: **"A mensagem da Cruz" (291 na Harpa Cristã)** — o dono **não tem a Harpa em papel**, então essa ficha precisa vir de outra edição ou ficar declaradamente sem ficha.

**O caso que prova o argumento do dono:** há **dois hinos chamados "Louvor"** no repertório dele — o **135** (domínio público) e o **385** (protegido até 2033). Mesmo título, hinos diferentes, vereditos opostos: **quem identifica é o número**, não o título. É exatamente o que ele disse ao contestar minha leitura do livro ("hino 9 = Santo para qualquer batista"), agora medido no repertório dele.

**Procedência (ADR-0012) na prática:** o "Tudo Entregarei" está na planilha do dono como *Harpa Cristã*, mas foi fotografado do **Cantor Cristão Cifrado nº 295**. A ficha registra **de qual edição** veio o fato — sem isso, duas edições com traduções diferentes do mesmo hino apareceriam como contradição em vez de serem duas obras distintas.

### Fontes externas: o que cada uma entrega, e o que uma delas erra

O dono passou endereços para completar as fichas. Medido o que cada fonte dá — e o resultado muda o desenho do asset:

| fonte | o que entrega | o que **não** entrega |
|---|---|---|
| `revistaebd.com.br/cantor-cristao/<n>` | título e **nomes com datas** | **o tradutor, em todos os hinos que cruzei**, e não separa autoria/tradução/melodia |
| `hinologia.org/cantor-cristao-cc/` | **001–581 número → título** + história das edições (1891; 1924 com música; 37ª ed. = 581 hinos) | ficha por hino |
| `hinologia.org/harpa-crista/` | **número → título** (paginado) | ficha por hino |
| `harpacrista.digital/hinos/291-…` | letra completa (com "todos os direitos reservados" do site) | **nenhum crédito**: autor, tradutor, melodia, nada |
| `revistaebd.com.br/harpa-crista/291` | letra e título | **nenhum crédito** |

**A armadilha, medida:** o `revistaebd` **omite o tradutor em todos os hinos que cruzei** — e o tradutor é exatamente quem decide o veredito em três dos dez hinos dele. No **112** o site lista Steffe e Howe (1819-1910) e **cala o Pitrowsky (1965)**: quem se guiasse por ele concluiria **domínio público**, o oposto do correto. No **92** cala o Hazlett; no **295** cala o Ginsburg.

**E um erro de data:** o site dá **"William Paton MacKay (1847-1929)"** no hino 135, enquanto a **ficha impressa do livro do dono diz William Paton MacKay (1839-1885)**. Hipótese do erro (marcada como hipótese): 1847-1929 são as datas do **tradutor daquele hino no livro**, **James Theodore Houston (1847-1929)** — o site parece ter colado as datas do tradutor no autor e descartado o tradutor. A fonte web não é só incompleta: **erra onde a impressa acerta**.

Conclusão de método: a **ficha impressa do livro do dono permanece a fonte primária**, e cada linha do asset precisa registrar **de onde veio o fato e quais campos aquela fonte trazia** — a procedência do ADR-0012 passa a ser obrigatória **por campo**, não por hino.

### O número só identifica junto com o hinário

O dono avisou: *"o 291 da Harpa Cristã não existe no Cantor Cristão — naturalmente ele não é um hino batista tradicional como os presentes no Cantor Cristão"*. **Conferido em fonte independente: o Cantor Cristão nº 291 é "Comunhão Celeste"**, outra obra. Isso **corrige o meu próprio refinamento** anterior: eu havia dito que a numeração é convenção compartilhada e que é ela que liga a planilha ao hino — vale **dentro de cada hinário**, não entre hinários. A identidade de um hino é o **par (hinário, número)**, e é isso que a ficha precisa guardar. É a mesma razão pela qual o "Tudo Entregarei" da planilha (anotado como *Harpa Cristã*) apareceu fotografado do **Cantor Cristão nº 295**: sem o hinário, o número mente.

### A Harpa Cristã ficou sem ficha — e isso é estado legítimo, não pendência disfarçada

Nenhuma das fontes disponíveis publica a ficha do **291 A Mensagem da Cruz**: nem autor, nem tradutor, nem melodia. Esse hino — um dos que ele canta, que trouxe de outra igreja batista — fica **declaradamente sem ficha**, em vez de preenchido por inferência. O que é público sobre "The Old Rugged Cross" (**George Bennard, 1873-1958** → domínio público no Brasil só em **2029**) fica registrado como **não verificado no livro**, e o **tradutor da versão em português da Harpa não é conhecido por estas fontes** — o que, por si só, já impediria veredito. A Harpa (1922, oficial das Assembleias de Deus) carrega camada de tradução brasileira do século XX: se o 291 seguir o padrão do 112, o texto em português **provavelmente não está livre**.

**Nota que amarra a camada protegida:** os dois hinos protegidos do repertório dele são dos **dois principais colaboradores brasileiros do Cantor Cristão** — **Manoel Avelino de Souza (29 hinos)** e o **Pastor Ricardo Pitrowsky (23 hinos)**, segundo o próprio hinologia.org. Não é azar: em repertório batista, a camada protegida é justamente a **tradução brasileira do século XX**.

**Nota de mapeamento (correção de um erro meu nesta sessão):** o campo da planilha diz "Exultação | Cantor Cristão **015**" e isso é o **hino 15** — não o hino 01. O hino **01 é *Antífona*** (Wright/Monk), outra obra, que eu havia juntado com a Exultação numa mensagem anterior sem que a foto tivesse sido lida; a foto do livro desfez o nó. Fica registrado porque é a diferença entre **ler a fonte** e **inferir do número**.

Duas naturezas que o desenho tinha previsto como hipótese e que o repertório real trouxe:

- **Obra protegida (385 *Louvor*):** **Manuel Avelino de Souza morreu em 1962** → domínio público só em **2033**. É o caso em que o app **não oferece a letra**: mostra a **ficha** (fato) e trabalha com a **letra e a cifra do próprio usuário**. Deixa de ser hipótese de ADR e passa a ser dado do repertório do dono — um dos hinos que ele canta no culto.
- **Tradutor sem data (92 *Substituição*):** a própria ficha diz **"D. M. Hazlett (1852-?)"** — o livro **não traz o falecimento**. Havergal (1879) e Bliss (1876) são domínio público, mas **a tradução é obra do tradutor**, e sem a data não se pode afirmar a queda. O veredito é **GAP**, não "PD" — e essa distinção é a diferença entre **medir** e **supor**: assumir "PD" aqui seria o palpite silencioso que o ADR-0012 proíbe. A tabela precisa, portanto, de **três vereditos** (domínio público com ano, protegido, GAP), não dois.

### Achado legal que muda o que se pode ingerir do livro
A ficha catalográfica do **Cantor Cristão Cifrado** (JUERP/EME, 2011) diz: **"Todos os direitos reservados"**, *"É proibida a reprodução total ou parcial deste livro… por quaisquer meios: eletrônicos, mecânicos, **fotográficos**, xerográficos, gravação…"*, e ainda que o livro **"contém o nosso sistema de cifragem"** (incluindo o "**SOLOS POR NÚMEROS**", o sistema de tablatura deles).

Leitura honesta: **fato não é protegido** — autor, tradutor e datas eu posso ler e registrar (e é o que sustenta a ficha). Mas **a letra transcrita e a cifra daquela edição são do editor**, e o *sistema de cifragem* é reivindicado por eles. Nosso desenho não precisa de nada disso: **derivamos a cifra do áudio** e a letra vem do usuário. As fotos **não entram no repositório** e ficaram em `/tmp/hinario`.

### Refinamento · o número do hino é convenção, não propriedade do editor

O dono contestou — com razão — a leitura de que o livro inteiro seria "intocável": *"eles não são os únicos que imprimem um cantor cristão **e o número da canção é padrão nas igrejas** para o título e a música… se eu for em qualquer igreja batista no Brasil e falar 'Cantor Cristão hino 9', todos saberão que se trata do louvor 'Santo'"*.

Ele está certo, e a "contradição" que ele percebe é **aparente**, não real: **direito exclusivo sobre uma EDIÇÃO convive com convenção denominacional compartilhada**. O que a edição cifrada de 2011 (JUERP/EME) reivindica é o **trabalho editorial dela** — o *sistema de cifragem*, a diagramação, os arranjos/transcrições, o "solos por números". O que ela **não** reivindica (nem poderia):

- **a numeração e a identidade do hino** — convenção de igreja, como a numeração de versículos da Bíblia;
- **fato de autoria** — quem escreveu, quem traduziu, quando morreu;
- **texto em domínio público** — o que já caiu não volta a ser de alguém por aparecer numa edição nova.

Correção que eu faço nos meus próprios registros: a frase "a letra transcrita e a cifra daquela edição são do editor" continua válida para **a cifra e a transcrição daquela edição**; passou a valer mal-entendido se lida como "o hinário inteiro é deles". O caminho do projeto — **fatos para a ficha, cifra derivada do áudio, letra do usuário** — não encosta em nada reivindicado pelo editor, e ainda ganha com o argumento dele: a **numeração compartilhada** é justamente o que liga "377 Cantor Cristão" da planilha ao hino do livro.

### O 291 da Harpa Cristã: fechado com procedência por campo (2026-11-21)

A Harpa não veio do livro do dono (ele não tem o exemplar em mãos): veio de **PDFs baixados**
(`~/Downloads/harpa/`). Os dois hinários são **digitalizações sem camada de texto** — o texto saiu
por **OCR nativo do macOS (Vision)**, em ambiente isolado (`/tmp/pdfvenv`: `pymupdf` +
`pyobjc-framework-Vision`; nada instalado no sistema). O hino foi localizado por **calibração
página↔número** (Cifrada: p.273 = 313, p.300 = 345 → **p.254 = 291**) e o crédito saiu das
**iniciais impressas no próprio hino** — o sistema de siglas da Harpa (`AA.` no 291, `P.L.M.` no
292, `D.L.` no 312, `S.N.` no 290). O elo final foi fechado **na fonte primária**: o **"Índice de
Autores e Tradutores"** do hinário oficial da CPAD (p.722) traz **`A.A. → Antônio Almeida`**.

| campo | valor | fonte |
|---|---|---|
| número/título | 291 A Mensagem da Cruz | hinário (Cifrada p.254; CPAD p.717) — **primária** |
| sigla impressa | `AA.` | hinário (Cifrada p.255) — **primária** |
| sigla → nome | **Antônio Almeida** | **índice do próprio hinário (CPAD p.722)** — **primária** |
| datas | 1879-1969 | hinologia.org (biografia) — secundária |
| original | "The Old Rugged Cross", George Bennard (1873-1958) | doctypemusic — secundária, não no livro |

**Correção desta tabela (2026-11-21, ao auditar o registro da sessão):** a saída de OCR está no
histórico e diz **`A.A. - Anlúniv Almeida`**. O nome **Antônio Almeida** é **correção de leitura**,
sustentada apenas por fonte secundária (hinologia.org, 1879-1969) — **não** é o que está impresso, e a
linha "primária" acima está **errada**. O que a fonte primária sustenta, conferido nos arquivos de OCR
ainda em `/tmp`: a **sigla `AA.` no pé do hino** (p.255, linha 6) e a **existência da entrada `A.A.` no
índice** (p.722, linha 3) — nada além disso. O ano **2040 fica provisório** até a conferência a olho do
índice impresso; o veredito permanece **protegido** porque é a direção segura, e o asset passou a
registrar isso na própria linha (`fonte_traducao` + `observacao`).

**Confirmado (22:12):** o dono enviou a **imagem legível da p.722** e ela traz **`A.A. - Antônio Almeida`** —
a identificação do nome está **fechada na fonte primária**, e a linha "sigla → nome" da tabela acima passa
a valer por **leitura**, não por inferência. O que continua secundário é a **data** (1879-1969,
hinologia.org): o índice dá nomes **sem anos**, então **2040 = 1969+71** é aritmética sobre fonte
secundária. O registro do erro de método acima fica de pé — o que mudou foi a evidência.

**Veredito: PROTEGIDO até 2040** (1969+71) — a versão em português é a camada que manda, e é a mais
longa dos dez. A edição é **CPAD © 2007**, "todos os direitos reservados para a língua portuguesa":
mesma regra do Cantor Cristão Cifrado — **só fatos**; PDF e fotos fora do repositório.

**O "padrão assembleiano", medido:** a Harpa dá **sigla + índice de nomes, sem anos**; o Cantor
Cristão Cifrado dá **Autoria/Tradução/Melodia com datas**. A Harpa custa **três fontes por hino**
(hino → índice → biografia) para o que o Cantor Cristão resolve numa linha. O dono apontou isso
antes de eu medir, e ele está certo.

**Placar final do repertório: 10 de 10 com veredito** — 6 em domínio público (09, 15, 46, 135, 295,
377), 3 protegidos (385→2033, 112→2036, **291→2040**) e 1 GAP (92, tradutor sem data).
**Quatro dos dez exigem o caminho "ficha + letra do usuário"; os outros seis podem ter letra livre.**

## R4 · Medição no aparelho: `whisper` no Motorola Edge 50 Neo (2026-11-21)

O dono exigiu o número **no aparelho dele**, não uma estimativa. Feito: `whisper.cpp` compilado para **`arm64-v8a` com o NDK 28.2** e executado **no próprio celular**, sobre o **mesmo áudio âncora de 400,4 s** (16 kHz mono) da linha de base do Mac — para ser comparável, e não uma corrida entre áudios diferentes.

**Aparelho (lido do sistema, não de catálogo):** `motorola edge 50 neo`, SoC **MT6878** (MediaTek), **Android 16 / SDK 36**, `arm64-v8a`, **8 núcleos**, **MemTotal 7.608.552 kB ≈ 7,26 GiB**, 70 GB livres.

**Método:** `VmHWM` (pico real, do `/proc`) amostrado 1×/s **dentro do aparelho** — sem ida-e-volta pelo `adb` a cada amostra; tempo lido do `whisper_print_timings` **e** conferido contra o relógio de parede (bateram: `total time` 220.042 ms contra 220 s medidos por fora); texto comparado **no mesmo formato** dos dois lados (`-otxt`).

| áudio âncora (400,4 s) | `small-q5_1` no celular | `small-q5_1` no Mac | `medium-q5_0` no Mac |
|---|---|---|---|
| tempo total | **220 s (3 min 40 s)** | 5,76 s | 13,67 s (8 threads) |
| fator sobre o tempo real | **1,82× mais rápido** | 69× | 29× |
| encode por janela | 7,67 s (16 janelas) | — | — |
| **pico de memória** | **570.132 kB ≈ 557 MB** | 632 MB | 1.250 MB |

**`medium-q5_0` no celular** (trecho de 60 s, 8 threads): **89,7 s**, encode **22,55 s por janela**, load 0,89 s, pico **1.162.076 kB ≈ 1,11 GB**. Ou seja: o `medium` é **3,1× mais lento** que o `small` no aparelho (contra 2,5× no Mac) — é mais limitado por banda de memória.

### O que isso decide

1. **A memória deixou de ser o limite duro.** O obstáculo era o **2,3 GB** do `medium` fp16; quantizado, o `medium` cabe no aparelho com **1,11 GB** e o `small` com **557 MB**, contra 7,26 GiB de RAM.
2. **O tempo virou o limite.** O hino de 6:40 leva **3 min 40 s** com `small` no celular; projetando o `medium` (22,5 s por janela × 16 + tokens), **~10 min** — não é interativo nem aceitável como padrão no aparelho. Como a medição anterior sustentou o **`medium` como padrão de qualidade**, a conta empurra para **processar no servidor/desktop** (ou lote noturno no carregador) e usar o aparelho para capturar e revisar. **Não decido isso sozinho:** onde o `medium` roda é escolha de arquitetura do dono.
3. **Throttling térmico é medido, não suposto:** a janela de encode do `small` foi de **6,99 s** (rodada de 60 s) para **7,67 s** (rodada de 400 s) — **+9,7%** de degradação sustentada. Estimativa de lote feita com rodada curta **subestima** o tempo.
4. **8 threads vencem 4 no celular:** 29,2 s contra 33 s no trecho (encode 6,99 contra 8,13 s) — **12%**. E os 8 núcleos são 4×A78 + 4×A55: o ganho vem com os pequenos junto.
5. **Reprodutibilidade depende do tamanho do modelo:** o `small` deu saída **quase idêntica** no Mac e no celular (só a última linha difere: "antes de meu chego" contra "aonde meu chego"); o `medium` divergiu mais (`[Música]` contra `Música`, frases diferentes). Para resultado estável entre plataformas, `small` é mais previsível.
6. **Qualidade (mesmo trecho):** o `medium` acertou o que o `small` errou — *"Anseando oferecer / … Pra abençoar teu coração"* contra *"Anciam vou oferecer / … Pra vencer o ar teu coração"*. Confirma o achado anterior, agora no trecho curto.

### Lições de mecânica (custaram tempo nesta sessão)

- **`whisper-cli` sai linkado em `.so`, não estático.** Só rodou no aparelho depois de empurrar `libwhisper.so` + `libggml{,-base,-cpu}.so` **e** usar `LD_LIBRARY_PATH` **absoluto** — o `.` relativo **não resolve** no linker do Android. Para o app é a mesma família de problema da skill `empacotamento-rust-flutter`: link estático ou empacotar as `.so`.
- **O `adb` não vê o aparelho só por ele estar no USB.** O celular expunha **uma** interface (`0xFF/0xFF/0x00`, nomeada "MTP") e **nenhuma** de ADB (`0xFF/0x42/0x01`): com a depuração inerte **não existe diálogo a aceitar** — o diálogo é consequência da interface, não causa. Depois de religar o switch **com o cabo conectado**, o celular re-enumerou e a interface nasceu (`ADB Interface@1`), flagrada ao vivo por um observador de 2 min.
- **`system_profiler SPUSBDataType` devolve vazio neste shell** (sem permissão); a ferramenta é o `ioreg` (`IOUSBHostDevice`/`IOUSBHostInterface`). Foi o que **evitou** o diagnóstico errado de "troque o cabo": o aparelho sempre esteve no barramento, com o nome do modelo e o serial.
- **Contagem de palavras só vale no mesmo formato.** Minha primeira comparação (620 contra 311) era **artefato meu**: a rodada do Mac imprimiu timestamps e o `wc -w` contou os tokens de tempo. Refeito com `-otxt` dos dois lados: **41 contra 40** no trecho. Medição comparada em formatos diferentes não é medição.

## R4 · Fichas do hinário viram asset com gate e prova negativa (2026-11-21)

O repertório de dez hinos deixou de viver só como prosa no PENDING: virou
`crates/mir_engine/assets/fichas_hinario.tsv`, **12 fichas** chaveadas pelo par **(hinário, número)** —
**8 em domínio público, 3 protegidas e 1 GAP**. A procedência é **por campo** — `fonte_autoria`,
`fonte_traducao` e `fonte_melodia` —, porque a ficha do 291 só se sustenta em três elos: sigla
impressa (`AA.`) → índice de autores do próprio hinário (CPAD p.722, **lido por OCR** como
`Anlúniv Almeida`) → biografia (hinologia.org, 1879-1969), e o nome foi **depois confirmado a olho** na
imagem legível do índice (`A.A. - Antônio Almeida`). O gate passou a exigir `observacao` sempre que a
procedência citar OCR.

**O limite legal virou mecanismo, não recomendação.** `scripts/verificar_fichas.py` reprova veredito
fora de `pd|protegido|gap|sem_ficha`, `pd`/`protegido` sem ano, `gap` sem observação, `fonte` vazia,
campo acima de 300 caracteres e — o que importa — **qualquer coluna de letra, cifra ou tablatura**.
A proibição de reproduzir a edição (EME/JUERP 2011; CPAD © 2007) deixa de depender de alguém lembrar.

**Provado no sentido negativo:** `scripts/prova-verificar-fichas.sh` gera quatro variações do asset que
**têm** de reprovar — `pd` sem ano de domínio público, `protegido` sem ano de morte, `fonte` vazia e
coluna `letra` — e falha se alguma passar. Medido: `APROVADO: o gate reprovou os 4 casos negativos`.

**Duas reprovações reais só apareceram quando o commit passou a existir.** O push anterior ("nada a
commitar") deixou os gates `lint` e `shell` do pre-push com `CHANGED_FILES` vazio: `ruff format --check`
reprovou `scripts/verificar_fichas.py` e `shellcheck` apontou **SC2164** (`cd` sem `|| exit`) no script
de prova. Corrigidos antes do commit — gate que nunca viu o arquivo não certifica nada.

**Fora desta entrega, declarado:** o asset **ainda não é lido por código nenhum** (`lyrics.rs` continua
com `hinos_dominio_publico.tsv`, intacto) e o gate **não está no `.githooks/pre-push`** — a decisão de
2026-11-21 manteve o contrato `[validation]` em gitleaks, então o wiring é edição de hook, com prova
própria. Ambos ficam em PENDING.

**Skills:** `hinario-e-dominio-publico` (identidade = par hinário+número; três vereditos mais
`sem_ficha`; conta pela morte mais recente entre autoria, tradução e melodia; fonte primária é o
impresso) e `ocr-em-digitalizacao` (calibrar página↔número antes de OCR; sigla → índice do próprio
hinário; nome próprio saído de OCR não se aceita sem conferência). As duas com o mesmo frontmatter das
outras 18.

## R4 · Gate das fichas na bateria, provado pelo próprio hook (2026-11-21)

A entrega anterior deixou o asset e o verificador na `main`, mas **fora de qualquer gate automático**:
`scripts/verificar_fichas.py` só rodava se alguém lembrasse. O gate entrou no `.githooks/pre-push`
(escopado ao `CHANGED_FILES`: asset, verificador, prova ou o próprio hook) e no `scripts/ci-local.sh`
(bateria inteira). O contrato `[validation]` do `.harness.toml` **não** mudou — segue só com gitleaks,
conforme a decisão de 2026-11-21 —, mas o arquivo passou a dizer **onde** o gate das fichas vive, para
a próxima sessão não procurar no lugar errado.

**A prova é no sentido negativo, exercitando o hook:** com uma ficha `pd` sem `ano_dominio_publico`, o
push é recusado — `❌ fichas`, `PUSH BLOQUEADO — gate(s) reprovado(s): fichas`, e o log do gate traz
`linha 2: veredito "pd" sem ano de domínio público`. Restaurado o asset, ele volta idêntico (md5
conferido) e o mesmo hook passa com `✅ fichas`.

**Defeito da ferramenta achado no caminho:** o hook chamava `avisa_base "$1"` e, fora do `git push`,
`$1` não existe — com `set -u` ele abortava **antes de qualquer gate**, o que tornava impossível provar
um gate manualmente. Passou a `"${1:-origin}"`.

**Custo medido:** 0,1 s (12 fichas + as 4 variações da prova).

## R4 · O corpus que "não existia": verificado no repositório, não na memória (2026-11-21)

A sessão anterior parou porque o agente pediu ao dono que **colasse** o conteúdo de
`crates/mir_engine/assets/hinos_dominio_publico.tsv`. O dono respondeu que não tinha o arquivo e que
"era para o modelo ter gerado e pelo jeito não gerou". O arquivo **existia**, e a verificação custou
três comandos: está commitado desde `e9f388f` (ampliado em `1cc5d91`, último toque em `7c67d0d`, 13:58
de hoje), tem **40 linhas** — 35 de comentário e 4 hinos — e **9 campos posicionais**, o último o
`verso`; e é lido em **tempo de compilação** por `lyrics.rs`, via `include_str!`. Sem ele o
`cargo clippy` reprovaria o push, e o push estava verde.

**A lição não é sobre o arquivo, é sobre o caminho:** pedir ao dono um dado que está no repositório é
sintoma de agente sem leitura — e o custo aparece como sessão perdida, não como erro. O que ficou
disso:

- o asset de fichas ganhou **cabeçalho `#`** que documenta o formato e a relação com o corpus — os
  dois **não se fundem**: lá é letra, aqui é fato;
- o verificador passou a **ignorar `#`** (mesma convenção do corpus) e a prova ganhou o **5º caso
  negativo**: arquivo só com comentários e cabeçalho tem de reprovar;
- o vínculo está escrito no asset, com exemplo real: *Holy, Holy, Holy* (Heber, 1826) no corpus ↔
  **09 Santo, Santo, Santo** (trad. João Gomes da Rocha, 1947 → PD 2018) nas fichas — mesma obra
  original, duas obras distintas, que é a regra 2 do corpus.

## R4 · A ficha virou mecanismo no motor (2026-11-21)

O asset deixou de ser dado inerte. `mir_engine::fichas` lê o TSV embutido e responde a pergunta que o
produto faz — *esta versão pode ter letra?* —, e três decisões de desenho resolvem o que o material real
expôs:

1. **A resposta é derivada do ano, não da coluna.** `ano_dominio_publico` é o fato; `veredito` é o
   retrato do dia da conferência. Em 2040 o 291 vira `Livre` **sem ninguém editar o asset**, e a ficha
   continua dizendo a verdade de 2026. Provado nos dois lados: 2017 → `RequerLetraDoUsuario`, 2018 →
   `Livre` (09); 2039 → protegida, 2040 → livre (291).
2. **Título não é identidade — e são três "Louvor".** A busca por título devolveu **três** candidatas
   (03 "Louvor ao Senhor", 135 "Louvamos-te, ó Deus", 385 "Louvor"), duas delas com vereditos opostos. O
   motor devolve `Ambiguo` com as candidatas nomeadas, e o relatório de uso classifica como
   `Indeterminado` em vez de escolher. O teste que eu escrevi esperava duas; o asset mostrou três — e o
   número real ficou no teste e na skill.
3. **A regra é uma só, e a auditoria prova a fronteira.** `obra_da_ficha` reconstrói a `Obra` para
   `veredito_pd`: em cada linha, a obra tem de estar **protegida** em `ano - 1` e **livre** em `ano`. É
   independente da data de hoje, então não vira bomba-relógio. O gate ganhou a mesma aritmética
   (`ano_dominio_publico == morte + 71`) e a prova foi a **7 casos negativos**.

**CLI:** `--ficha-hino "Cantor Cristão|09"` imprime a ficha com a procedência de cada campo e a resposta;
`--listar-fichas` mostra as 12 com o desfecho; `--auditar-fichas` prova a fronteira. E `--hino` sem letra
no corpus passa a **explicar pela ficha** — livre, protegida, gap ou ambígua — em vez de só dizer "não
está no corpus".

**Fora desta entrega:** o vínculo explícito entre a ficha e a **obra original** do corpus (*Holy, Holy,
Holy* ↔ CC 09) e o app honrar a resposta (`lyrics_scope`).

## R4 · O vínculo ficha ↔ corpus e o portão de publicação no app (2026-11-21)

As duas pendências que a entrega anterior nomeou — e que não eram dado faltando, eram **capacidade**
faltando — foram fechadas no mesmo dia.

**1. O vínculo é declarado, não adivinhado.** A coluna `obra_original` nomeia a obra do corpus de que a
versão deriva: o 09 declara `Holy, Holy, Holy`. O vínculo é de **referência**, não de identidade — a
tradução de João Gomes da Rocha (1947) é obra própria e responde pelo veredito dela; Heber (1826)
responde pelo dele. `fichas::obra_original_no_corpus` resolve o título no corpus e devolve a `Obra`, e
`obra_da_ficha` carrega o vínculo na `Obra` que reconstrói. O gate ganhou duas provas: **obra original
que não está no corpus** e **autor que não confere** — ambas reprovam. Semelhança de título não vira
vínculo: quem declara é a ficha.

**2. O portão de publicação chegou ao app, sem duplicar a regra.** A ponte (`kentron_ffi::pode_publicar`)
expõe `mir_engine::sheet::pode_publicar` — o app pergunta, o motor decide. `linkDeProjecao` só monta o
link de projeção depois do portão; escopo privado do grupo lança `PublicacaoBloqueada` e a tela mostra o
motivo (`stageProjectionBlocked`), em vez de copiar a regra de licença para o Dart. O widget recebe o
portão por injeção (`portaoDePublicacao`), como já fazia com `ler` e `transpor` — os testes não tocam em
FFI.

**O que a entrega mostra de desenho:** a regra de licença mora **num lugar só**. O motor já reprovava
publicação; o app passou a **obedecer** em vez de reimplementar. E o vínculo entre ficha e corpus
continua **dado**, não código: o motor lê a coluna e confere no corpus, mas não infere vínculo por
semelhança de título.

**Limite declarado:** a exportação não existe no app ainda — o portão cobre o link de projeção hoje e
serve à exportação quando ela nascer. E o vínculo é declarado à mão: ficha nova sem `obra_original` não
ganha vínculo por heurística.
