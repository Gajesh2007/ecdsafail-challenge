//! Experimental "round" circuit builders from the research loop.
//!
//! Each `build_roundNNN_*` emits a self-contained component (and its
//! phase-resource profile) probing one optimization idea. Builders are grouped
//! by lineage:
//!
//! - [`frontier`]         — round 629–671 packed-iteration / predicate-stream work
//! - [`dialog`]           — "dialog GCD" raw & compressed half-GCD experiments
//! - [`b5`]               — round 218 / 3xx source-live coefficient transport
//! - [`d1`]               — in-place product / quotient lowerers
//! - [`mid`]              — round 499–592 Solinas raw-splice and square tails
//! - [`low`]              — round 8–199 square tails, half-GCD PA, JSF, numeric steps
//! - [`high`]             — round >= 700 experiments
//! - [`direct_centered`]  — direct-centered branch fits and binary-trie QROM
//! - [`source_live`]      — source-live cubic product-tail helpers

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;


mod b5;
mod d1;
mod dialog;
mod direct_centered;
mod frontier;
mod high;
mod low;
mod mid;
mod source_live;

pub(crate) use b5::*;
pub(crate) use d1::*;
pub(crate) use dialog::*;
pub(crate) use direct_centered::*;
pub(crate) use frontier::*;
pub(crate) use high::*;
pub(crate) use low::*;
pub(crate) use mid::*;
pub(crate) use source_live::*;
