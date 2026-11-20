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

