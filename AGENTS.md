# Instruções para agentes de IA — "Kentron"

> Fonte única das instruções do projeto. `GEMINI.md` é symlink para este arquivo; `CLAUDE.md` carrega apenas
> as instruções do RTK e não duplica este conteúdo;
> o Antigravity o recebe pela regra `always_on` em `.agents/rules/`; Codex e OpenCode o leem nativamente.
> Mantenha curto: o que é verificável vira gate, o que é julgamento vira skill.

## Leitura obrigatória

- Organização: [docs/README.md](docs/README.md).
- Arquitetura: [docs/architecture/README.md](docs/architecture/README.md).
- Pendências: [docs/PENDING.md](docs/PENDING.md) — somente trabalho aberto.
- Direção e sequência: [docs/ROADMAP.md](docs/ROADMAP.md).
- Histórico: [docs/CONCLUSION.md](docs/CONCLUSION.md).
- Quadro de trabalho: `.agents/agents_work.md`; configurar fonte canônica quando houver várias sessões.

## Perfil do projeto

- **Topologia**: repositório único (`topology = "repository"` em `.harness.toml`), sem membros
  independentes — um histórico e um conjunto de hooks. O harness não infere isso do nome das pastas.
- **Fronteiras**: workspace Rust (`crates/kentron_ffi`, o núcleo exposto ao app — ADR-0007/ADR-0008),
  app Flutter (`apps/kentron_app`, alvos Android, iOS e Web), pacote de localização
  (`packages/core_l10n`) e ferramentas Python (`tools/`, `scripts/`).
- **Branch principal e remote**: `main`, protegida no guard contra commit direto; único remote
  `origin` (`github.com/jefersonlop3s/kentron-music`).
- **Stack**: Rust 1.97.1 (pinado em `crates/kentron_ffi/rust-toolchain.toml`), Flutter 3.50.1
  (pinado em `.flutter-version`), JDK 21, NDK 28.2.13676358, Gradle 9.5.0, AGP 9.3.1, KGP 2.4.20,
  compileSdk/targetSdk 36, minSdk 24.
- **Comandos reais de validação**: `./scripts/ci-local.sh` (bateria completa, mesma matriz do
  workflow) e `.githooks/pre-push` (gates por push: gitleaks, trivy, semgrep, clippy, testes,
  rustfmt, shellcheck, cargo-deny, ARBs, Flutter). O contrato `[validation]` do `.harness.toml`
  declara o **gitleaks** como validação de toda invocação do `validation.py`; a bateria completa
  fica no hook, não no contrato (decisão de 2026-11-21).
- **Idiomas e locales**: documentação e comunicação em português (Brasil). Os ARBs são nomeados pelo
  **idioma-base** — `en` (en-US), `es` (es-419), `pt` (pt-BR) — porque o `gen-l10n` exige um arquivo
  base para cada idioma (um `pt_BR` sem `pt` faz o gerador abortar); `es_ES` e `pt_PT` sobrescrevem
  as variantes europeias. O `pt` é **português do Brasil por gate**: `tools/check_arb.py` reprova
  grafia europeia nesse arquivo (`ecrã`, `secção`, `ficheiro`, `utilizador`…) e manda a variante
  europeia para `pt_PT`. A faixa 1 da ADR-0002 (pt-BR, en-US, es-419) é atendida por essa
  correspondência.
- **Autoridades de aprovação**: o dono do projeto aprova decisão estrutural (ADR), remoção de
  branches e release.

## Comunicação

- Idioma: português (Brasil). Técnico e direto, sem bajulação.
- Senso crítico: revise a própria sugestão contra o histórico do projeto antes de propor.

## Segurança e soberania

1. **Nenhum fallback silencioso.** Falhou: logue e estoure o erro (fail-loud).
2. **Credenciais** só por ambiente/cofre; nunca no código, em fixture, commit, PR ou log (skill `manuseio-segredos`).
3. **Dependência de sistema ausente**: reproduza o erro, identifique o pacote e siga as permissões e autorizações vigentes do projeto; nunca contorne o requisito silenciosamente.

## Testes e validação

1. Executar os testes necessários ao comportamento alterado e os gates obrigatórios do projeto. Validar integrações reais quando disponíveis; doubles não certificam um serviço ou ambiente real.
2. Escolher ferramenta E2E conforme a interface. Projeto sem interface não precisa de teste de navegador; registrar a razão.
3. Todo verificador novo é provado no sentido negativo (skill `disciplina-de-validacao`).
4. Bateria completa antes de qualquer push — aplicada pelo `.githooks/pre-push`.

## Git, gates e release

1. **Branch por trabalho** conforme política do projeto; configurar ramos protegidos no guard.
2. **Cadência**: alterou, commitou — commits atômicos, mensagem com o quê, o porquê e o objetivo. Mudança indivisível em vários arquivos vai num commit só.
3. **Autoria** segue a identidade Git configurada e a política de contribuição. Não alterar identidade global.
4. **Gates locais**: configurar comandos reais em `.harness.toml` antes de ativar `git config core.hooksPath .githooks` em cada clone. Gate reprovou: corrija o código ou o gate (skill `gate-reprovou`). O runner portátil não oferece bypass por variável; escapes do guard e do perfil legado são mecanismos distintos e não certificam validação.
5. **Merge** pelo mecanismo aprovado pelo projeto, com validação do resultado integrado. Adaptador GitHub só se aplica quando essa for a forge; repositório local não exige PR remoto. Não desativar os checks requeridos.
6. **Release**: ainda não há release publicada — versão `0.0.0` em `Cargo.toml` e em `apps/kentron_app/pubspec.yaml`. A primeira é o piloto da R1 (R1-D em `docs/ROADMAP.md`, depois do alpha da R1-C — ADR-0009); a versão semântica é conferida nos dois manifestos antes de qualquer push de versão.

## Documentação viva

- `CHANGELOG.md` registra mudanças; `docs/ROADMAP.md` registra sequência. Respeitar arquivos já existentes e definir uma única fonte por escopo.
- **Toda entrega fecha nos dois registros centrais** (PENDING e CONCLUSION), de qualquer tamanho.
- Decisão estrutural: ADR **antes** de executar; GAP de Código não tem ADR; nunca ADR preventiva (skill `governanca-de-decisao`).
- Pressuposto de produto assumido por uma decisão é nomeado na ADR/CHANGELOG.
- Use `classificar-documentacao` ao criar/mover documentos e `encerrar-documentacao` no fechamento. Épicos e ADRs mantêm vínculos nos dois sentidos; IDs e decisões históricas permanecem estáveis.

## Coexistência multi-agente

1. Antes de tarefa não trivial: `git status`/`git diff` nos repositórios alvo e leitura do quadro, quando adotado pelo projeto.
2. Nunca `git add -A`/`.`; confira `git diff --cached --stat` antes do commit.
3. Worktree deriva da referência atualizada escolhida pelo projeto; conferir remote e branch. Sem remote, a referência local é a fonte.
4. `rm`/`mv`/`npm install` em worktree atravessam symlinks: `readlink -f` antes (skill `estado-compartilhado-multi-agente`).
5. Limpeza de branches segue a política e autorização do projeto, com prova de integração e preservação de trabalho exclusivo.
6. Ao terminar: quadro atualizado, se adotado, branches substituídas apagadas e handover. Nome de branch é unidade de trabalho, não escopo — branch de escopo vira segundo trunk e faz sessões lerem o checkout errado (skill `estado-compartilhado-multi-agente` §1c).

## Skills do projeto

**Onde ficam**: skills se criam e se editam só em `.agents/skills/<nome>/SKILL.md`. `.claude/skills` é symlink para `../.agents/skills` (o Claude Code só descobre esse caminho); nunca o transforme em diretório real.

Transversais (carregar sempre): `vies-de-plausibilidade`, `disciplina-de-validacao`, `manuseio-segredos`.

| Skill | Quando ler |
|---|---|
| `empacotamento-rust-flutter` | ao mexer em `crate-type`, toolchain, `hook/build.dart`, ao regerar bindings, ou quando o build falhar com "Unable to find bundle for dynamic library" |
| `higiene-de-dispositivos` | ao terminar prova em emulador/simulador e ao liberar máquina |
| `lote-nao-e-sequencia` | antes de agrupar comandos dependentes num mesmo lote |
| `toolchain-unica` | ao ver aviso de duplicata no doctor ou gate que reprova só num contexto |

Toda lição que custou uma sessão vira skill (julgamento) ou gate (mecanismo) — skill `prosa-vira-gate`.