# ADR-0012 · Letra: de onde vem, como casa com o áudio, e o que não entra

- **Status:** proposto (2026-11-21) — aguarda aceite do dono
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
