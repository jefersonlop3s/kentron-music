//! Notation bundle: song package model and chord handling.
//!
//! R0 only implements [`chord_naming`]. The package layout (`meta.json`, `.cho`,
//! `.musicxml`, `tabs/*.atex`) and the projection-friendly verse pagination arrive in R1.

pub mod bridge;
pub mod chord_naming;
pub mod chordpro;
pub mod musicxml;
