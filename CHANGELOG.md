# Changelog

Todos os eventos notáveis do projeto Kentron Music Hub.

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
