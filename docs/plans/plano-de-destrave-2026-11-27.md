# Plano de destrave — 2026-11-27

> Registro da sessão de 26–27/11, depois de **duas rodadas paralelas** (gates, agenda, letra/ordem cantada, bundle, instrumentos) e do **aceite da ADR-0018**. O objetivo do documento é um: **transformar bloqueio em decisão de uma linha** — e listar o que já roda sem o dono.
>
> Estado na abertura: `main` = `origin/main` = `c9489d8 → 722d40b → 4ecf1e5` (**publicada**), bateria local verde, **10 branches mergeadas** e **5 worktrees** pendentes de limpeza, disco em **94%**.

## 1 · Todo bloqueio é de um destes quatro tipos

| tipo | remédio | quem executa |
|---|---|---|
| **Decisão** | não decidir por prosa: **folha** com recomendação e consequência, uma linha por item | agente prepara, dono responde |
| **Ação humana** (jurídico, nativos, aparelho, loja, agenda) | **preparar até a ação levar 10 minutos** | agente prepara, dono executa |
| **Terceiro** (gerador do Flutter, fonte que não existe) | **gatilho com data de reavaliação** — espera vigiada, não trabalho | ninguém |
| **Só falta medir** | **medir** (não decidir) | agente |

## 2 · Rodada 3 — o que roda **sem** o dono

| frente | paths | entrega |
|---|---|---|
| **F0 · folhas de decisão** | `docs/plans/**`, `docs/legal/**` | este plano · folha da ADR-0018 · minuta da Política de Privacidade · mapeamento do export · mecanismo do gabarito |
| **F1 · gates/infra** | `.harness/**`, `tools/**`, `scripts/ci-local.sh`, `scripts/exportar_onnx.py` | `verificar_refs.py` que não mente na `main` · licenças Dart/Flutter no pipeline · `exportar_onnx.py` marcado como **registro** |
| **F2 · ASR** | `scripts/medir-timestamp-asr.py`, `scripts/medir-asr-*.py` | viés **calibrado** por modelo · `--grammar` (GBNF) medido · custo do `medium` no **desktop** |
| **F3 · MIR no repertório** | `crates/mir_engine/**`, `scripts/medir-mir-*.py` | terça (limiar 0,4) e alinhamento reconferidos no repertório · **repetição sem declaração** · dinâmica por instrumento |
| **F4 · agenda 2** | `apps/**`, `crates/kentron_ffi/**`, `packages/core_l10n/**` | montar/editar o evento · **"repetir a agenda anterior com versões novas"** · relatório de uso na tela |

Método (o mesmo das rodadas 1 e 2): **um dono por caminho**, entrada no quadro **antes** de editar, documentação fechada pelo integrador, e `main` andando uma vez por frente.

## 3 · Folha de decisões — o mínimo que se precisa do dono

| # | decisão | recomendação | o que destrava |
|---|---|---|---|
| 1 | limpar as 10 branches + 5 worktrees? | **sim** (todas com 0 commits fora da `main`) | ≥6 GB só de `target/`; disco em 94% |
| 2 | frente 5 (web): dupla carga do `wasm_bindgen` | **sim** — última do trio original; mexe em arquivo **gerado** do frb | fim do erro de console |
| 3 | `scripts/exportar_onnx.py` | **manter como registro** (custo zero; o docstring já explica que não exporta) | fecha o GAP sem apagar nada |
| 4 | importador do export do app atual | **sim** — o mapeamento coluna a coluna fica pronto na F0 | 313 músicas sem digitação |
| 5 | gabarito humano (cifra de referência) | **`gabaritos/` fora do repo + ponteiro/hash no repo** (como `~/Downloads` já funciona) | a acurácia da cifra passa a ter gate |
| 6 | ASR: onde o `medium` roda | escolher **entre dois números medidos** (F2: `small` no aparelho × `medium` no desktop + lote) | transcrever o repertório |
| 7 | ASR: léxico/dicionário | decidir com o número do **`--grammar`** (F2) | qualidade da letra |
| 8 | formato do campo de licença | **manter texto livre** (já funciona; a ADR não inventa formato) | nada — só fechar o item |
| 9 | onboarding sem catálogo | decidir depois da folha da F0 (proposta de 3 telas) | produto instalável sem catálogo |
| 10 | R128 à mão × crate `ebur128` | **manter R128** (já contraprovado contra o `ffmpeg`) | nada — fechar o item |
| 11 | ADR-0018: os 6 itens "Em aberto" | **folha com recomendação por item** (F0) | implementação da presença web |
| 12 | `cline/1dce1` — Trilha 1 | a sessão está **parada desde 26/11 21:09** com "aguardando decisão do dono" **sem dizer qual**: pedir o detalhe | destrava uma sessão ociosa |
| 13 | proteger a branch `main` no GitHub | autorizar via `gh` (escopo `repo` já autenticado) | fecha o último item da R0 |
| 14 | uso de Holyrics (qual versão?) | se **não** usa, o item da matriz **morre** | item fantasma a menos |

## 4 · Só o dono faz — mas com o trabalho pronto (≈10 min cada)

- **Jurídico:** a minuta da **Política de Privacidade** fica pronta (F0) e os três blanks do termo (§7.5 retenção, §9.1 canal do titular, §13.2 foro) fecham com uma resposta; sobra **revisar**.
- **Nativos:** pacote do revisor montado (glossário, telas, checklist por idioma) — ele só lê.
- **Alpha/piloto:** roteiro e script já existem (`scripts/piloto-obs.sh`, alpha assistido); precisa da **agenda**.
- **Play Console e aparelho:** ação de conta/dispositivo (o `adb` fora do PATH é problema nosso; o aparelho é dele).
- **Fichas de hinário:** precisa do **impresso** (a ficha vem em cada hino do Cantor Cristão Cifrado).

## 5 · Espera vigiada (ninguém destrava hoje)

| item | espera por | reavaliar em |
|---|---|---|
| migração do `core_l10n` (`flutter_localizations` → `material_ui`) | o **gerador** do Flutter | cada bump do `.flutter-version` |
| índice de autoria do Cantor Cristão | o Herança Batista **publicar** as seções | ao revisar as fichas dos hinários |

## 6 · O que **não** destravar agora

- **Spikes S1–S4/S6 (R2, R3, R5) e a implementação da ADR-0018**: o portão da R1 (piloto de dois cultos) **não fechou** — abrir frente de release futura antes de fechar a atual espalha esforço em vez de reduzir risco.
- **Compasso `4/4` × `6/8` × `3/4`**: não falta decisão nem permissão — falta **sinal**. É pesquisa medida (a F3 cobre a parte que dá).

## 7 · Sequência

1. Dono responde a folha (§3) — tudo em uma mensagem.
2. A rodada 3 corre em paralelo **sem esperar** por ela (só o item 2 depende de um "ok").
3. Com a folha: limpeza (1) → frente 5 (2) → importador (4) → higiene dos itens que eram só fechar (3, 8, 10).
4. Fechamento nos dois registros centrais + quadro, e push.

**Nota de infraestrutura (paga no mesmo dia):** o `pre-push` estava **reprovando todo push de 2+ crates** — `gate_types`/`gate_test` deduplicavam a lista de crates **com os `-p` dentro**, e o `sort -u` apagava os flags. Corrigido e provado nos dois sentidos; foi o que destravou a publicação da `main`.

## Anexo A · Folha da ADR-0018 (os 6 itens de "Em aberto")

Cada linha é uma decisão com **recomendação** — o dono responde `ok` ou corrige. Nada aqui foi implementado.

| # | item | recomendação | por quê (o que a pesquisa da ADR mostra) |
|---|---|---|---|
| 1 | **Host final** | **Cloudflare Pages** | gratuito (500 deploys/mês), aceita `_headers` — que é o que o app web exige (COOP/COEP) — e dá subdomínio pronto. Netlify fica como alternativa se o fluxo de conta incomodar; **GitHub Pages está fora** para o app (não expõe cabeçalhos). |
| 2 | **Endereço canônico** | **subdomínio do host agora** (`kentron.pages.dev`), domínio próprio só quando houver razão | zero custo e zero registro; Freenom (`.tk`/`.cf`/`.gq`) está **suspenso desde 2023** e não entra; um TLD próprio custa e não destrava nada hoje. |
| 3 | **Onde a landing mora** | **`site/` no próprio repositório** | um histórico, um guard, um pipeline e os mesmos tokens do `DESIGN.md`; repositório próprio exigiria harness/CI próprios (custo hoje > benefício). Publicar em `docs/` **não serve**: sem `_headers`, o wasm não sobe. |
| 4 | **Canais de apoio fora do app** | **PIX + GitHub Sponsors** | PIX é o padrão brasileiro (zero taxa de plataforma); Sponsors não tem taxa de plataforma e é o canal natural de projeto em GitHub. Ko-fi/BuyMeACoffee/Padrim ficam como "se um dia". **A landing só linka** — dentro do app a regra das duas lojas proíbe. |
| 5 | **IAP agora ou depois** | **depois** | a ADR fixa o **mecanismo** (gorjeta por IAP é permitida na Apple 3.1.1); implementar exige conta de loja, produto cadastrado e revisão — nada disso é preciso para o alpha da R1, e a taxa (15–30%) só passa a existir quando houver receita. |
| 6 | **PF × PJ** | **PF agora** | não existe PJ; o mecanismo escolhido (link externo + doação) não exige CNPJ. Revisitar se a receita aparecer ou se entrar *nonprofit* aprovado, o que mudaria o pressuposto de produto declarado na ADR. |

**Pressuposto de produto (da ADR, repetido aqui para não se perder):** *um mantenedor, sem nonprofit aprovado, app gratuito, sem venda de bem digital.* Se qualquer um mudar, a ADR precisa ser revista **antes** de implementar.

