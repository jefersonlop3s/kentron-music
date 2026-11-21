# ADR-0011 · MIR: primeiro incremento da análise de áudio (cifra, estrutura, dinâmica)

- **Status:** aceito (2026-11-21) — decisão do dono do projeto: o alvo é a música dele, por transcrição do áudio, sem consultar bibliotecas de cifras
- **Decisão do plano:** R4 (Motor MIR) antecipado como instrumento de trabalho, fora do portão da R1

## Contexto

O ROADMAP coloca o MIR (Music Information Retrieval) no **R4**, com portão de entrada nos spikes S1/S2/S3. O dono pediu o caminho curto: apontar uma música e obter **cifra, estrutura, dinâmica e nuances** a partir do áudio — sem consultar repositórios de cifras.

Levantamento medido nesta máquina (2026-11-21):

- `ffmpeg`/`ffprobe` presentes; **nenhum** DSP ou ML em Python (`numpy`, `scipy`, `librosa`, `torch`, `onnxruntime`, `whisper` ausentes).
- `models/` tem apenas o manifesto (schema, licenças de pesos, `CC BY-NC*` proibido) — **nenhum peso baixado**.
- crates.io e PyPI alcançáveis; `rustfft` disponível (MIT/Apache-2.0, coberto pelo `deny.toml`).

O pedido tem cinco partes com custos muito diferentes. Tratá-las como uma só seria prometer o que não se entrega:

| Parte pedida | Natureza | Veredito |
|---|---|---|
| Cifra (acordes) | DSP: croma + templates | **entregável agora** |
| Dinâmica / pressão sonora | DSP: EBU R128 | **entregável agora, com contraprova** |
| Estrutura (introdução, refrões, interlúdio, final) | DSP: novidade + auto-similaridade | **entregável como inferência**, não semântica |
| Letra | ASR de canto | **não agora** — exige peso (MANIFEST) e correção humana certa |
| Tablatura de solo | pitch tracking + casamento de corda/traste | **não agora** — pesquisa, e ambíguo por natureza |

## Decisão

Primeiro incremento: um crate **`mir_engine`** que lê **WAV** (o `ffmpeg` converte qualquer formato antes) e produz um **relatório** com:

1. **Áudio**: duração, taxa, canais.
2. **Dinâmica**: LUFS integrado (R128), curva de curto prazo, pico verdadeiro, faixa de volume (LRA), e a marcação dos trechos mais baixos/altos.
3. **Andamento**: BPM estimado por autocorrelação de onsets, com a grade de batidas.
4. **Cifra**: croma por quadro, casamento por template (maior, menor, 7, m7, sus4, dim) e suavização temporal → **sequência de acordes por trecho**.
5. **Estrutura**: cortes por novidade espectral, agrupamento por auto-similaridade e **rótulos por repetição** (A/B/C) com papel inferido por posição e repetição — introdução no começo, final no fim, seção que volta é candidata a refrão, seção de contraste no meio é candidata a interlúdio.

**Sem modelos** neste incremento: é DSP puro, então nada a registrar no `models/MANIFEST.toml`. Letra e tablatura ficam para incrementos próprios, cada um com o peso declarado (licença, fonte, hash, ADR) antes de entrar.

## Consequências

- **Ganho:** a cadeia inteira é auditável e roda local; nada de rede no momento da análise. O relatório vira entrada do bundle e da projeção.
- **Custo:** DSP escrito à mão tem armadilha de sinal. Mitigação: prova em áudio **sintetizado** (progressão conhecida, andamento conhecido, trecho piano) e **contraprova do LUFS contra o filtro `ebur128` do ffmpeg** — dois medidores independentes.
- **Risco assumido e declarado:** acurácia de acorde em mixagem real é **moderada**, não exata; a saída é *candidata* com confiança por trecho, para o músico confirmar. Sem essa ressalva, o número vira mentira.
- **Estrutura não é semântica:** o motor detecta repetição e contraste; chamar um trecho de "refrão" é inferência de rótulo, não compreensão musical.
- **R1 intocado:** este trabalho não muda o portão da R1 (ADR-0009/ADR-0010).

## Referências

- [MANIFEST de modelos](../../models/MANIFEST.toml) — contrato de pesos (licença, fonte, hash, ADR) para os incrementos com ML.
- [ROADMAP](../ROADMAP.md) — R4, Motor MIR.
- [ADR-0002](0002-primeira-release-cifra-e-projecao.md) — faixa 1 de idiomas e escopo da primeira release.
