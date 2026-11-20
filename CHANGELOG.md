# Changelog

Todos os eventos notáveis do projeto Kentron Music Hub.

## [R1] Cifra e Projeção (em andamento, 2026-11-19)
### Adicionado
- Parser ChordPro com modelo canônico e testes dourados.
- Placeholder de parser MusicXML.
- Branch `feature/R1-cifra-projecao` criada a partir da `main`.

### Corrigido
- Lints do parser ChordPro e remoção do módulo `harmony` incompleto da árvore ativa.
- Workflow de CI com ações fixadas por SHA e escopo correto do Flutter.
- App Flutter sem dependências inexistentes de `core_l10n` e `flutter_rust_bridge`.
- Exclusão dos scripts vendorizados do Impeccable da análise Semgrep.
- App migrado para `material_ui` 1.11.1 e token explícito "Sovereign Emerald".
- CI do GitHub verde no run `44665120771` (Rust, licenças, i18n e Flutter).

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
