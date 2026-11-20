# Roadmap · Kentron Music Hub

Este documento detalha a trajetória de evolução do Kentron, dividida em seis releases verticais. Cada release é liberada por um **Spike de Validação (S)** e validada por uma **Matriz de Testes** rigorosa.

## 🗺️ Visão Geral do Fluxo
`R0 (Fundação) ➔ R1 (Cifra/Projeção) ➔ R2 (Banda Conectada) ➔ R3 (Palco ao Vivo) ➔ R4 (Motor MIR) ➔ R5 (Rede Global)`

---

## 🏁 R0 · Fundação e Guardas (Concluído)
**Objetivo:** Estabelecer a infraestrutura de engenharia e a identidade do produto.
- [x] Nome definido: **Kentron Music Hub**.
- [x] Licença: Apache-2.0.
- [x] Sovereign Harness: Guardrails e Git Gates ativos.
- [x] Impeccable UI: `DESIGN.md` e vocabulário visual definidos.
- [x] Infra: Cargo Workspace, Melos, Flutter Scaffold e CI Pipeline.
- [x] Core: `notation_bundle` (chord_naming) com testes dourados.

---

## 🎸 R1 · Cifra, Palco Local e Projeção (Atual)
**Objetivo:** Resolver a dor imediata de leitura e projeção de cifras offline.
**Portão de Entrada:** Spike S5 (Renderização de Partituras/Tabs).

### Etapas:
1. **Parser de Cifras:** Leitor de `.cho` (ChordPro) e `.musicxml`.
2. **Motor Harmônico:** Transposição matemática e 4 sistemas de notação.
3. **UI de Palco:** Leitor de cifras, setlists locais e gestão de estilos.
4. **Projeção:** Exportação OpenLyrics/Holyrics e Browser Source para OBS.
5. **L10n:** Suporte total à Faixa 1 (pt-BR, en-US, es-419).

**Validação R1:** $\text{Unit} \rightarrow \text{Regression} \rightarrow \text{Exploratory} \rightarrow \text{Accessibility} \rightarrow \text{E2E (Piloto Igreja)}$.

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

