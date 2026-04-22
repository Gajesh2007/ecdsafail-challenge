//! quantum_ecc library: exposes classical/reference modules for tests and
//! analysis. The main circuit build lives in `src/point_add/` and is
//! driven by the binary crate `src/main.rs`.
//!
//! Modules here are analysis and classical-reference code; they do NOT
//! participate in the binary build and cannot affect the Toffoli metric.

pub mod classical_by;
