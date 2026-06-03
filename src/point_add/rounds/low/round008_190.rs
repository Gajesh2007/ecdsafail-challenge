
#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

pub(crate) fn round84_emit_fused_square_xtail(
    b: &mut B,
    tx: &[QubitId],
    lam: &[QubitId],
    ox: &[BitId],
    p: U256,
) {
    b.set_phase("round84_fused_square_xtail_dx_sub_lam_square_lowq");
    if std::env::var("ROUND84_XTAIL_KARATSUBA").ok().as_deref() == Some("1") {
        // Squaring-aware 1-level Karatsuba square (default OFF). Overrides the
        // ROUND84_XTAIL_SCHOOLBOOK default set in configure_ecdsafail_submission_route.
        squaring_sub_from_acc_karatsuba(b, tx, lam, p);
    } else if std::env::var("ROUND84_XTAIL_WALK_SQUARE").ok().as_deref() == Some("1") {
        squaring_sub_from_acc_walk_controls_lowq(b, tx, lam, p);
    } else if std::env::var("ROUND84_XTAIL_SCHOOLBOOK").ok().as_deref() == Some("1") {
        squaring_sub_from_acc_schoolbook(b, tx, lam, p);
    } else {
        squaring_sub_from_acc_schoolbook_lowq_shift22(b, tx, lam, p);
    }
    b.set_phase("round84_fused_square_xtail_add_double_ox");
    mod_add_double_qb(b, tx, ox, p);
    b.set_phase("round84_fused_square_xtail_negate_to_x3");
    mod_neg_inplace_fast(b, tx, p);
}
