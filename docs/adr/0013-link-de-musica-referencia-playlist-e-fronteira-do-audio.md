# ADR-0013 · Link de música: referência, playlist e a fronteira do áudio

- **Status:** proposto (2026-11-22) — aguarda aceite do dono
- **Relaciona-se:** emenda a [ADR-0012](0012-letra-fonte-alinhamento-e-limites.md) §2 e §108 e destrava a etapa 4 da R4 do [ROADMAP](../ROADMAP.md)
- **Contexto de origem:** o dono corrigiu uma premissa registrada — o registro **não** dizia "não usar link"; dizia que *o app não busca letra em site de terceiro* e que *o áudio é obtido fora do app*. As duas coisas foram lidas como uma só.

## Contexto

O dono precisa que o link sirva para **extrair cifra, letra, nuances e tonalidade** e para **compor a playlist que a banda ouve antes do ensaio**. O áudio é insumo de análise, **não** biblioteca: nada é mantido no dispositivo.

As três coisas que estavam fundidas no registro, e o que cada uma é de fato:

| Item registrado | Onde | O que diz de verdade |
|---|---|---|
| "o app não os busca" | ADR-0012 §126 | proíbe **raspar site de letra/cifra** (`letras.mus.br`, `cifraclub.com.br`) — é sobre letra, não sobre link |
| "o áudio é obtido fora do app, por decisão do dono" | ROADMAP R4, etapa 4 | não é proibição: é **etapa aberta**, com a aquisição do arquivo deixada fora |
| "não há `yt-dlp` nem downloader versionado" | PENDING, fim da R4 | constatação de repositório, com "a decisão de como obter o áudio é do dono" |

**Medido nesta sessão (2026-11-22), com fonte primária:**

- **YouTube API Services – Developer Policies** (última atualização 2026-11-06 UTC), em "Additional Prohibitions", sobre Conteúdo audiovisual: *"separate, isolate, or modify the audio or video components of any YouTube audiovisual content made available as part of, or in connection with, YouTube API Services"* e *"create, include, or promote features that play content, including audio or video components, from a background player"*. Ou seja: **pela via oficial, isolar o áudio é proibido**, e só a superfície de reprodução exibida é permitida.
- **Spotify Developer Policy**, efetiva em **15/05/2025**: as seções de áudio são "IV. Streaming and Commercial Use" e "V. Widgets and Audio Preview Clips"; sobre os clipes de prévia, *"must not offer Audio Preview Clips as a standalone service or product"*. **Não há seção, cláusula ou endpoint de download.**
- **Spotify Web API – Quota modes**: desde **15/05/2025** a quota estendida **só aceita pedido de organizações** (entidade legalmente registrada, serviço ativo, mínimo de 250 mil MAU). O modo de desenvolvimento limita a **5 usuários** autenticados, com quota contada.
- **Spotify Web API – referência**: existem `Get Playlist`, `Get Playlist Items`, `Create Playlist` (referências e metadados) e os endpoints de **Player** (`Start/Resume Playback`, Web Playback SDK) — reprodução, nunca arquivo.

**Não verificado nesta sessão** (declarado, não usado como base): a cláusula de "Permissions and Restrictions" dos Termos de Uso de consumo do YouTube — a página de 32 KB volta truncada exatamente nesse ponto em três tentativas; e o bloqueio de `audio-features`/`audio-analysis` citado no plano v2.5 — a URL do changelog do Spotify devolve 404, e os dois endpoints continuam no índice da referência.

## Decisão

1. **O link entra como referência, sempre.** É identidade da obra e do fonograma (título, artista, duração, versão) e a procedência que viaja na cifra. O app **nunca** emite veredito de domínio público sobre o fonograma — não existe essa categoria (ADR-0012 §93–95).
2. **Playlist é lista de referências.** Montar a playlist da banda é montar uma lista de links/IDs, com a API oficial onde ela existe. Nada de áudio é copiado por isso.
3. **A etapa "link → identidade da obra" é implementável hoje**, com API oficial, e é o primeiro terço da etapa 4 da R4. É ela que casa o áudio que o usuário traz com a obra e a versão certas.
4. **A seta "identidade da obra → cifra" continua exigindo o áudio**, e o áudio continua vindo do usuário. O produto **não baixa de provedor nenhum**:
   - **Spotify: não existe caminho.** Não há endpoint de áudio e a quota estendida é fechada para pessoa física. Não há nada a "travar" porque não há direito a liberar — o desenho "o usuário apresenta a licença e o download destrava" não tem contraparte no provedor.
   - **YouTube: o caminho oficial é proibido** pela própria Developer Policy (separar/isolar o áudio). A via não oficial fica **fora do repositório**, como já está.
5. **Das três formas levantadas, C vence; B é rejeitada com motivo.**
   - **A** (o áudio vem de fora) — é o que existe hoje.
   - **C** (o app resolve o link até a **identidade** sempre; pede o arquivo quando a análise precisa de áudio) — **escolhida**: entrega o que o dono pediu sem colocar o produto como autor da extração.
   - **B** (o produto baixa o áudio) — **rejeitada**: colidiria com a ADR-0012 §108, que separa `ExtraidaPeloUsuario` (ato do usuário) de `ASR do produto` (proibida sem licença), e não sobrevive ao ToS de nenhum dos dois provedores. Rejeitada **com motivo escrito**, para não voltar como "melhoria óbvia".

## Consequências

- **Ganho:** a ambiguidade morre. A playlist entra, a identidade da obra entra, e a procedência fica declarada nos dois eixos (obra e fonograma) sem que o produto reproduza áudio de terceiro.
- **Custo aceito:** o passo do arquivo continua manual. A "letra por link" que o dono quer depende de o áudio chegar — o link acelera a *identificação*, não a aquisição.
- **Risco nomeado:** pedir a **quota estendida** do Spotify é inviável para o dono hoje (só organização, ≥250k MAU). Se a playlist do Spotify for requisito, ela viverá no modo de desenvolvimento (5 usuários) até essa decisão mudar — e a lista **local** de links (que não depende de API) continua sendo o caminho sem esse teto.
- **Dívida rastreável:** a cláusula de consumo do YouTube e a mudança de dev mode de fevereiro/2026 do Spotify ficam **não verificadas**, com a tentativa registrada acima. Quem retomar confere na fonte, não herda a hipótese.
- **Pressuposto declarado:** "o áudio que o usuário traz é dele ou ele responde por ele" — nomeado, não implícito.

## Referências

- [ADR-0012](0012-letra-fonte-alinhamento-e-limites.md) — procedência em dois eixos e `ExtraidaPeloUsuario`.
- [ROADMAP](../ROADMAP.md) — R4, etapa 4 (Resolução de Links).
- [PENDING](../PENDING.md) — aquisição de áudio e importação do export.
- YouTube API Services – Developer Policies, "Additional Prohibitions" (última atualização 2026-11-06 UTC).
- Spotify Developer Policy, efetiva 15/05/2025 (seções IV e V); Spotify Web API – Quota modes e referência.