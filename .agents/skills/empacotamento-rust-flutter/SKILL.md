---
name: empacotamento-rust-flutter
description: Como o núcleo Rust entra no app Flutter neste projeto (Native Assets/build hooks, ADR-0008) e as armadilhas que já custaram builds — crate-type sem staticlib, rust-toolchain.toml fora do diretório do crate, bindings gerados desatualizados, JDK 27 no Gradle 9.5. Carregar ao mexer em hook/build.dart, crate-type, toolchain, ao regerar bindings, e quando o build falhar com "Unable to find bundle for dynamic library" ou "Unsupported class file major version 71".
---

# Empacotamento Rust ↔ Flutter (Native Assets)

Caso de origem (2026-11-20): a ponte existia e funcionava, mas o Rust entrava no
app como **plugin** (cargokit, CocoaPods-only). O Flutter avisava que o plugin não
suporta Swift Package Manager e que isso *viraria erro*. A migração para **Native
Assets** (ADR-0008) tirou o plugin do caminho — e o caminho até ela custou três
erros com assinatura própria, todos abaixo.

## Como o núcleo entra hoje

- `apps/kentron_app/hook/build.dart` compila `crates/kentron_ffi` como **code
  asset** via `native_toolchain_rust` (pacote `flutter_rust_bridge_hooks: 2.13.0`).
- Sem plugin: `ios/` **não tem** `Podfile`, `Podfile.lock` nem `Pods/`; o plugin
  do Gradle também sai do caminho. É o que faz o aviso de SPM desaparecer por
  construção (não por flag).
- Os bindings gerados (`apps/kentron_app/lib/src/rust/`) ficam **versionados**;
  o dia a dia não precisa do gerador.

## Predicados técnicos (quebram o build quando ausentes)

| Predicado | Por quê |
|---|---|
| `crate-type = ["lib", "cdylib", "staticlib"]` | `cdylib` serve Android/testes/desktop; `staticlib` é o que o iOS consome (`.a`). Só `cdylib` → `Unable to find bundle for dynamic library`. |
| `rust-toolchain.toml` **dentro de `crates/kentron_ffi/`** | O `native_toolchain_rust` valida a toolchain no diretório do crate e **não sobe a árvore**; a cópia na raiz não basta. |
| Toolchain pinada (`1.97.1`) | Pin transforma bump de Rust em decisão deliberada; a lista de alvos acompanha. |
| **JDK 21** para Android | Gradle 9.5 reprova com Java 27: `Unsupported class file major version 71`. Aponte `flutter config --jdk-dir .../openjdk@21/...`. |
| NDK 28.2.13676358 · compileSdk/targetSdk 36 · minSdk 24 · Gradle 9.5.0 · AGP 9.3.1 · KGP 2.4.20 | Matriz medida na fonte; não de memória. |

## Erros com assinatura conhecida

- **`Unable to find bundle for dynamic library`** → falta `staticlib` no
  `crate-type`. O cargokit usava o `.a` **se existisse** (`if (staticLibs.isNotEmpty)`)
  e só exigia Framework quando não existia; lido no código da ferramenta, não por
  tentativa e erro.
- **`Unsupported class file major version 71`** → JDK errado (27 em vez de 21).
- **Aviso de SPM em todo build de iOS** → ainda há plugin no caminho. Não é
  silenciável por `flutter config --no-enable-swift-package-manager` (medido:
  build verde e aviso presente). Saída: sem plugin, por build hooks.

## Regerar bindings (quando `bridge.rs` muda)

```bash
cargo install flutter_rust_bridge_codegen --version 2.13.0
# comando e caminhos no flutter_rust_bridge.yaml, na raiz
```

Depois de regerar, rode o teste que carrega o **cdylib real**
(`apps/kentron_app/test/bridge_test.dart`) — teste hermético não prova a ponte.

## Julgamento: ler o código da ferramenta de terceiro

Quando o erro está numa ferramenta de terceiro, **o código dela é o runbook**.
Ler o caminho que levanta o erro deu a correção exata em minutos; tentativa e erro
custaria builds de 5 min cada. Antes de acusar a ferramenta, reproduza isolado e
compare com a máquina livre (skill `vies-de-plausibilidade` §3 e §5).

## Ritual de validação

1. `flutter build ios --debug --simulator` **e** `flutter build apk --debug` —
   mudança de `crate-type` ou de toolchain exige **revalidar a outra plataforma**.
2. Instale e abra o app no dispositivo; **leia o print você mesmo** e confira
   `núcleo: kentron_ffi …` (prova de que o núcleo respondeu).
3. `flutter test` (inclui o teste com cdylib real) e `scripts/ci-local.sh`.
4. Encerre os dispositivos depois da prova (`scripts/encerrar-dispositivos.sh`,
   skill `higiene-de-dispositivos`).

## O que não fazer

- Commitar `Podfile`/`Pods/` de volta: é o retorno do plugin.
- Trocar a toolchain global em vez de editar o pin do crate.
- Declarar o app "rodando" pelo build verde: build não abre o app.
