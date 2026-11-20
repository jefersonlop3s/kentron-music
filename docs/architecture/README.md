# Arquitetura · Kentron Music Hub

## 📐 Visão Geral
O Kentron utiliza uma **Arquitetura Hexagonal (Ports and Adapters)** para garantir a separação total entre a lógica de negócio musical (estável) e as interfaces de usuário/transporte (voláteis).

### 🛠️ Stack Tecnológica
- **Core:** Rust (Alta performance, segurança de memória, portabilidade).
- **UI:** Flutter (Interface declarativa, multiplataforma, alta fidelidade).
- **Ponte:** `flutter_rust_bridge` (Comunicação eficiente via FFI).
- **Sincronia:** Matrix (Estado assíncrono) + Iroh (Dados efêmeros/LAN).

## 📦 Decomposição de Módulos

### 1. Rust Core (`crates/`)
- `notation_bundle`: Modelo canônico de música. Lida com transposição, notação e parsing de arquivos.
- `mir_engine`: Motor de análise de áudio (MIR) via modelos ONNX.
- `harmonic_index`: Gestão do índice de obras e busca global.
- `band_mesh`: Implementação do protocolo Matrix e sincronia de estado.
- `stage_clock`: Lógica de regência, tempo e sincronia de palco.
- `trust_safety`: Moderação e listas de bloqueio assinadas.

### 2. Flutter Frontend (`apps/` e `packages/`)
- `kentron_app`: Aplicação principal.
- `core_l10n`: Recurso transversal de tradução (ARB).
- `feature_stage`: UI de performance e regência.
- `feature_library`: Gestão de setlists e biblioteca de músicas.
- `feature_projection`: Exportação e controle de projeção.

## 🛡️ Fluxo de Dados
`Arquivo (.cho) ➔ Rust Parser ➔ Objeto Canônico ➔ Flutter Bridge ➔ UI Impeccable`

O dado musical nunca é armazenado como string localizada; ele vive como estrutura canônica no Rust e é traduzido apenas na "borda" (Flutter) conforme a preferência do usuário.


## 📌 Estado atual (R0/R1)

- Existe apenas o crate `notation_bundle`, com `chord_naming`, parser ChordPro e placeholder de MusicXML.
- Os módulos `mir_*`, `band_*`, `stage_*` e `trust_*` ainda não foram criados.
- O app Flutter é um scaffold; `core_l10n` e `flutter_rust_bridge` ainda não estão integrados.
- A ponte Rust ↔ Flutter é um placeholder (`bridge.rs`).

