//! Letra: fonte com procedência, corpus de domínio público e alinhamento ao áudio.
//!
//! **Não existe fonte oficial de letra gratuita** (ADR-0012): provedor licenciado
//! é pago e proíbe armazenar o texto. O que entra aqui é o que é barato e
//! legítimo — texto **do próprio usuário** e **obra em domínio público** (corpus
//! embutido, só obra pré-1900, com procedência linha a linha). Transcrição
//! automática (ASR) está declarada e **não implementada**: exige peso no
//! `models/MANIFEST.toml`.
//!
//! O alinhamento é o "cruzar com o que o áudio apresenta": frases vocais são
//! achadas por energia na banda da voz, as linhas são distribuídas por elas pela
//! ordem e pelo tamanho, e cada linha volta com **confiança** — alinhamento por
//! frase é aproximado, não é fonema a fonema.

use serde::Serialize;

use crate::report::Report;
use crate::spectrum::Spectrogram;

/// Corpus embutido (ADR-0012): obra pré-1900 com procedência declarada no
/// próprio arquivo. Nada entra sem autor, ano e base.
const CORPUS_DOMINIO_PUBLICO: &str = include_str!("../assets/hinos_dominio_publico.tsv");

/// De onde veio a letra. Nunca é opcional: fonte sem procedência não entra.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Origem {
    /// Texto fornecido pelo próprio usuário (biblioteca, transcrição dele).
    DoUsuario,
    /// Corpus embutido de domínio público.
    DominioPublico {
        titulo: String,
        autor: String,
        ano: u16,
    },
    /// Transcrição automática do áudio — **não implementada** (ver ADR-0012).
    Asr,
}

impl Origem {
    /// Descrição legível, para o relatório e para a cifra.
    #[must_use]
    pub fn descricao(&self) -> String {
        match self {
            Self::DoUsuario => "fornecida pelo usuário".to_owned(),
            Self::DominioPublico { titulo, autor, ano } => {
                format!("domínio público — {titulo}, {autor} ({ano})")
            }
            Self::Asr => "transcrição automática (não implementada)".to_owned(),
        }
    }
}

/// Letra com procedência declarada.
#[derive(Debug, Clone, Serialize)]
pub struct Letra {
    pub origem: Origem,
    /// Linhas de letra, na ordem. Linha vazia não entra.
    pub linhas: Vec<String>,
}

/// Lê letra de texto livre ou de ChordPro.
///
/// Em ChordPro, o que está entre colchetes é acorde: sai. Metadados (`{...}`),
/// comentários (`#`) e linhas vazias também saem — sobra o texto cantado.
#[must_use]
pub fn ler_do_usuario(texto: &str) -> Letra {
    let mut linhas = Vec::new();
    for linha in texto.lines() {
        let limpa = linha.trim();
        if limpa.is_empty() || limpa.starts_with('#') {
            continue;
        }
        if limpa.starts_with('{') && limpa.ends_with('}') {
            continue;
        }
        // Se for só um bloco entre colchetes (nome de seção), não é letra.
        let sem_acordes = sem_colchetes(limpa);
        if sem_acordes.trim().is_empty() {
            continue;
        }
        linhas.push(sem_acordes.trim_end().to_owned());
    }
    Letra {
        origem: Origem::DoUsuario,
        linhas,
    }
}

/// Remove trechos entre colchetes (acordes e nomes de seção).
fn sem_colchetes(linha: &str) -> String {
    let mut saida = String::with_capacity(linha.len());
    let mut dentro = false;
    for caractere in linha.chars() {
        match caractere {
            '[' => dentro = true,
            ']' => dentro = false,
            _ if !dentro => saida.push(caractere),
            _ => {}
        }
    }
    saida
}

/// Busca no corpus de domínio público embutido, pelo título.
///
/// Comparação sem diferenciar maiúsculas nem acentos de caixa; aceita prefixo
/// (`{ titulo = "amazing" }`).
#[must_use]
pub fn do_corpus(titulo: &str) -> Option<Letra> {
    let alvo = titulo.trim().to_lowercase();
    for linha in CORPUS_DOMINIO_PUBLICO.lines() {
        let linha = linha.trim();
        if linha.is_empty() || linha.starts_with('#') {
            continue;
        }
        let mut campos = linha.split('\t');
        let (Some(titulo_dp), Some(autor), Some(ano), Some(verso)) =
            (campos.next(), campos.next(), campos.next(), campos.next())
        else {
            continue;
        };
        if titulo_dp.to_lowercase().starts_with(&alvo) {
            return Some(Letra {
                origem: Origem::DominioPublico {
                    titulo: titulo_dp.to_owned(),
                    autor: autor.to_owned(),
                    ano: ano.parse().unwrap_or(0),
                },
                linhas: verso
                    .split(" | ")
                    .map(|l| l.trim().to_owned())
                    .filter(|l| !l.is_empty())
                    .collect(),
            });
        }
    }
    None
}

/// Títulos disponíveis no corpus embutido: `(título, autor, ano)`.
#[must_use]
pub fn titulos_do_corpus() -> Vec<(String, String, u16)> {
    CORPUS_DOMINIO_PUBLICO
        .lines()
        .filter(|l| {
            let l = l.trim();
            !l.is_empty() && !l.starts_with('#')
        })
        .filter_map(|linha| {
            let mut campos = linha.split('\t');
            Some((
                campos.next()?.to_owned(),
                campos.next()?.to_owned(),
                campos.next()?.parse().ok()?,
            ))
        })
        .collect()
}

/// Vogal — inclusive `y`, que em inglês faz núcleo de sílaba ("every", "my").
fn eh_vogal(caractere: char) -> bool {
    matches!(
        caractere.to_lowercase().next(),
        Some(
            'a' | 'e'
                | 'i'
                | 'o'
                | 'u'
                | 'y'
                | 'á'
                | 'é'
                | 'í'
                | 'ó'
                | 'ú'
                | 'â'
                | 'ê'
                | 'ô'
                | 'ã'
                | 'õ'
                | 'à'
                | 'ü'
        )
    )
}

/// Início de cada sílaba, em **índices de caractere**.
///
/// Heurística barata e declarada: núcleo é grupo de vogais, e o início da sílaba
/// recua sobre as consoantes de ataque (é onde o acorde fica na cifra). Não é
/// silabação de dicionário — serve para posicionar acorde, não para cantar.
#[must_use]
pub fn silabas(texto: &str) -> Vec<usize> {
    let caracteres: Vec<char> = texto.chars().collect();
    let mut inicios: Vec<usize> = Vec::new();
    let mut i = 0;
    while i < caracteres.len() {
        if eh_vogal(caracteres[i]) {
            let limite = inicios.last().map_or(0, |&anterior| anterior + 1);
            let mut inicio = i;
            while inicio > limite
                && !eh_vogal(caracteres[inicio - 1])
                && !caracteres[inicio - 1].is_whitespace()
            {
                inicio -= 1;
            }
            // Ataque válido: consoante sozinha, ou o par cujo segundo elemento é
            // `l`, `r` ou `h` (gra, bra, ch, th). Sem isso, "San-to" saía
            // "Sa-nto" — o acorde cairia na consoante errada.
            let quantas_consoantes = i - inicio;
            if quantas_consoantes >= 2 {
                let ultima = caracteres[i - 1].to_lowercase().next();
                let par_valido = matches!(ultima, Some('l' | 'r' | 'h'));
                inicio = if par_valido {
                    (i - 2).max(limite)
                } else {
                    (i - 1).max(limite)
                };
            }
            inicios.push(inicio);
            while i < caracteres.len() && eh_vogal(caracteres[i]) {
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    inicios
}

/// Uma linha da letra posicionada no tempo, com o que o áudio diz dela.
#[derive(Debug, Clone, Serialize)]
pub struct LinhaAlinhada {
    pub indice: usize,
    pub inicio_s: f32,
    pub fim_s: f32,
    pub texto: String,
    /// Rótulo do trecho onde a linha caiu.
    pub trecho: String,
    /// Papel inferido do trecho (refrão candidato, final…).
    pub papel: String,
    /// Loudness local, em LUFS, pela curva de curto prazo.
    pub lufs_medio: f32,
    /// A mesma letra já apareceu antes — é refrão de fato, não rótulo inferido.
    pub repetida: bool,
    /// Quanta confiança merece este alinhamento (0–1). Baixa = conferir.
    pub confianca: f32,
}

/// Média móvel simples.
fn suavizar(serie: &[f32], janela: usize) -> Vec<f32> {
    if janela <= 1 || serie.is_empty() {
        return serie.to_vec();
    }
    let mut saida = Vec::with_capacity(serie.len());
    for i in 0..serie.len() {
        let inicio = i.saturating_sub(janela / 2);
        let fim = (i + janela / 2 + 1).min(serie.len());
        let fatia = &serie[inicio..fim];
        saida.push(fatia.iter().sum::<f32>() / fatia.len() as f32);
    }
    saida
}

/// Funde trechos separados por pausa menor que `pausa_s` (respiro não é fim de frase).
fn fundir(trechos: &mut Vec<(f32, f32)>, pausa_s: f32) {
    let mut fundidos: Vec<(f32, f32)> = Vec::with_capacity(trechos.len());
    for &(inicio, fim) in trechos.iter() {
        match fundidos.last_mut() {
            Some(anterior) if inicio - anterior.1 < pausa_s => anterior.1 = fim,
            _ => fundidos.push((inicio, fim)),
        }
    }
    *trechos = fundidos;
}

/// Frases vocais: onde a voz soa, por energia na banda 200–3500 Hz.
///
/// A referência é o **máximo recente** (4 s, causal), não a média local. Duas
/// medições levaram até aqui:
/// 1. limiar sobre o pico **global** devolveu **uma frase de 385 s em 400** no
///    primeiro áudio real — em mixagem densa a banda da voz nunca cai a 12% do
///    pico;
/// 2. limiar de 1,15× a **média local** (causal, 3 s) não achava o primeiro bloco
///    de um áudio cantado: nível constante nunca fica acima da própria média, e
///    as frases só abriam depois de cada pausa, truncadas em ~2,8 s.
///
/// O que marca presença de voz é **pausa** — a energia cai muito abaixo do
/// máximo recente —, e a frase é o que fica entre pausas.
#[must_use]
pub fn frases_vocais(spectro: &Spectrogram, taxa: u32) -> Vec<(f32, f32)> {
    let energia = spectro.energia_banda(taxa, 200.0, 3500.0);
    if energia.len() < 4 {
        return Vec::new();
    }
    let fps = spectro.quadros_por_segundo(taxa).max(1.0);
    let suave = suavizar(&energia, (0.15 * fps).max(1.0) as usize);
    let pico = suave.iter().copied().fold(0.0f32, f32::max);
    if pico <= f32::EPSILON {
        return Vec::new();
    }
    let piso = pico * 0.02;
    // Janela e limiares: medidos nos dois casos que existem (áudio sintetizado com
    // quatro blocos cantados e uma música real). Mais largos, o motor devolve
    // frases longas demais (`16,7 s` onde há verso); mais estreitos, ele picota a
    // voz. O repertório inteiro é o que falta medir (PENDING).
    let janela_referencia = (2.0 * fps).max(1.0) as usize;
    let abrir = 0.65f32;
    let fechar = 0.50f32;
    let mut brutos: Vec<(f32, f32)> = Vec::new();
    let mut dentro: Option<usize> = None;
    for (i, &valor) in suave.iter().enumerate() {
        let inicio_referencia = i.saturating_sub(janela_referencia);
        let referencia = suave[inicio_referencia..=i]
            .iter()
            .copied()
            .fold(piso, f32::max);
        match dentro {
            None if valor >= referencia * abrir => dentro = Some(i),
            Some(inicio) if valor < referencia * fechar => {
                brutos.push((inicio as f32 / fps, i as f32 / fps));
                dentro = None;
            }
            _ => {}
        }
    }
    if let Some(inicio) = dentro {
        brutos.push((inicio as f32 / fps, suave.len() as f32 / fps));
    }
    fundir(&mut brutos, 0.6);
    brutos.retain(|&(inicio, fim)| fim - inicio >= 0.5);
    brutos
}

/// Loudness local pela curva de curto prazo; sem bloco na janela, usa o mais próximo.
fn lufs_local(relatorio: &Report, inicio_s: f32, fim_s: f32) -> f32 {
    let curva = &relatorio.dynamics.curva_curto_prazo;
    let na_janela: Vec<f64> = curva
        .iter()
        .filter(|&&(t, _)| t >= f64::from(inicio_s) && t <= f64::from(fim_s))
        .map(|&(_, l)| l)
        .collect();
    if !na_janela.is_empty() {
        return (na_janela.iter().sum::<f64>() / na_janela.len() as f64) as f32;
    }
    curva
        .iter()
        .min_by(|a, b| {
            (a.0 as f32 - inicio_s)
                .abs()
                .partial_cmp(&(b.0 as f32 - inicio_s).abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map_or(f32::NAN, |&(_, l)| l as f32)
}

/// Alinha as linhas da letra ao áudio.
///
/// As linhas entram nas **frases vocais** na ordem, proporcional ao tempo de cada
/// frase; dentro da frase, o tempo é dividido pelo número de sílabas. Sem frase
/// detectada, a letra é distribuída pela duração inteira e a confiança cai para
/// o piso — o motor prefere dizer "não sei onde" a inventar precisão.
#[must_use]
pub fn alinhar(
    relatorio: &Report,
    letra: &Letra,
    spectro: &Spectrogram,
    taxa: u32,
) -> Vec<LinhaAlinhada> {
    let n = letra.linhas.len();
    if n == 0 {
        return Vec::new();
    }
    let frases = frases_vocais(spectro, taxa);
    let duracao = relatorio.audio.duracao_s;
    let cobertura: f32 = frases.iter().map(|&(inicio, fim)| fim - inicio).sum();
    // Uma frase só cobrindo (quase) a música inteira não diz **onde** cada linha
    // começa: é o mesmo caso de nenhuma frase. Aconteceu com ruído contínuo de
    // banda larga, que o limiar relativo ao pico lê como um bloco único.
    if frases.is_empty() || (frases.len() == 1 && cobertura > 0.85 * duracao) {
        let tudo = [(0.0f32, duracao)];
        return montar_linhas(relatorio, letra, &tudo, &[n], 0.15);
    }
    let pesos: Vec<f32> = frases.iter().map(|&(a, b)| (b - a).max(0.01)).collect();
    let mut quantas: Vec<usize> = vec![0; frases.len()];
    let mut restantes = n;
    for i in 0..frases.len() {
        if restantes == 0 {
            break;
        }
        if i == frases.len() - 1 {
            quantas[i] = restantes;
            break;
        }
        let peso_restante: f32 = pesos[i..].iter().sum();
        let alvo = (pesos[i] / peso_restante * restantes as f32).round() as usize;
        // Toda frase recebe ao menos uma linha, e cada frase seguinte ainda
        // precisa da sua — o resto não pode ser consumido de uma vez.
        let maximo = restantes.saturating_sub(frases.len() - i - 1).max(1);
        let escolhidas = alvo.clamp(1, maximo.min(restantes));
        quantas[i] = escolhidas;
        restantes -= escolhidas;
    }
    montar_linhas(relatorio, letra, &frases, &quantas, 0.6)
}

/// Distribui as linhas dentro das frases e mede a confiança de cada uma.
fn montar_linhas(
    relatorio: &Report,
    letra: &Letra,
    frases: &[(f32, f32)],
    quantas: &[usize],
    base_confianca: f32,
) -> Vec<LinhaAlinhada> {
    let mut saida: Vec<LinhaAlinhada> = Vec::new();
    let mut proxima = 0usize;
    for (i, &(inicio, fim)) in frases.iter().enumerate() {
        let k = quantas.get(i).copied().unwrap_or(0);
        if k == 0 || proxima >= letra.linhas.len() {
            continue;
        }
        let fim_indice = (proxima + k).min(letra.linhas.len());
        let linhas = &letra.linhas[proxima..fim_indice];
        let silabas_por_linha: Vec<usize> =
            linhas.iter().map(|l| silabas(l).len().max(1)).collect();
        let total_silabas: usize = silabas_por_linha.iter().sum();
        let mut instante = inicio;
        for (j, linha) in linhas.iter().enumerate() {
            let fatia = (fim - inicio) * (silabas_por_linha[j] as f32 / total_silabas as f32);
            let linha_inicio = instante;
            let linha_fim = if j + 1 == linhas.len() {
                fim
            } else {
                instante + fatia
            };
            instante = linha_fim;

            let indice = proxima + j;
            let secao = relatorio
                .sections
                .iter()
                .find(|s| linha_inicio >= s.inicio_s - 0.01 && linha_inicio < s.fim_s);
            let (trecho, papel) = secao.map_or_else(
                || ("?".to_owned(), "?".to_owned()),
                |s| (s.rotulo.clone(), s.papel.clone()),
            );

            let mut confianca = base_confianca;
            if secao.is_some_and(|s| (linha_inicio - s.inicio_s).abs() < 0.6) {
                confianca += 0.15;
            }
            if k > 2 {
                confianca -= 0.2;
            }
            let silabas_por_segundo =
                silabas_por_linha[j] as f32 / (linha_fim - linha_inicio).max(0.1);
            if !(1.0..=8.0).contains(&silabas_por_segundo) {
                confianca -= 0.2;
            }

            saida.push(LinhaAlinhada {
                indice,
                inicio_s: linha_inicio,
                fim_s: linha_fim,
                texto: linha.clone(),
                trecho,
                papel,
                lufs_medio: lufs_local(relatorio, linha_inicio, linha_fim),
                repetida: letra.linhas[..indice]
                    .iter()
                    .any(|anterior| anterior.trim().eq_ignore_ascii_case(linha.trim())),
                confianca: confianca.clamp(0.05, 0.95),
            });
        }
        proxima = fim_indice;
    }
    saida
}
