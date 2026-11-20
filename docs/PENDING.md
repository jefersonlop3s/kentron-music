# Pendências · Kentron Music Hub

## 🚩 R0 (publicado, com ressalvas)
- [x] `flutter analyze` de `packages/core_l10n` no pipeline local (cópia temporária, `--no-fatal-infos`).
- [x] Versão do Flutter fixada em `.flutter-version` (3.50.1), verificada pelo script e espelhada no workflow.
- [ ] Manter `scripts/ci-local.sh` e `.github/workflows/ci.yml` sincronizados: a matriz está duplicada e pode divergir.
- [ ] Migrar `core_l10n` de `flutter_localizations` para `material_ui`: o código gerado emite `info` de depreciação (`Global*Localizations`).
- [ ] Integrar `core_l10n` ao app e criar `lib/core_l10n.dart`.
- [ ] Substituir o placeholder da ponte Rust ↔ Flutter por chamada real.
- [ ] Configurar proteção da branch `main` no GitHub.
- [ ] Registro final na verificação de desenvolvedor do Android.
- [ ] Adicionar verificação de licenças para dependências Dart/Flutter no pipeline local (`cargo-deny` cobre apenas Rust).
- [ ] Teste de fumaça do app Flutter em dispositivo real (Android/iOS).
- [ ] Revisar traduções de es-419, pt-BR e en-US com falantes nativos.

## 🎸 R1 (prioridades imediatas)

Parser ChordPro e testes dourados já integrados à `main`; o restante da R1 segue em branch própria.
- [ ] **Spike S5:** validar renderização de partituras/tabs via alphaTab/OSMD.
- [ ] Ampliar o parser ChordPro: acordes invertidos, `6/9`, acidentes duplos e letras multilíngues.
- [ ] Implementar transposição no `chord_naming` e reintroduzir o módulo `harmony`. Rascunho preservado na branch local `wip/harmony-transpose` (`35971c4`); não compila e não foi publicada.
- [ ] Implementar persistência local (Song Bundle), migrações e exportação/restauração.
- [ ] Criar UI de leitor de cifras (Modo Palco) e setlists.
- [ ] Implementar exportação OpenLyrics/Holyrics e validar XML.
- [ ] Validar Browser Source para OBS sem `obs-websocket`.
- [ ] Executar piloto de dois cultos e fechar a R1.

## 🌐 R2+ (backlog)
- [ ] Spike S4 (Matrix/Push iOS).
- [ ] Spike S2 (sincronia de tempo).
- [ ] Spike S1 (MIR on-device).
- [ ] Spike S6 (índice global).

