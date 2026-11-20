//! Notation bundle: song package model and chord handling.
//!
//! Implemented: [`chord_naming`], [`chordpro`], [`harmony`] and the in-memory
//! [`bundle`] model (library and setlists). Persisting the package on disk
//! (`meta.json`, `.cho`, `.musicxml`, `tabs/*.atex`) depends on ADR-0006 and is
//! not implemented yet; the projection-friendly verse pagination arrives in R1.

pub mod bridge;
pub mod bundle;
pub mod chord_naming;
pub mod chordpro;
pub mod harmony;
pub mod musicxml;
