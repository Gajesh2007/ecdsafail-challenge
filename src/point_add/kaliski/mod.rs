//! Kaliski binary almost-inverse: the reversible modular inversion that
//! dominates the point-addition cost.
//!
//! Provides the forward / backward iteration drivers, the per-iteration step
//! logic (including bulk-prefix and branch-merged variants), coefficient
//! channel updates, branch-state allocation / freeing, HMR-based discard of
//! phase-dirty scratch, and the iteration-count safety gating that keeps the
//! sweep within proven bounds.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

mod branch1;
mod branch2;
mod coeff;
mod config;
mod iteration1;
mod iteration2;
mod raw;
mod step;
mod util;

pub(crate) use branch1::*;
pub(crate) use branch2::*;
pub(crate) use coeff::*;
pub(crate) use config::*;
pub(crate) use iteration1::*;
pub(crate) use iteration2::*;
pub(crate) use raw::*;
pub(crate) use step::*;
pub(crate) use util::*;
