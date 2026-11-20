# Roadmap · Kentron Music Hub

Este documento detalha a trajetória de evolução do Kentron, dividida em seis releases verticais. Cada release é liberada por um **Spike de Validação (S)** e validada por uma **Matriz de Testes** rigorosa.

## 🗺️ Visão Geral do Fluxo
`R0 (Fundação) ➔ R1 (Cifra/Projeção) ➔ R2 (Banda Conectada) ➔ R3 (Palco ao Vivo) ➔ R4 (Motor MIR) ➔ R5 (Rede Global)`

---

## 🏁 R0 · Fundação e Guardas (publicado na `main`)
**Objetivo:** Estabelecer a infraestrutura de engenharia e a identidade do produto.
- [x] Nome definido: **Kentron Music Hub**.
- [x] Licença: Apache-2.0, com aviso explícito de uso "como está".
- [x] Sovereign Harness: guardrails e git hooks ativos.
- [x] Impeccable UI: [DESIGN.md](../DESIGN.md) e vocabulário visual definidos.
- [x] Infra: Cargo Workspace, Melos, scaffold Flutter e CI com ações fixadas por SHA.
- [x] Pipeline local `scripts/ci-local.sh` como portão principal (ADR-0004); Actions como rede de segurança em PRs e semanal.
- [x] Core: `notation_bundle` (`chord_naming`) com testes dourados.
- [x] Validação Flutter local (`flutter analyze`/`flutter test`) com Flutter 3.50.1.
- [ ] Integração real de `core_l10n` e da ponte Rust ↔ Flutter.
- [x] Pipeline local validado de ponta a ponta (Rust, licenças, ARBs e Flutter).
- [x] CI remoto mantido como rede de segurança: sem gatilho em `push` para `main`; `pull_request`, `workflow_dispatch` e `schedule` semanal.
- [x] App migrado para `material_ui` 1.11.1.

---

## 🎸 R1 · Cifra, Palco Local e Projeção (parcialmente integrado à `main`)
**Objetivo:** Resolver a dor imediata de leitura e projeção de cifras offline.
**Portão de Entrada:** Spike S5 (Renderização de Partituras/Tabs).

### R1-A · Cifra local utilizável
- [x] Parser ChordPro e modelo canônico com testes dourados.
- [ ] Fixtures de parser para acordes invertidos, `6/9` e acidentes duplos (a nomeação já os cobre).
- [ ] Biblioteca local, setlists e Song Bundle versionado com exportação/restauração. Modelo em memória pronto (`bundle`: `SongBundle`, `Library`, `Setlist`); persistência depende da ADR-0006.
- [x] Transposição e renderização nos quatro sistemas (29 testes, incluindo acordes invertidos, `6/9` e acidentes duplos).
- [ ] Testes de propriedade e regressão ampliada.

### R1-B · Projeção manual
- [ ] Exportação OpenLyrics com validação XML e arquivos dourados.
- [ ] Matriz de compatibilidade com versões fixadas de Holyrics e outros consumidores.
- [ ] Browser Source para OBS, sem `obs-websocket` neste marco.
- [ ] Paginação, acentos, pseudolocalização e projeção bilíngue.

### R1-C · Piloto e release
- [ ] Dois cultos consecutivos sem assistência do desenvolvedor.
- [ ] Nenhuma perda de dados, crash bloqueante ou falha de projeção.
- [ ] pt-BR, en-US e es-419 revisados.
- [ ] MusicXML e ATEX ficam para R1.1, depois do S5 — sequência de execução; o escopo de formatos da ADR-0002 permanece integral (ADR-0005 rejeitada).

**Validação R1:** unitária, regressiva, exploratória, acessibilidade e E2E (piloto de igreja).

---

## 🌐 R2 · Banda Conectada
**Objetivo:** Sincronização de estado e comunicação entre músicos.
**Portão de Entrada:** Spike S4 (Matrix e Push no iOS).

### Etapas:
1. **Identidade:** Perfis, chaves de dispositivo e convites por QR Code.
2. **Sincronização:** Agenda, escalas e setlists via Matrix/Iroh.
3. **Comunicação:** Chat mínimo (texto e salas) integrado ao fluxo de ensaio.
4. **Trust & Safety:** Sistema de denúncia e lista de bloqueio assinada.

---

## ⏱️ R3 · Palco ao Vivo
**Objetivo:** Regência em tempo real e sincronia de palco.
**Portão de Entrada:** Spike S2 (Sincronia de Tempo/Bateria).

### Etapas:
1. **Stage Clock:** Maestro manual, Sync de Start e auto-scroll por compasso.
2. **Regência:** Transições litúrgicas e avisos (cues) via rede LAN efêmera.
3. **Integração OBS:** Controle remoto via `obs-websocket`.
4. **Painel FOH:** Interface para o técnico de som.

---

## 🧠 R4 · Motor MIR (Music Information Retrieval)
**Objetivo:** Automação de cifras e tablaturas via análise de áudio.
**Portão de Entrada:** Spikes S1 (MIR on-device), S2 e S3 (Identidade da Obra).

### Etapas:
1. **Análise Local:** Áudio $\rightarrow$ Acordes, Tom e Compasso (Desktop primeiro).
2. **Geração Automática:** Tablaturas e partituras via inferência ONNX.
3. **Auto-Maestro:** Sugestão de dinâmica e formação baseada no áudio.
4. **Resolução de Links:** Link de streaming $\rightarrow$ Identidade da Obra $\rightarrow$ Cifra.

---

## 🌍 R5 · Rede Global e Lojas
**Objetivo:** Distribuição pública, indexação global e monetização.
**Portão de Entrada:** Spike S6 (Backend do Índice Global).

### Etapas:
1. **Índice Harmônico:** Busca global de mapas harmônicos via ID canônico.
2. **Lojas:** Publicação oficial na App Store, Google Play e F-Droid.
3. **Backup:** Sincronização opcional com Drive/OneDrive.
4. **L10n Expandida:** Faixas 2 (regionais) e 3 (comunidade).

---

## 🛠️ Matriz de Validação Global
Para cada etapa, aplica-se o seguinte rigor:
- **Funcional:** Testes unitários e de integração cobrindo 100% dos casos de uso.
- **Regressivo:** Snapshots de UI (Golden Tests) e tabelas de referência musical.
- **Exploratório:** Testes de estresse, limites de memória e casos de borda.
- **Acessibilidade:** Contraste WCAG, alvos de toque $\ge 48\text{px}$ e Screen Readers.
- **E2E:** Fluxos completos do usuário em dispositivos de referência.

**Execução:** o pipeline local (`./scripts/ci-local.sh`) roda a matriz antes de cada push; o Actions cobre PRs em ambiente limpo e o agendamento semanal detecta *drift* de dependências (ADR-0004).

