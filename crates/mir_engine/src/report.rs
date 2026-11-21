//! Relatório da análise: estrutura de dados serializável e resumo legível.

use serde::Serialize;

use crate::chords::ChordSegment;
use crate::dynamics::Dynamics;
use crate::structure::{Section, Tempo};

/// O que se sabe do arquivo analisado.
#[derive(Debug, Clone, Serialize)]
pub struct AudioInfo {
    pub taxa: u32,
    pub canais: u16,
    pub duracao_s: f32,
}

/// Relatório completo da música.
#[derive(Debug, Clone, Serialize)]
pub struct Report {
    pub audio: AudioInfo,
    pub dynamics: Dynamics,
    pub tempo: Tempo,
    pub chords: Vec<ChordSegment>,
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

    /// Tom **estimado**: a fundamental mais frequente e a qualidade dominante
    /// dela. É palpite fundamentado, não análise de centro tonal.
    #[must_use]
    pub fn tom_estimado(&self) -> String {
        let mut contagem: Vec<(String, usize, usize)> = Vec::new(); // raiz, maior, menor
        for trecho in &self.chords {
            let (raiz, qualidade) = raiz_e_qualidade(&trecho.acorde);
            let menor = qualidade.starts_with('m') && !qualidade.starts_with("maj");
            match contagem.iter_mut().find(|(r, _, _)| *r == raiz) {
                Some((_, maior, menor_cont)) => {
                    if menor {
                        *menor_cont += 1;
                    } else {
                        *maior += 1;
                    }
                }
                None => contagem.push((raiz, (!menor) as usize, menor as usize)),
            }
        }
        contagem
            .iter()
            .max_by_key(|(_, maior, menor)| maior + menor)
            .map_or_else(
                || "C".to_owned(),
                |(raiz, maior, menor)| {
                    if menor > maior {
                        format!("{raiz}m")
                    } else {
                        raiz.clone()
                    }
                },
            )
    }

    /// Cifra em ChordPro: uma seção por trecho, acordes na linha.
    ///
    /// A letra **não** entra — vem de ASR, que é outro incremento (ADR-0011).
    #[must_use]
    pub fn para_chordpro(&self, titulo: &str) -> String {
        let mut texto = format!(
            "{{title: {titulo}}}\n{{key: {}}}\n{{tempo: {:.0}}}\n{{duration: {:.0}}}\n{{source: mir_engine}}\n",
            self.tom_estimado(),
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

/// Separa o nome do acorde em fundamental e qualidade.
///
/// "Db" tem `b` minúsculo: cortar na primeira minúscula devolvia `D`, e "B5"
/// devolvia `B5` como se o `5` fosse da fundamental — medido na primeira cifra
/// gerada (`{key: B5}`).
fn raiz_e_qualidade(acorde: &str) -> (String, String) {
    let mut chars = acorde.chars();
    let mut raiz = match chars.next() {
        Some(letra) => letra.to_string(),
        None => return (String::new(), String::new()),
    };
    if let Some(segundo) = chars.clone().next()
        && (segundo == 'b' || segundo == '#')
    {
        raiz.push(segundo);
        chars.next();
    }
    (raiz, chars.collect())
}
