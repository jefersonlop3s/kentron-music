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
        let mut texto = format!(
            "{{title: {titulo}}}\n{{key: {}}}\n{{tempo: {:.0}}}\n{{duration: {:.0}}}\n{{source: mir_engine}}\n",
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
