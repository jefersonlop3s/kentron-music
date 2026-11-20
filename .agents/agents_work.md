# Quadro de trabalho — quem está fazendo o quê, agora

> Único lugar onde uma sessão enxerga a outra. Caminho canônico: o deste arquivo **no diretório principal,
> na branch principal** — nunca a cópia de um worktree. Edite pelo `.agents/quadro.sh editar` e commite com
> `.agents/quadro.sh commit "docs(quadro): <o que mudou>"` (leva só este arquivo). Protocolo completo na skill
> `quadro-de-trabalho`.

## Como usar (3 momentos obrigatórios)

1. **Antes**: leia tudo; se alguém declarou os mesmos paths, combine em "Notas entre agentes". Acrescente sua entrada.
2. **Durante**: atualize ao mudar de repositório, branch ou escopo.
3. **Ao terminar**: mova para "Concluído recentemente" com PRs e o que ficou aberto; limpe as branches.

Timestamp sempre de `date -u '+%F %H:%M'`. **Paths sob edição** é o campo mais importante.

```markdown
### <agente>/<slug> — <título curto>
- **Início**: AAAA-MM-DD HH:MM (UTC) · **Última atualização**: AAAA-MM-DD HH:MM (UTC)
- **Repositórios**: ... · **Branches**: ... · **Worktrees**: ...
- **Demanda**: ...
- **Paths sob edição**: ...
- **Status**: em andamento | aguardando decisão do dono | em revisão | bloqueado
- **PRs**: ...
```

---

## 🔴 Em andamento

### cline/de6b0 — Rascunho de harmonia (WIP)
- **Início**: 2026-11-20 03:35 (UTC) · **Última atualização**: 2026-11-20 03:35 (UTC)
- **Repositório**: `kentron-music` · **Branches**: `wip/harmony-transpose` (`35971c4`, local) · **Worktrees**: nenhum
- **Demanda**: transposição e harmonia; o rascunho não compila.
- **Paths sob edição**: `crates/notation_bundle/src/harmony.rs` e `crates/notation_bundle/tests/harmony_tests.rs` na branch.
- **Status**: aguardando decisão do dono (concluir ou descartar).
- **PRs**: nenhum.

## 🟡 Interrompido / órfão

## 🟢 Concluído recentemente

### cline/de6b0 — CI: versão fixada e core_l10n no pipeline
- **Início**: 2026-11-20 04:25 (UTC) · **Última atualização**: 2026-11-20 04:45 (UTC)
- **Repositório**: `kentron-music` · **Branches**: `main` (`e211ae7`) · **Worktrees**: `~/Develop/Projects/app-music`
- **Demanda**: fechar as pendências do replanejamento: versão do Flutter, `core_l10n` no pipeline e risco de *drift*.
- **Paths sob edição**: nenhum.
- **Status**: concluído.
- **PRs**: nenhum.

**Feito:**
- `.flutter-version` (3.50.1) como fonte única; o script falha se a versão instalada divergir (provado no sentido negativo).
- `packages/core_l10n` coberto com `gen-l10n` + `analyze --no-fatal-infos` em cópia temporária.
- Workflow espelha a versão fixada e cobre `core_l10n`.
- Localizações geradas (`app_localizations*.dart`) no `.gitignore`.
- Pipeline local verde (`exit 0`) após as mudanças.

**Ficou aberto:**
- Sincronia entre script e workflow continua manual (risco de *drift* registrado em `PENDING`).
- Migração de `core_l10n` de `flutter_localizations` para `material_ui` (dois `info` de depreciação).

### cline/de6b0 — CI local-first (ADR-0004)
- **Início**: 2026-11-20 04:05 (UTC) · **Última atualização**: 2026-11-20 04:20 (UTC)
- **Repositório**: `kentron-music` · **Branches**: `main` (`9051b5b`) · **Worktrees**: `~/Develop/Projects/app-music`
- **Demanda**: replanejar as execuções de pipeline para o modelo local-first, com o Actions como rede de segurança.
- **Paths sob edição**: nenhum.
- **Status**: concluído.
- **PRs**: nenhum.

**Feito:**
- ADR-0004 aceita e registrada no índice.
- `scripts/ci-local.sh` criado e verde de ponta a ponta (Rust, `cargo deny`, ARBs, Flutter `analyze`/`test`/`gen-l10n`).
- `.githooks/pre-push` com gates transversais `deny`, `arb` e `flutter`.
- `.github/workflows/ci.yml` sem gatilho em `push`; mantém `pull_request`, `workflow_dispatch` e `schedule` semanal.
- Documentos vivos atualizados: README, ROADMAP, PENDING, CONCLUSÃO, CHANGELOG e plano v2.6.
- Dois pushes (`3a8dc58`, `9051b5b`) sem disparar Actions — confirmado em `gh run list`.

**Ficou aberto:**
- Matriz duplicada entre script e workflow; risco de *drift* registrado em `PENDING`.
- Versão do Flutter não fixada em nenhum dos dois.
- `packages/core_l10n` sem `analyze` no pipeline local.

### cline/de6b0 — R1 · Cifra e projeção (branch ativa)
- **Início**: 2026-11-19 21:31 (UTC) · **Última atualização**: 2026-11-20 03:44 (UTC)
- **Repositório**: `kentron-music` · **Branches**: `feature/R1-cifra-projecao` integrada à `main` (`7460f57`) · **Worktrees**: `~/Develop/Projects/app-music`
- **Demanda**: entregar a R1 (cifra local, projeção manual) e consolidar o parser na `main`.
- **Paths sob edição**: nenhum.
- **Status**: concluído — branch integrada e a apagar; o restante da R1 começa em nova branch.

**Feito:**
- `analysis_options.yaml` commitado em `91de756` (exclusão de `build/**` no analyzer).
- `feature/R1-cifra-projecao` integrada por merge `50c7323`.
- Documentação atualizada em `7460f57`.
- Validação do resultado integrado: 15 testes Rust, Clippy, formatação, `check_arb.py`, `cargo deny`, `flutter analyze` e `flutter test` verdes.

**Ficou aberto:**
- O restante da R1 (transposição, persistência, UI de palco, projeção) deve começar em nova branch a partir da `main`.
- `wip/harmony-transpose` permanece como rascunho local, aguardando decisão.

### cline/de6b0 — Análise dos planos, R0 publicado e R1 iniciada
- **Início**: 2026-11-19 21:31 (UTC) · **Última atualização**: 2026-11-20 03:13 (UTC)
- **Repositórios**: `kentron-music` (github.com/jefersonlop3s/kentron-music)
- **Branches**: `main` (`d21d1fe`), `feature/R1-cifra-projecao` (`4beb085`), `wip/harmony-transpose` (`35971c4`, local, não publicada) · **Worktrees**: `/Users/jefersonlopes/.cline/worktrees/de6b0/app-music`
- **Demanda**: validar e mitigar os planos anteriores, publicar o R0 na `main` e iniciar a R1 com bateria de testes.
- **Paths sob edição**: nenhum (sessão encerrada).
- **Status**: concluído
- **PRs**: sem PR; integração por *fast-forward* na `main`.

**O que ficou aberto:**
- Flutter não instalado neste ambiente; `flutter analyze`/`flutter test` rodaram apenas no CI.
- `core_l10n` não integrado ao app; ponte Rust ↔ Flutter é placeholder.
- R1 parcial: parser ChordPro e placeholder MusicXML; sem transposição, persistência, UI de palco ou projeção.
- `wip/harmony-transpose` não compila e não foi publicada; reimplementar `Chord::transpose` antes de mesclar.

**Hipóteses refutadas:**
- "R0 commitado e enviado": falso — os commits estavam apenas em `cline/de6b0`, a `main` local seguia em `fd7d588` e o remoto só tinha `master`.
- "Commit direto na `main` é permitido": o guard bloqueia; o fluxo correto é branch + merge *fast-forward*.
- "A validação local de Flutter já ocorreu": não ocorreu; apenas o CI executou `flutter analyze`/`flutter test`.

### cline/de6b0 — Instalação do Flutter e validação local
- **Início**: 2026-11-20 03:20 (UTC) · **Última atualização**: 2026-11-20 03:35 (UTC)
- **Repositório**: `kentron-music` · **Branches**: `main`
- **Demanda**: instalar Flutter 3.50.x e fechar a pendência de validação local do R0.
- **Paths sob edição**: nenhum.
- **Status**: concluído.

**Feito:**
- Flutter 3.50.1 (stable) instalado em `~/development/flutter` (clone parcial, canal stable).
- `flutter analyze` no app: sem achados. `flutter test`: teste de fumaça passando.
- `flutter gen-l10n` em `packages/core_l10n`: geração sem erro; artefatos gerados removidos por serem reproduzíveis.
- Documentos atualizados: README, ROADMAP, PENDING, CONCLUSION, CHANGELOG e plano v2.6.

**Ficou aberto:**
- O SDK não está no `PATH` padrão; usar `~/development/flutter/bin/flutter` ou exportar o caminho.
- CI ainda não analisa `packages/core_l10n`; item registrado em PENDING.
- `apps/kentron_app/analysis_options.yaml` foi assumido e commitado em `91de756` (exclusão de `build/**` no analyzer).

## Notas entre agentes

- Flutter 3.50.1 instalado em `~/development/flutter` (não está no `PATH` padrão). Para usar: `export PATH="$HOME/development/flutter/bin:$PATH"`.
- Remoção de worktrees validada em 2026-11-20 03:13 (UTC): o worktree `de6b0` está limpo; os diretórios `40e55` e `b5088` em `~/.cline/worktrees` estão vazios. Nenhum stash pendente. O rascunho de harmonia está na branch local `wip/harmony-transpose`, que sobrevive à remoção do worktree.
- `feature/R1-cifra-projecao` foi integrada à `main` em `50c7323` e pode ser apagada após o push do merge.
