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

1. A letra entra por **fonte declarada**, com procedência explícita no relatório (`Origem`): arquivo do próprio usuário, corpus de **domínio público** embutido, ou (no futuro, com ADR e peso declarado) transcrição por ASR local. **Nenhum raspador de site de letra é escrito**, nem para teste.
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
