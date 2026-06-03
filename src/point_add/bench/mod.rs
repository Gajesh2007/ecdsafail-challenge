//! Benchmark and experiment scaffolds (not part of the scored circuit).
//!
//! Houses the `*_for_bench` primitives and `benchmark_scaffold` harness used by
//! the research microbenchmarks to measure isolated sub-circuits in isolation
//! (centered-by microsteps, scaled-by replay, single-inversion shapes). Nothing
//! here is reachable from the scored [`build`](super::build) entry point.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

mod by_ops;
mod centered1;
mod centered2;
mod flags;
mod misc;
mod scaffold;
mod scaled;

pub(crate) use by_ops::*;
pub(crate) use centered1::*;
pub(crate) use centered2::*;
pub(crate) use flags::*;
pub(crate) use misc::*;
pub(crate) use scaffold::*;
pub(crate) use scaled::*;
