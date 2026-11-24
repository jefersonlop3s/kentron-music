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
        texto.push_str(&format!(
            "tonalidade {} · correlação {:.2} · margem {:.2}\n",
            self.tonalidade.rotulo(),
            self.tonalidade.correlacao,
            self.tonalidade.margem
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
