//! Fronteira Rust ↔ Flutter (ADR-0007).
//!
//! Este crate é o adaptador: só ele é `cdylib`, só ele depende do
//! `flutter_rust_bridge`, e só aqui a cola gerada vive. O domínio
//! (`notation_bundle`) segue puro, com `unsafe_code = "forbid"`.
//!
//! Para regerar os bindings depois de mudar [`bridge`]:
//!
//! ```text
//! flutter_rust_bridge_codegen generate --config-file flutter_rust_bridge.yaml
//! ```

// O módulo `frb_generated` é escrito pelo gerador e contém um bloco `unsafe`
// (desserialização do wire). O allow é escopado a ele — este crate usa `deny`
// (ver Cargo.toml), então nenhuma outra parte pode usar `unsafe` sem allow.
#[allow(unsafe_code)]
mod frb_generated;

pub mod bridge;
