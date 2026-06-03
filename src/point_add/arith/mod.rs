//! Reversible field-arithmetic primitives for secp256k1.
//!
//! Aggregates the low-level building blocks consumed by the inversion,
//! multiplication, and point-addition layers. Submodules:
//!
//! - [`registers`]   — load / unload constants and classical bits, register extension
//! - [`adder`]       — Cuccaro ripple-carry adder family and n-bit add / sub
//! - [`const_arith`] — add / sub of compile-time constants (controlled, fast, direct)
//! - [`modular`]     — add / sub / neg / double / halve / shift, all mod p
//! - [`multiply`]    — schoolbook, Karatsuba, Montgomery, squaring, Solinas reduction
//! - [`compare`]     — less-than / greater-than / equality predicates into a flag
//! - [`shift_ctrl`]  — single-bit logical shifts and controlled Cuccaro lanes
//! - [`config`]      — environment-variable feature gates
//! - [`util`]        — assorted helpers (curve reference, probes, construction)

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;


mod adder;
mod compare;
mod config;
mod const_arith;
mod modular;
mod multiply;
mod registers;
mod shift_ctrl;
mod util;

pub(crate) use adder::*;
pub(crate) use compare::*;
pub(crate) use config::*;
pub(crate) use const_arith::*;
pub(crate) use modular::*;
pub(crate) use multiply::*;
pub(crate) use registers::*;
pub(crate) use shift_ctrl::*;
pub(crate) use util::*;
