# Plano de execução v2.6 · Kentron Music Hub

**Status:** vigente para execução. Substitui a v2.5 como referência operacional; a v2.5 permanece como histórico de arquitetura e auditoria.

## 1. Decisões consolidadas

- Nome do produto: **Kentron Music Hub**; identificador técnico provisório `kentron`.
- Licença: **Apache-2.0**, com aviso explícito de uso "como está" no README.
- UI: metodologia **Impeccable**, com [DESIGN.md](../../DESIGN.md) como fonte de verdade visual.
- Engenharia assistida: **Sovereign Harness**, com gates locais de segredo, SCA, SAST, tipos, testes e lint.
- CI: **local-first** (ADR-0004). `scripts/ci-local.sh` é o portão principal; o GitHub Actions permanece como rede de segurança em PRs e semanal.
- Repositório: `https://github.com/jefersonlop3s/kentron-music`.
- Branch principal: `main`. O parser ChordPro e a transposição da R1 foram integrados; o restante da R1 segue em branch por unidade de trabalho, derivada da `main`.
- Escopo da R1: a ADR-0002 vale integralmente (`.cho`, `.musicxml`, `.atex`). A **ordem de execução** é cifra primeiro e MusicXML/ATEX no marco R1.1; a redução de escopo proposta na ADR-0005 foi **rejeitada** (2026-11-20).
- Dados musicais canônicos no Rust; tradução apenas na borda Flutter.
- MIR, Matrix/Iroh e índice global ficam fora do primeiro release público.

## 2. Estado real da fundação (R0)

Concluído e publicado na `main`:

- Workspace Rust com `notation_bundle` e 15 testes passando.
- Nomeação de acordes em quatro sistemas, com testes dourados.
- Workspace Flutter/Melos e scaffold de `kentron_app`.
- ARBs da Faixa 1 e validador `tools/check_arb.py`.
- Pipeline local `scripts/ci-local.sh` verde: Rust, licenças, i18n e Flutter. Actions como rede de segurança, sem gatilho em `push` para `main`.
- App Flutter migrado para `material_ui` 1.11.1.
- `cargo clippy`, `cargo fmt`, `cargo deny` e Semgrep sem achados no código do projeto.

Ressalvas abertas:

- Flutter 3.50.1 instalado em `~/development/flutter`; `flutter analyze`, `flutter test` e `flutter gen-l10n` rodaram localmente com sucesso.
- `core_l10n` ainda não está integrado ao app; o pacote não possui `lib/core_l10n.dart`.
- A ponte `flutter_rust_bridge` é apenas um placeholder; não há chamada real Rust ↔ Flutter.
- Pipeline local executado com sucesso; a validação local de Flutter foi executada com sucesso.

## 3. R1 · Cifra, palco local e projeção

### R1-A · Cifra local utilizável

- Parser ChordPro com modelo canônico e testes dourados. **Feito.**
- Fixtures de parser para acordes invertidos, `6/9` e acidentes duplos. **Pendente** (a nomeação já os cobre; falta provar a preservação na leitura do `.cho`).
- Letras multilíngues (projeção bilíngue). **Pendente** em R1-B.
- Biblioteca local, setlists e Song Bundle versionado com exportação/restauração. **Pendente.**
- Transposição e renderização nos quatro sistemas. **Feito** (29 testes, incluindo acordes invertidos, `6/9` e acidentes duplos).
- Testes: unitários, regressivos, exploratórios e de propriedade. **Parcial.**

### R1-B · Projeção manual

- Exportação OpenLyrics com validação XML e arquivos dourados. **Pendente.**
- Matriz de compatibilidade com versões fixadas de Holyrics e outros consumidores. **Pendente.**
- Browser Source para OBS, sem `obs-websocket` neste marco. **Pendente.**
- Testes de paginação, acentos, pseudolocalização e projeção bilíngue. **Pendente.**

### R1-C · Piloto e release

- Dois cultos consecutivos em uma igreja, sem assistência do desenvolvedor. **Pendente.**
- Nenhuma perda de dados, crash bloqueante ou falha de projeção. **Pendente.**
- pt-BR, en-US e es-419 revisados. **Pendente.**
- MusicXML e ATEX ficam para **R1.1**, depois do spike S5 — ordem de execução, não redução de escopo (ADR-0005 rejeitada; ADR-0002 integral).

## 4. Riscos e mitigações

| Risco | Mitigação |
|---|---|
| R1 grande demais | Entregar R1-A, R1-B e R1-C como marcos separados. |
| S5 não cobre interoperabilidade | Separar spike de renderização de teste com Holyrics/OpenLyrics/OBS. |
| Sem portão de persistência | Song Bundle v1, migrações e backup por arquivo antes do piloto. |
| Sem spike de relógio/LAN | Spike específico antes da R3. |
| MIR sem métrica de aceite | Dataset lícito, métricas e limiares antes dos spikes. |
| Decisões D1–D9 bloqueando R0 | Apenas decisões imediatas bloqueiam; D3–D8 ficam para as releases afetadas. |
| `cargo-deny` não cobre Dart | Inventário e verificação de licenças Dart no pipeline local. |
| Matriz duplicada (script + workflow) | **Resolvido (2026-11-20):** o workflow não repete a matriz — instala as ferramentas e chama `scripts/ci-local.sh`, que é a fonte de verdade. |
| Política de runners de terceiros | Pipeline local como portão principal; Actions apenas como rede de segurança. |
| Versão do Flutter não fixada | Resolvido: `.flutter-version` (3.50.1) verificado pelo script e espelhado no workflow. |
| `core_l10n` sem verificação | Resolvido: `gen-l10n` + `analyze --no-fatal-infos` na cópia temporária. |
| Depreciação em `core_l10n` | Migrar de `flutter_localizations` para `material_ui` (registrado em `PENDING`). |

## 5. Próximos passos

0. Rodar `./scripts/ci-local.sh` antes de cada push; é o portão principal (ADR-0004).
1. ~~Rodar `flutter analyze` e `flutter test` em ambiente com Flutter~~ — feito; o pipeline cobre app e `core_l10n`.
2. Integrar `core_l10n` e criar a chamada real Rust ↔ Flutter.
3. Fechar o parser ChordPro e iniciar Song Bundle/persistência.
4. Implementar OpenLyrics e Browser Source manual.
5. Executar o piloto de dois cultos e só então publicar a R1.
