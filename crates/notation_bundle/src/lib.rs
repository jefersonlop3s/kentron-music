//! Notation bundle: song package model and chord handling.
//!
//! Implemented: [`chord_naming`], [`chordpro`], [`harmony`], the in-memory
//! [`bundle`] model (library and setlists) and its persistence ([`bundle_store`],
//! ADR-0006). The bundle is a directory with `meta.json` and `song.cho`; the
//! optional `score.musicxml` and `tabs/*.atex` still arrive later, as does the
//! projection-friendly verse pagination.

pub mod bridge;
pub mod bundle;
pub mod bundle_store;
pub mod chord_naming;
pub mod chordpro;
pub mod harmony;
pub mod musicxml;
