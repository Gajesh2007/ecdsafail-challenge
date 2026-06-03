//! Cuccaro ripple-carry adder family.
//!
//! The MAJ / UMA primitives, the textbook `cuccaro_add` / `cuccaro_sub`, and
//! measurement-based "fast" variants (windowed, low-to-extended, borrowed
//! carry), surfaced as n-bit `add_nbit_qq` / `sub_nbit_qq` (and their fast
//! forms). Includes the windowed-adder self-test.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

mod cuccaro;
mod majuma;
mod misc;
mod nbit;

pub(crate) use cuccaro::*;
pub(crate) use majuma::*;
pub(crate) use misc::*;
pub(crate) use nbit::*;
