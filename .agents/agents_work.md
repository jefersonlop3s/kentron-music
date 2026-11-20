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

## 🟡 Interrompido / órfão

## 🟢 Concluído recentemente

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

## Notas entre agentes

- Remoção de worktrees validada em 2026-11-20 03:13 (UTC): o worktree `de6b0` está limpo e na branch `feature/R1-cifra-projecao`, publicada em `origin` (`4beb085`); os diretórios `40e55` e `b5088` em `~/.cline/worktrees` estão vazios. Nenhum stash pendente. O rascunho de harmonia está na branch local `wip/harmony-transpose`, que sobrevive à remoção do worktree.
