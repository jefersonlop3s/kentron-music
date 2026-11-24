# As faixas de Hz de cada peça da bateria — e o que não separa (2026-11-22)

**Origem:** autorização do dono para pesquisar, baixar e medir as músicas que ele
indicou, para sair do repertório do hinário e testar a discriminação de metro em
material **com bateria de verdade**.
**Uso aqui:** base de vocabulário para a **partitura de bateria** (a "tablatura")
e teste do perfil de acento por instrumento que o motor ganhou.

## Material

Nove faixas, três por família, todas com kit acústico audível:

| família | faixas | o que se espera |
|---|---|---|
| `3/4` com bateria | Manic Depression · My Favorite Things · Someday My Prince Will Come | um acento por compasso, caixa nos tempos 2 e 3 |
| `6/8` com bateria | House of the Rising Sun · Nothing Else Matters · Since I've Been Loving You | dois acentos por compasso, caixa no tempo 4 |
| `4/4` (mapa de Hz) | Back in Black · Billie Jean · Another One Bites the Dust | bumbo 1 e 3, caixa 2 e 4, chimbau em oitavas |

O áudio fica fora do repositório (`~/Music/EstudoMetro`), como manda a ADR-0013.

## O mapa de Hz, medido

Cada ataque foi medido pela **novidade por banda** — a subida de energia naquele
instante, normalizada pela média da própria banda — e classificado pela banda que
mais subiu. Medianas das nove faixas:

| instrumento | ≤300 Hz | 1,5–5 kHz | 6,5–13 kHz |
|---|---|---|---|
| **bumbo** | **53%** (25–68) | 1% | 0% |
| **corpo** (110–300) | 17% | 0% | 5% |
| **caixa** | 13% | **65%** (48–76) | 25% |
| **chimbau** | 13% | 25% | **60%** (47–75) |

A distribuição dos ataques por família de banda é estável e plausível nas nove:

```
musica                   baixo   medio    alto
another-bites-dust         51%     25%     24%
back-in-black              33%     35%     32%
billie-jean                39%     28%     33%
house-rising-sun           41%     27%     32%
manic-depression           33%     38%     30%
my-favorite-things         34%     21%     45%
nothing-else-matters       32%     26%     42%
since-been-loving-you      45%     26%     29%
someday-my-prince          57%     30%     13%
```

**Três conclusões que servem à partitura de bateria:**

1. **O bumbo é inequívoco:** 53% da energia do evento abaixo de 300 Hz, e só 1%
   em 1,5–5 kHz. Um limiar em 300 Hz acerta o bumbo sem confundir com nada.
2. **A caixa é inequívoca contra o bumbo:** 65% em 1,5–5 kHz contra 1% do bumbo.
3. **Caixa e chimbau NÃO se separam por banda.** A caixa tem 25% da energia na
   faixa do chimbau e o chimbau tem 25% na faixa da caixa — simétrico, medido nas
   nove. Os fios da caixa chegam a 13 kHz e o chimbau tem corpo em 1,5–5 kHz. **O
   que os separa não é a faixa, é o tempo:** a caixa ressoa 150–250 ms, o chimbau
   fechado ~50 ms. Próximo passo para a tablatura: o **decaimento**, não a banda.

## O que NÃO funcionou, e é importante

**O perfil de acento por instrumento sai chapado em mix denso.** Medido com o
motor (`compasso.bandas`) nas nove faixas: os valores por posição do compasso
ficam entre **0,5 e 1,5** em oito delas — indistinguíveis de ruído. Só
`Nothing Else Matters`, que é um mix esparso e guiado pela bateria, mostra
estrutura (**1,8–2,0**).

**Diagnóstico:** em um mix denso **toda banda sobe em toda batida** — há sempre
algo soando. E a banda de 30–110 Hz carrega o **baixo elétrico** junto com o
bumbo, então o bumbo não é isolável por energia de banda. No material sintético
(isolado, puro) o teste passa; no áudio real não.

**Efeito na decisão de metro:** o motor devolve `3` tanto para o grupo `3/4`
quanto para o `6/8`, com margem **0,00 a 0,04** em oito das nove — ou seja,
**abstenção disfarçada**. A confiança não sustenta a leitura, e a discriminação
`3/4` contra `6/8` **continua aberta**.

## Encaminhamento

- **Para a tablatura:** classificar cada ataque por **forma espectral** (a partilha
  entre bandas, como acima) e depois por **decaimento**, para separar caixa de
  chimbau. A partilha sozinha resolve bumbo e caixa; o decaimento resolve o par
  caixa/chimbau.
- **Para o metro:** o perfil por banda não serve em mix. As alternativas medidas
  como promissoras são (a) a **forma espectral por ataque** e (b) a **separação de
  fontes** antes da medição — nunca a energia de banda no mix.
- **Pendência de material:** validar o lado `3/4` contra um **3/4 com bateria de
  banda** (os hinos do gabarito são corais: a banda de 1,5–5 kHz é o coral).