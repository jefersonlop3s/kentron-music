# Changelog

Todos os eventos notáveis do projeto Kentron Music Hub.

## [R1] Android local pronto e ponte provada no navegador (2026-11-20)
### Verificado
- **Prova no navegador:** a tela web mostrou dados vindos do núcleo Rust — `Exemplo`, `Kentron · tom C · 2 seção(ões)`, acordes `C G Am F` e `núcleo: kentron_ffi 0.0.0`. Cadeia Rust → wasm → Flutter web → tela provada no browser.

### Toolchain Android (medida)
- `flutter doctor`: **Android toolchain ✓** (SDK 36.0.0), com Java **Temurin 27**; comando de versões tirado do próprio SDK: compileSdk/targetSdk 36, minSdk 24, **NDK 28.2.13676358**, Gradle 9.5.0, AGP 9.3.1, KGP 2.4.20.
- Pacotes instalados em `~/Library/Android/sdk`: build-tools 36.0.0, emulator 37.3.10, NDK 28.2.13676358, platform-tools 37.0.3, platforms;android-36, system-images;android-36;google_apis;arm64-v8a. AVD **`kentron`** criado.
- `flutter config --android-sdk` aponta para esse SDK.

### Armadilhas registradas
- O `avdmanager` **do cask** resolve o SDK root pela própria localização (o cask), onde não há system-images — daí `Package path is not valid ... null`. O SDK root só fica íntegro com uma **cópia real** de `cmdline-tools` dentro dele (symlink não engana a detecção). O SDK em `~/Library/Android/sdk` também fica imune a `brew upgrade`.
- `flutter doctor` avisa de **dois `adb`** (SDK + cask `android-platform-tools`) e existem **dois Flutter** (o do pipeline em `~/development/flutter` e o do cask). Ver PENDING.

## [R1] Tela inicial consumindo o núcleo Rust (2026-11-20)
### Adicionado
- `HomeScreen` deixou de ser estática: chama `parseSong` e mostra título, artista/tom/seções, os acordes em chips e **qual núcleo respondeu** (`núcleo: kentron_ffi …`) — a ponte visível para quem usa o app.
- `cifraDeExemplo`: cifra **só de acordes** (sem letra de terceiros) usada pela tela; o objetivo é provar a ponte, não entregar repertório.
- Carregador injetável (`carregar`) em `KentronApp`/`HomeScreen`: o default é o núcleo real, e os testes herméticos injetam outro.

### Verificado
- `bridge_test.dart`: teste de **widget com o `cdylib` real**, provando a tela renderizar o parse do Rust (título, tom, seções, acordes e o núcleo que respondeu). `flutter test`: 8/8 verdes; `flutter analyze`: sem issues.
- O parse de linhas **só com acordes** (`[C] [G] [Am] [F]`) ficou provado por esse teste.

### Decisão de teste
- O teste com a ponte real **não** usa `pumpAndSettle`: o indicador de carregamento anima indefinidamente e a resposta chega por uma porta do FFI, que o relógio falso do teste não avança. A espera é em tempo real, via `runAsync`, até o indicador sair da árvore.

### Não provado
- A execução **no navegador**: o `build/web` está servível, mas ninguém abriu ainda. Ver a nota no PENDING sobre canvas/DOM antes de investir em E2E automatizado.

## [R1] Alvo web do app (2026-11-20)
### Adicionado
- `apps/kentron_app/web/`: scaffold web (`flutter create --platforms=web`) com `index.html` carregando o glue wasm-bindgen (`pkg/kentron_ffi.js`) e título/descrição do Kentron.
- `scripts/servir-web.py`: servidor estático que envia `Cross-Origin-Opener-Policy: same-origin` e `Cross-Origin-Embedder-Policy: require-corp` (sem eles não há `SharedArrayBuffer` e a ponte wasm não carrega), além de `application/wasm`; falha alto com instrução quando o build não existe.

### Alterado
- `apps/kentron_app/analysis_options.yaml`: `web/**` fora do analyzer (o próprio `flutter create` acrescenta).
- Removido o `README.md` boilerplate que o `flutter create` gerou: documentação vive em `docs/`.

### Verificado
- `flutter build web --release` → `✓ Built build/web`; servido: `/` 200 com COOP, `/pkg/kentron_ffi_bg.wasm` 200 `application/wasm`, `/pkg/kentron_ffi.js` e `/main.dart.js` 200, `index.html` com a tag da ponte. Negativo: `servir-web.py` com dir inexistente → exit=1.

### Pré-requisitos (medidos)
- `build-web` compila com `-Z build-std` para atomics/shared memory: exige toolchain **nightly** com `rust-src` e o alvo `wasm32-unknown-unknown`.
- O próprio gerador **auto-instala** ferramentas quando faltam: `cargo-expand` 1.0.127 e `wasm-pack` 0.15.1 (divulgado).

### Decisão
- O build web **não** entra no `ci-local.sh` por ora: exigiria nightly + `build-std` em toda bateria, custo alto para o benefício atual. Revisar quando o alvo web passar a ser entregue.

## [R1] Ponte Rust ↔ Flutter real (ADR-0007 aceita, 2026-11-20)
### Adicionado
- `crates/kentron_ffi`: crate adaptador — o único `cdylib` do workspace e o único a depender do `flutter_rust_bridge` 2.13.0. Expõe `parse_song(input) -> SongSummary` (título, tom, artista, seções, acordes e qual núcleo respondeu).
- Bindings gerados (`crates/kentron_ffi/src/frb_generated.rs` + `apps/kentron_app/lib/src/rust/`) e `flutter_rust_bridge.yaml` com o comando de regeração.
- `apps/kentron_app/test/bridge_test.dart`: carrega o `cdylib` e chama o núcleo de verdade — prova a ponte, não um dublê.

### Alterado
- `notation_bundle` volta a ser domínio puro: sai o `cdylib` e a dependência de FFI, e `unsafe_code = "forbid"` segue valendo.
- `scripts/ci-local.sh` e `.githooks/pre-push`: constroem o `cdylib` antes dos passos do app, porque o `flutter test` carrega o artefato.

### Emenda à ADR-0007 (o que a medição obrigou)
- A cláusula "`unsafe_code = "forbid"` permanece" era **inalcançável**: a cola gerada tem um bloco `unsafe` e `forbid` não pode ser relaxado por `allow`. O FFI passou para `crates/kentron_ffi` com `unsafe_code = "deny"` e `#[allow(unsafe_code)]` escopado ao módulo gerado — o domínio mantém `forbid`.

### Validação
- 58 testes Rust (3 do adaptador) e 5 de Flutter (2 da ponte) verdes; `cargo clippy --workspace --all-targets -- -D warnings` limpo; `ci-local.sh` verde.

## [R0] core_l10n integrado ao app (2026-11-20)
### Adicionado
- `packages/core_l10n/lib/core_l10n.dart`: caminho único de import das localizações geradas — o app deixa de conhecer o caminho do arquivo gerado.
- ARB `homeWelcome` (en/pt/es) e a tela inicial usando a string do ARB no lugar de texto fixo.
- Testes do app cobrindo en, pt e es na mesma tela, mais o fallback regional `pt_PT` → `pt`.

### Alterado
- `apps/kentron_app` passou a depender de `core_l10n` e monta o `MaterialApp` com `CoreL10n.delegate` e `...GlobalMaterialLocalizations.delegates`; `supportedLocales` vem de `CoreL10n.supportedLocales`.
- `scripts/ci-local.sh`: a geração do `core_l10n` passou a ser **no lugar** (não mais em cópia temporária) e **antes** dos passos do app, que precisam do arquivo gerado.
- `.githooks/pre-push`: o gate de Flutter gera o `core_l10n` antes de analisar e testar — sem isso o push reprovaria por um arquivo que o pipeline encontra pronto.

### Descoberto (e registrado no PENDING)
- O `MaterialLocalizations` do `material_ui` **não** é satisfeito pelas `Global*` do `flutter_localizations` que o `gen-l10n` emite: o app reprovava com "No MaterialLocalizations found". A saída é usar as `Global*` do `material_ui` — e isso revisou o item que pedia "migrar `core_l10n` para `material_ui`", que **não** é resolvível no `pubspec` do pacote.


### Alterado
- `.github/workflows/ci.yml` deixou de duplicar a matriz: instala Rust, Flutter (3.50.1, casando com `.flutter-version`) e `cargo-deny`, e chama `./scripts/ci-local.sh`. Quatro jobs viraram um; a cobertura é a mesma, agora definida num lugar só.
- Gatilhos preservados: `pull_request`, `workflow_dispatch` e `schedule` semanal — segue sem disparar em push para a `main` (ADR-0004).

### Corrigido
- O risco de divergência silenciosa entre script e workflow (item aberto no PENDING e na tabela de riscos do plano v2.6) deixa de existir: não há duas cópias para divergir.

## [Infra] Duas lições do kentron publicadas no sovereign-harness 0.7.2 (2026-11-20)
### Publicado
- Release [v0.7.2](https://github.com/personal-digital-sovereignty/sovereign-harness/releases/tag/v0.7.2): skill `estado-compartilhado-multi-agente` §1c (PR [#21](https://github.com/personal-digital-sovereignty/sovereign-harness/pull/21)) e o gate de lint lendo a edição do manifesto (PR [#22](https://github.com/personal-digital-sovereignty/sovereign-harness/pull/22), tag `v0.7.2`), com teste novo provado no negativo.
- Versões → 0.7.2 nos 7 manifestos do kit; `sincronizar --verificar` 45/45; CI verde em ubuntu e macos.
- O projeto e o kit voltam a concordar sobre a edição do `rustfmt` — a correção do kentron (`a68c85c`) e a do template (0.7.2) são a mesma leitura do `Cargo.toml`.

## [Infra] Gate de lint do pre-push alinhado à edição do crate (2026-11-20)
### Corrigido
- `.githooks/pre-push` formatava com `rustfmt --edition 2021` **fixo**, enquanto o workspace é edition 2024 e o `ci-local.sh` usa `cargo fmt` (que lê a edição real). Os dois gates do projeto passaram a discordar sobre a ordenação de itens de import e o push era bloqueado por arquivo que o próprio pipeline aprovava.
- A edição passa a ser lida do `Cargo.toml`. Medição que fecha o diagnóstico: **mesmo arquivo e mesmo `rustfmt`** — `--edition 2024` limpo, `--edition 2021` com diff. Provado no negativo: um arquivo deliberadamente mal formatado continua reprovando.

## [R1] Song Bundle v1: persistência (ADR-0006 aceita, 2026-11-20)
### Adicionado
- Módulo `bundle_store`: bundle em diretório com `meta.json` versionado (`schema_version`), `song.cho` como fonte de verdade do conteúdo, índice da biblioteca (`library.json`), setlists (`*.json`) e exportação por cópia do diretório.
- Serializador ChordPro (`chordpro::write`): metadados em ordem determinística e cada acorde recolocado na posição original; ida e volta pelo parser preserva o modelo.
- Cadeia de migração por degrau (`MIGRATIONS`), vazia hoje porque v1 é o primeiro esquema: versão sem degrau é recusada em voz alta, nunca lida pela metade.
- Primeira dependência externa do workspace: `serde` + `serde_json`, 11 pacotes com transitivas, aprovados pelo `cargo deny`.

### Validação
- 55 testes no workspace (12 novos: 4 do serializador ChordPro e 8 de persistência, com diretório descartável). `ci-local.sh` verde e `cargo deny check` ok.

### Premissa corrigida
- "Persistir exige embutir o modelo em JSON": o conteúdo vai como `.cho` (texto) e o JSON carrega só identidade — o modelo em `bundle` segue sem nenhum atributo de serialização.

## [R1] Song Bundle v1: modelo em memória (2026-11-20)
### Adicionado
- Módulo `bundle` em `crates/notation_bundle`: `SongId`, `SetlistId`, `SongBundle`, `Library`, `Setlist` e `BundleError` com `message_key()` estável.
- 14 testes: 11 unitários (invariantes de id, título, duplicata, setlist e resolução) e 3 de integração em `tests/bundle_tests.rs` sobre o fixture `worship.cho`.
- Composição provada com parser e harmonia: o bundle carrega o `.cho` e a música transpõe sem perder a identidade bibliográfica.

### Alterado
- `lib.rs` documenta o que está implementado e o que depende da ADR-0006.

### Premissa corrigida
- "Persistência é o primeiro passo do Song Bundle": o dono decidiu o contrário — o modelo em memória vem primeiro e o formato é decidido depois. Nenhuma dependência externa foi adicionada; o workspace segue sem dependências.

## [Docs] ADR-0006 proposta: Song Bundle v1 (2026-11-20)
### Adicionado
- ADR-0006 (proposta): Song Bundle como diretório com `meta.json` versionado, `serde`/`serde_json` como primeira dependência externa, migração por degrau e exportação por cópia do diretório na v1.

### Alterado
- Índice de ADRs: ADR-0006 registrada como proposta; próximo número livre passa a ADR-0007.

## [Docs] ADR-0005 rejeitada e branches limpas (2026-11-20)
### Alterado
- ADR-0005 marcada como **rejeitada**: a redução de escopo da R1 tratava uma ordem de execução como corte de escopo de uma ADR já aceita. A ADR-0002 permanece integral.
- Plano v2.6, ROADMAP, PENDING e CONCLUSION alinhados: cifra primeiro é ordem de execução; MusicXML/ATEX ficam no marco R1.1, depois do S5, sem emenda de escopo.
- Referências da ADR-0005 corrigidas (caminhos relativos quebrados a partir de `docs/adr/`).

### Adicionado
- Skill `estado-compartilhado-multi-agente` §1c: branch de escopo vira segundo trunk; como distinguir trabalho de sombra antes de ler estado.
- `AGENTS.md`: ao terminar, apagar branches substituídas no mesmo passo; nome de branch é unidade de trabalho, não escopo.

### Removido
- Branches `feature/R1-restante`, `feature/R1-transposicao-harmonia` e `wip/harmony-transpose`: nenhuma tinha conteúdo exclusivo em relação à `main`.

### Corrigido
- `AGENTS.md` afirmava que `CLAUDE.md` e `GEMINI.md` eram symlinks; `CLAUDE.md` é arquivo próprio com as instruções do RTK.
- Referências a `DESIGN.md` em `docs/` apontavam para `docs/DESIGN.md` (inexistente); agora resolvem para a raiz e `verificar_refs.py` sai 0.

### Premissa corrigida
- "O piloto público aguarda o S5" era inferência da proposta, não cláusula da ADR-0002. O portão do piloto é o uso real por uma igreja, offline.

## [Infra] Pipeline local-first (ADR-0004, 2026-11-20)
### Adicionado
- `scripts/ci-local.sh` como portão principal: Rust (`fmt --check`, `clippy`, testes), `cargo deny`, paridade de ARBs e Flutter (`analyze`, `test`, `gen-l10n`).
- ADR-0004 e entrada no índice de ADRs.
- Gates transversais no `.githooks/pre-push` (`deny`, `arb`, `flutter`), com escape por `HARNESS_SKIP`.

### Alterado
- `.github/workflows/ci.yml` sem gatilho em `push` para `main`; mantém `pull_request`, `workflow_dispatch` e `schedule` semanal.
- README, ROADMAP, PENDING e CONCLUSÃO alinhados ao pipeline local.

### Corrigido
- Versão do Flutter fixada em `.flutter-version` (3.50.1), verificada pelo script e espelhada no workflow.
- `packages/core_l10n` incluído no pipeline com `gen-l10n` e `analyze --no-fatal-infos`.
- Localizações geradas (`app_localizations*.dart`) no `.gitignore`; deixam de poluir `git status`.

### Premissa corrigida
- O repositório é público, portanto o GitHub Actions é gratuito e ilimitado. A mudança é por soberania, redução de ruído e não duplicação de validação — não por custo.

## [R1] Transposição e harmonia (2026-11-20)
### Adicionado
- `Chord::transpose` em `chord_naming.rs`, com aritmética de letra e acidente.
- Módulo `harmony.rs` com transposição de música inteira e renderização nos quatro sistemas.
- Testes dourados `golden_harmony/worship.cho` e saídas Common/Latin.
- ADR-0005 (proposta): escopo da R1 com ChordPro primeiro e MusicXML/ATEX em R1.1.

### Corrigido
- Lints de formatação no código de harmonia e nos testes.

### Validação
- 29 testes Rust passando, incluindo acordes invertidos, `6/9`, acidentes duplos e acordes desconhecidos.

## [R1] Cifra e Projeção (parcial integrado, 2026-11-20)
### Adicionado
- Parser ChordPro com modelo canônico e testes dourados.
- Placeholder de parser MusicXML.
- Branch `feature/R1-cifra-projecao` criada a partir da `main` e integrada pelo merge `50c7323`.

### Corrigido
- Lints do parser ChordPro e remoção do módulo `harmony` incompleto da árvore ativa.
- Workflow de CI com ações fixadas por SHA e escopo correto do Flutter.
- App Flutter sem dependências inexistentes de `core_l10n` e `flutter_rust_bridge`.
- Exclusão dos scripts vendorizados do Impeccable da análise Semgrep.
- App migrado para `material_ui` 1.11.1 e token explícito "Sovereign Emerald".
- CI do GitHub verde no run `44665120771` (Rust, licenças, i18n e Flutter).
- Validação local com Flutter 3.50.1: `flutter analyze` sem achados, `flutter test` passando e `flutter gen-l10n` de `core_l10n` sem erro.

## [R0] Fundação e Identidade (2026-11-19)
### Adicionado
- Identidade oficial **Kentron Music Hub** e licença Apache-2.0 com aviso de uso.
- Sovereign Harness com guardrails e git gates.
- Impeccable UI documentado em `DESIGN.md`.
- Monorepo Melos, scaffold Flutter e core Rust `notation_bundle`.
- Ponte `flutter_rust_bridge` como placeholder.
- CI para Rust, licenças, i18n e Flutter.
- ARBs da Faixa 1 (pt-BR, en-US, es-419) e validador `check_arb.py`.
- ADR-0001, ADR-0002 e ADR-0003.

### Publicado
- `main` enviada ao GitHub com `[skip ci]` no primeiro push.
- Branch padrão do repositório alterada para `main`.
