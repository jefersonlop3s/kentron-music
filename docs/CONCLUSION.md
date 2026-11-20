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
- Persistência, transposição, UI de palco e projeção ainda não existem.

**Próxima Fase:** concluir R1-A, R1-B e R1-C conforme o plano v2.6, em nova branch a partir da `main`.

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

