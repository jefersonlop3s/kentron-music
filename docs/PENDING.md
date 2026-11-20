# Pendências · Kentron Music Hub

## 🚩 R0 (publicado, com ressalvas)
- [ ] Rodar `flutter analyze` e `flutter gen-l10n` de `packages/core_l10n` no CI (hoje o job cobre apenas `apps/kentron_app`).
- [ ] Integrar `core_l10n` ao app e criar `lib/core_l10n.dart`.
- [ ] Substituir o placeholder da ponte Rust ↔ Flutter por chamada real.
- [ ] Configurar proteção da branch `main` no GitHub.
- [ ] Registro final na verificação de desenvolvedor do Android.
- [ ] Adicionar verificação de licenças para dependências Dart/Flutter.
- [ ] Teste de fumaça do app Flutter em dispositivo real (Android/iOS).
- [ ] Revisar traduções de es-419, pt-BR e en-US com falantes nativos.

## 🎸 R1 (prioridades imediatas)
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

