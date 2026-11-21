# ADR-0008 · Empacotamento do Rust: Native Assets em vez de Cargokit

- **Status:** **aceito** (2026-11-20) — decisão do dono do projeto; adotado a partir do commit `bb44789` (branch `spike/native-assets`), com iOS e Android buildando e **rodando** sem CocoaPods e sem o aviso de SPM
- **Decisão do plano:** continuação da ADR-0007 (ponte Rust ↔ Flutter)

## Contexto

A ADR-0007 decidiu o *mecanismo* (`flutter_rust_bridge` 2.13.0), mas não o *backend de empacotamento*. Seguimos o padrão do gerador: **cargokit** — um plugin Flutter (`rust_builder/`) que compila o crate e o embarca, usando **CocoaPods** no iOS e o plugin do **Gradle** no Android.

Estado medido em 2026-11-20:

- O `flutter build ios` avisa: *"The following plugins do not support Swift Package Manager for ios: `kentron_ffi`. **This will become an error in a future version of Flutter**"* — o cargokit é CocoaPods-only: não gera `Package.swift` e só tem builders `pod`/`gradle`/`cmake`.
- Desligar SPM no Flutter (`--no-enable-swift-package-manager`) **não** silencia o aviso: o build segue verde com a flag desligada e o aviso continua (é alerta da resolução de plugins, independente da flag). A configuração foi restaurada ao default.
- O frb 2.13.0 já oferece o backend **`native-assets`** (`integrate --integration-backend native-assets`), que usa **build hooks** do Dart/Flutter (`hook/build.dart` → `native_toolchain_rust`) e registra o Rust como **code asset** — a direção para onde o próprio Flutter está indo.
- Upstream documentou a migração (PR #3285, *merged*) e rodou teste manual — porém **só em Linux desktop**; iOS e Android não foram cobertos por esse teste.

## Decisão

Migrar o empacotamento do Rust para o backend **Native Assets**, eliminando o plugin e, com ele, a dependência de CocoaPods e do plugin do Gradle.

**O que muda no projeto (medido no spike):**

| Antes | Depois |
|---|---|
| `apps/kentron_app/rust_builder/` (plugin cargokit, 65 arquivos) | **removido** |
| `pubspec`: `kentron_ffi: path: rust_builder` | **removido**; entra `flutter_rust_bridge_hooks: 2.13.0` |
| `analysis_options`: exclude `rust_builder/cargokit/**` | **removido** |
| `rust-toolchain.toml`: `channel = "stable"` | **`channel = "1.97.1"` + `targets = [...]`** — e o **mesmo arquivo dentro do crate**, porque o `native_toolchain_rust` valida no diretório do crate (não sobe a árvore) |
| `crates/kentron_ffi/Cargo.toml`: `crate-type = ["lib", "cdylib", "staticlib"]` | **mantido** — é requisito do backend: o alvo precisa produzir artefato estático **e** dinâmico |
| `ios/` gerado com CocoaPods | **regenerado sem CocoaPods** (`rm -rf ios && flutter create --platforms=ios .`) |
| — | **novo** `apps/kentron_app/hook/build.dart`, apontando para `../../crates/kentron_ffi` |
| Web | inalterado (`flutter_rust_bridge_codegen build-web` / wasm) |

## Provas (no spike, 2026-11-20)

- `flutter build ios --debug --simulator` → `✓ Built Runner.app` (23,9 s), **sem CocoaPods** e com **zero** ocorrências do aviso de SPM.
- App instalado e aberto no simulador (PID 64657); o print mostra `Exemplo`, `Kentron · tom C · 2 seção(ões)`, os acordes `C G Am F` e `núcleo: kentron_ffi 0.0.0` — o code asset carrega e responde em execução.
- `flutter build apk --debug` → `✓ Built app-debug.apk` (32,7 s).
- `flutter test` 8/8 — o teste que carrega o `cdylib` do host segue válido com a toolchain pinada.

## Alternativas consideradas

- **Manter o cargokit.** Custo zero hoje, mas o aviso *"will become an error"* é bomba-relógio e mantém **duas** máquinas de build (pods + gradle) para uma coisa só.
- **Escrever `Package.swift` à mão no plugin.** O SPM não tem a integração de build do cargokit: exigiria `binaryTarget` com XCFramework pré-construído — piora o pipeline (build fora do Flutter) para *manter* justamente o plugin que é o problema.
- **Esperar o frb estável com SPM.** Deixa o problema em pé e não ataca a raiz: mesmo com SPM, continuaria sendo plugin.

## Consequências

- **Ganho:** sai o CocoaPods do caminho do Rust (e o `pod install` do build), sai o plugin do Gradle, sai o aviso e o erro futuro de SPM — um mecanismo só para Android e iOS.
- **Custo:** a toolchain Rust passa a ser **pinada por versão** (bump manual, deliberado — coerente com o `.flutter-version` do projeto) e o `rust-toolchain.toml` fica duplicado (raiz + crate), por exigência da ferramenta.
- **Risco assumido:** o backend é novo e o teste manual do upstream cobriu **apenas Linux**; por isso a prova local em iOS e Android fez parte da aceitação.
- **Reversão:** o spike é um commit isolado em `spike/native-assets`; voltar é `git revert` + `flutter create --platforms=ios .` (o commit do cargokit continua no histórico).

## Referências

- ADR-0007 (mecanismo da ponte) e sua emenda.
- Doc upstream (no repositório do `flutter_rust_bridge`, não neste): `manual/integrate/03-native-assets` e `manual/integrate/05-migrate-cargokit-to-native-assets` (PR #3285).
- Spike local: branch `spike/native-assets`, com builds e execução medidos em 2026-11-20.
