# ADR-0017 — Separação de bateria: o primeiro modelo no motor

Status: **aceito (2026-11-24)** — decisão do dono do projeto, com a prova conceitual executada.
Autoridade de aceite: dono do projeto. As três ressalvas da prova estão registradas no `PENDING` como pendência de tratativa posterior.
Repositórios/contratos afetados: `crates/mir_engine` (novo módulo de stems), `models/MANIFEST.toml`, ROADMAP R4 (portão S1/S2/S3).

## Contexto

### O problema, medido

A pergunta do dono — *"como você diferencia o tempo forte quando é bumbo e quando é caixa?"* — expôs a falha da medida de metro do motor.

A decisão de `3/4` × `6/8` (`Compasso.dobra`) mede autocorrelação sobre o **fluxo espectral de banda inteira**: bumbo, caixa, chimbau, violão e voz somados. Para essa medida, o acento do bumbo e o da caixa são o mesmo tipo de evento. **Não são:** o bumbo marca o tempo 1 e a caixa marca o contratempo — funções métricas diferentes, e o metro está na relação entre os dois. Somados, `6/8` (bumbo em 1, caixa em 4) e `3/4` (bumbo em 1, caixa em 2 e 3) produzem o mesmo envelope: contraste **0,42 nos dois**, empate exato medido em `Me Derramar`.

A medida musicalmente correta — o **offset da caixa em relação ao bumbo** — foi testada em quatro variantes e **nenhuma separou** os dois compassos:

| variante | resultado |
|---|---|
| offset da caixa sobre o mix | sem estrutura |
| offset sobre a componente percussiva (HPSS, `fe5f317`) | sem estrutura |
| offset com ataques isolados (tolerância 70 ms) | sem estrutura |
| razão entre as taxas dos dois | sem estrutura |

Duas causas, medidas:

1. **A banda de 30–110 Hz carrega o baixo.** A "barra do bumbo" saiu **0,49–1,28 s** — não os 2–3 s de um compasso. Eram notas de baixo, não bumbos.
2. **No arranjo real, bumbo e caixa batem juntos no tempo 1.** O histograma de offset fica dominado por zero: 28–57% na primeira fatia. O contratempo que carrega a informação está encoberto pelo ataque conjunto.

O gargalo **não é o nível do `T`** — é que a bateria não está separada. Nenhum ajuste de limiar resolve: sem separar as peças, não há relação a medir.

### O que existe, medido nesta máquina (2026-11-24)

- **Demucs v4 (htdemucs)** — separação de fontes por rede híbrida transformer. **MIT**. Repositório arquivado em jan/2025; o fork vivo é `adefossez/demucs`.
- **`htdemucs_6s`** — seis fontes: `drums`, `bass`, `other`, `vocals`, `guitar`, `piano`. **Peso medido: 52,4 MB** (`content-length` de `dl.fbaipublicfiles.com`). O de quatro fontes é 80,2 MB — o de seis é **menor**.
- **`sevagh/demucs.onnx`** — **MIT**, inferência por **ONNX Runtime**, com exportação dos modelos de 4 e 6 fontes. O STFT/iSTFT é **movido para fora da rede**, porque não é exportável: o motor já tem o seu próprio.
- **ONNX Runtime tem crate Rust (`ort`)** — o motor não precisaria de Python nem de `torch` em tempo de execução.

O que `htdemucs_6s` resolve é exatamente o que foi medido como causa: **o baixo sai da banda do bumbo e a voz sai da banda da caixa**. A separação interna da bateria (bumbo/caixa/pratos) continua sendo banda + decaimento — o que já existe em `PerfilBandas` e na medição de decaimento.

## Decisão

Adotar **`htdemucs_6s` em ONNX, executado por `ort` no motor**, como **instrumento de trabalho** — o mesmo enquadramento da ADR-0011: aberto por decisão do dono, fora do portão da R1, com o portão do R4 (S1/S2/S3) **seguindo valendo para o produto**.

Alternativas rejeitadas:

- **Só DSP (refinar limiares e bandas).** Rejeitada por medição: quatro variantes, nenhuma separou. O HPSS já está no motor e **não isola** bumbo de baixo nem caixa de voz — o que ele separa é harmônico de percussivo, não uma peça de outra.
- **Modelo por peça de bateria** (bumbo/caixa/pratos como saídas separadas). É a escalada natural, mas **não há opção MIT mantida** identificada hoje. Fica como evolução caso banda + decaimento sobre o stem de bateria não baste.
- **Modelo de 4 fontes.** Rejeitada: o de 6 é **menor** (52,4 contra 80,2 MB) e dá `guitar`/`piano` separados, o que ajuda a banda da caixa.

Escopo declarado desta decisão:

1. **O motor funciona sem o peso.** O caminho DSP continua sendo o padrão; o stem é um refinamento opcional. Sem o arquivo, nada quebra — o relatório sai como hoje.
2. **O STFT é do motor**, não da rede: a `Spectrogram` alimenta o modelo e o inverso é feito no motor.
3. **Determinismo:** mesmos pesos e mesma entrada dão a mesma saída. As fixtures douradas seguem válidas para o caminho DSP; o caminho com stem ganha fixture própria.

## Consequências

- **Ganho:** destrava a medida que falhou — o offset da caixa em relação ao bumbo passa a ser calculável, e com ele a discriminação `3/4` × `6/8` que hoje acerta 1 de 8 no gabarito independente.
- **Custo de peso:** 52,4 MB, a declarar no `models/MANIFEST.toml` (schema 1) com `format = "onnx"`.
- **Licença:** código MIT (Demucs e `demucs.onnx`). **A licença dos PESOS precisa ser confirmada na fonte dos pesos**, como o próprio MANIFEST exige — não basta o repositório dizer MIT.
- **Custo de tempo:** a medir. O README do Demucs estima processamento ≈ 1,5× a duração da faixa em CPU para o modelo original; o ONNX Runtime tende a ser mais rápido, e o número real entra aqui **antes do aceite**.
- **Portão:** esta é a **primeira dependência de modelo** do motor. O ROADMAP diz que o portão vale para a parte com ML (stems, letra, tablatura). **A decisão do dono sobre abrir como instrumento de trabalho ou fazer S1/S2/S3 antes é o que falta para o aceite.**
- **On-device (S1):** o alvo de produto exige que a análise rode no dispositivo. O ONNX Runtime cobre desktop e móvel, mas **o custo em telefone não foi medido** — fica para o spike S1, e não é pré-requisito do incremento de trabalho.
- **Reversibilidade:** alta. O módulo é aditivo e o motor não depende dele; desligar é remover a chamada e o peso.
- **Testes:** herméticos no caminho DSP (como hoje). O caminho com stem precisa de um teste que **não** exija o peso no repositório: o modelo entra por caminho configurável e o teste de integração roda só quando o arquivo existe, declarado como tal.

## Prova conceitual — executada (2026-11-24)

### Licenciamento — conferido na fonte

- **Código:** `LICENSE` do repositório = **MIT**, "Copyright (c) Meta Platforms, Inc. and affiliates".
- **Pesos:** hospedados por Meta em `dl.fbaipublicfiles.com`, **sem termo separado**; o README declara a release como MIT. Os espelhos no Hugging Face (`adefossez/HTDemucs-6s`, do próprio autor) **não declaram licença** — a fonte autoritativa é o LICENSE do repositório.
- **Hash medido, e ele se confere:** `sha256 = 34c22ccb381c6f9fdbf324f04e1e2fe21aaaf293f5ded163a162697ff9a02ddd`. O nome do arquivo é `5c90dfd2-34c22ccb.th` — o sufixo é o **prefixo do sha256**, que é a convenção do Demucs. O arquivo é o que diz ser.

### O modelo

**54.996.327 bytes = 52,4 MB**, exatamente o `content-length` previsto.

### Desempenho medido

**13,5× o tempo real** em CPU (46,8 s de áudio em 3,5 s; 30 s em 16,2 s incluindo a carga do modelo). Uma faixa de 4 minutos sai em **~18 s**. O README estimava ≈1,5× — o número real é **9× melhor**.

### O que a separação resolve — medido em `Me Derramar` (que é `6/8`)

| | mix | stem de bateria |
|---|---|---|
| barra do bumbo | 1,25 s | **2,28 s** |
| bumbos detectados | 22 | **10** |
| caixa no ciclo | 43% · 24% · 10% · 10% · 10% · 5% | **58% · 0% · 0% · 8% · 33% · 0%** |

**A assinatura apareceu:** com o baixo fora da banda, a barra do bumbo salta para 2,28 s e a caixa cai em **0 e na metade do ciclo** — a leitura de `6/8`. Era exatamente a causa medida no contexto desta ADR.

### O que a separação ainda não resolve

No gabarito independente, o resultado **não fecha**: em `binky` (`3/4`) a caixa cai em **1/3 e 2/3** com `T = 0,93 s` (assinatura de `3/4` ✓), mas nas outras cinco o histograma continua dominado pela fatia zero — bumbo e caixa batendo **juntos** no tempo 1.

**A causa mudou de lado.** Antes era a bateria não separada; agora é o **estimador de ataque e de período**, que é meu e é cru: limiar por desvio, espaçamento mínimo e mediana de intervalos. **O modelo não é o gargalo.**

### Veredito para o aceite

A separação **removeu a causa medida** (o baixo na banda do bumbo, verificado em `Me Derramar`) e **não é suficiente sozinha** — o que falta agora é detector de ataque e estimador de período no motor, que é trabalho de DSP deste projeto.

**Entrada pronta para o `models/MANIFEST.toml`, a registrar no aceite:**

```toml
[[model]]
id              = "htdemucs_6s"
purpose         = "separação de fontes para medir a relação bumbo×caixa no metro"
code_license    = "MIT"
weights_license = "MIT"   # LICENSE do repositório; sem termo separado para os pesos
source_url      = "https://github.com/facebookresearch/demucs"
weights_url     = "https://dl.fbaipublicfiles.com/demucs/hybrid_transformer/5c90dfd2-34c22ccb.th"
sha256          = "34c22ccb381c6f9fdbf324f04e1e2fe21aaaf293f5ded163a162697ff9a02ddd"
format          = "pytorch"
approved_in     = "ADR-0017"
```

## Referências e evolução

- Origem: a pergunta do dono (2026-11-24) e o `PENDING` com as quatro variantes medidas e a causa nomeada.
- [`../CONCLUSION.md`](../CONCLUSION.md) · R4 · "O gabarito independente derruba o `5 de 6`" — o gabarito de nove músicas fica como **régua** do que esta decisão precisa destravar.
- [`../CONCLUSION.md`](../CONCLUSION.md) · R4 · "O metro: cinco medidas, uma funciona, um gargalo só".
- ADR-0011 (MIR, primeiro incremento) — o enquadramento de "instrumento de trabalho, fora do portão da R1" é o mesmo.
- ADR-0001 e `models/MANIFEST.toml` — a regra de licença de pesos (nada de `CC BY-NC*`) e o registro obrigatório antes de adotar.


