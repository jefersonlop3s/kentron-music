//! Relatório da análise: estrutura de dados serializável e resumo legível.

use serde::Serialize;

use crate::chords::ChordSegment;
use crate::dynamics::Dynamics;
use crate::structure::{Section, Tempo};
use crate::tonality::Tonalidade;

/// O que se sabe do arquivo analisado.
#[derive(Debug, Clone, Serialize)]
pub struct AudioInfo {
    pub taxa: u32,
    pub canais: u16,
    pub duracao_s: f32,
    /// Desvio global de afinação, em cents (0 = A440). Medido **antes** do croma
    /// e corrigido nele; positivo = a gravação está acima do padrão.
    pub afinacao_cents: f32,
}

/// O que a melodia tocou num trecho — a evidência que arbitra a terça.
///
/// Existe separado porque **a melodia é extraída e usada mas não aparecia em
/// lugar nenhum**: sem publicá-la, a hipótese de que ela carrega a terça (e as
/// extensões da pentatônica) não era conferível.
#[derive(Debug, Clone, Serialize)]
pub struct MelodiaSegment {
    pub inicio_s: f32,
    pub fim_s: f32,
    /// Quantos quadros do trecho deram F0 confiável.
    pub quadros_com_f0: usize,
    /// Classe de altura dominante, na convenção do motor (`C`, `Db`, …).
    pub dominante: Option<String>,
    /// Fração dos quadros com F0 que a dominante representa.
    pub fracao_dominante: f32,
    /// Histograma por classe de altura (`0` = C).
    pub contagem: [usize; 12],
    /// A melodia **confirma** a terça que o croma acrescentou?
    ///
    /// `None` quando não há o que confirmar (o croma não acrescentou terça).
    /// `Some(false)` é o caso que importa: o croma devolveu o acorde inteiro e a
    /// melodia, que é a outra testemunha, não mostra a terça — candidato a
    /// "terça que não existe". É o risco do limiar, medido com um instrumento
    /// independente do croma.
    pub terca_confirmada: Option<bool>,
}

/// O baixo estimado num trecho de acorde.
///
/// Existe separado do croma de propósito: o croma descarta o registro, e é o
/// registro que separa `C` de `C/E`. Ver `crate::baixo`.
#[derive(Debug, Clone, Serialize)]
pub struct BaixoSegment {
    pub inicio_s: f32,
    pub fim_s: f32,
    /// Nota MIDI do baixo (`28` = E1).
    pub midi: u8,
    /// Nome com oitava, como o músico lê (`E2`).
    pub nota: String,
    /// Fração dos quadros do trecho que sustentam esta nota.
    pub confianca: f32,
    /// Cifra com a barra quando o baixo é terça, quinta ou sétima do acorde
    /// (`C/E`). `None` quando a barra não informaria nada — ver
    /// `crate::chords::com_baixo`.
    pub cifra: Option<String>,
    /// RMS da banda do baixo (40–250 Hz) no trecho, relativo à mediana da
    /// música. **Perto de zero = não há baixo tocando ali**, e a abstenção está
    /// certa: nem toda introdução tem grave, e nem sempre o baixo fala.
    pub energia: f32,
    /// Fração da energia da banda que está sobre os harmônicos da nota (0 a 1).
    ///
    /// **Baixa = o que há ali é percussivo** (bumbo), não nota. É o que julga a
    /// abstenção: energia no grave não distingue baixo de bumbo.
    pub tonalidade: f32,
}

/// Relatório completo da música.
#[derive(Debug, Clone, Serialize)]
pub struct Report {
    pub audio: AudioInfo,
    pub dynamics: Dynamics,
    pub tempo: Tempo,
    /// Centro tonal por perfil (ADR-0014, bloco 1). Substitui a contagem de
    /// fundamental de acorde, que devolvia `B` numa música em E maior.
    pub tonalidade: Tonalidade,
    pub chords: Vec<ChordSegment>,
    /// Baixo por trecho de acorde (ADR-0014, bloco 4). Vazio quando a banda do
    /// baixo não tem evidência — o motor não inventa inversão.
    pub baixo: Vec<BaixoSegment>,
    /// O que a melodia tocou em cada trecho de acorde — a evidência da terça.
    pub melodia: Vec<MelodiaSegment>,
    /// A relação entre o bumbo e a caixa (ADR-0017). **Pressupõe o stem de
    /// bateria** — no mix o número sai contaminado pelo baixo. Ver
    /// `crate::percussao`.
    pub percussao: crate::percussao::BumboCaixa,
    /// Análise comparativa com stem de bateria isolado, quando disponível.
    /// O `mix` reflete o mesmo valor que `percussao`; `com_stems` só existe
    /// quando um diretório de stems foi fornecido.
    pub stems: crate::stems::ComparativoStems,
    /// Compasso e tempo 1 (ADR-0014, bloco 5). `batidas_por_compasso = 0` quando
    /// não há evidência para decidir — ver `crate::compasso`.
    pub compasso: crate::compasso::Compasso,
    pub sections: Vec<Section>,
}

impl Report {
    /// JSON indentado.
    ///
    /// # Errors
    ///
    /// Falha de serialização (na prática, nenhuma com estes tipos).
    pub fn para_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|erro| erro.to_string())
    }

    /// Resumo em texto, para leitura humana no terminal.
    #[must_use]
    pub fn resumo(&self) -> String {
        let mut texto = String::new();
        texto.push_str(&format!(
            "duraçao   {:.1} s · {} Hz · {} canal(is)\n",
            self.audio.duracao_s, self.audio.taxa, self.audio.canais
        ));
        // Abaixo de 5 cents é ruído de medição; acima disso, o arquivo saiu do
        // padrão e quem lê precisa saber que o croma foi corrigido por isso.
        if self.audio.afinacao_cents.abs() >= 5.0 {
            texto.push_str(&format!(
                "afinação  {:+.0} cents fora de A440 (corrigida no croma)\n",
                self.audio.afinacao_cents
            ));
        }
        texto.push_str(&format!(
            "dinâmica  {:.1} LUFS · LRA {:.1} LU · pico {:.1} dBFS\n",
            self.dynamics.lufs_integrado, self.dynamics.lra_lu, self.dynamics.pico_amostra_dbfs
        ));
        texto.push_str(&format!(
            "          mais alto em {:.1} s · mais baixo em {:.1} s\n",
            self.dynamics.mais_alto_s, self.dynamics.mais_baixo_s
        ));
        texto.push_str(&format!(
            "andamento {:.1} BPM · {} batida(s)\n",
            self.tempo.bpm,
            self.tempo.batidas_s.len()
        ));
        // A razão vem junto: correlação alta com margem baixa é decisão frágil,
        // e quem lê precisa saber disso para não tratar como certeza.
        let segundo = self.tonalidade.segunda_nome.as_deref().unwrap_or("?");
        let fragil = if self.tonalidade.margem < 0.10 {
            " · DECISÃO FRÁGIL"
        } else {
            ""
        };
        texto.push_str(&format!(
            "tonalidade {} · correlação {:.2} · margem {:.2} · segunda {} ({:.2}){fragil}\n",
            self.tonalidade.rotulo(),
            self.tonalidade.correlacao,
            self.tonalidade.margem,
            segundo,
            self.tonalidade.segunda_correlacao,
        ));
        // O compasso sai **com a evidência e com a alternativa**, e o nome vem
        // marcado como **sugestão**: ele erra 7 de 8 no gabarito independente de
        // `David Bennett`, então apresentá-lo como fato seria vender palpite.
        let leitura = self.compasso.leitura.sugestao();
        texto.push_str(&if leitura.is_empty() {
            "compasso  — (sem evidência para decidir)\n".to_owned()
        } else {
            format!("{leitura}\n")
        });
        if self.percussao.bumbos > 0 {
            texto.push_str(&format!("percussão {}\n", self.percussao.regra));
        }
        // Pecas: contagem de peças individuais do stem de bateria, quando disponível.
        if let Some(ref pecas) = self.stems.pecas {
            let contagem: Vec<String> = pecas
                .contagem
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect();
            texto.push_str(&format!(
                "pecas    {} · período do bumbo {:.2} s\n",
                contagem.join(", "),
                pecas.kick_periodo_s
            ));
        }
        // Stems: quando o stem de bateria está disponível, mostra a comparação
        // lado a lado, destacando as diferenças que interessam.
        if !self.stems.dinamica.is_empty() {
            // A dinâmica **por instrumento**: quem carrega o loudness da música e
            // quem só preenche. Sai em ordem do mais forte para o mais fraco —
            // ordenado por LUFS, que é a leitura útil, e não por nome de arquivo.
            let mut ordenada = self.stems.dinamica.clone();
            ordenada.sort_by(|a, b| {
                b.lufs_integrado
                    .partial_cmp(&a.lufs_integrado)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            let por_instrumento: Vec<String> = ordenada
                .iter()
                .map(|s| {
                    if s.tem_conteudo() {
                        format!("{} {:.1}", s.nome, s.lufs_integrado)
                    } else {
                        // Sem conteúdo: dizer o número como se fosse dinâmica de
                        // instrumento seria apresentar silêncio como música.
                        format!("{} sem conteúdo", s.nome)
                    }
                })
                .collect();
            texto.push_str(&format!(
                "stems LUFS por instrumento (mono do leitor): {}\n",
                por_instrumento.join(" · ")
            ));
            // O "mais fraco" ignora stem sem conteúdo: −61 LUFS não é o instrumento
            // mais fraco da música, é um instrumento que não toca nela.
            let com_conteudo: Vec<&crate::stems::StemDinamica> =
                ordenada.iter().filter(|s| s.tem_conteudo()).collect();
            if let (Some(forte), Some(fraco)) = (com_conteudo.first(), com_conteudo.last()) {
                texto.push_str(&format!(
                    "           mais forte {} ({:.1} LUFS) · mais fraco {} ({:.1} LUFS) · mix {:.1}\n",
                    forte.nome,
                    forte.lufs_integrado,
                    fraco.nome,
                    fraco.lufs_integrado,
                    self.dynamics.lufs_integrado
                ));
            }
        }
        if let Some(ref stem) = self.stems.com_stems {
            let diff_periodo = (self.stems.mix.periodo_bumbo_s - stem.periodo_bumbo_s).abs();
            let diff_68 = (self.stems.mix.escore_68 - stem.escore_68).abs();
            let diff_34 = (self.stems.mix.escore_34 - stem.escore_34).abs();
            let diff_dobra = self.stems.mix.dobra != stem.dobra;
            texto.push_str(&format!(
                "stems     mix  → período {:.2} s · escore 6/8 {:.2} · 3/4 {:.2} · dobra {}\n\
                           stem → período {:.2} s · escore 6/8 {:.2} · 3/4 {:.2} · dobra {}\n",
                self.stems.mix.periodo_bumbo_s,
                self.stems.mix.escore_68,
                self.stems.mix.escore_34,
                self.stems.mix.dobra,
                stem.periodo_bumbo_s,
                stem.escore_68,
                stem.escore_34,
                stem.dobra,
            ));
            let mut diffs: Vec<String> = Vec::new();
            if diff_periodo > 0.05 {
                diffs.push(format!("período difere em {diff_periodo:.2} s"));
            }
            if diff_68 > 0.05 {
                diffs.push(format!("escore 6/8 difere em {diff_68:.2}"));
            }
            if diff_34 > 0.05 {
                diffs.push(format!("escore 3/4 difere em {diff_34:.2}"));
            }
            if diff_dobra {
                diffs.push(format!(
                    "dobra diverge: mix={} stem={}",
                    self.stems.mix.dobra, stem.dobra
                ));
            }
            if !diffs.is_empty() {
                texto.push_str(&format!("           diferenças: {}\n", diffs.join(" · ")));
            }
        }
        // A terça é a decisão que o croma mais erra, e até aqui ela era invisível
        // no resumo. Sai com o número que decidiu (a razão medida) e com a
        // testemunha independente: quantos trechos a melodia NÃO confirma.
        let por_croma: Vec<&ChordSegment> = self
            .chords
            .iter()
            .filter(|t| t.origem_terca == crate::chords::OrigemDaTerca::Croma)
            .collect();
        if !por_croma.is_empty() {
            let mut razoes: Vec<f32> = por_croma.iter().map(|t| t.terca_razao).collect();
            razoes.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            // A testemunha não é uma: o croma devolveu o acorde inteiro e a melodia
            // (a) mostra outra classe — **contradiz** —, ou (b) não tem quadros com
            // F0 suficientes para dizer nada — **não confirma por falta de dado**.
            // Somar as duas num número só transformaria falta de dado em erro.
            let mut contradita = 0;
            let mut sem_dados = 0;
            for (i, trecho) in self.chords.iter().enumerate() {
                if trecho.origem_terca != crate::chords::OrigemDaTerca::Croma {
                    continue;
                }
                let Some(m) = self.melodia.get(i) else {
                    continue;
                };
                if (m.inicio_s - trecho.inicio_s).abs() >= 0.01 || m.terca_confirmada != Some(false)
                {
                    continue;
                }
                if m.quadros_com_f0 < crate::chords::MINIMO_DE_QUADROS {
                    sem_dados += 1;
                } else {
                    contradita += 1;
                }
            }
            texto.push_str(&format!(
                "terça     {} trecho(s) pelo croma (razão mediana {:.2}) · {} contradito(s) pela \
                 melodia · {} sem F0 para confirmar{}\n",
                por_croma.len(),
                razoes[razoes.len() / 2],
                contradita,
                sem_dados,
                if contradita > 0 {
                    " · DECISÃO DO CROMA SEM TESTEMUNHA"
                } else {
                    ""
                }
            ));
        }
        texto.push_str("\nestrutura (rótulo é repetição; papel é inferência):\n");
        for secao in &self.sections {
            let acordes = self.acordes_do_trecho(secao.inicio_s, secao.fim_s);
            texto.push_str(&format!(
                "  {:>6.1}s–{:>6.1}s  {}  {:<20} energia {:.2}  {}\n",
                secao.inicio_s,
                secao.fim_s,
                secao.rotulo,
                secao.papel,
                secao.energia_relativa,
                acordes
            ));
        }
        texto
    }

    /// Acordes que caem dentro de um intervalo, em ordem, sem repetir vizinho.
    #[must_use]
    pub fn acordes_do_trecho(&self, inicio_s: f32, fim_s: f32) -> String {
        let mut nomes: Vec<&str> = Vec::new();
        for trecho in &self.chords {
            if trecho.fim_s <= inicio_s || trecho.inicio_s >= fim_s {
                continue;
            }
            if nomes.last() != Some(&trecho.acorde.as_str()) {
                nomes.push(&trecho.acorde);
            }
        }
        nomes.join(" ")
    }

    /// Cifra em ChordPro: uma seção por trecho, acordes na linha.
    ///
    /// A letra **não** entra — vem de ASR, que é outro incremento (ADR-0011).
    #[must_use]
    pub fn para_chordpro(&self, titulo: &str) -> String {
        // A leitura vai como **comentário**, não como `time`: o nome do motor é
        // sugestão (1 de 8 no gabarito independente), e `{time: …}` é a decisão
        // do músico.
        let sugestao = self.compasso.leitura.sugestao();
        let compasso = if sugestao.is_empty() {
            String::new()
        } else {
            format!("{{comment: {sugestao}}}\n")
        };
        let mut texto = format!(
            "{{title: {titulo}}}\n{{key: {}}}\n{compasso}{{tempo: {:.0}}}\n{{duration: {:.0}}}\n{{source: mir_engine}}\n",
            self.tonalidade.rotulo(),
            self.tempo.bpm,
            self.audio.duracao_s
        );
        for secao in &self.sections {
            texto.push_str(&format!(
                "\n[{}-{:.0}s {}]\n",
                secao.rotulo, secao.inicio_s, secao.papel
            ));
            let mut anterior = "";
            for trecho in &self.chords {
                if trecho.fim_s <= secao.inicio_s || trecho.inicio_s >= secao.fim_s {
                    continue;
                }
                if trecho.acorde != anterior {
                    texto.push_str(&format!("[{}] ", trecho.acorde));
                    anterior = &trecho.acorde;
                }
            }
            texto.push('\n');
        }
        texto
    }
}
