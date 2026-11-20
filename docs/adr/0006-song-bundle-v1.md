# ADR-0006 · Song Bundle v1: formato de persistência e primeira dependência externa

- **Status:** proposto (2026-11-20) — aguarda aceite do dono do projeto
- **Decisão do plano:** R1-A (biblioteca local, setlists e Song Bundle versionado)
- **Não substitui:** ADR-0002 (o layout `meta.json` / `.cho` / `.musicxml` / `tabs/*.atex` já vem de lá)

## Contexto

O [PENDING](../PENDING.md) pede "persistência local (Song Bundle), migrações e exportação/restauração", e o plano v2.6 lista o risco "Sem portão de persistência → Song Bundle v1, migrações e backup por arquivo antes do piloto". Nada disso existe ainda: não há modelo de bundle, I/O, versão de esquema nem teste.

Estado medido do workspace (2026-11-20):

- `Cargo.lock` tem **1 pacote** — o próprio `notation_bundle`. **Zero dependências externas.**
- `deny.toml` registra a intenção: "O workspace ainda não tem dependências externas".
- O parser ChordPro já produz `Song` (metadata + seções) e o `harmony` transpoe/renderiza; falta empacotar isso em disco.
- O layout do pacote já está fixado nos planos v2.1–v2.4 e na ADR-0002: `meta.json`, `song.cho`, `score.musicxml`, `tabs/*.atex`, com referências relativas.

Forças em jogo:

- **Soberania da cadeia de suprimentos:** cada dependência nova é superfície e precisa passar por `cargo deny`. O projeto escolheu zero dependências até aqui.
- **Migração:** formato persistido sem versão de esquema vira dívida irreversível no primeiro release com usuários reais.
- **Interoperabilidade:** o app Flutter consome o bundle; JSON é o formato de troca natural, e o layout já pede `meta.json`.

## Decisão

O Song Bundle v1 é um **diretório** com um manifesto versionado:

```
<song-id>/
  meta.json          # manifesto: schema_version, id, título, artista, tom, datas, arquivos
  song.cho           # fonte ChordPro (fonte de verdade do conteúdo)
  score.musicxml     # opcional
  tabs/*.atex        # opcional
```

1. **Serialização:** `serde` + `serde_json` para o manifesto e para os arquivos de biblioteca e setlist. É a primeira dependência externa do workspace; licenças MIT/Apache-2.0 já estão em `deny.toml`.
2. **Versão de esquema:** `schema_version` inteiro no `meta.json`, com função de migração por degrau (`v1 → v2 → ...`). Ler um bundle com versão maior que a suportada **falha com erro explícito**, nunca ignora campos em silêncio.
3. **Exportação/restauração v1:** copiar o diretório do bundle. Um contêiner de arquivo único (zip) fica para a v1.1 — exigiria uma dependência a mais e não bloqueia o piloto.
4. **Biblioteca e setlist:** `library.json` (índice de ids) e `setlist.json` (ids ordenados) ao lado dos bundles. Setlist referencia id inexistente = erro de validação, não entrada órfã.

## Consequências

- **Ganho:** formato legível, diffável e inspecionável sem ferramenta; o piloto pode fazer backup copiando uma pasta.
- **Ganho:** a versão de esquema entra desde o primeiro commit, quando migrar ainda é barato.
- **Custo:** `serde` + `serde_json` trazem ~5 crates transitivas; o workspace deixa de ser zero-dependência. `cargo deny` passa a valer como gate real, não decorativo.
- **Custo:** exportar como diretório é desconfortável no celular; até a v1.1 o compartilhamento depende do app empacotar.
- **Dívida assumida:** o contêiner de arquivo único fica para a v1.1. Se o piloto exigir compartilhamento por arquivo antes disso, a v1.1 sobe de prioridade.
- **Reversão:** se o dono recusar a dependência, a alternativa é um JSON artesanal (sem dependências) — mais código de escape e parsing para manter, e nenhum ganho de formato.

## Referências

- ADR-0002 · Primeira release pública: cifra, palco local e projeção (R1).
- [Plano de execução v2.6](../plans/plano-mestre-v2.6.md), seção 3 (R1-A) e tabela de riscos.
- [Pendências](../PENDING.md), seção R1 (persistência local).
- `crates/notation_bundle/src/chordpro.rs` (modelo `Song`) e `crates/notation_bundle/src/lib.rs` (layout declarado).
