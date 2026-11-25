//! Notation bundle: song package model and chord handling.
//!
//! Implemented: [`chord_naming`], [`chordpro`], [`harmony`], the in-memory
//! [`bundle`] model (library and setlists) and its persistence ([`bundle_store`],
//! ADR-0006). The bundle is a directory with `meta.json` and `song.cho`; the
//! optional `score.musicxml` and `tabs/*.atex` still arrive later, as does the
//! projection-friendly verse pagination.
//!
//! This crate is domain only: no FFI, and `unsafe_code = "forbid"` holds. The
//! Rust ↔ Flutter boundary lives in the `kentron_ffi` crate (ADR-0007).

pub mod bundle;
pub mod bundle_store;
pub mod chord_naming;
pub mod chordpro;
pub mod harmony;
pub mod musicxml;
pub mod atex;
pub mod openlyrics;
