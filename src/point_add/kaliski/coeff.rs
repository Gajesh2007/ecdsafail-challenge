
#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

/// Optional side-channel coefficient transform used by the tagged-DIV probe.
/// It applies the same linear Kaliski coefficient update to an external
/// `(cr, cs)` pair while the ordinary inverse state still carries the
/// qrisp sentinel needed to uncompute branch flags.
pub(crate) fn coeff_channel_cswap(b: &mut B, ctrl: QubitId, cr: &[QubitId], cs: &[QubitId]) {
    assert_eq!(cr.len(), cs.len());
    for i in 0..cr.len() {
        cswap(b, ctrl, cr[i], cs[i]);
    }
}

pub(crate) fn coeff_channel_cadd(b: &mut B, p: U256, cr: &[QubitId], cs: &[QubitId], ctrl: QubitId) {
    cmod_add_qq(b, cs, cr, ctrl, p);
}

pub(crate) fn coeff_channel_double(b: &mut B, p: U256, cr: &[QubitId]) {
    // The data coefficient is an arbitrary field element, not the bounded
    // qrisp inverse coefficient, so the early no-correction shift is invalid.
    mod_double_inplace_fast(b, cr, p);
}
