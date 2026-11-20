# ADR-0007 · Ponte Rust ↔ Flutter: mecanismo e alvo do primeiro incremento

- **Status:** proposto (2026-11-20) — aguarda aceite do dono do projeto
- **Decisão do plano:** plano v2.6, "Próximos passos" item 2 (integrar `core_l10n` e criar a chamada real Rust ↔ Flutter)

## Contexto

O núcleo musical já faz parser ChordPro, transposição/render e persistência do bundle (43 testes + 12 da persistência), mas **nada disso alcança a tela**: `crates/notation_bundle/src/bridge.rs` é um placeholder que devolve `greet()`, e o app não depende dele. A ponte é o que destrava tudo o que já existe.

Estado medido em 2026-11-20:

- `apps/kentron_app/` **não tem diretórios de plataforma** — só `lib/` (um `main.dart` de 859B), `test/`, `pubspec.yaml` e `analysis_options.yaml`. Não existe `android/`, `ios/`, `macos/`, `linux/` nem `windows/`.
- `flutter_rust_bridge_codegen` **não está instalado**; o crate `flutter_rust_bridge` não está em nenhum cache local (cargo ou pub).
- Rede alcança crates.io e pub.dev. Última versão **estável** do `flutter_rust_bridge`: **2.13.0**; a mais recente publicada é um *pre-release* (`2.14.0-beta.2`).
- A [arquitetura](../architecture/README.md) promete a ponte: "`flutter_rust_bridge` (Comunicação eficiente via FFI)".
- O crate hoje tem duas dependências externas (`serde`, `serde_json`) e `unsafe_code = "forbid"` no workspace.

Forças em jogo:

- **Contrato documentado vs. custo da ferramenta.** A arquitetura já elegeu `flutter_rust_bridge`; trocar isso é mudar uma promessa, não escolher uma lib.
- **Sem plataforma não há app.** Enquanto não existirem os diretórios de plataforma, a ponte só é exercitável em teste (`flutter test` roda na VM do Dart, que tem `dart:ffi`); o ganho no dispositivo vem depois.
- **Beta não é opção num projeto que fixa versões.** A ferramenta mais recente é beta; a decisão precisa apontar a estável.

## Decisão

1. **Mecanismo: `flutter_rust_bridge` na versão estável 2.13.0** — mantém o contrato da arquitetura e evita a manutenção manual de ABI. A instalação de `flutter_rust_bridge_codegen` (ferramenta de sistema) é pré-requisito e fica registrada como custo, com a decisão de instalar sendo **do dono**.
2. **Alvo do primeiro incremento: uma chamada real, não a integração completa.** `bridge.rs` deixa de ser placeholder e passa a expor o núcleo (parsear ChordPro e devolver algo consumível pelo Dart), com o crate virando `cdylib` além de `rlib`. A prova é um teste de Flutter que chama Rust e verifica o resultado — sem depender de dispositivo.
3. **Diretórios de plataforma: item próprio.** `flutter create --platforms=...` no app é trabalho separado, e é o que transforma a ponte em algo que roda fora do teste. Não bloqueia o incremento 1, mas bloqueia o app de verdade.
4. **`unsafe_code = "forbid"` permanece.** O que o FFI exigir fica isolado no código gerado pelo `flutter_rust_bridge`; se em algum ponto for preciso `unsafe` escrito à mão, isso volta para ADR própria.

## Consequências

- **Ganho:** o núcleo deixa de ser inalcançável; a UI de leitor/setlists (R1-A) passa a ter caminho.
- **Ganho:** seguir o contrato documentado evita uma segunda decisão depois ("por que não usamos o que a arquitetura dizia?").
- **Custo:** uma ferramenta de sistema a instalar (`flutter_rust_bridge_codegen` 2.13.0, casando com o crate), com download e compilação.
- **Custo:** nova dependência no crate + código gerado no repositório (ou passo de geração no pipeline) — a matriz do CI cresce e precisa cobrir a geração.
- **Dívida assumida:** sem diretórios de plataforma, o incremento 1 é provado só em teste. O app não roda em dispositivo até o item 3.
- **Reversão:** se o dono recusar a ferramenta, a alternativa é `dart:ffi` manual com `extern "C"` — sem dependência externa, mas com ABI mantida à mão, sem gerador e contrariando a arquitetura documentada.

## Referências

- [Arquitetura](../architecture/README.md) · stack e promessa da ponte.
- [Plano de execução v2.6](../plans/plano-mestre-v2.6.md), "Próximos passos" item 2.
- [Pendências](../PENDING.md) · ponte como placeholder e integração do `core_l10n`.
- `crates/notation_bundle/src/bridge.rs` (placeholder atual) e `crates/notation_bundle/src/lib.rs`.
