# Kentron Music Hub

App livre e descentralizado para bandas e equipes de louvor: cifras, tablaturas e partituras,
setlists, agenda e escalas, regência ao vivo e ponte para projeção (Holyrics/OpenLyrics) e OBS.
Multilíngue desde o início (pt-BR, es-419, en-US primeiro).

**Estado: R0 publicado na `main`; R1 parcialmente integrado à `main` (parser ChordPro, transposição e placeholder MusicXML), com o restante em branch própria.** O portão principal é o **pipeline local** (`./scripts/ci-local.sh`), que roda Rust, licenças, ARBs e Flutter antes de cada push; o GitHub Actions permanece como rede de segurança em pull requests e no agendamento semanal (ADR-0004). O app usa `material_ui` 1.11.1. O nome definitivo é **Kentron**.

Repositório: `https://github.com/jefersonlop3s/kentron-music`.

---

## ⚠️ Aviso de Uso e Contribuição
Este projeto é distribuído "como está" (*as-is*), sem garantias de qualquer tipo, expressas ou implícitas. Os mantenedores e contribuidores não assumem responsabilidade por qualquer dano, perda de dados ou problemas operacionais decorrentes do uso do software. O uso deste aplicativo é de inteira responsabilidade do usuário.

---

## Documentos

- [Plano de execução v2.6](docs/plans/plano-mestre-v2.6.md): decisões vigentes, estado real e marcos da R1.
- [Plano mestre v2.5](docs/plano-mestre-v2.5.html): arquitetura, auditoria da v2.4 e histórico.
- [Roadmap](docs/ROADMAP.md) · [Pendências](docs/PENDING.md) · [Conclusões](docs/CONCLUSION.md) · [Changelog](CHANGELOG.md).
- [Arquitetura](docs/architecture/README.md) · [Design System](docs/design-system/README.md).
- [ADR-0001](docs/adr/0001-licenca-apache-2-0.md) licença · [ADR-0002](docs/adr/0002-primeira-release-cifra-e-projecao.md) primeira release · [ADR-0003](docs/adr/0003-nome-de-trabalho.md) nome · [ADR-0004](docs/adr/0004-ci-local-first.md) CI local-first.

## Estrutura

| Caminho | O que é |
|---|---|
| `crates/notation_bundle` | Modelo canônico de acordes, nomeação em 4 sistemas e parser ChordPro. |
| `apps/kentron_app` | Scaffold Flutter; ainda sem integração com `core_l10n` nem com a ponte Rust. |
| `packages/core_l10n` | ARB e configuração de tradução (Flutter). Ver o README do pacote. |
| `models/MANIFEST.toml` | Manifesto obrigatório de modelos de IA (licença do código **e** dos pesos). |
| `deny.toml` | Lista de licenças permitidas (`cargo-deny`). |
| `tools/check_arb.py` | Portão de i18n que não depende do Flutter. |

## Requisitos

- Rust estável (edition 2021) com `cargo`, `clippy`, `rustfmt` e `cargo-deny` (`cargo install cargo-deny --locked`).
- Flutter na versão fixada em `.flutter-version` (3.50.1), canal `stable`. O pipeline falha se a versão instalada divergir. Se o SDK não estiver no `PATH`, use `~/development/flutter/bin/flutter`.
- Python 3 para `tools/check_arb.py`.
- Para mexer na ponte Rust ↔ Flutter, `flutter_rust_bridge_codegen` 2.13.0 — **só** para quem muda
  `crates/kentron_ffi/src/bridge.rs`: os bindings gerados são versionados (ver *Ponte Rust ↔ Flutter*).

O hook de pré-push (`.githooks/pre-push`) executa os portões locais; ferramenta ausente **bloqueia** o push em vez de pular a validação.

## Comandos

Pipeline local completo (portão principal, ADR-0004):

```bash
./scripts/ci-local.sh
```

```bash
cargo test --workspace
```

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

```bash
python3 tools/check_arb.py
```

```bash
cargo deny check
```

```bash
cd apps/kentron_app && flutter pub get && flutter analyze && flutter test
```

```bash
cd packages/core_l10n && flutter pub get && flutter gen-l10n
```

## Ponte Rust ↔ Flutter (bindings)

Os bindings de `crates/kentron_ffi/src/bridge.rs` para Dart são **gerados** e **versionados**:

- `crates/kentron_ffi/src/frb_generated.rs` (lado Rust);
- `apps/kentron_app/lib/src/rust/` — `bridge.dart`, `frb_generated.dart`, `frb_generated.io.dart` e `frb_generated.web.dart`.

Eles entram no commit **junto com a mudança que os gerou**, e é isso que faz o **dia a dia não precisar
da ferramenta**: um clone novo compila e testa o app sem instalar nada.

`flutter_rust_bridge_codegen` só é necessária para **quem muda o que a ponte expõe** (função, struct
ou enum em `bridge.rs`):

```bash
cargo install flutter_rust_bridge_codegen --version 2.13.0
flutter_rust_bridge_codegen generate --config-file flutter_rust_bridge.yaml
cargo fmt --all
```

O `flutter_rust_bridge.yaml` é a fonte dos caminhos (`rust_input: crate::bridge`,
`rust_root: crates/kentron_ffi`, `dart_output: apps/kentron_app/lib/src/rust`) — os comandos acima
rodam da raiz do repositório.

Três coisas que o gate cobra:

- a versão do gerador é **fixa**: o crate pede `flutter_rust_bridge = "=2.13.0"`, e 2.13.0 do gerador
  produz código casado com o crate 2.13.0;
- o `cargo fmt --all` depois de gerar **não é opcional**: o rustfmt que o gerador usa não segue a
  edição do crate (2024) e o `cargo fmt --check` do pipeline reprova o arquivo como ele sai;
- quem regera, **formata e commita** o gerado no mesmo commit da mudança de `bridge.rs`.

## Licença

[Apache-2.0](LICENSE).
