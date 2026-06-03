//! Modular multiplication and squaring.
//!
//! Schoolbook and one/two-level Karatsuba products, Montgomery multiply and
//! square, symmetric squaring, Solinas reduction of the double-width product,
//! and the controlled add / subtract cores those algorithms are built from.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

mod cores;
mod karatsuba1;
mod karatsuba2;
mod montgomery;
mod schoolbook1;
mod schoolbook2;
mod solinas;
mod squaring;

pub(crate) use cores::*;
pub(crate) use karatsuba1::*;
pub(crate) use karatsuba2::*;
pub(crate) use montgomery::*;
pub(crate) use schoolbook1::*;
pub(crate) use schoolbook2::*;
pub(crate) use solinas::*;
pub(crate) use squaring::*;
