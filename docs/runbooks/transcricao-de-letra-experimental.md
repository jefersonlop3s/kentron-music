# Transcrição de letra por áudio — procedimento experimental

- **Status:** **experimental**, por decisão do dono (2026-11-27): *"melhor erro que silêncio"*, **sem garantia de sucesso** e **sujeito a revisão**. Não é recurso prometido; é recurso medido.
- **Onde vive:** nada entra no repositório — áudio, letras e stems são material de terceiros ([ADR-0013](../adr/0013-link-de-musica-referencia-playlist-e-fronteira-do-audio.md) e [ADR-0012 §126](../adr/0012-letra-fonte-alinhamento-e-limites.md)).
- **Origem:** [PENDING — áudio real de voz cantada](../PENDING.md); [Insight — como os serviços de transcrição fazem](../insights/2026-11-27-como-os-servicos-de-transcricao-fazem.md).

## 1. Pré-condições

| Item | Valor medido |
|---|---|
| `whisper-cli` (whisper.cpp) | `/tmp/whisper.cpp/build/bin/whisper-cli` — build local |
| Modelo | `ggml-medium-q5_0.bin` (~1,5 GB) |
| Separador de voz | Demucs 4.1.0, modelo `htdemucs_6s`, `--two-stems=vocals` |
| `ffmpeg` | conversão para 16 kHz mono |
| Verdade | a letra do dono, **fora do repositório** |

## 2. Procedimento (por música)

1. **Separar a voz** — é o passo que mais custa e o que muda o resultado.
   `python3 -m demucs --two-stems=vocals -n htdemucs_6s -o <saida> <audio>`
2. **Normalizar para o ASR** — `ffmpeg -i <vocals.wav> -ar 16000 -ac 1 <voz16k.wav>`
3. **Transcrever** — `whisper-cli -m ggml-medium-q5_0.bin -f <voz16k.wav> -l pt -np -nt -sns`
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
| Vinho e Pão | 0,995 | 0,633 | **0,151** | 0,342 |
| Meu Prazer | 1,265 | 0,993 | 0,901 | 0,828 |
| Desde os confins | 4,312 | 1,792 | 0,896 | 1,521 |
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
| **média** | **1,136** | **1,065** | **0,957** | **0,931** |
| **mediana** | — | 1,120 | 0,921 | 0,828 |
| **usáveis (WER < 0,5)** | **4 de 17** | 3 de 17 | 2 de 17 | 4 de 17 |

**O que estes números afirmam:**

1. **Nenhum braço domina.** A separação de voz e o `--prompt` melhoram a **média**, mas cada um **piora músicas específicas** (*O Verbo* 1,23 → voz 1,45; *Videira* 0,52 → +domínio 0,92).
2. **O dicionário não conserta um modelo que não ouve.** Com o **vocabulário exato da música** o WER médio ainda é **0,99** — o gargalo é **acústico**, não lexical. Restrição dura (gramática GBNF) ainda **piora** o caso bom.
3. **A separação de voz tira a alucinação do caminho.** *Vinho e Pão*: no mix a saída é `Criando Criando Criando… Amém!` (WER 0,995); na voz aparece a letra real (0,633 → **0,151** com léxico).
4. **Não há "próximo da perfeição".** O melhor caso é 0,151 (85% das palavras certas); o resto fica entre 0,5 e 2,1. **Conferência humana é etapa, não opção.**

## 5. O interruptor entre os dois métodos (decisão do dono)

O dono aceitou o custo da separação **como caminho condicional**: roda no **mix** primeiro; o **sinal** decide se vale reprocessar na voz. Medido em 17:

- **mix já bom (WER < 0,5, 4 músicas):** manter o mix **nunca perdeu** (2 empates, 2 vitórias do mix).
- **mix ruim (13 músicas):** a família "voz" venceu em **8 de 13**.

**Limite honesto:** esse limiar foi medido **com a verdade em mãos** (pós-hoc). No produto **não existe WER** — o sinal tem de ser um **proxy da própria saída** (`[música]`, loop/repetição, proporção de tokens). **Esse proxy ainda não foi medido como preditor.** É a próxima medição, e sem ela o interruptor é hipótese.

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
