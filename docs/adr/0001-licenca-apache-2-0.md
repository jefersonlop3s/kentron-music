# ADR-0001 · Licença do projeto: Apache-2.0

- **Status:** aceita (2026-11-18)
- **Decisão do plano:** D9 (ver `docs/plano-mestre-v2.5.html`)

## Contexto

O projeto é software livre, sem empresa por trás, mantido por doação. A meta é "100% permissivo": nenhuma dependência que obrigue a abrir o código do app ou impeça distribuição nas lojas. A preocupação com patentes apareceu desde o início da conversa de planejamento.

## Decisão

O código do projeto é licenciado sob **Apache-2.0**. Ela concede patentes de forma explícita (MIT não concede) e é compatível com dependências MIT, BSD e Apache.

Dependências só entram se estiverem na lista permitida do `deny.toml`:

| Permitidas | Observação |
|---|---|
| Apache-2.0, MIT, BSD-2-Clause, BSD-3-Clause, ISC, Zlib, CC0-1.0, Unicode-3.0 | uso livre |
| MPL-2.0 | copyleft por arquivo; só como dependência sem modificação (ex.: alphaTab) |

Qualquer GPL, AGPL, LGPL ou licença não comercial (CC BY-NC*) é **negada** por padrão. Exceção exige ADR própria.

Pesos de modelos de IA também contam: cada arquivo de modelo precisa constar em `models/MANIFEST.toml` com licença, fonte e SHA-256 (o código de um projeto pode ser MIT enquanto os pesos não são).

## Consequências

- O CI roda `cargo-deny` e reprova licença fora da lista (Rust).
- Dependências Dart/Flutter ainda não têm checagem automática; entra quando o Flutter for instalado no ambiente.
- Achados da auditoria que motivaram esta regra: `chromaprint-next` é LGPL-2.1 e o `matrix-dart-sdk` é AGPL-3.0 (itens A1 e A6 do plano v2.5).
