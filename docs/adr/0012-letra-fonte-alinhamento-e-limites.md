# ADR-0012 · Letra: de onde vem, como casa com o áudio, e o que não entra

- **Status:** **aceito pelo dono do projeto (2026-11-21)**, emendado no mesmo dia pela seção "Emenda · Proveniência em dois eixos e extração pelo usuário"
- **Depende de:** [ADR-0011](0011-mir-primeiro-incremento-de-analise-de-audio.md) (motor de análise)

## Contexto

O dono pediu que a **letra apareça na cifra**, com um método que não crie compromisso legal: buscar a letra numa base barata e cruzar com o que o áudio apresenta, mostrando o mais fiel ao áudio.

Achado que muda o desenho: **não existe fonte oficial de letra que seja gratuita.**

- Os provedores "oficiais" de letra (Musixmatch, LyricFind, Genius, letras do Spotify) são **licenciados e pagos**, e seus termos **proíbem armazenar ou reproduzir** a letra — é exatamente o compromisso legal que o dono quer evitar.
- Portanto "oficial" e "sem custo legal" são incompatíveis hoje. Escolher entre eles é decisão de produto, não detalhe de implementação.

O que é barato **e** legítimo, medido:

| Caminho | Custos | Situação |
|---|---|---|
| Texto do próprio dono (biblioteca, transcrição dele, repertório da igreja) | nenhum | **entra agora** |
| **Letra em domínio público** (obras antigas, hinos pré-1900) | nenhum | **entra agora**, em corpus local com procedência |
| ASR local (Whisper) | peso e CPU reais; exige `models/MANIFEST.toml` | depois, com ADR própria |
| Provedor licenciado de letra | dinheiro + contrato + proibição de armazenar | **não entra** |

E o que o pedido chama de "cruzar com o que o áudio apresenta" é técnica conhecida: **alinhamento forçado** — texto conhecido sendo posicionado no tempo pelo próprio áudio. Isso não depende de licença nenhuma; depende de texto e áudio.

## Decisão

1. A letra entra por **fonte declarada**, com procedência explícita no relatório (`Origem`): arquivo do próprio usuário, corpus de **domínio público** embutido, **letra sob licença declarada do usuário** (licença coletiva de igreja/CCLI e equivalentes, ou provedor licenciado — ver emenda do crédito) ou, no futuro e com peso declarado, transcrição por ASR local. **Nenhum raspador de site de letra é escrito**, nem para teste.
2. O motor **alinha** a letra ao áudio sem modelo: detecta **frases vocais** por energia na banda da voz (200–3500 Hz) com histerese, atribui as linhas às frases pela ordem e pelo tamanho (núcleos vocálicos), encosta no começo de trecho quando o encaixe é bom, e devolve **confiança por linha**.
3. Os acordes são **colocados sobre a sílaba** certa usando o tempo de cada acorde dentro da linha alinhada — a colocação é mecânica e não depende de o nome do acorde estar certo (ADR-0011 já declara a acurácia como candidata).
4. A cifra sai como **objeto do domínio** (`notation_bundle::chordpro::Song`), escrita por `chordpro::write`. O formato deixa de ser texto montado à mão no motor: é o mesmo do app, por construção. O leitor do Modo Palco já desenha "acorde sobre a letra", então **não há binding novo** — o app recebe ChordPro pela ponte que já existe.
5. As **nuances** viram dado: cada linha carrega o trecho (rótulo e papel inferido), o loudness local e a marca de repetição; a seção da cifra leva o papel no nome. O que não se pode sustentar não é mostrado.

## Consequências

- **Ganho:** o caminho inteiro é auditável e sem licença de terceiro; a letra de domínio público e a do próprio usuário funcionam offline; o app renderiza sem mudança de código Dart.
- **Limite aceito e declarado:** para música protegida cuja letra o dono **não** tenha, o motor entrega o **esqueleto** (onde cada linha cai no tempo, seção, dinâmica) e **não** as palavras. Isso é limitação, não bug, e é o preço de não assumir o compromisso legal.
- **Pressuposto de produto nomeado:** "a letra que o usuário já possui é fornecida por ele". O app não é fonte de letra de terceiro; ele é a régua que posiciona a letra sobre o áudio.
- O alinhamento por frase é **aproximado**: não é alinhamento fonema a fonema (que exigiria modelo acústico). A confiança por linha existe para o músico ver onde confiar.
- Corpus de domínio público exige **procedência verificável** por entrada (autor, ano, base) — obra de 1923+ não entra sem checagem, e tradução moderna de hino antigo tem proteção própria.

## Referências

- [models/MANIFEST.toml](../../models/MANIFEST.toml) — contrato de pesos, para o dia do ASR.
- `crates/notation_bundle/src/chordpro.rs` — modelo e escrita do `.cho` (ADR-0006).
- `apps/kentron_app/lib/src/stage/song_reader_page.dart` — o desenho "acorde sobre a letra" que recebe o resultado.


## Emenda proposta · O argumento do crédito (2026-11-21)

O dono levantou: se numa live (Facebook/YouTube) a banda executa música sem crédito e a live cai — o que é justo —, então por que extrair o texto **do áudio** seria diferente de obtê-lo de um **arquivo ou link**, desde que se informe na tela quem detém o direito? Não seria a mesma coisa?

Registro do argumento e da resposta, para que nenhuma sessão futura trate isso como exceção óbvia.

### Crédito não é licença — são direitos diferentes

- **Crédito** cumpre o **direito moral** de paternidade: dizer de quem é. **Reprodução** e **comunicação ao público** (tela, projeção, transmissão) são **direitos patrimoniais**, e são eles que a autorização precisa cobrir. Informar o autor não autoriza nada: quem creditou e não licenciou continua sem licença.
- A derrubada de live costuma vir de duas coisas distintas: (a) a plataforma exigindo **identificação/lincenciamento** da obra executada, e (b) o **Content ID** casando o **fonograma** — a gravação. É por isso que a banda ao vivo frequentemente passa e a gravação original cai: o que cai é o áudio do titular, não a performance.
- **A letra é obra autônoma**, protegida separadamente da melodia e do fonograma. Extrair o texto cantado — **por ouvido, por OCR ou por ASR** — produz uma **cópia** dessa obra. O meio não muda a natureza do ato: se o texto protegido aparece na sua saída, houve reprodução. Por isso "é praticamente transcrição" não é defesa; é descrição do ato.

### O que muda de fato entre os caminhos: quem responde

A diferença não é a "fonte", é **de quem é o ato**:

| Caminho | De quem é o ato | Consequência |
|---|---|---|
| Usuário transcreve para uso próprio | do usuário | ele invoca as exceções de uso privado que existirem |
| **O app** transcreve por ASR e distribui | **do produto** | um produto publicado não invoca uso privado; vira distribuidor de letra |
| Corpus de domínio público | não há titular | sem licença a obter |
| Provedor licenciado | do provedor | o titular autorizou — e o provedor proíbe armazenar |
| **Letra que o usuário já tem, sob licença dele** | do usuário licenciado | **o app é instrumento da licença do usuário** |

### O caminho que resolve — e é o que a ADR passa a nomear

Não é ASR. É **a licença que o usuário já tem**: no caso da igreja, a licença coletiva de igreja (CCLI e equivalentes, que autoriza **reproduzir e projetar letra** e dá cobertura para transmissão; no Brasil, o licenciamento local para execução). Com essa licença declarada, o app deixa de ser fonte concorrente de letra e passa a ser **ferramenta do licenciado** — e o motor agrega o que o texto sozinho não tem: **alinhamento ao áudio, dinâmica por linha, estrutura e tempo**.

Consequência prática na ADR: a decisão não muda (`ASR` continua atrás de decisão de peso e de licença), mas o conjunto de fontes legítimas ganha um membro explícito — **letra sob licença declarada do usuário** —, e a procedência que já viaja na cifra passa a poder carregar *sob qual licença* ela é usada.

### Ressalva

Isto é registro de decisão de produto e engenharia, **não parecer jurídico**. Quem confirma a operação para o caso concreto (igreja, transmissão, país) é a entidade de licenciamento e o contador/jurídico do dono. O que a ADR faz é não fingir que crédito resolve, e não colocar o produto no lugar do titular.


## Emenda · Proveniência em dois eixos e extração pelo usuário (2026-11-21, aceita com a ADR)

O dono aprovou a ADR e trouxe dois pontos operacionais que a fecham.

### 1. A obra e a gravação são coisas diferentes — e o app precisa dizer as duas

Caso do dono: **"a música/letra é de 1822 e a gravação é de ontem"**. Ele está certo, e o erro a evitar é tratar as duas como uma só:

- **Obra** (letra e música): o que é protegido por si. Domínio público quando a regra da jurisdição o declarar — no Brasil, vida do autor **+ 70 anos**, contados de 1º de janeiro do ano seguinte ao falecimento.
- **Fonograma** (a gravação): tem vida própria. **Uma gravação de ontem de obra de 1822 continua protegida**, e o titular é de quem gravou. Regravação em "outra roupagem", com link de YouTube, é fonograma de terceiro.

Consequência de desenho: a procedência passa a ter **dois eixos declarados** — obra **e** fonograma —, cada um com seus campos, e **o app nunca dá veredito de domínio público sobre o fonograma**: não existe essa categoria. O que ele faz é dizer de quem é a gravação que está sendo analisada.

**Como identificar** (não é adivinhação, é ficha):
1. O usuário informa a ficha da obra: **título, autor, ano da obra, ano de falecimento do autor e a base** (hinário, partitura, edição). Sem esses campos, o veredito é **`Indeterminado`** — o motor não chuta (fail-loud, princípio do projeto).
2. **Armadilha nomeada: tradução é obra própria.** Hinário antigo (Harpa Cristã, Cantor Cristão) reúne traduções que podem ser **muito mais novas** que o original. A ficha precisa do **tradutor** e da data dele; sem isso, `Indeterminado`.
3. O veredito sai **com a regra que o produziu** (`vida do autor (1868) + 70 anos`) e **por jurisdição** (Brasil e EUA divergem: no Brasil vale vida+70; nos EUA, obra publicada antes de 1930).
4. A ficha da obra e a origem da gravação viajam na cifra (`{work_status: ...}`, `{recording_source: ...}`), então quem projeta sabe **o que** está usando e **de onde veio o áudio**.

### 2. Extração de letra pelo usuário entra — com o ato no nome dele e circulação privada

O dono decidiu: extrair a letra do áudio é **necessidade real**, e o que sustenta o uso é ficar claro que **quem fornece é o usuário** — a extração é ato dele, não do produto —, com a cifra resultante **privada no grupo da banda**.

Decisão registrada:
- A origem passa a ter `ExtraidaPeloUsuario` (**ato do usuário**), separada de `ASR do produto` — esta última **continua proibida** sem licença, porque aí quem reproduz é o produto.
- Cifra com essa origem nasce com **escopo `grupo`**: o motor marca e **recusa publicação** (`pode_publicar`), e a projeção/exportação pública é caminho bloqueado. Publicar é o que a licença coletiva cobre; o app não substitui a licença do usuário.
- A ficha da obra continua obrigatória nesse caminho: extrair do áudio **não** torna a obra de domínio público nem transfere direito.
- **Pesos:** implementar a extração de verdade exige modelo no `models/MANIFEST.toml` com licença, fonte e hash. O que esta emenda decide é o **enquadramento** — a responsabilidade e o escopo —, que é o que faltava para a extração ser prevista sem colocar o produto no lugar do titular.

### Ressalva que se mantém
Continua **não sendo parecer jurídico**. O que a ADR garante é que o app não afirme domínio público sem ficha, não trate gravação como obra, e não publique o que nasceu privado.

## Emenda · A entrada é do usuário; o produto não decide repertório nem versão (2026-11-21)

O dono fixou a fronteira de produto, ao contar como a equipe trabalha hoje:

> "O OBS Studio + Holyrics é facilitar que as versões que tocamos, as letras sejam transmitidas conforme nossa execução; logo, é **entrada de usuário**, não que o produto decida sobre isso ou repertório."

### Decisão

1. **Repertório e versão são entrada do usuário.** O produto **não** escolhe o que se canta, **não** decide qual versão é a certa e **não** substitui a decisão da equipe sobre arranjo, tom ou letra. O que ele faz é **medir, posicionar e declarar procedência**.
2. **Medido no repertório real (313 linhas exportadas do app atual, 2026-11-21):** o export do usuário **já carrega os dois eixos desta ADR** — `Song name`+`Artist` (obra), `Album`/`Version name`/`Version notes` (versão), `Video` (fonograma com link), `Key` e `BPM` (execução). O modelo do produto **não inventa campos**: ele casa com o que a equipe já preenche.
3. **Os links de letra e cifra que a planilha aponta (`letras.mus.br`, `cifraclub.com.br`, em 54–56 das 313 linhas) são do usuário, não do produto.** O app não os busca: ele **deriva a cifra do áudio** e recebe a letra do usuário, sob a responsabilidade e a licença dele (decisão anterior desta ADR).
4. **O corpus de domínio público atual não cobre este repertório — medido: 0 de 313.** Ele é de hinos ingleses pré-1900; o repertório é português, com **10 linhas** de hinário (Cantor Cristão, Harpa) que exigem **ficha do tradutor** e **303** obras contemporâneas, que nenhuma ficha torna livres. Consequência honesta: para o hinário a ficha resolve; para o contemporâneo, o que resolve é **atribuição + decisão de licença do dono** — e o relatório de uso é o dado dessa decisão, não um veredito que eu dou.

### Consequências

- **Ganho:** a fronteira fica escrita — nem o produto escolhe repertório, nem o corpus se arvora em juiz do que a igreja canta. E o modelo já provado casa com o export existente, o que torna a migração do app atual mecânica.
- **Limite aceito:** `SEM FICHA` continuará sendo a resposta da maior parte do repertório contemporâneo, e isso **não é falha do motor** — é a natureza daquelas obras.
- **Escala real corrigida:** o repertório limpo tem **cerca de 300 obras distintas** (295 normalizando sufixos como "(Ao Vivo)"), não as 1000+ que eu havia lido — aquele número é de execuções por ano, não de obras distintas.

