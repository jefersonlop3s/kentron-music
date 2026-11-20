# Changelog

Todos os eventos notáveis do projeto Kentron Music Hub.

## [R0] Fundação e Identidade (2026-11-19)
### Adicionado
- **Identidade:** Nome oficial definido como **Kentron Music Hub**.
- **Soberania:** Implementação do `sovereign-harness` com guardrails, git hooks e sistema de skills.
- **Design:** Adoção do sistema `Impeccable` com a definição do `DESIGN.md` (Sovereign Emerald).
- **Arquitetura:** 
  - Estrutura de Monorepo configurada com Melos.
  - Scaffold do app Flutter (`apps/kentron_app`).
  - Core em Rust (`crates/notation_bundle`) com módulo de nomeação de acordes.
  - Ponte inicial para Flutter Rust Bridge (`bridge.rs`).
- **Infraestrutura:** 
  - Pipeline de CI configurado para Rust, Licenças (cargo-deny) e i18n.
  - Adicionado suporte a análise de Flutter no CI.
- **i18n:** Estrutura de ARB para Faixa 1 (pt-BR, en-US, es-419) e validador `check_arb.py`.
- **Documentação:** ADR-0001, ADR-0002 e ADR-0003 formalizadas.
