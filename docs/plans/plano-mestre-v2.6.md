# Plano de execução v2.6 · Kentron Music Hub

**Status:** vigente para execução. Substitui a v2.5 como referência operacional; a v2.5 permanece como histórico de arquitetura e auditoria.

## 1. Decisões consolidadas

- Nome do produto: **Kentron Music Hub**; identificador técnico provisório `kentron`.
- Licença: **Apache-2.0**, com aviso explícito de uso "como está" no README.
- UI: metodologia **Impeccable**, com `DESIGN.md` como fonte de verdade visual.
- Engenharia assistida: **Sovereign Harness**, com gates locais de segredo, SCA, SAST, tipos, testes e lint.
- Repositório: `https://github.com/jefersonlop3s/kentron-music`.
- Branch principal: `main`. O trabalho da R1 ocorre em `feature/R1-cifra-projecao`.
- Dados musicais canônicos no Rust; tradução apenas na borda Flutter.
- MIR, Matrix/Iroh e índice global ficam fora do primeiro release público.

## 2. Estado real da fundação (R0)

Concluído e publicado na `main`:

- Workspace Rust com `notation_bundle` e 15 testes passando.
- Nomeação de acordes em quatro sistemas, com testes dourados.
- Workspace Flutter/Melos e scaffold de `kentron_app`.
- ARBs da Faixa 1 e validador `tools/check_arb.py`.
- CI com Rust, licenças, i18n e Flutter; ações fixadas por SHA. Verde no run `44665120771`.
- App Flutter migrado para `material_ui` 1.11.1.
- `cargo clippy`, `cargo fmt`, `cargo deny` e Semgrep sem achados no código do projeto.

Ressalvas abertas:

- Flutter não está instalado neste ambiente; `flutter analyze` e `flutter test` ainda não rodaram localmente.
- `core_l10n` ainda não está integrado ao app; o pacote não possui `lib/core_l10n.dart`.
- A ponte `flutter_rust_bridge` é apenas um placeholder; não há chamada real Rust ↔ Flutter.
- CI do GitHub verde no run `44665120771`; a validação local de Flutter segue pendente.

## 3. R1 · Cifra, palco local e projeção

### R1-A · Cifra local utilizável

- Parser ChordPro com modelo canônico e testes dourados. **Feito.**
- Tratamento de acordes invertidos, `6/9`, acidentes duplos e letras multilíngues. **Pendente.**
- Biblioteca local, setlists e Song Bundle versionado com exportação/restauração. **Pendente.**
- Transposição e renderização nos quatro sistemas. **Pendente.**
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
- MusicXML e ATEX ficam para **R1.1**, condicionados ao spike S5.

## 4. Riscos e mitigações

| Risco | Mitigação |
|---|---|
| R1 grande demais | Entregar R1-A, R1-B e R1-C como marcos separados. |
| S5 não cobre interoperabilidade | Separar spike de renderização de teste com Holyrics/OpenLyrics/OBS. |
| Sem portão de persistência | Song Bundle v1, migrações e backup por arquivo antes do piloto. |
| Sem spike de relógio/LAN | Spike específico antes da R3. |
| MIR sem métrica de aceite | Dataset lícito, métricas e limiares antes dos spikes. |
| Decisões D1–D9 bloqueando R0 | Apenas decisões imediatas bloqueiam; D3–D8 ficam para as releases afetadas. |
| `cargo-deny` não cobre Dart | Inventário e verificação de licenças Dart no CI. |

## 5. Próximos passos

1. Rodar `flutter analyze` e `flutter test` em ambiente com Flutter; corrigir o app.
2. Integrar `core_l10n` e criar a chamada real Rust ↔ Flutter.
3. Fechar o parser ChordPro e iniciar Song Bundle/persistência.
4. Implementar OpenLyrics e Browser Source manual.
5. Executar o piloto de dois cultos e só então publicar a R1.
