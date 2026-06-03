//! Top-level affine point-addition (P + Q) circuit assemblies.
//!
//! Wires the field-arithmetic ([`arith`](super::arith)) and modular-inversion
//! ([`kaliski`](super::kaliski)) layers into complete point-add circuits:
//! the standard builder, the compact builder (and its early-inverse-clean
//! variant), and the one-inversion `dx^3` fail-closed affine path.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

mod compact;
mod one_inv;
mod standard;

pub(crate) use compact::*;
pub(crate) use one_inv::*;
pub(crate) use standard::*;
