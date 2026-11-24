---
name: relatorio-de-status
description: Template de relatório de status do Kentron Music Hub organizado por Release (R) e Semanas. Carregar quando o usuário perguntar "qual o status do projeto", "status", "como está o projeto", "progresso", ou qualquer variação destas. Produz o relatório no formato padronizado: visão geral de commits/semanas, depois cada release com tabelas de itens ✅/❌/⚠️, depois resumo por release com indicador 🟢🟡🔴.
---

# Template de relatório de status do Kentron Music Hub

## Formato do relatório

### 1. Cabeçalho — visão geral
```
## 📊 Visão geral: <N> commits em <N> dias (<data início>–<data fim>)
```

Extrair de `git rev-list --count HEAD` e `git log --reverse --format='%ai' | head -1` + `git log --format='%ai' | tail -1`.

---

### 2. Por semana — cada semana tem seu bloco

```
## Semana <N> · <data início>–<data fim> (dia_sem) — <tema da semana>
```

Dentro de cada semana, agrupar por release (R0, R1-0, R1-A, R1-B, R1-C, R1-D, R2, R3, R4, R5).

---

### 3. Tabela de itens por release

Para cada release dentro de uma semana, usar tabela:

```
### 🏁 R0 · Fundação e Guardas  (exemplos de emoji por release)
### 🎸 R1-0 · Fundação técnica
### 🎸 R1-A · Cifra local utilizável
### 🎸 R1-B · Projeção manual
### 🎸 R1-C · Alpha utilizável
### 🎸 R1-D · Piloto automatizado
### 🌐 R2 · Banda Conectada
### ⏱️ R3 · Palco ao Vivo
### 🧠 R4 · Motor MIR
### 🌍 R5 · Rede Global e Lojas
```

Formato da tabela:

| Item | Status |
|---|---|
| Descrição do item | ✅ |
| Descrição do item | ❌ Aberto |
| Descrição do item | ❌ Refutado |

Usar:
- `✅` — entregue/verde
- `❌ Aberto` — pendente, não iniciado
- `❌ Refutado` — tentativa medida que falhou e foi revertida
- `⚠️ Parcial` — release com itens mistos (✅ e ❌)

---

### 4. Tabelas de métricas quando aplicável (ex: compasso)

```
| Métrica | Antes | Depois |
|---|---|---|
| item | valor | **valor** |
```

---

### 5. Resumo por Release

Tabela final:

```
## 📈 Resumo por Release

| Release | Progresso | Status |
|---|---|---|
| **R0** · Fundação | 100% (X/X itens) | 🟢 Publicado |
| **R1-0** · Fundação técnica | 100% (X/X) | 🟢 Entregue |
| **R1-A** · Cifra local | ~X% (X/X itens) | 🟡 Aberto: item1, item2 |
| **R1-B** · Projeção | ~X% (X/X) | 🟡 Aberto: item1, item2 |
| **R1-C** · Alpha | ~X% (X/X) | 🟡 Aberto: item1, item2, item3 |
| **R1-D** · Piloto | ~X% (X/X) | 🟡 Aberto: item1, item2 |
| **R4** · Motor MIR | ~X% dos blocos | 🟢 Entregue parcialmente |
```

Legenda:
- 🟢 — publicado / entregue
- 🟡 — em andamento / parcial
- 🔴 — bloqueado

---

### 6. Próximos passos

```
## Próximos passos imediatos (PENDING)

1. Item prioritário
2. Item prioritário
3. ...
```

---

## Fontes de dados obrigatórias

Sempre consultar (nesta ordem) antes de produzir o relatório:

1. `docs/ROADMAP.md` — releases e marcos
2. `docs/PENDING.md` — pendências abertas por release
3. `docs/CONCLUSION.md` — conclusões de entregas passadas
4. `CHANGELOG.md` — registro cronológico das mudanças
5. `git log --oneline --format='%ai %s'` (últimos commits) — atividade recente
6. `.agents/agents_work.md` — quadro de trabalho (sessões ativas)
7. `git rev-list --count HEAD` — total de commits