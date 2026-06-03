//! Modular arithmetic over the secp256k1 (Solinas) prime field.
//!
//! Modular addition / subtraction (including vented, low-peak variants),
//! negation, doubling, halving, and shift-by-k mod p, together with their
//! controlled forms and the conditional-swap helper.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

mod add;
mod controlled;
mod misc;
mod neg;
mod scale;
mod sub;

pub(crate) use add::*;
pub(crate) use controlled::*;
pub(crate) use misc::*;
pub(crate) use neg::*;
pub(crate) use scale::*;
pub(crate) use sub::*;
