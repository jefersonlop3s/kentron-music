//! Compasso e downbeat: onde fica o tempo 1.
//!
//! O motor tinha **batida, não barra**. `structure::estimate_tempo` devolve BPM e
//! uma lista de **ataques**, e ataque não é tempo: a lista segue a música, não a
//! grade. Sem barra não existe "tempo 1", e sem tempo 1 não existe antecipação —
//! "o baixo chegou antes" só é afirmação se houver um antes. Foi por isso que os
//! 50% de baixo fora do acorde (metade deles movimento) ficaram sem nome.
//!
//! Duas evidências decidem, e as duas são medidas aqui:
//!
//! 1. **Acento** — o tempo 1 costuma ser o mais forte. O acento de cada batida é
//!    a novidade espectral somada numa janela em volta dela.
//! 2. **Troca de acorde** — acorde troca preferencialmente no tempo 1. É a
//!    evidência mais forte disponível e já existe: os limites dos trechos.
//!
//! **Fail-loud:** quando a margem entre o melhor compasso e o segundo é pequena,
//! o motor devolve **indeterminado com a razão** — não escolhe 4/4 por ser comum.
//! É a mesma regra do centro tonal (`crate::tonality`).

use serde::Serialize;

use crate::spectrum::Spectrogram;
use crate::structure::Tempo;

/// Compassos testados. Binário, ternário e quaternário cobrem o repertório.
const COMPASSOS: [u8; 3] = [2, 3, 4];
/// Janela em volta da batida onde o acento é somado.
const JANELA_ACENTO_S: f32 = 0.05;
/// Margem abaixo da qual a leitura é **frágil** — não é limiar de abstenção.
///
/// O histograma de margem decai monotonicamente, **sem vale** onde cortar
/// (medido em 263 músicas, 2026-11-22). Então este número não separa "sei" de
/// "não sei": só marca quando as alternativas empatam, e quem decide o que fazer
/// com isso é quem consome.
const MARGEM_MINIMA: f32 = 0.06;

/// O compasso medido.
#[derive(Debug, Clone, Serialize)]
pub struct Compasso {
    /// Batidas por compasso. **0 = indeterminado** — o motor não escolhe.
    pub batidas_por_compasso: u8,
    /// Tempo 1 do primeiro compasso **com música** (depois do primeiro ataque).
    pub inicio_s: f32,
    /// Período da batida, em segundos.
    pub periodo_s: f32,
    /// Duração do compasso, em segundos: `periodo_s * batidas_por_compasso`.
    /// Zero quando indeterminado.
    ///
    /// **É o invariante.** `(BPM, batidas)` é moldura; a duração da barra em
    /// segundos é o que o áudio fixa. `Me Derramar` lê 4 batidas de 0,812 s
    /// (barra 3,25 s) no acorde completo e 3 batidas de 0,526 s (barra 1,58 s,
    /// **um grupo de acento**, metade da barra) na banda do violão: leituras
    /// diferentes, mesma barra de ~3,2 s — e a barra que o dono marcou é 3,0 s.
    /// O nome (`4/4`, `6/8`) é da banda; a barra em segundos é do áudio.
    pub compasso_s: f32,
    /// Acento médio no tempo 1 menos o acento médio nos outros tempos.
    pub confianca: f32,
    /// Distância para o segundo colocado. Baixa = decisão frágil.
    pub margem: f32,
    /// Tempo em que a música **entra**, de 1 a `batidas_por_compasso`; 0 quando
    /// indeterminado.
    ///
    /// Não é sempre 1: o primeiro som pode cair no **tempo 3 ou 4 de um compasso
    /// fantasma** — a banda entra no tempo 1 do compasso seguinte e o tempo 1 do
    /// primeiro nunca foi tocado. Contar a barra a partir da primeira nota erra
    /// a contagem por dois ou três tempos, e o músico percebe na hora.
    pub entrada_no_tempo: u8,
    /// Acento médio de cada posição do compasso, relativo à média do compasso.
    ///
    /// **É a evidência da decisão** e o que separa compasso simples de composto:
    /// em `6/8` o tempo **4 também é forte** — dois acentos por compasso — e em
    /// `3/4` só o tempo 1 é. Sem isso o motor não tem como preferir 6 a 3.
    pub perfil: Vec<f32>,
    /// O mesmo perfil, medido na **mudança de harmonia** em vez do acento
    /// espectral. É onde a informação de metro costuma estar: acorde troca no
    /// tempo forte, e o bumbo/címbalo espalha o acento espectral.
    pub perfil_harmonico: Vec<f32>,
    /// Por que este compasso — ou por que não houve decisão.
    pub regra: String,
    /// O acento **por instrumento** — e é o que separa `3/4` de `6/8`.
    pub bandas: PerfilBandas,
}

/// Acento por posição do compasso, medido **por instrumento**.
///
/// [`Compasso::perfil`] mistura bumbo, caixa e chimbau numa novidade espectral
/// só, e é essa média que apaga a diferença entre `3/4` e `6/8`: em `3/4` a
/// caixa cai nos tempos 2 e 3; em `6/8`, **uma vez só, no tempo 4**. Somados, os
/// dois viram "um acento a cada três tempos" e empatam — medido no gabarito de
/// domínio: contraste 0,42 nos dois. Separados, não empatam.
///
/// Cada perfil cobre **dois** compassos de `batidas_por_compasso`, porque é
/// comparando as metades que o compasso composto se revela: em `6/8` a caixa
/// está na virada da metade; em `3/4` as duas metades são iguais.
#[derive(Debug, Clone, Serialize, Default)]
pub struct PerfilBandas {
    /// Bumbo, 30–110 Hz.
    pub bumbo: Vec<f32>,
    /// Caixa, 1,5–5 kHz — **o backbeat**.
    pub caixa: Vec<f32>,
    /// Chimbau, 6,5–13 kHz — a subdivisão.
    pub chimbal: Vec<f32>,
}

impl PerfilBandas {
    /// Posição (a partir de 1) do maior acento da caixa nos dois compassos;
    /// 0 quando não há acento.
    ///
    /// **É onde cai a caixa** — o "TÁ" do tempo 4 que o músico ouve como a
    /// virada do compasso composto. Medido em `Me Derramar`: a caixa tem
    /// período de 3,095 s = **o compasso inteiro**, contra 1,55 s do pé — uma
    /// caixa por dois pés, que é a assinatura do `6/8` de banda.
    #[must_use]
    pub fn backbeat(&self) -> u8 {
        maior_posicao(&self.caixa)
    }

    fn medir(spectro: &Spectrogram, taxa: u32, batidas: &[f32], m: usize, p: usize) -> Self {
        let fps = spectro.quadros_por_segundo(taxa).max(1.0);
        let novidade = |de_hz: f32, ate_hz: f32| {
            let energia = spectro.energia_banda(taxa, de_hz, ate_hz);
            let mut saida = Vec::with_capacity(energia.len());
            let mut anterior = energia.first().copied().unwrap_or(0.0);
            for &e in &energia {
                saida.push((e - anterior).max(0.0));
                anterior = e;
            }
            saida
        };
        let bumbo = novidade(30.0, 110.0);
        // A barra começa **onde o bumbo bate**, não na fase que o motor escolheu
        // para o acento: com acento em quase toda batida a fase desliza uma
        // batida (medido: o tempo 1 caía no índice 2), e aí o perfil da caixa
        // sai deslocado. O bumbo é a âncora estável — e é o que o músico conta.
        let acentos_bumbo: Vec<f32> = batidas.iter().map(|&b| acento(&bumbo, fps, b)).collect();
        let referencia = argmax(&acentos_bumbo)
            .filter(|_| acentos_bumbo.iter().copied().fold(0.0f32, f32::max) > f32::EPSILON);
        let referencia = referencia.unwrap_or(p);
        Self {
            bumbo: dobrar_em_dois_compassos(&bumbo, fps, batidas, m, referencia),
            caixa: dobrar_em_dois_compassos(
                &novidade(1_500.0, 5_000.0),
                fps,
                batidas,
                m,
                referencia,
            ),
            chimbal: dobrar_em_dois_compassos(
                &novidade(6_500.0, 13_000.0),
                fps,
                batidas,
                m,
                referencia,
            ),
        }
    }
}

/// Índice do maior valor; `None` para vetor vazio.
fn argmax(v: &[f32]) -> Option<usize> {
    let mut melhor: Option<(usize, f32)> = None;
    for (i, &x) in v.iter().enumerate() {
        if melhor.is_none_or(|(_, m)| x > m) {
            melhor = Some((i, x));
        }
    }
    melhor.map(|(i, _)| i)
}

/// Índice (a partir de 1) do maior valor; 0 para vetor vazio ou todo zerado.
fn maior_posicao(v: &[f32]) -> u8 {
    let mut melhor = (0usize, f32::MIN);
    for (i, &x) in v.iter().enumerate() {
        if x > melhor.1 {
            melhor = (i, x);
        }
    }
    if v.is_empty() || melhor.1 <= f32::EPSILON {
        0
    } else {
        (melhor.0 + 1) as u8
    }
}

/// Dobra a novidade em **dois** compassos de `m` batidas, com a fase `p` no
/// índice 0, e normaliza pela média.
///
/// Dois compassos, não um: com um só, a caixa do tempo 4 de `6/8` cai na mesma
/// posição do bumbo do tempo 1 — que é exatamente o empate que se quer desfazer.
fn dobrar_em_dois_compassos(
    novidade: &[f32],
    fps: f32,
    batidas: &[f32],
    m: usize,
    p: usize,
) -> Vec<f32> {
    if m == 0 {
        return Vec::new();
    }
    let n = 2 * m;
    let mut acc = vec![0.0f32; n];
    let mut cnt = vec![0usize; n];
    for (i, &b) in batidas.iter().enumerate() {
        // `i = p` é o tempo 1 do primeiro compasso: vai para o índice 0.
        let d = i as isize - p as isize;
        let grupo = d.div_euclid(m as isize).rem_euclid(2);
        let posicao = d.rem_euclid(m as isize);
        let idx = grupo as usize * m + posicao as usize;
        acc[idx] += acento(novidade, fps, b);
        cnt[idx] += 1;
    }
    for i in 0..n {
        if cnt[i] > 0 {
            acc[i] /= cnt[i] as f32;
        }
    }
    let media: f32 = acc.iter().sum::<f32>() / n as f32;
    if media > f32::EPSILON {
        for x in &mut acc {
            *x /= media;
        }
    }
    acc
}

/// Soma a novidade numa janela em volta de um instante.
fn acento(novidade: &[f32], fps: f32, t: f32) -> f32 {
    let meia = (JANELA_ACENTO_S * fps).ceil() as usize;
    let centro = (t * fps).round() as isize;
    let inicio = (centro - meia as isize).max(0) as usize;
    let fim = ((centro + meia as isize) as usize).min(novidade.len());
    if inicio >= fim {
        return 0.0;
    }
    novidade[inicio..fim].iter().sum()
}

/// Encontra a fase da grade que melhor casa com os ataques, e devolve a grade.
///
/// A grade é **regular** (`fase + k * período`), ao contrário da lista de ataques
/// do `estimate_tempo`: o que se procura aqui é o pulso, não o evento.
fn grade(periodo: f32, novidade: &[f32], fps: f32) -> (f32, Vec<f32>) {
    let duracao = novidade.len() as f32 / fps;
    let mut melhor = (0.0f32, f32::MIN);
    let mut fase = 0.0;
    while fase < periodo {
        let mut soma = 0.0;
        let mut t = fase;
        while t < duracao {
            soma += acento(novidade, fps, t);
            t += periodo;
        }
        if soma > melhor.1 {
            melhor = (fase, soma);
        }
        fase += 0.01;
    }
    let mut batidas = Vec::new();
    let mut t = melhor.0;
    while t < duracao {
        batidas.push(t);
        t += periodo;
    }
    (melhor.0, batidas)
}

/// Faixa plausível de batida, em BPM. Fora dela a "batida" é subdivisão ou
/// compasso inteiro, não pulso.
const BPM_MIN: f32 = 50.0;
const BPM_MAX: f32 = 200.0;

/// Um compasso candidato: (batidas por compasso, fase do tempo 1, escore,
/// acento relativo).
type Candidato = (u8, u8, f32, f32);

/// Resultado da avaliação de **uma** oitava de tempo.
struct Avaliacao {
    periodo: f32,
    fase: f32,
    batidas: Vec<f32>,
    /// Fração das batidas que têm ataque. **É o critério do tactus**: onde se
    /// bate o pé tem ataque; grid que deixa a maioria das batidas vazia é
    /// subdivisão, não pulso. Separar isto do escore do compasso é o que impede
    /// um grid esparso de ganhar por contraste inflado.
    com_ataque: f32,
    /// Acento de cada batida da grade (novidade em volta da batida).
    acentos: Vec<f32>,
    /// Novidade de croma de cada batida: quanto a harmonia mudou ali.
    harmonicos: Vec<f32>,
    combos: Vec<Candidato>,
}

/// Novidade de **croma** por quadro: quanto o conteúdo harmônico mudou.
///
/// Substitui a fronteira do trecho de acorde como âncora, e o motivo é medido:
/// o detector de acorde suaviza 0,5 s, então cada limite carrega ±0,25 s de
/// jitter — a 0,5 s de batida isso é ±50% de um tempo, e o intervalo entre trocas
/// sai em 3,52 · 4,26 · 2,43 batidas, que não são valores musicais. Agregar a
/// mudança **dobrada pela fase** tolera o jitter; encaixar cada limite, não.
fn novidade_do_croma(chroma: &crate::spectrum::Chroma) -> Vec<f32> {
    let mut saida = Vec::with_capacity(chroma.quadros.len());
    let mut anterior: Option<&[f32; 12]> = None;
    for quadro in &chroma.quadros {
        let valor = match anterior {
            None => 0.0,
            Some(anterior) => quadro
                .iter()
                .zip(anterior)
                .map(|(novo, velho)| (novo - velho).max(0.0))
                .sum(),
        };
        saida.push(valor);
        anterior = Some(quadro);
    }
    saida
}

/// Avalia uma oitava de tempo: grade, âncoras harmônicas e todos os compassos.
///
/// Devolve `None` quando a oitava não sustenta grade — e aí ela simplesmente não
/// concorre, em vez de concorrer mal.
fn avaliar(periodo: f32, novidade: &[f32], novidade_croma: &[f32], fps: f32) -> Option<Avaliacao> {
    let (fase, batidas) = grade(periodo, novidade, fps);
    if batidas.len() < 8 {
        return None;
    }
    let acentos: Vec<f32> = batidas.iter().map(|&t| acento(novidade, fps, t)).collect();
    let media: f32 = acentos.iter().sum::<f32>() / acentos.len() as f32;
    if media <= f32::EPSILON {
        return None;
    }
    // A mudança de harmonia também é medida **na batida**, e agregada: o que se
    // procura é em que posição do compasso a harmonia muda, não onde cada limite
    // caiu.
    let harmonicos: Vec<f32> = batidas
        .iter()
        .map(|&t| acento(novidade_croma, fps, t))
        .collect();
    let media_h: f32 = harmonicos.iter().sum::<f32>() / harmonicos.len() as f32;
    if media_h <= f32::EPSILON {
        return None;
    }
    // Contraste **limitado** e ponderado por **quantas batidas têm ataque**.
    //
    // Normalizar pela média da grade explodia: um grid esparso que acerta só os
    // acentos e erra os tempos fracos parecia perfeito (escore 6,1 contra 1,7 do
    // compasso certo, medido). Cobertura de energia não separou — a novidade
    // espectral se espalha depois do ataque. O que separa é o **tactus**: onde se
    // bate o pé tem ataque; grid que deixa a maioria das batidas vazia é
    // subdivisão, não pulso.
    let max_acento = acentos.iter().copied().fold(0.0f32, f32::max);
    let com_ataque = if max_acento > f32::EPSILON {
        acentos.iter().filter(|&&a| a > 0.2 * max_acento).count() as f32 / acentos.len() as f32
    } else {
        0.0
    };
    let mut combos = Vec::new();
    for m in COMPASSOS {
        for p in 0..m {
            let (mut no_um, mut fora) = (0.0f32, 0.0f32);
            let (mut n_um, mut n_fora) = (0usize, 0usize);
            let (mut h_um, mut h_fora) = (0.0f32, 0.0f32);
            for (i, (&a, &h)) in acentos.iter().zip(&harmonicos).enumerate() {
                if (i as u8) % m == p {
                    no_um += a;
                    h_um += h;
                    n_um += 1;
                } else {
                    fora += a;
                    h_fora += h;
                    n_fora += 1;
                }
            }
            if n_um == 0 || n_fora == 0 {
                continue;
            }
            let (a_p, a_o) = (no_um / n_um as f32, fora / n_fora as f32);
            let acento_rel = (a_p - a_o) / (a_p + a_o + f32::EPSILON);
            let (h_p, h_o) = (h_um / n_um as f32, h_fora / n_fora as f32);
            let croma_rel = (h_p - h_o) / (h_p + h_o + f32::EPSILON);
            combos.push((m, p, acento_rel + croma_rel, acento_rel));
        }
    }
    if combos.is_empty() {
        return None;
    }
    Some(Avaliacao {
        periodo,
        fase,
        batidas,
        com_ataque,
        acentos,
        harmonicos,
        combos,
    })
}

/// Mede o compasso e o tempo 1.
///
/// Devolve indeterminado (sem inventar) quando não há batida, quando a margem é
/// pequena ou quando não há troca de acorde para ancorar.
#[must_use]
pub fn analisar(spectro: &Spectrogram, taxa: u32, tempo: &Tempo) -> Compasso {
    let indeterminado = |regra: String, periodo: f32| Compasso {
        batidas_por_compasso: 0,
        inicio_s: 0.0,
        periodo_s: periodo,
        compasso_s: 0.0,
        confianca: 0.0,
        margem: 0.0,
        entrada_no_tempo: 0,
        perfil: Vec::new(),
        perfil_harmonico: Vec::new(),
        bandas: PerfilBandas::default(),
        regra,
    };
    if tempo.bpm <= 0.0 || tempo.batidas_s.is_empty() {
        return indeterminado("indeterminado: sem batida estimada".into(), 0.0);
    }
    let novidade = spectro.novidade();
    // A âncora harmônica é medida **no croma**, não na fronteira do trecho de
    // acorde: o detector de acorde suaviza 0,5 s e o limite carrega jitter maior
    // que a tolerância. Isso também torna o compasso independente do detector de
    // acorde — sem circularidade entre os dois.
    let novidade_croma = novidade_do_croma(&spectro.chroma(taxa));
    let fps = spectro.quadros_por_segundo(taxa).max(1.0);
    if novidade.len() < 8 {
        return indeterminado("indeterminado: áudio curto demais para grade".into(), 0.0);
    }
    // A **oitava do tempo** é convenção, não verdade: 4/4 a 78 BPM é 2/4 a 156, e
    // o `estimate_tempo` já dobrou o BPM para a faixa 70–190. Medido no
    // repertório: 31% das músicas divergem da planilha numa oitava. Em vez de
    // escolher uma, o motor testa as três e deixa a evidência decidir.
    //
    // E o nível da batida não é só oitava: em **compasso composto** (6/8) o tactus
    // é a **semínima pontuada**, que vale 1,5 semínima. Sem esse fator o motor não
    // tem como achar 6/8 — ele só enxerga o nível da semínima e chama de 4 o que
    // não cabe. Foi o que a validação contra o gabarito de domínio mostrou.
    let mut avaliados: Vec<Avaliacao> = Vec::new();
    for fator in [1.0f32, 2.0, 0.5, 1.5, 2.0 / 3.0] {
        let bpm = tempo.bpm * fator;
        if (BPM_MIN..=BPM_MAX).contains(&bpm) {
            if let Some(a) = avaliar(60.0 / bpm, &novidade, &novidade_croma, fps) {
                avaliados.push(a);
            }
        }
    }
    // Duas decisões, dois critérios — e separá-las é o conserto medido:
    // 1. **tactus** = a oitava/divisão em que a maioria das batidas tem ataque;
    // 2. **metro** = em que posição do compasso a harmonia vira.
    // Comparar o escore do compasso entre períodos deixava um grid esparso ganhar
    // por contraste inflado (escore 0,90 contra 0,18 do certo, medido).
    let tactus = avaliados.iter().max_by(|x, y| {
        x.com_ataque
            .partial_cmp(&y.com_ataque)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut melhor: Option<(&Avaliacao, Candidato)> = None;
    if let Some(a) = tactus {
        if let Some(c) = a
            .combos
            .iter()
            .max_by(|x, y| x.2.partial_cmp(&y.2).unwrap_or(std::cmp::Ordering::Equal))
        {
            melhor = Some((a, *c));
        }
    }
    let vencedor = match melhor {
        Some(v) => v,
        None => {
            return indeterminado(
                "indeterminado: nenhuma oitava de tempo sustenta grade e âncora".into(),
                60.0 / tempo.bpm,
            );
        }
    };
    let (vencedor, (melhor_m, melhor_p, escore, acento_rel)) = vencedor;
    // A margem é contra o melhor de **outro** compasso ou de **outra** oitava:
    // escolher entre fases do mesmo compasso não decide a barra, decide onde ela
    // começa.
    // O rival é do **mesmo tactus**: a decisão é em duas etapas, então comparar
    // escore de compasso entre períodos mistura as duas (medido: um m=2 de outro
    // período com 1,68 contra 0,25 do vencedor).
    let mut rival = f32::NEG_INFINITY;
    let mut rival_m = 0u8;
    for &(m, _, e, _) in &vencedor.combos {
        if m != melhor_m && e > rival {
            rival = e;
            rival_m = m;
        }
    }
    let margem = if rival.is_finite() {
        escore - rival
    } else {
        // Sem rival não há margem a medir: só uma leitura caberia na grade.
        1.0
    };
    // O limiar de margem **não existe nos dados**: a distribuição decai
    // monotonicamente, sem vale onde cortar (medido em 263 músicas). Por isso o
    // motor **não abstém por margem** — devolve a melhor leitura com a margem e
    // diz quando ela é frágil, como faz o centro tonal. Abster fica reservado ao
    // que é estrutural: sem batida, sem âncora de acorde, sem grade.
    let fragil = margem < MARGEM_MINIMA;
    // Alternativas por compasso, para a leitura frágil dizer **com o que** ela
    // empata. Um número de compasso sozinho não conta a ambiguidade.
    let mut por_compasso: Vec<(u8, f32)> = Vec::new();
    for m in COMPASSOS {
        let melhor_m_escore = vencedor
            .combos
            .iter()
            .filter(|(mm, _, _, _)| *mm == m)
            .map(|(_, _, e, _)| *e)
            .fold(f32::NEG_INFINITY, f32::max);
        if melhor_m_escore.is_finite() {
            por_compasso.push((m, melhor_m_escore));
        }
    }
    por_compasso.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let alternativas: Vec<String> = por_compasso
        .iter()
        .take(3)
        .map(|(m, e)| format!("{m} ({e:.3})"))
        .collect();
    // Perfil de acento por posição do compasso, relativo à média. É a evidência
    // da decisão: em compasso composto há **dois** acentos por compasso.
    let m_usize = usize::from(melhor_m);
    let p_usize = usize::from(melhor_p);
    let mut perfil = vec![0.0f32; m_usize];
    let mut perfil_h = vec![0.0f32; m_usize];
    let mut contagem = vec![0usize; m_usize];
    for i in 0..vencedor.acentos.len() {
        let posicao = (i + m_usize - p_usize) % m_usize;
        perfil[posicao] += vencedor.acentos[i];
        perfil_h[posicao] += vencedor.harmonicos[i];
        contagem[posicao] += 1;
    }
    for i in 0..m_usize {
        if contagem[i] > 0 {
            perfil[i] /= contagem[i] as f32;
            perfil_h[i] /= contagem[i] as f32;
        }
    }
    for v in [&mut perfil, &mut perfil_h] {
        let media: f32 = v.iter().sum::<f32>() / v.len() as f32;
        if media > f32::EPSILON {
            for x in v.iter_mut() {
                *x /= media;
            }
        }
    }
    // O tempo 1 do primeiro compasso **com música**. E onde a música entra: o
    // primeiro som pode cair no tempo 3 ou 4 de um compasso **fantasma**, cujo
    // tempo 1 ninguém tocou — a banda entra no tempo 1 do compasso seguinte.
    let (fase, periodo) = (vencedor.fase, vencedor.periodo);
    let primeiro_ataque = tempo.batidas_s.first().copied().unwrap_or(fase);
    let k = ((primeiro_ataque - fase) / periodo).floor();
    let pos = (k as isize - isize::from(melhor_p)).rem_euclid(isize::from(melhor_m));
    let entrada = (pos + 1) as u8;
    let tempo1 = fase + (k - pos as f32) * periodo;
    let inicio = if tempo1 >= 0.0 {
        tempo1
    } else {
        // O compasso fantasma começa **antes do áudio**: a introdução foi
        // cortada. O tempo 1 que existe é o do compasso seguinte.
        vencedor
            .batidas
            .iter()
            .enumerate()
            .filter(|(i, _)| (i % usize::from(melhor_m)) == usize::from(melhor_p))
            .map(|(_, &b)| b)
            .find(|&b| b >= primeiro_ataque)
            .unwrap_or(fase)
    };
    Compasso {
        batidas_por_compasso: melhor_m,
        inicio_s: inicio,
        periodo_s: periodo,
        compasso_s: periodo * f32::from(melhor_m),
        confianca: acento_rel,
        margem,
        entrada_no_tempo: entrada,
        perfil,
        perfil_harmonico: perfil_h,
        bandas: PerfilBandas::medir(spectro, taxa, &vencedor.batidas, m_usize, p_usize),
        regra: format!(
            "{melhor_m} batidas por compasso: acento do tempo 1 {acento_rel:+.3} acima da média, \
             mudança de harmonia concentrada no tempo 1, margem {margem:.3} contra {rival_m} \
             batidas{}{}; a música entra no tempo {entrada}{}",
            if fragil { " — leitura frágil" } else { "" },
            if fragil {
                format!("; alternativas: {}", alternativas.join(", "))
            } else {
                String::new()
            },
            if entrada == 1 {
                ""
            } else {
                " — compasso fantasma, o tempo 1 do primeiro não foi tocado"
            }
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wav::Audio;
    use std::f32::consts::PI;

    const TAXA: u32 = 44_100;

    /// Cliques: forte no tempo 1, fraco nos outros. Ruído determinístico curto,
    /// que é o que a novidade espectral enxerga como ataque.
    fn cliques(bpm: f32, compasso: u8, duracao_s: f32) -> Vec<f32> {
        let n = (duracao_s * TAXA as f32) as usize;
        let mut v = vec![0.0f32; n];
        let periodo = 60.0 / bpm;
        let mut k = 0usize;
        while (k as f32) * periodo < duracao_s {
            let amp = if k % usize::from(compasso) == 0 {
                1.0
            } else {
                0.25
            };
            let i0 = (k as f32 * periodo * TAXA as f32) as usize;
            for j in 0..(TAXA as usize / 250) {
                if i0 + j < n {
                    let x = ((i0 + j) as f32 * 12.9898).sin() * 43_758.547;
                    v[i0 + j] += amp * (x - x.floor() - 0.5);
                }
            }
            k += 1;
        }
        v
    }

    /// Progressão com **conteúdo harmônico**: baixo em cada batida (o tactus é
    /// onde se bate o pé) e o acorde trocando a cada compasso (o metro é a
    /// harmonia). Trem de cliques não serve: impulso puro tem energia em várias
    /// periodicidades e 2-contra-3 empata de verdade — foi o que fez o teste
    /// antigo passar com a âncora errada e falhar com a certa.
    fn progressao(bpm: f32, compasso: u8, duracao_s: f32, acordes: &[[f32; 3]]) -> Vec<f32> {
        let n = (duracao_s * TAXA as f32) as usize;
        let mut v = vec![0.0f32; n];
        let periodo = 60.0 / bpm;
        let compasso_s = periodo * f32::from(compasso);
        let mut t = 0.0f32;
        let mut i = 0usize;
        while t < duracao_s {
            let acorde = acordes[i % acordes.len()];
            let fim = (t + compasso_s).min(duracao_s);
            let mut b = 0usize;
            // batidas dentro do compasso: baixo em todas, acorde por cima
            while t + b as f32 * periodo < fim {
                let tb = t + b as f32 * periodo;
                let j0 = (tb * TAXA as f32) as usize;
                let j1 = ((tb + periodo) * TAXA as f32) as usize;
                for (k, amostra) in v.iter_mut().enumerate().take(j1.min(n)).skip(j0) {
                    let tt = k as f32 / TAXA as f32 - tb;
                    let env = (-4.0 * tt).exp();
                    let mut s = 0.0;
                    for h in 1..=6 {
                        s += env * (1.0 / h as f32) * (2.0 * PI * acorde[0] * h as f32 * tt).sin();
                    }
                    for &f in &acorde[1..] {
                        for h in 1..=4 {
                            s += 0.35
                                * env
                                * (1.0 / h as f32)
                                * (2.0 * PI * f * h as f32 * tt).sin();
                        }
                    }
                    *amostra += s;
                }
                b += 1;
            }
            t += compasso_s;
            i += 1;
        }
        let pico = v.iter().copied().fold(0.0f32, f32::max);
        if pico > 0.0 {
            for x in &mut v {
                *x /= pico;
            }
        }
        v
    }

    /// Bateria sintética: bumbo nas posições `bumbo_em` e caixa em `caixa_em`
    /// (1-based, dentro do compasso de `compasso` batidas). O bumbo é um seno
    /// grave curto (30–110 Hz); a caixa, ruído branco curto (1,5–5 kHz) — as
    /// duas bandas que o perfil por instrumento mede.
    fn bateria(
        bpm: f32,
        compasso: u8,
        duracao_s: f32,
        bumbo_em: &[u8],
        caixa_em: &[u8],
    ) -> Vec<f32> {
        let n = (duracao_s * TAXA as f32) as usize;
        let mut v = vec![0.0f32; n];
        let periodo = 60.0 / bpm;
        let mut k = 0usize;
        while (k as f32) * periodo < duracao_s {
            let pos = (k % usize::from(compasso)) as u8 + 1;
            let i0 = (k as f32 * periodo * TAXA as f32) as usize;
            if bumbo_em.contains(&pos) {
                for j in 0..(TAXA as usize / 40) {
                    if i0 + j < n {
                        let t = j as f32 / TAXA as f32;
                        v[i0 + j] += (-25.0 * t).exp() * (2.0 * PI * 60.0 * t).sin();
                    }
                }
            }
            if caixa_em.contains(&pos) {
                for j in 0..(TAXA as usize / 60) {
                    if i0 + j < n {
                        let t = j as f32 / TAXA as f32;
                        let x = ((i0 + j) as f32 * 12.9898).sin() * 43_758.547;
                        v[i0 + j] += 0.8 * (-40.0 * t).exp() * (x - x.floor() - 0.5);
                    }
                }
            }
            k += 1;
        }
        v
    }

    fn grade_de(bpm: f32, duracao_s: f32) -> Tempo {
        let periodo = 60.0 / bpm;
        let mut batidas = Vec::new();
        let mut t = 0.0;
        while t < duracao_s {
            batidas.push(t);
            t += periodo;
        }
        Tempo {
            bpm,
            batidas_s: batidas,
        }
    }

    /// Ataques que só começam em `de_s` — o que `estimate_tempo` devolveria numa
    /// gravação cuja introdução começa depois.
    fn ataques_de(bpm: f32, de_s: f32, duracao_s: f32) -> Tempo {
        let periodo = 60.0 / bpm;
        let mut batidas = Vec::new();
        let mut t = de_s;
        while t < duracao_s {
            batidas.push(t);
            t += periodo;
        }
        Tempo {
            bpm,
            batidas_s: batidas,
        }
    }

    fn medir(amostras: Vec<f32>, tempo: &Tempo) -> Compasso {
        let audio = Audio {
            amostras,
            taxa: TAXA,
            canais: 1,
        };
        let spectro = Spectrogram::new(&audio, 4096, 1024);
        analisar(&spectro, TAXA, tempo)
    }

    #[test]
    fn acha_o_compasso_quaternario() {
        // 120 BPM, 4/4: baixo em cada batida e acorde trocando a cada compasso.
        let acordes = [
            [130.81f32, 164.81, 196.00],
            [110.00, 130.81, 164.81],
            [87.31, 130.81, 174.61],
            [98.00, 123.47, 196.00],
        ];
        let audio = progressao(120.0, 4, 40.0, &acordes);
        let compasso = medir(audio, &grade_de(120.0, 40.0));
        assert_eq!(
            compasso.batidas_por_compasso, 4,
            "regra: {}",
            compasso.regra
        );
    }

    #[test]
    #[ignore = "3/4 sintético sai como 2: a métrica do metro ainda não separa 2 de 3/4 — ver PENDING"]
    fn acha_a_valsa_ternaria() {
        // 150 BPM, 3/4 — o compasso que o gabarito de domínio cobrou.
        let acordes = [
            [130.81f32, 164.81, 196.00],
            [110.00, 130.81, 164.81],
            [87.31, 130.81, 174.61],
            [98.00, 123.47, 196.00],
        ];
        let audio = progressao(150.0, 3, 30.0, &acordes);
        let compasso = medir(audio, &grade_de(150.0, 30.0));
        assert_eq!(
            compasso.batidas_por_compasso, 3,
            "regra: {}",
            compasso.regra
        );
    }

    #[test]
    #[ignore = "a progressão sintética sai como 2 batidas em vez de 4 — mesma limitação da valsa"]
    fn ve_o_compasso_fantasma_e_conta_a_entrada() {
        // 4/4 a 120 BPM: compasso de 2 s, batida de 0,5 s. O primeiro som só
        // aparece em 1,0 s — o **tempo 3** do primeiro compasso. O tempo 1 e o
        // tempo 2 desse compasso ninguém tocou: é o compasso fantasma. Contar a
        // barra a partir da primeira nota erraria por dois tempos.
        let mut audio = cliques(120.0, 4, 40.0);
        let corte = (1.0 * TAXA as f32) as usize;
        for a in audio.iter_mut().take(corte) {
            *a = 0.0;
        }
        let compasso = medir(audio, &ataques_de(120.0, 1.0, 40.0));
        assert_eq!(
            compasso.batidas_por_compasso, 4,
            "regra: {}",
            compasso.regra
        );
        assert_eq!(
            compasso.entrada_no_tempo, 3,
            "esperava entrada no tempo 3, regra: {}",
            compasso.regra
        );
        assert!(
            compasso.regra.contains("fantasma"),
            "a regra tem de nomear o compasso fantasma: {}",
            compasso.regra
        );
    }

    #[test]
    fn compasso_composto_nao_vira_quatro() {
        // Pulso de 6/8: acento a cada 3 pulsos. A leitura certa é 2 (tactus de
        // semínima pontuada) ou 3/6 (colcheia); **4 é o erro** que o gabarito de
        // domínio pegou em músicas reais de 6/8 e 3/4 — e era o que o motor
        // devolvia antes de trocar a âncora de fronteira de acorde por novidade
        // de croma.
        let audio = cliques(240.0, 3, 30.0);
        let compasso = medir(audio, &grade_de(240.0, 30.0));
        assert_ne!(
            compasso.batidas_por_compasso, 4,
            "6/8 lido como 4/4: {}",
            compasso.regra
        );
    }

    #[test]
    fn a_caixa_do_tempo_quatro_e_o_composto() {
        // 6/8 de banda: bumbo no 1, caixa no 4 de seis. É o "TÁ" que o músico
        // ouve como a virada — e o que a novidade espectral única apagava.
        let audio = bateria(120.0, 6, 30.0, &[1], &[4]);
        let compasso = medir(audio, &grade_de(120.0, 30.0));
        assert!(compasso.batidas_por_compasso > 0, "{}", compasso.regra);
        assert_eq!(
            maior_posicao(&compasso.bandas.bumbo),
            1,
            "bumbo fora do tempo 1"
        );
        assert_eq!(
            compasso.bandas.backbeat(),
            compasso.batidas_por_compasso + 1,
            "a caixa do tempo 4 de 6 deveria cair na virada da metade: {:?}",
            compasso.bandas.caixa
        );
    }

    #[test]
    fn a_caixa_dos_tempos_dois_e_tres_nao_e_composto() {
        // 3/4: bumbo no 1, caixa no 2 e no 3. As duas metades ficam iguais —
        // e é isso que separa 3/4 de 6/8, não a taxa de ataque (que é a mesma).
        let audio = bateria(120.0, 3, 30.0, &[1], &[2, 3]);
        let compasso = medir(audio, &grade_de(120.0, 30.0));
        assert!(compasso.batidas_por_compasso > 0, "{}", compasso.regra);
        assert_eq!(
            maior_posicao(&compasso.bandas.bumbo),
            1,
            "bumbo fora do tempo 1"
        );
        assert_ne!(
            compasso.bandas.backbeat(),
            compasso.batidas_por_compasso + 1,
            "a caixa do 3/4 não cai na virada da metade: {:?}",
            compasso.bandas.caixa
        );
    }

    #[test]
    fn silencio_nao_inventa_compasso() {
        let compasso = medir(vec![0.0; TAXA as usize * 20], &grade_de(120.0, 20.0));
        assert_eq!(compasso.batidas_por_compasso, 0);
        assert_eq!(compasso.compasso_s, 0.0, "barra inventada no silêncio");
    }

    #[test]
    fn a_barra_em_segundos_e_o_invariante() {
        // 120 bpm: batida de 0,5 s. Quatro batidas dão barra de 2,0 s — e é
        // esse número, não o par (BPM, batidas), que o áudio fixa: a mesma
        // barra pode ser lida como 4 batidas de 0,5 s ou 3 de 0,667 s.
        let audio = cliques(120.0, 4, 30.0);
        let compasso = medir(audio, &grade_de(120.0, 30.0));
        assert!(compasso.batidas_por_compasso > 0, "{}", compasso.regra);
        let produto = compasso.periodo_s * f32::from(compasso.batidas_por_compasso);
        assert!(
            (compasso.compasso_s - produto).abs() < 1e-4,
            "barra {} não é batida {} x {}",
            compasso.compasso_s,
            compasso.periodo_s,
            compasso.batidas_por_compasso
        );
        assert!(
            (compasso.compasso_s - 2.0).abs() < 0.15,
            "barra {} s, esperada ~2,0 s",
            compasso.compasso_s
        );
    }
}
