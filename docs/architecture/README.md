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
- `notation_bundle`: Modelo canônico de música. Lida com transposição, notação e parsing de arquivos. É **domínio puro**: sem FFI e com `unsafe_code = "forbid"`.
- `kentron_ffi`: **Adaptador** Rust ↔ Flutter (ADR-0007). Único `cdylib` e único a depender do `flutter_rust_bridge`; declara a API que o Dart enxerga (`bridge.rs`) e hospeda o código gerado.
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

- O domínio é `notation_bundle`: `chord_naming`, parser ChordPro, `harmony`, o modelo de bundle/setlists com persistência (ADR-0006) e `musicxml` **como stub** — o módulo assina o contrato (`MusicXmlParser::parse`) e recusa com erro explícito; a implementação depende do spike S5 (R1.1). Os módulos `mir_*`, `band_*`, `stage_*` e `trust_*` ainda não existem.
- A fronteira FFI é `kentron_ffi` (ADR-0007): expõe `parse_song` ao Dart e é a única a carregar código gerado — o domínio segue sem FFI, com `unsafe_code = "forbid"`.
- `core_l10n` e `flutter_rust_bridge` **estão integrados**; a ponte deixou de ser placeholder e é provada por um teste de Flutter que carrega o `cdylib` e chama o núcleo.
- O app Flutter ainda **não tem diretórios de plataforma**: a ponte roda em teste, mas o app não abre em dispositivo até isso ser criado.

