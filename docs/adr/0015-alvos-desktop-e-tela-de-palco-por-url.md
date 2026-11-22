# ADR-0015 · Alvos desktop (Windows, macOS, Linux) e a tela de palco por URL

- **Status:** proposto (2026-11-22) — aguarda aceite do dono; sequência recomendada: **depois da release da R1**
- **Relaciona-se:** [ADR-0008](0008-native-assets-em-vez-de-cargokit.md) (empacotamento), [ADR-0013](0013-link-de-musica-referencia-playlist-e-fronteira-do-audio.md) e [ADR-0014](0014-analise-harmonica-pos-extracao.md) (o que o desktop destrava)

## Contexto

O dono trouxe a demanda: **tecladistas usam notebook**, e o palco precisa de uma tela com cifra para o músico — não a projeção da congregação. Os alvos do projeto hoje são **Android, iOS e Web**; não há desktop.

Três coisas que o desktop resolve e o alvo web não:

1. **Arquivo de áudio local** — abrir um WAV de 40 MB sem upload e sem servidor.
2. **MIR em CPU com memória** — a análise da ADR-0014 (F0 de melodia e de baixo, downbeat) e um eventual modelo de stems cabem num notebook; no wasm, não.
3. **Uso offline de verdade** — sem depender de `servir-web.py` na mesma máquina.

A objeção que eu tinha levantado em 2026-11-22 era a **tela secundária**: no Flutter não há multi-janela de primeira classe, e sem alvo desktop não havia como mandar a cifra para a TV. A resposta está na arquitetura que o projeto **já tem pronta**:

- A tela de palco existe (`SongReaderPage`: acorde sobre a letra, transposição, contraste).
- O **bootstrap por URL** existe (`?projecao=` abre direto na tela, sem passar pela home).
- **Já foi provado** que essa URL renderiza fora do app: `scripts/piloto-obs.sh` abriu a projeção num Browser Source do OBS, em outra máquina, por túnel reverso.

Ou seja: **a tela de palco não precisa de uma segunda pipeline de renderização — precisa de um endereço.**

## Decisão

1. **Adotar os três alvos desktop** (`flutter create --platforms=windows,macos,linux`). O núcleo Rust continua empacotado pelo **mesmo hook de Native Assets** (ADR-0008); os triples de desktop entram no `rust-toolchain.toml` e no pin do crate.
2. **A tela de palco é uma view endereçada por URL**, não uma janela paralela de estado próprio: `?palco=<base64url do ChordPro>&tom=G` abre `SongReaderPage`. O mesmo endereço serve três destinos — navegador da TV, segunda janela do desktop e Browser Source do OBS.
3. **A segunda tela não vira multi-janela nativa.** O desktop abre uma **segunda janela web** (webview do SO, ou o próprio `servir-web.py` local) apontando para `?palco=`, e o sistema operacional manda a janela para o monitor. Isso evita depender de pacote comunitário de multi-janela e mantém **um único caminho de renderização** — o que já está provado.
4. **Sequência: fora da R1.** Adotar desktop mexe em empacotamento, assinatura por SO e matriz de CI; não entra no portão da release atual. Fica como R1.1/R2.

## Consequências

- **Ganho:** o notebook vira cidadão de primeira classe; a análise de áudio deixa de depender de upload; o modelo de stems (ADR-0014, etapa 3) passa a caber; e a "tela secundária" deixa de ser um problema de framework para virar um problema resolvido de endereço.
- **Custo:** a matriz de CI triplica; `rust-toolchain.toml` e o hook ganham triples novos; e cada SO tem seu empacotamento — Windows (MSIX ou zip, com certificado), macOS (app assinado **e notarizado**, exige conta Apple), Linux (AppImage ou deb).
- **Ordem recomendada: Windows → Linux → macOS.** O público-alvo é igreja com notebook Windows, e a notarização do macOS é o item mais caro e o que menos entrega a esse público. Fazer macOS primeiro é pagar o preço mais alto pelo menor alcance.
- **Risco nomeado:** a segunda janela no desktop depende de **webview** (plugin por SO). É dívida declarada, não escondida — e por isso o caminho de escape é o navegador, que não depende de plugin nenhum.
- **Risco nomeado:** `?palco=` amplia a superfície pública do bootstrap. Ele precisa do **mesmo portão de publicação** do link de projeção (`pode_publicar`), senão a cifra privada do grupo vazaria por outra porta.
- **Pressuposto declarado:** "um notebook por equipe, com app e palco na mesma rede local, sem depender de internet no culto".

## Referências

- [ADR-0008](0008-native-assets-em-vez-de-cargokit.md) — o mecanismo de empacotamento que já cobre desktop.
- [ADR-0013](0013-link-de-musica-referencia-playlist-e-fronteira-do-audio.md) — o áudio que o desktop passa a poder analisar localmente.
- [ADR-0014](0014-analise-harmonica-pos-extracao.md) — a análise que o desktop destrava.
- [ADR-0010](0010-piloto-automatizado-antes-do-uso-ao-vivo.md) — a prova por URL no OBS, que é a base do item 2.
- [PENDING](../PENDING.md) — tela secundária e monitor de palco.
- `scripts/piloto-obs.sh` — a prova de que a URL renderiza fora do app.