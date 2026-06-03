//! Round 629–671 "frontier" packed-iteration experiments.
//!
//! Predicate and equality streams, Cuccaro predicate schedules, borrow / carry
//! retap taps, and packed Kaliski iteration skeletons.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

mod builders1;
mod builders2;
mod builders3;
mod cswap;
mod latch;
mod misc1;
mod misc2;
mod misc3;
mod packed;
mod sidecar;
mod streams;
mod uncompute;

pub(crate) use builders1::*;
pub(crate) use builders2::*;
pub(crate) use builders3::*;
pub(crate) use cswap::*;
pub(crate) use latch::*;
pub(crate) use misc1::*;
pub(crate) use misc2::*;
pub(crate) use misc3::*;
pub(crate) use packed::*;
pub(crate) use sidecar::*;
pub(crate) use streams::*;
pub(crate) use uncompute::*;
