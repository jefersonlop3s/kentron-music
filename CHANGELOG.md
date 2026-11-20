# Changelog

Todos os eventos notáveis do projeto Kentron Music Hub.

## [Infra] Gate de lint do pre-push alinhado à edição do crate (2026-11-20)
### Corrigido
- `.githooks/pre-push` formatava com `rustfmt --edition 2021` **fixo**, enquanto o workspace é edition 2024 e o `ci-local.sh` usa `cargo fmt` (que lê a edição real). Os dois gates do projeto passaram a discordar sobre a ordenação de itens de import e o push era bloqueado por arquivo que o próprio pipeline aprovava.
- A edição passa a ser lida do `Cargo.toml`. Medição que fecha o diagnóstico: **mesmo arquivo e mesmo `rustfmt`** — `--edition 2024` limpo, `--edition 2021` com diff. Provado no negativo: um arquivo deliberadamente mal formatado continua reprovando.

## [R1] Song Bundle v1: persistência (ADR-0006 aceita, 2026-11-20)
### Adicionado
- Módulo `bundle_store`: bundle em diretório com `meta.json` versionado (`schema_version`), `song.cho` como fonte de verdade do conteúdo, índice da biblioteca (`library.json`), setlists (`*.json`) e exportação por cópia do diretório.
- Serializador ChordPro (`chordpro::write`): metadados em ordem determinística e cada acorde recolocado na posição original; ida e volta pelo parser preserva o modelo.
- Cadeia de migração por degrau (`MIGRATIONS`), vazia hoje porque v1 é o primeiro esquema: versão sem degrau é recusada em voz alta, nunca lida pela metade.
- Primeira dependência externa do workspace: `serde` + `serde_json`, 11 pacotes com transitivas, aprovados pelo `cargo deny`.

### Validação
- 55 testes no workspace (12 novos: 4 do serializador ChordPro e 8 de persistência, com diretório descartável). `ci-local.sh` verde e `cargo deny check` ok.

### Premissa corrigida
- "Persistir exige embutir o modelo em JSON": o conteúdo vai como `.cho` (texto) e o JSON carrega só identidade — o modelo em `bundle` segue sem nenhum atributo de serialização.

## [R1] Song Bundle v1: modelo em memória (2026-11-20)
### Adicionado
- Módulo `bundle` em `crates/notation_bundle`: `SongId`, `SetlistId`, `SongBundle`, `Library`, `Setlist` e `BundleError` com `message_key()` estável.
- 14 testes: 11 unitários (invariantes de id, título, duplicata, setlist e resolução) e 3 de integração em `tests/bundle_tests.rs` sobre o fixture `worship.cho`.
- Composição provada com parser e harmonia: o bundle carrega o `.cho` e a música transpõe sem perder a identidade bibliográfica.

### Alterado
- `lib.rs` documenta o que está implementado e o que depende da ADR-0006.

### Premissa corrigida
- "Persistência é o primeiro passo do Song Bundle": o dono decidiu o contrário — o modelo em memória vem primeiro e o formato é decidido depois. Nenhuma dependência externa foi adicionada; o workspace segue sem dependências.

## [Docs] ADR-0006 proposta: Song Bundle v1 (2026-11-20)
### Adicionado
- ADR-0006 (proposta): Song Bundle como diretório com `meta.json` versionado, `serde`/`serde_json` como primeira dependência externa, migração por degrau e exportação por cópia do diretório na v1.

### Alterado
- Índice de ADRs: ADR-0006 registrada como proposta; próximo número livre passa a ADR-0007.

## [Docs] ADR-0005 rejeitada e branches limpas (2026-11-20)
### Alterado
- ADR-0005 marcada como **rejeitada**: a redução de escopo da R1 tratava uma ordem de execução como corte de escopo de uma ADR já aceita. A ADR-0002 permanece integral.
- Plano v2.6, ROADMAP, PENDING e CONCLUSION alinhados: cifra primeiro é ordem de execução; MusicXML/ATEX ficam no marco R1.1, depois do S5, sem emenda de escopo.
- Referências da ADR-0005 corrigidas (caminhos relativos quebrados a partir de `docs/adr/`).

### Adicionado
- Skill `estado-compartilhado-multi-agente` §1c: branch de escopo vira segundo trunk; como distinguir trabalho de sombra antes de ler estado.
- `AGENTS.md`: ao terminar, apagar branches substituídas no mesmo passo; nome de branch é unidade de trabalho, não escopo.

### Removido
- Branches `feature/R1-restante`, `feature/R1-transposicao-harmonia` e `wip/harmony-transpose`: nenhuma tinha conteúdo exclusivo em relação à `main`.

### Corrigido
- `AGENTS.md` afirmava que `CLAUDE.md` e `GEMINI.md` eram symlinks; `CLAUDE.md` é arquivo próprio com as instruções do RTK.
- Referências a `DESIGN.md` em `docs/` apontavam para `docs/DESIGN.md` (inexistente); agora resolvem para a raiz e `verificar_refs.py` sai 0.

### Premissa corrigida
- "O piloto público aguarda o S5" era inferência da proposta, não cláusula da ADR-0002. O portão do piloto é o uso real por uma igreja, offline.

## [Infra] Pipeline local-first (ADR-0004, 2026-11-20)
### Adicionado
- `scripts/ci-local.sh` como portão principal: Rust (`fmt --check`, `clippy`, testes), `cargo deny`, paridade de ARBs e Flutter (`analyze`, `test`, `gen-l10n`).
- ADR-0004 e entrada no índice de ADRs.
- Gates transversais no `.githooks/pre-push` (`deny`, `arb`, `flutter`), com escape por `HARNESS_SKIP`.

### Alterado
- `.github/workflows/ci.yml` sem gatilho em `push` para `main`; mantém `pull_request`, `workflow_dispatch` e `schedule` semanal.
- README, ROADMAP, PENDING e CONCLUSÃO alinhados ao pipeline local.

### Corrigido
- Versão do Flutter fixada em `.flutter-version` (3.50.1), verificada pelo script e espelhada no workflow.
- `packages/core_l10n` incluído no pipeline com `gen-l10n` e `analyze --no-fatal-infos`.
- Localizações geradas (`app_localizations*.dart`) no `.gitignore`; deixam de poluir `git status`.

### Premissa corrigida
- O repositório é público, portanto o GitHub Actions é gratuito e ilimitado. A mudança é por soberania, redução de ruído e não duplicação de validação — não por custo.

## [R1] Transposição e harmonia (2026-11-20)
### Adicionado
- `Chord::transpose` em `chord_naming.rs`, com aritmética de letra e acidente.
- Módulo `harmony.rs` com transposição de música inteira e renderização nos quatro sistemas.
- Testes dourados `golden_harmony/worship.cho` e saídas Common/Latin.
- ADR-0005 (proposta): escopo da R1 com ChordPro primeiro e MusicXML/ATEX em R1.1.

### Corrigido
- Lints de formatação no código de harmonia e nos testes.

### Validação
- 29 testes Rust passando, incluindo acordes invertidos, `6/9`, acidentes duplos e acordes desconhecidos.

## [R1] Cifra e Projeção (parcial integrado, 2026-11-20)
### Adicionado
- Parser ChordPro com modelo canônico e testes dourados.
- Placeholder de parser MusicXML.
- Branch `feature/R1-cifra-projecao` criada a partir da `main` e integrada pelo merge `50c7323`.

### Corrigido
- Lints do parser ChordPro e remoção do módulo `harmony` incompleto da árvore ativa.
- Workflow de CI com ações fixadas por SHA e escopo correto do Flutter.
- App Flutter sem dependências inexistentes de `core_l10n` e `flutter_rust_bridge`.
- Exclusão dos scripts vendorizados do Impeccable da análise Semgrep.
- App migrado para `material_ui` 1.11.1 e token explícito "Sovereign Emerald".
- CI do GitHub verde no run `44665120771` (Rust, licenças, i18n e Flutter).
- Validação local com Flutter 3.50.1: `flutter analyze` sem achados, `flutter test` passando e `flutter gen-l10n` de `core_l10n` sem erro.

## [R0] Fundação e Identidade (2026-11-19)
### Adicionado
- Identidade oficial **Kentron Music Hub** e licença Apache-2.0 com aviso de uso.
- Sovereign Harness com guardrails e git gates.
- Impeccable UI documentado em `DESIGN.md`.
- Monorepo Melos, scaffold Flutter e core Rust `notation_bundle`.
- Ponte `flutter_rust_bridge` como placeholder.
- CI para Rust, licenças, i18n e Flutter.
- ARBs da Faixa 1 (pt-BR, en-US, es-419) e validador `check_arb.py`.
- ADR-0001, ADR-0002 e ADR-0003.

### Publicado
- `main` enviada ao GitHub com `[skip ci]` no primeiro push.
- Branch padrão do repositório alterada para `main`.
