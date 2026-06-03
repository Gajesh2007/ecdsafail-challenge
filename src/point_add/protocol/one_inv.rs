//! `protocol::one_inv` — verbatim split of the original `protocol` module.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

pub(crate) fn one_inv_dx3_affine_pa_enabled() -> bool {
    std::env::var(ONE_INV_DX3_AFFINE_PA_ENV).ok().as_deref() == Some("1")
}

pub(crate) fn build_one_inv_dx3_affine_pa_or_break(
    b: &mut B,
    tx: &[QubitId],
    ty: &[QubitId],
    ox: &[BitId],
    oy: &[BitId],
    p: U256,
) -> ! {
    assert_eq!(p, SECP256K1_P);
    assert_eq!(tx.len(), N);
    assert_eq!(ty.len(), N);
    assert_eq!(ox.len(), N);
    assert_eq!(oy.len(), N);
    b.set_phase("one_inv_dx3_affine_pa_blocked_cleanup");
    panic!("{ONE_INV_DX3_AFFINE_PA_BLOCKER}");
}
