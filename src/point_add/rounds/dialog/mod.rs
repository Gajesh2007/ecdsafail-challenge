//! "Dialog GCD" half-GCD experiments (raw and compressed).
//!
//! Bitvector `tobitvector` / `apply` steps, compressed-sidecar block
//! lifecycles, high-tail transcript handling, quotient transport, and the raw
//! point-add assembly built on top of them.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

mod compressed1;
mod compressed2;
mod compressor;
mod misc1;
mod misc2;
mod raw;

pub(crate) use compressed1::*;
pub(crate) use compressed2::*;
pub(crate) use compressor::*;
pub(crate) use misc1::*;
pub(crate) use misc2::*;
pub(crate) use raw::*;
