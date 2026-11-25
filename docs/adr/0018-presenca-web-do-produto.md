# ADR-0018 · Presença web do produto: landing page, domínio e hospedagem

- **Status:** Proposto (2026-11-25)
- **Autoridade de aceite:** dono do projeto
- **Repositórios/contratos afetados:** novo diretório de site (a definir, ex.: `site/`), `apps/kentron_app` (tela "Sobre o projeto", quando existir), `docs/legal/` (política de apoio, futura), [`README.md`](../../README.md), e o processo de submissão às lojas (teste fechado da Google Play no R1 — ADR-0002; publicação na R5).

## Contexto

O produto precisa de uma **presença web institucional** — uma landing page que cumpra três papéis ao mesmo tempo:

1. **Página institucional** — o que o produto é, changelog, suporte, licença, como contribuir.
2. **Destino do link neutro do app** — o botão "Sobre o projeto" aponta para cá (conformidade com as lojas).
3. **Vitrine do apoio financeiro** — que, pelas regras das lojas, **não pode ficar atrás do link do app** (ver seção de conformidade).

A restrição de custo é explícita: **domínio e hospedagem gratuitos**, sem custo recorrente. E há uma restrição técnica **medida no próprio projeto**: o app web (Flutter + wasm da ponte) **exige COOP/COEP** — é o que `scripts/servir-web.py` configura, e é o que o piloto do OBS contorna por túnel reverso. Logo, o host precisa permitir **cabeçalhos HTTP customizados**.

Três forças de contexto:

- **ADR-0001** já declara o projeto "software livre, sem empresa por trás, **mantido por doação**" — a premissa do apoio existe; falta o mecanismo.
- **ADR-0002:** o teste fechado da Google Play (12 testadores/14 dias) começa **no R1** → conformidade com loja é relevante agora.
- **R5** lista "monetização" como objetivo.

### Fontes verificadas (consultadas em 2026-11-24/25)

| Serviço | Gratuito? | Cabeçalhos customizados | Endereço grátis |
|---|---|---|---|
| **Cloudflare Pages** | sim (500 deploys/mês) | **sim** — arquivo `_headers` (COOP/COEP) | `projeto.pages.dev` |
| **Netlify** | sim | **sim** — `_headers` / `netlify.toml` `[[headers]]` | `projeto.netlify.app` |
| **GitHub Pages** | sim (1 GB, ~100 GB/mês, 10 builds/h) | **não** — a plataforma não expõe configuração de cabeçalhos | `usuario.github.io/repo` |

- **Domínio gratuito:** o caminho real e sem risco é o **subdomínio do host** (zero custo, zero registro). Serviços de subdomínio gratuito existem (`is-a.dev`, `eu.org` — este último com aprovação manual), mas o domínio pertence ao serviço. **Freenom** (`.tk`/`.cf`/`.gq`) ainda se anuncia como gratuito, mas os registros estão **suspensos desde 2023** — não é confiável. Um TLD próprio (`.com`, `.com.br`) **custa**.
- **Consequência técnica:** o **GitHub Pages não serve para o app web** (sem `_headers`, o wasm não sobe em contexto seguro). Serve, no máximo, para documentação estática.

### Conformidade com as lojas (o que decide o desenho do apoio)

Verificado nas fontes primárias (Apple App Store Review Guidelines e Google Play Payments policy):

- **Apple 3.1.1:** desbloquear função exige IAP; e, textualmente, *"Apps may use in-app purchase currencies to enable customers to **'tip' the developer**"* → **gorjeta por IAP é permitida**.
- **Apple 3.1.1(a) "Link to Other Purchase Methods":** links externos de compra só por *entitlement* em regiões específicas; fora delas — e **exceto a loja dos EUA** — *"apps and their metadata may not include buttons, external links, or other calls to action that direct customers to purchasing mechanisms other than in-app purchase."* (o número "3.1.3" das diretrizes trazidas pelo dono corresponde hoje a **3.1.1(a)**).
- **Apple 3.2.1(vi):** **apenas nonprofits aprovados** podem arrecadar dentro do app.
- **Apple 3.2.1(vii):** **presente monetário entre indivíduos sem IAP** é permitido, se opcional e **100% ao recebedor**.
- **Apple 3.2.2(iv):** fora do caso de nonprofit aprovado, **arrecadar para caridade dentro do app é inaceitável**; o app deve ser **gratuito** e **só pode coletar fora do app** (Safari/SMS).
- **Apple 2.2:** TestFlight **não** pode dar recompensa por financiamento coletivo a testadores.
- **Google Play:** billing exigido para bens digitais; **gorjeta direta que nada concede = peer-to-peer, billing não exigido**; **dentro do app não se pode levar a um método de pagamento alternativo** — *"including directly linking to a webpage that could lead to an alternate payment method"*; links administrativos só valem **se a página não levar a pagamento**.

**Conclusão que inverte a intuição:** a "landing page com seção de apoio" **não é segura como destino do link do app** se essa seção levar a pagamento. A seção de apoio vive **fora do caminho do link do app** (site/README/redes).

## Decisão

1. **A landing page é a casa do produto na web** — página estática institucional (descrição, changelog, suporte, licença, como contribuir) e o destino do link neutro do app.

2. **Hospedagem: Cloudflare Pages** (recomendado). É gratuito, permite **`_headers`** (COOP/COEP para o app web) e não cobra banda. **Netlify** é alternativa equivalente. **GitHub Pages fica descartado para o app web** (sem cabeçalhos); serve só para documentação estática, se for o caso.

3. **Um projeto de host, dois papéis:** a landing page e o app web podem viver no **mesmo projeto Cloudflare Pages**, com o `_headers` escopando COOP/COEP ao caminho do app (ex.: `/app/*`) e deixando a landing page sem esses cabeçalhos.

4. **Domínio: começar pelo subdomínio gratuito do host** (`kentron.pages.dev`). **Domínio próprio é opcional e pago** — decisão futura, não pré-requisito. Não usar Freenom nem depender de subdomínio de terceiro como endereço canônico.

5. **Apoio financeiro fora do app.** A seção de apoio vive na landing page/README/redes, **nunca atrás do link do app**. Canais (GitHub Sponsors, Ko-fi, Pix por chave aleatória) são decisão do dono e não entram no código.

6. **Dentro do app, dois caminhos:** (a) **IAP consumível "gorjeta"** (StoreKit / Play Billing), amparado em Apple 3.1.1 e na política do Play; (b) **link institucional neutro** ("Sobre o projeto") → landing page **sem caminho para pagamento**.

7. **Rótulos — o ponto sensível do iOS:**

   | Evitar no app | Usar no app |
   |---|---|
   | "Apoiar o projeto", "Doar", "Pague um café" + link externo | "Sobre o projeto", "Site oficial", "Saiba mais" |
   | destino = checkout (Ko-fi/Pix) | destino = landing page institucional, sem pagamento |
   | página só com pedido de dinheiro | descrição + changelog + suporte |

8. **Apoio não desbloqueia nada** — não concede função, não remove anúncio, não dá VIP (senão vira bem digital e exige billing/IAP).

9. **Fiscal (PF × PJ): em aberto** — decisão do dono, registrada como **pressuposto a nomear**. O risco conhecido (rendimento tributável em PF; Simples/CNPJ quando recorrente) fica declarado, não resolvido aqui.

10. **Segredos:** chaves Pix, Stripe, PayPal e Ko-fi são **credenciais** (skill `manuseio-segredos`) — nada no repositório, em fixture, log ou commit. Pix por **chave aleatória**.

11. **Teste fechado (R1):** nenhuma recompensa por financiamento coletivo a testadores (Apple 2.2).

## Consequências

- **Ganhos:** presença web **sem custo recorrente**; um host que atende landing page **e** app web (COOP/COEP); conformidade nas duas lojas; o caminho neutro **não paga taxa**; alinhado ao ethos "sem empresa por trás".
- **Custos, nomeados:** o IAP introduz **taxa de 15–30%** e **dependência de loja** — tensão real com "livre e descentralizado"; o endereço inicial é um **subdomínio do host** (menos "próprio" que um domínio); a seção de apoio **não pode ficar atrás do link do app** (limita a conversão); o quadro de *steering* está em movimento (DMA na EEA, decisão judicial nos EUA) e pode mudar o permitido.
- **Reversibilidade:** alta. A landing page é estática e portável; trocar de host é reapontar o DNS; o IAP é aditivo.
- **Pressuposto de produto (declarado):** *um mantenedor, sem nonprofit aprovado, app gratuito, sem venda de bem digital.* Se qualquer um mudar, esta ADR precisa ser revista.

## Em aberto (decisão do dono)

1. **Host final:** Cloudflare Pages (recomendado) ou Netlify.
2. **Domínio:** subdomínio do host agora, domínio próprio depois? Qual nome/endereço canônico?
3. **Onde a landing page mora no repositório:** `site/` na raiz, `docs/` publicada, ou repositório próprio?
4. **Lista de canais** de apoio fora do app.
5. **IAP agora ou depois** — a ADR fixa o mecanismo; a implementação pode esperar o aceite.
6. **PF × PJ** (item 9).

## Referências

- [ADR-0001](0001-licenca-apache-2-0.md) — "software livre, sem empresa por trás, mantido por doação".
- [ADR-0002](0002-primeira-release-cifra-e-projecao.md) — teste fechado da Google Play durante o R1.
- [ADR-0013](0013-link-de-musica-referencia-playlist-e-fronteira-do-audio.md) — precedente de "fronteira fora do repositório, por decisão do dono".
- [ROADMAP](../ROADMAP.md) · R5 — "monetização".
- **Apple App Store Review Guidelines** (consultado em 2026-11-24): 3.1.1, 3.1.1(a), 3.2.1(vi), 3.2.1(vii), 3.2.2(iv), 2.2. <https://developer.apple.com/app-store/review/guidelines/>
- **Google Play Payments policy** (consultado em 2026-11-24): billing para bens digitais; FAQ "direct tips"; FAQ "communicate with users about alternative ways to pay". <https://support.google.com/googleplay/android-developer/answer/10281818>
- **Cloudflare Pages** (consultado em 2026-11-25): `_headers` e limites do plano gratuito. <https://developers.cloudflare.com/pages/configuration/headers/>
- **Netlify** (consultado em 2026-11-25): `_headers` e `netlify.toml`. <https://docs.netlify.com/build/configure-builds/file-based-configuration/>
- **GitHub Pages** (consultado em 2026-11-25): limites e ausência de cabeçalhos customizados. <https://docs.github.com/en/pages/getting-started-with-github-pages/github-pages-limits>
- Diretrizes trazidas pelo dono (referência; as correções de número estão registradas no Contexto).
