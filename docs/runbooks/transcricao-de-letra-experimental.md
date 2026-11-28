# Transcrição de letra por áudio — procedimento experimental

- **Status:** **experimental**, por decisão do dono (2026-11-27): *"melhor erro que silêncio"*, **sem garantia de sucesso** e **sujeito a revisão**. Não é recurso prometido; é recurso medido.
- **Onde vive:** nada entra no repositório — áudio, letras e stems são material de terceiros ([ADR-0013](../adr/0013-link-de-musica-referencia-playlist-e-fronteira-do-audio.md) e [ADR-0012 §126](../adr/0012-letra-fonte-alinhamento-e-limites.md)).
- **Origem:** [PENDING — áudio real de voz cantada](../PENDING.md); [Insight — como os serviços de transcrição fazem](../insights/2026-11-27-como-os-servicos-de-transcricao-fazem.md).

## 1. Pré-condições

| Item | Valor medido |
|---|---|
| `whisper-cli` (whisper.cpp) | `/tmp/whisper.cpp/build/bin/whisper-cli` — build local |
| Modelo | **`ggml-large-v3-turbo.bin`** (~1,62 GB) — o de **melhor erro medido** (ver §9); `ggml-medium-q5_0.bin` (~1,5 GB) como alternativa mais leve |
| Separador de voz | Demucs 4.1.0, modelo `htdemucs_6s`, `--two-stems=vocals` |
| `ffmpeg` | conversão para 16 kHz mono |
| Verdade | a letra do dono, **fora do repositório** |

## 2. Procedimento (por música)

1. **Separar a voz** — é o passo que mais custa e o que muda o resultado.
   `python3 -m demucs --two-stems=vocals -n htdemucs_6s -o <saida> <audio>`
2. **Normalizar para o ASR** — `ffmpeg -i <vocals.wav> -ar 16000 -ac 1 <voz16k.wav>`
3. **Transcrever** — `whisper-cli -m ggml-large-v3-turbo.bin -f <audio16k.wav> -l pt -np -nt -sns`
4. **Reexecutar com léxico** (opcional, ver §4) — acrescentar `--prompt "<vocabulário>"`.
5. **Conferir à mão.** A saída é **ponto de partida**, nunca letra final.

**`-sns` (`--suppress-nst`) é obrigatório em voz cantada.** Sem ele o modelo classifica o canto como música e devolve `[música] [música] …` — medido, não hipótese.

## 3. Custo medido (17 músicas, 2026-11-27, CPU local)

| Etapa | Tempo |
|---|---|
| Demucs + 3 passes de whisper | **63 s por música** (mín. 37 s, máx. 101 s) |
| Um passe de whisper sobre a voz | 8–15 s |
| Disco por música (stems + 16 kHz) | ~100 MB (**1,7 GB** para 17 faixas) |
| Lote de 17 músicas, ponta a ponta | **16,7 min** |

## 4. Resultado medido — 17 músicas contra a letra do dono (WER; 0 = perfeito)

Braços: **mix** (sem separação) · **voz** (separada) · **voz+domínio** (léxico de 140 palavras dos *outros* 16, fora da amostra) · **voz+própria** (a letra inteira como contexto — teto declarado).

| música | mix | voz | +domínio | +própria |
|---|---|---|---|---|
| Vinho e Pão | 0,658 ᶜ | 0,633 | **0,151** | 0,342 |
| Meu Prazer | 0,874 ᶜ | 0,993 | 0,901 | 0,828 |
| Desde os confins | 1,750 ᶜ | 1,792 | 0,896 | 1,521 |
| Aos Pés da Cruz | 0,426 | 0,398 | 0,778 | 1,102 |
| O Verbo | 1,233 | 1,452 | 1,740 | 2,151 |
| Dom de Amar | 1,320 | 1,897 | 1,495 | 1,412 |
| Diante de Ti | 0,468 | 0,814 | 0,996 | 0,468 |
| Perdão | **0,159** | 0,163 | 0,469 | 0,196 |
| Alto Preço | 1,474 | 1,742 | 1,381 | 1,619 |
| Juntos como família | 1,451 | 1,423 | 1,423 | 1,056 |
| Raridade | 0,899 | 0,331 | 0,943 | 0,501 |
| Maior é Jesus | 1,253 | 1,182 | 0,737 | 0,523 |
| Videira | 0,517 | 0,715 | 0,921 | 0,732 |
| Tudo Posso | 0,450 | 0,579 | 0,737 | 0,450 |
| Graças te Damos | 1,224 | 1,120 | 1,062 | 0,502 |
| Marca da Promessa | 1,216 | 1,646 | 0,959 | 1,357 |
| Eu Te Quero | 0,652 | 1,220 | 0,684 | 1,071 |
| **média** | **0,943** | 1,065 | 0,957 | **0,931** |
| **mediana** | 0,899 | 1,120 | 0,921 | 0,828 |
| **usáveis (WER < 0,5)** | **4 de 17** | 3 de 17 | 2 de 17 | 4 de 17 |

**Vencedor por música:** **mix 7** · voz+domínio 4 · voz+própria 4 · voz 2. ᶜ = número corrigido — ver §8.

**O que estes números afirmam:**

1. **Nenhum braço domina — e o mix é o melhor ponto de partida.** Média: mix **0,943**, a menor; a separação de voz sozinha **piora** (1,065). Por música, o mix vence **7 de 17**.
2. **O dicionário não conserta um modelo que não ouve.** Com o **vocabulário exato da música** (restrição dura, GBNF) o WER médio ainda é **~0,99** e ele **piora** o caso bom — re-medido em *Vinho e Pão* na entrada correta: mix 0,658, gramática **1,000**. O gargalo é **acústico**, não lexical.
3. **O `--prompt` é a única alavanca de ganho real, e é condicional.** *Vinho e Pão*: 0,658 → **0,151**; *Desde os confins*: 1,750 → **0,896**. Mas **piora** *Aos Pés da Cruz* (0,426 → 0,778) e *O Verbo*.
4. **Não há "próximo da perfeição".** O melhor caso é 0,151 (85% das palavras certas); 13 das 17 ficam acima de 0,5. **Conferência humana é etapa, não opção.**

## 5. O interruptor entre os dois métodos — e o proxy **medido** (2026-11-27)

O dono aceitou o custo da separação **como caminho condicional**: roda no **mix** primeiro e um **sinal da própria saída** decide se vale reprocessar. Os três candidatos a sinal foram medidos como **preditor**, contra a verdade (média do WER resultante e quantas vezes a regra escolhe o melhor braço):

| sinal (computável sem verdade) | regra | média do WER | acerta o melhor |
|---|---|---|---|
| — | **sempre mix** | 0,943 | 7/17 |
| — | sempre voz+domínio | 0,957 | 4/17 |
| `[música]`/`♪` | fração > 0 → voz+domínio | 0,942 | 6/17 |
| repetição de 4-grama | ≥ 5 → voz+domínio | 0,944 | 3/17 |
| **confiança do modelo** | **média de `p` por token < 0,85 → voz+domínio** | **0,898** | **8/17** |
| tokens vs letra | > 1,5× → voz+domínio | 0,925 | 7/17 |

**Portanto: o sinal não é `[música]` nem repetição — é a confiança do próprio modelo** (média das probabilidades por token; sai de `-ojf` no `whisper-cli`). Ele disparia em **4 das 17** e baixa a média de 0,943 para **0,898**.

**Limite honesto, e é sério:** o limiar 0,85 foi escolhido **olhando estes mesmos 17 resultados** — é sobreajuste. Valer como regra exige **conjunto separado** (as 2 letras novas ou as 13 restantes do plano). Antes disso, o interruptor é **hipótese com indício**, não recurso.

## 8. Correção de medição (2026-11-27) — o erro que estava nos números

**O que aconteceu:** o primeiro comparativo reusou wavs de uma sessão **anterior** em `/tmp/wav/` (o instrumento tinha `if not w.exists()`), e **nas 3 primeiras músicas da amostra** eles estavam **truncados** — 139 s contra **192 s reais** de *Vinho e Pão*, 173 contra 211, 192 contra 229. As outras 14 coincidiam.

**Consequência:** a média do mix era **1,136** e é **0,943**; e a "prova" de que a separação resgatava a alucinação (*Vinho e Pão* "0,995 → 0,648") era **artefato do arquivo truncado** — na entrada correta, mix **0,658** contra voz **0,633**: ganho desprezível. Quem resgata aquela música é o **`--prompt`** (0,151).

**Lição de método:** (1) instrumento que reusa intermediário **precisa provar a origem** do que reusa — aqui, duração/tamanho contra o original; (2) repetir a decodificação na **mesma entrada** (md5 idêntico) foi o que apontou a **entrada**, e não o modelo, como causa; (3) **agregado não substitui conferir por música**.

## 9. Modelo e decodificação: medidos (2026-11-28, 17 músicas)

Cinco braços no mesmo material e no mesmo placar (`-sns`, verdade = a letra do dono):

| braço | média | mediana | usáveis (< 0,5) |
|---|---|---|---|
| `medium` no mix (baseline anterior) | 0,943 | 0,899 | 4 |
| `medium --no-fallback -bo 1` | **0,970** | 0,986 | 3 |
| **`large-v3-turbo` no mix** | **0,847** ⭐ | **0,702** | **5** |
| `large-v3-turbo` na voz separada | 0,885 | 0,706 | 5 |
| `medium` na voz separada | 1,065 | 1,120 | 3 |

**Três conclusões, todas medidas:**

1. **O melhor modelo é o `large-v3-turbo`**, no **mix** (sem separação de voz): melhor média **e** melhor mediana (**−22%**), sem os 1,7 GB de stems e sem os 63 s/música do Demucs — **11 s/música** no total. O ganho é de **erro**, não de tempo.
2. **A separação de voz não paga** (turbo: mix 0,847 × voz 0,885); ela só transforma a *Juntos como família* (1,465 → **0,324**). O caminho padrão é o áudio como veio.
3. **A hipótese de que os laços de invenção eram artefato do fallback de temperatura está REFUTADA:** desligá-lo (`--no-fallback -bo 1`) **piora** a média e a mediana. Os laços são do material cantado.

**E o `--prompt` não é alavanca portável** — ele **ajuda o `medium` e atrapalha o `turbo`** (mesma construção de léxico, mesmas 17 músicas):

| par | sem prompt → com prompt | efeito |
|---|---|---|
| `medium` na voz | 1,065 → 0,957 | **ajuda** (−0,108) |
| `turbo` na voz | 0,885 → 0,925 | atrapalha (+0,040) |
| `turbo` no mix | 0,847 → 1,043 | **atrapalha muito** (+0,196) |

No turbo o prompt piora **12 de 17** no mix e **12 de 17** na voz (usáveis caem de 5 para 1 e para 2). Ele ainda salva casos isolados (*Raridade* 1,160 → **0,364**; *Desde os confins* 2,208 → 1,333; *Dom de Amar* 0,856 → 0,670): é **efeito por música**, não alavanca de produto. **O desenho que fica: `large-v3-turbo` no mix, sem prompt e sem separação de voz.**

### 9.1 Instrução, fusão e `-mc 0` (2026-11-28, mesma rodada de 17)

| braço | média | mediana | usáveis (< 0,5) |
|---|---|---|---|
| **mix** (referência) | **0,847** | 0,702 | 5 |
| voz | 0,885 | 0,706 | 5 |
| **mix + instrução** (`--prompt "<instrução curta>" --carry-initial-prompt`) | **1,097** | 1,062 | 5 |
| voz + instrução | 1,126 | 1,040 | 4 |
| mix + instrução + `-mc 0` | 0,991 | 0,898 | 5 |
| **fusão (mix + voz), por confiança** | 0,896 | **0,656** | 5 |
| fusão com instrução | 1,107 | 1,078 | 6 |

**Três achados, todos medidos:**

1. **Instrução não é lista de palavras, mas também não é ganho.** `--prompt` no whisper é **texto prévio**, não ordem — e por padrão **só vale na primeira janela de 30 s**; sem `--carry-initial-prompt` a instrução nem chega às demais. Com o carry ligado, a instrução **piora a média** (0,847 → **1,097**).
2. **O mecanismo do dano é verbosidade:** a instrução faz o decoder **gerar texto além do áudio**. Medido em palavras emitidas: *Marca da Promessa* **177 → 430**, *Eu Te Quero* 205 → 319, *Aos Pés da Cruz* 196 → 261. O começo costuma ficar melhor; o fim é invenção.
3. **A fusão por confiança é estabilizadora, não média-melhoradora:** melhora a **mediana** (0,702 → **0,656**, a melhor de todas) mas não a média — corrige o caso em que um dos dois áudios falha (*Juntos* 1,465 → 1,028; *Raridade* 0,938 → 0,599; *Aos Pés* 0,255 → 0,227) sem estragar os outros.

**Teto medido (oráculo):** escolhendo a melhor arma **por música**, a média é **0,656** (contra 0,847 do mix sozinho). Vitórias: voz 5 · mix 3 · mix+`-mc 0` 3 · mix+instrução 2 · fusão 2 · fusão+instrução 2. É esse o ganho máximo que um interruptor por música pode capturar — e os sinais para tentar capturá-lo (confiança `p` e **verbosidade** contra o esperado) já têm indício medido.



## 6. Falhas conhecidas e recuperação

| Falha | Causa | Ação |
|---|---|---|
| Saída é só `[música] …` | canto classificado como música | **faltou `-sns`** — obrigatório |
| Uma linha repetida por minutos | loop de alucinação | áudio de voz ausente/abafado; separar a voz e reexecutar |
| Texto pior que o mix | separação removeu harmonia que ajudava | manter o mix (o interruptor de §5) |
| Demucs sem saída | entrada em formato não suportado | converter com `ffmpeg` antes |
| Disco cheio | ~100 MB por música | apagar os stems após medir |

## 7. O que falta

- Medir o **proxy** do §5 como preditor real.
- Estender a amostra a **19 músicas** (as 17 + as 2 letras novas) e, se o dono quiser, às 30 do plano de amostragem (11% do repertório).
- Testar **timestamps**: o alinhamento forçado (ADR-0012) dá sílaba a sílaba; a transcrição crua **não** dá.
