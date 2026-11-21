# ADR-0006 · Song Bundle v1: formato de persistência e primeira dependência externa

- **Status:** **aceito** (2026-11-20) — decisão do dono do projeto. O modelo em memória (`SongBundle`, `Library`, `Setlist`) foi entregue antes, como o dono pediu; o formato desta ADR passa a valer a partir dele
- **Decisão do plano:** R1-A (biblioteca local, setlists e Song Bundle versionado)
- **Não substitui:** ADR-0002 (o layout `meta.json` / `.cho` / `.musicxml` / `tabs/*.atex` já vem de lá)

## Emenda · A agenda vira o relatório de uso (2026-11-21)

O dono respondeu como a igreja trabalha de verdade, e isso redesenha o pedido:

- **Não existe relatório de uso** — o que existe é **agenda por culto/evento**, **uma playlist por evento**, músicas que **repetem entre eventos** e, quando repetem, **nem sempre na mesma versão**.
- **Não há formato de licença** a respeitar, e o repertório passa de **1000 músicas diferentes por ano** (hinos do Cantor Cristão e alguns da Harpa Cristã somados a música contemporânea: Fernandinho, Projeto Sola, Aline Barros, Renascer Praise). Enumerar é inviável.

### Decisão

1. **A agenda é o registro.** `Setlist` deixa de ser só uma lista e passa a ser o **evento**: ganha `data`, `tipo` (culto, ensaio, evento) e entradas com **versão** e **tom**. O relatório de uso **não pede hábito novo** — ele é subproduto do que a equipe já mantém. Pressuposto nomeado: *a agenda do culto é preenchida de qualquer forma*.
2. **Versão é campo de primeira classe** da execução: a mesma obra em dois cultos pode ser outra versão (outro arranjo, outra gravação, outro tom). Isso liga direto aos dois eixos da ADR-0012 — a **obra** se repete, a **versão** muda.
3. **`schema_version` vai a 2**, com passo de migração: entrada antiga (id solto) vira execução sem versão declarada. **Não se adivinha versão:** ausente é ausente, e o relatório diz "versão não declarada".
4. **O relatório classifica o que a equipe realmente toca**, em vez de exigir a enumeração do repertório: para cada obra, o veredito de domínio público **quando houver ficha** (ADR-0012) e, quando não houver, `sem ficha` — o que produz a **lista de fichas a conferir, ditada pela própria playlist**. É a resposta ao "impossível enumerar": não se enumera; classifica-se o que se canta.
5. **Licença continua sendo decisão do dono.** Sem formato declarado, o app **não inventa** um: ele diz, por evento, quantas obras são de domínio público e quantas são **sem ficha** — que é a informação de que ele precisa para decidir sobre licença coletiva, sem eu arbitrar por ele.

### Consequências

- **Ganho:** o relatório sai do dado que já existe; a lista de fichas fica finita e priorizada pelo uso real (os hinos que a igreja de fato canta, não os 400 do hinário); a versão distingue execuções que antes se confundiriam.
- **Custo:** mudança de formato persistido com migração — feita agora, enquanto não há dados de produção a converter.
- **Limite declarado:** sem ficha da obra, o relatório **não** afirma domínio público nem protegido; afirma `sem ficha`. Mentir aqui seria pior que não responder.


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
- **Custo:** medido em 2026-11-20 — `serde` + `serde_json` somam **11 pacotes** no `Cargo.lock` (`serde_core`, `serde_derive`, `proc-macro2`, `quote`, `syn`, `unicode-ident`, `itoa`, `memchr`, `zmij`); o workspace deixa de ser zero-dependência. `cargo deny` passa a valer como gate real, não decorativo.
- **Custo:** exportar como diretório é desconfortável no celular; até a v1.1 o compartilhamento depende do app empacotar.
- **Dívida assumida:** o contêiner de arquivo único fica para a v1.1. Se o piloto exigir compartilhamento por arquivo antes disso, a v1.1 sobe de prioridade.
- **Reversão:** se o dono recusar a dependência, a alternativa é um JSON artesanal (sem dependências) — mais código de escape e parsing para manter, e nenhum ganho de formato.

## Referências

- ADR-0002 · Primeira release pública: cifra, palco local e projeção (R1).
- [Plano de execução v2.6](../plans/plano-mestre-v2.6.md), seção 3 (R1-A) e tabela de riscos.
- [Pendências](../PENDING.md), seção R1 (persistência local).
- `crates/notation_bundle/src/chordpro.rs` (modelo `Song`) e `crates/notation_bundle/src/lib.rs` (layout declarado).
