//! "D1" in-place product / quotient lowerer experiments.
//!
//! In-place product and quotient lowerers and the direct-quotient arithmetic
//! derived from raw and prescaled inverses.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

mod direct_quotient;
mod inplace;

pub(crate) use direct_quotient::*;
pub(crate) use inplace::*;
