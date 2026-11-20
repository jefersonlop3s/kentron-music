# Kentron Music Hub

App livre e descentralizado para bandas e equipes de louvor: cifras, tablaturas e partituras,
setlists, agenda e escalas, regência ao vivo e ponte para projeção (Holyrics/OpenLyrics) e OBS.
Multilíngue desde o início (pt-BR, es-419, en-US primeiro).

**Estado: R0 (fundação).** Ainda não há app. O nome definitivo é **Kentron**.

---

## ⚠️ Aviso de Uso e Contribuição
Este projeto é distribuído "como está" (*as-is*), sem garantias de qualquer tipo, expressas ou implícitas. Os mantenedores e contribuidores não assumem responsabilidade por qualquer dano, perda de dados ou problemas operacionais decorrentes do uso do software. O uso deste aplicativo é de inteira responsabilidade do usuário.

---

## Documentos

- [Plano mestre v2.5](docs/plano-mestre-v2.5.html): arquitetura, auditoria da v2.4, i18n, roteiro por releases e decisões.
- [ADR-0001](docs/adr/0001-licenca-apache-2-0.md) licença · [ADR-0002](docs/adr/0002-primeira-release-cifra-e-projecao.md) primeira release · [ADR-0003](docs/adr/0003-nome-de-trabalho.md) nome.

## Estrutura

| Caminho | O que é |
|---|---|
| `crates/notation_bundle` | Modelo canônico de acordes e nomeação em 4 sistemas (`chord_naming`), com testes dourados. Primeiro módulo do R0. |
| `packages/core_l10n` | ARB e configuração de tradução (Flutter). Ver o README do pacote. |
| `models/MANIFEST.toml` | Manifesto obrigatório de modelos de IA (licença do código **e** dos pesos). |
| `deny.toml` | Lista de licenças permitidas (`cargo-deny`). |
| `tools/check_arb.py` | Portão de i18n que não depende do Flutter. |

## Comandos

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

## Licença

[Apache-2.0](LICENSE).
