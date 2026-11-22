# ADR-0014 · Análise harmônica pós-extração: tonalidade, baixo e qualidade do acorde

- **Status:** proposto (2026-11-22) — **bloco 1 (centro tonal por perfil) aceito pelo dono**; os demais blocos seguem em proposta
- **Especificação:** ditada pelo dono em prosa (2026-11-22). O texto dele é **a especificação**, não exemplo ilustrativo.
- **Causa comum de:** três itens abertos do [PENDING](../PENDING.md) — inversões, terça fraca não resolvida e tensão sem adivinhação.

## Contexto

O que o motor faz hoje, medido: croma por quadro + casamento por template → **conjunto de classes de altura**, sem a nota do baixo; e `tom_estimado` devolve a **fundamental mais frequente**. Resultados registrados: devolveu `B` numa música em **E maior**, e leu **`Am` como A maior**.

**Por que a leitura pende para maior — e não é só limiar.** O 5º harmônico da fundamental fica uma **terça maior** acima (razão 5/4, ~386 cents contra 400 do temperamento igual). Um acorde de fundamental + quinta, sem terça executada, **já põe energia real na terça maior** pela série harmônica da própria fundamental. Somar tudo num croma único e casar contra template de maior/menor faz o template "maior" vencer por construção. Aumentar o limiar não corrige isso; muda só o ponto onde o erro aparece.

**O processo que o dono usa em palco** (resumo fiel, para não reescrever a ideia dele):

1. Localiza a **sensível** e a qualidade do acorde no grau 7: `G#dim` versus `G#m` decide entre Lá maior e Mi maior; e a **presença de D ou D#** (grau 4) é o discriminador mais forte.
2. Com terça fraca, toca o acorde como **quinta** (ou 2ª/9ª) — são intervalos "seguros", presentes nas duas qualidades.
3. Método do **baixo primeiro**: achar o pulso e o compasso, identificar as notas mais graves e como se repetem, e comparar contra escalas maior/menor (natural, harmônica, melódica).
4. Em compasso 4/4, a nota estrutural aparece no **primeiro e terceiro tempo**; em 6/8 e tercinas, noutra posição. A primeira nota grave do compasso diz se o acorde está **na tônica, na terça ou na quinta** — é a inversão.
5. **Empréstimo modal e mudança de tonalidade** (estrofe × refrão, cantatas, jazz, prog) precisam ser identificados, não tratados como erro.
6. A **melodia fecha a análise**: linha melódica (subida, descida, tempo), comparada contra a escala e depois contra as frequências harmônicas **dentro dos tempos do compasso**. Mesmo com terça instrumental indefinida, a melodia (voz ou solo) define maior ou menor.
7. **Ciclo de quartas e quintas** como eixo para achar tonalidade em música popular.

**O que o motor tem e o que falta:** tem batida (autocorrelação de onsets) e grade de batidas; **não tem compasso/downbeat**. Tem estrutura por novidade timbrística; **não tem região harmônica**. Tem faixa vocal por energia (200–3500 Hz); **não tem F0 da melodia**.

## Decisão

Camada de análise **pós-extração**, rodando depois do DSP atual, em seis blocos. O eixo do ciclo de quintas do dono **é** o eixo do algoritmo padrão de centro tonal (perfis de Krumhansl/BP) — a intuição e o método registrado convergem.

1. **Centro tonal por região**, por correlação com perfis de tonalidade no eixo de quintas. Substitui `tom_estimado` (fundamental mais frequente). Saída com a regra que a produziu, como na ADR-0012.
2. **Região harmônica** segmentada por **mudança de centro**, separada da segmentação por novidade timbrística — as duas não coincidem, e a estrofe/refrão com tonalidades diferentes exige a primeira.
3. **Compasso e downbeat**, pré-requisito declarado dos blocos 4 e 5. Sem ele, "primeiro e terceiro tempo" não existe no dado.
4. **Baixo por compasso**: F0 predominante na banda grave com continuidade harmônica, separando o **transiente do bumbo** (percussivo, banda larga) do **tom sustentado do baixo** (harmônico). O baixo define a **inversão**, não a tônica.
5. **Melodia como árbitro da terça**: F0 da linha melódica sobre os tempos do compasso decide a qualidade quando o croma é ambíguo. Sem melodia e sem terça, o veredito é **`Indeterminado`** — fail-loud aplicado à harmonia, não palpite.
6. **Afinamento global e empréstimos**: estimar o desvio de A440 antes de casar contra escalas (gravação varispeed ou afinada fora do padrão contamina tudo), e manter um **conjunto explícito de empréstimos modais permitidos** por modo — senão a rejeição de outlier descarta exatamente o que o dono quer identificar.

**Ordem de execução** (barato e de alto valor primeiro): centro tonal → afinamento → terça pela melodia → baixo/inversão → compasso/downbeat.

## O que a prosa não cobre — e exige etapa própria

A prosa é fiel ao que o ouvido faz; as lacunas abaixo não são musicais, são de **sinal**. Cada uma é um bloco de trabalho nomeado, para não virar surpresa:

1. **Compasso/downbeat.** "Primeiro e terceiro tempo do compasso" pressupõe o compasso. O motor tem batida, não tem barra.
2. **Bumbo × baixo.** A nota mais grave do compasso é, com frequência, o **bumbo** (fundamental na faixa de 50–80 Hz), não o baixo. E há baixo de cinco cordas e afinações graves. A separação é transiente × sustentado, não corte de frequência.
3. **Série harmônica.** Já explicada no Contexto: a terça maior "aparece" sem ter sido tocada. É a causa mecânica do viés, e continuará existindo depois de qualquer ajuste de limiar.
4. **Modo sem sensível.** Ausência de acorde diminuto no grau 7 **não** significa "não é maior": em Mixolídio ou Dórico o grau 7 é natural. Exige perfis modais, não só o par maior/menor.
5. **Empréstimo é intencional.** Um filtro que "limpa" acorde fora da escala apaga justamente o dado que o dono quer identificar. Precisa de conjunto permitido, não de rejeição de outlier.
6. **Região harmônica ≠ seção.** As duas segmentações usam sinais diferentes e não vão coincidir.
7. **F0 é algoritmo novo, não ajuste do croma.** A resolução exigida é de semitom (~6% em Dó4), com vibrato e portamento, e a voz pode estar dobrando o instrumento.
8. **Afinamento.** Gravação varispeed ou afinada fora de A440 desalinha **toda** comparação contra escala.
9. **Textura sem terça e sem melodia** (introdução em power chord): `Indeterminado` é a resposta correta, não falha.
10. **Peso dos graus.** `C#m` é diatônica em E maior (vi) **e** em A maior (iii) — **não discrimina** as duas. Quem discrimina é o grau 4 (`D` × `D#`) e o grau 7 (`G#dim` × `G#m`). A prosa cita `C#m`/`F#m` junto do `G#m`; só o grau 7 e o grau 4 têm poder de decisão.
11. **Janela maior que um compasso.** A melodia pode cantar a quinta naquele compasso e a terça no seguinte; a decisão precisa de janela harmônica, não de um compasso isolado.

**Retificação de registro:** o PENDING afirma que inversão "exige baixo isolado (stems)". Isso é **hipótese, não medição**. Transcrição de linha de baixo e extração de melodia predominante são tarefas de MIR estabelecidas **sem** separação de fontes. Esta ADR adota o caminho sem stems e rebaixa o stems ao que ele é aqui: **melhoria de acurácia, não pré-requisito**.

## Consequências

- **Ganho:** ataca a causa comum dos três itens abertos. A terça deixa de ser palpite, a inversão passa a existir e a tonalidade deixa de ser "a nota mais frequente".
- **Custo:** é pesquisa, não ajuste. F0 de melodia, F0 de baixo e downbeat são algoritmos novos, cada um com a sua própria validação.
- **Risco assumido e declarado:** a acurácia continua **candidata com confiança**, como a ADR-0011 já declara. A diferença é que agora o candidato carrega a **razão** da escolha (perfil, grau 4, grau 7, melodia) em vez de só o rótulo.
- **Risco de regressão:** substituir `tom_estimado` muda a saída do relatório. O valor antigo (fundamental mais frequente) fica registrado no CHANGELOG para comparação.
- **Custo evitado:** sem esta camada, cada correção de acorde vira ajuste de limiar — e o viés de série harmônica volta por baixo.
- **Pressuposto declarado:** a análise é de **uma execução** (a gravação trazida pelo usuário), não de "a música" em abstrato. Versões diferentes dão resultados diferentes, e isso é o desenho da ADR-0012.
- **Fora de escopo:** cifra em monitor de palco (TV) não entra aqui; é transporte de tela, não harmonia.

## Referências

- [ADR-0011](0011-mir-primeiro-incremento-de-analise-de-audio.md) — DSP do primeiro incremento e a ressalva de acurácia "candidata".
- [ADR-0012](0012-letra-fonte-alinhamento-e-limites.md) — procedência e o que o motor pode afirmar.
- [PENDING](../PENDING.md) — inversões, terça fraca, tensão, centro tonal e stems.
- [ROADMAP](../ROADMAP.md) — R4, etapa 1 (Análise Local).
- Perfis de tonalidade (Krumhansl–Schmuckler) e o eixo de quintas; transcrição de linha de baixo e melodia predominante como tarefas de MIR.

## Emenda · Bloco 1 aceito e a separação ritmo × baixo (2026-11-22)

O dono **aceitou o centro tonal por perfil** — primeiro item da ordem de execução — e acrescentou à prosa: a nota mais grave pode ser o bumbo, e isso se resolve **separando o kit de bateria do baixo/contrabaixo**. Pediu o que existe em open source.

**Achados medidos (2026-11-22):**

- **HPSS — harmonic-percussive source separation** (Fitzgerald, 2010): filtro de **mediana no espectrograma** — mediana ao longo do **tempo** dá o componente **harmônico** (sustentado), ao longo da **frequência** dá o **percussivo** (transiente), com máscaras suaves. A implementação de referência (`librosa.decompose.hpss`) tem `kernel_size=31`, `power=2` e `margin=1`. **Não usa modelo nem pesos** — o módulo se descreve como "HPSS, nearest-neighbor filtering e decomposição matricial (NMF)". Em Rust, cabe no `rustfft` que o crate já usa.
- **Demucs v4 (`htdemucs`)** — 4 stems, incluindo `drums` e `bass`. O README declara **MIT** para o projeto; o repositório foi **arquivado em 2025-01-01** (fork ativo: `adefossez/demucs`). Pesado: PyTorch, ~7 GB de GPU nos argumentos padrão, ~1,5× a duração da faixa.
- **Open-Unmix (UMX/UMXL)** — MIT no repositório; 4 stems; SDR no MUSDB18: `bass 5,23 / drums 5,73` (UMX) e `bass 6,02 / drums 7,15` (UMXL).
- **madmom** — tem transcrição de baixo, mas os pesos são **CC BY-NC-SA**: **proibido** pelo `deny.toml` e pelo `MANIFEST`.
- **Ressalva honesta:** o MIT do **repositório** não declara a licença dos **pesos** em nenhum dos dois primeiros. O `MANIFEST` exige confirmar na fonte dos pesos, não no repositório — então nenhum dos dois entra hoje.

**Decisão da emenda — sem modelo primeiro, em duas etapas:**

1. **HPSS** remove o ataque e a banda larga do bumbo da banda grave. Barato, sem modelo, sem `MANIFEST`.
2. **Discriminador de harmonicidade** no resíduo: a nota do baixo é uma **série harmônica sustentada e estável** (2f, 3f…, contínua no tempo); o corpo do bumbo é decaimento quase senoidal, com harmônicos fracos e vida curta. Decide bumbo × baixo sem modelo.
3. **Modelo de 4 stems só se 1 + 2 não bastarem** — e aí com spike S1, entrada no `MANIFEST` e a licença dos pesos confirmada na fonte.

**Consequência:** o baixo deixa de depender de stems para existir. O ponto de partida é o `rustfft` que já está no workspace; stems viram **melhoria de acurácia**, definitivamente.