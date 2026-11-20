# ADR-0004 · CI local-first com GitHub Actions como rede de segurança

- **Status:** aceita (2026-11-20)
- **Decisão do plano:** replanejamento das execuções de pipeline

## Contexto

O pedido partiu da premissa de que os runners gratuitos do GitHub se esgotariam em breve. A verificação mostrou o contrário: o repositório é **público**, e o GitHub Actions é gratuito e ilimitado para repositórios públicos. O uso atual é de 12 execuções, sem custo.

O portão local (`.githooks/pre-push`) já roda gitleaks, Trivy, Semgrep, Clippy, `cargo test` e `rustfmt` nos arquivos do push. Ele **não** cobre `cargo fmt --all --check`, `cargo deny`, `tools/check_arb.py`, `flutter analyze` e `flutter test`. O CI remoto cobre esses itens, mas dispara em todo push para `main`, gerando ruído e duplicação.

## Decisão

1. **Pipeline local como portão principal:** criar `scripts/ci-local.sh`, que executa a matriz completa (Rust: formatação, Clippy e testes; `cargo-deny`; paridade de ARBs; Flutter: `analyze` e `test`).
2. **GitHub Actions como rede de segurança:** manter o workflow, mas remover o gatilho de `push` em `main`; manter `pull_request` e `workflow_dispatch`, com `schedule` semanal para detectar drift de dependências.
3. **Premissa corrigida:** a decisão não é motivada por custo. É motivada por soberania, redução de ruído e não duplicação. Se o GitHub mudar a política para repositórios públicos, o pipeline local já cobre o essencial.

## Consequências

- Pushes diretos em `main` não disparam CI remoto; a validação local passa a ser obrigatória pelo hook de pré-push.
- Pull requests continuam validados em ambiente limpo no GitHub.
- `workflow_dispatch` permite executar o CI remoto sob demanda.
- Custo adicional: nenhum. Ganho: menos ruído e independência de política de terceiros.
- Validação: `scripts/ci-local.sh` deve passar no estado atual e o push de teste não deve criar execução remota.
- Versão do Flutter fixada em `.flutter-version` (3.50.1); o script falha se a versão instalada divergir e o workflow espelha o mesmo valor.
- `packages/core_l10n` entra no pipeline com `gen-l10n` e `analyze --no-fatal-infos` (o código gerado emite `info` de depreciação).

## Referências e evolução

- `.githooks/pre-push`, `.github/workflows/ci.yml`, `scripts/ci-local.sh`.
- Política de cobrança do GitHub Actions (documentação oficial).
- Substitui o gatilho `push` do workflow `ci`; não substitui a validação remota em PRs.
