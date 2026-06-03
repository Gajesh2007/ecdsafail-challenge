//! `d1::direct_quotient` — verbatim split of the original `d1` module.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

pub(crate) fn d1_direct_quotient_compute_product_from_raw_inverse(
    b: &mut B,
    numerator: &[QubitId],
    quotient: &[QubitId],
    inv_raw: &[QubitId],
    p: U256,
) {
    let compute_mul = std::env::var("D1_DIRECT_QUOTIENT_COMPUTE_MUL").ok();
    let compute_mul = compute_mul.as_deref().or_else(|| {
        if d1_phase_corrected_arith_core_enabled() {
            Some("schoolbook")
        } else {
            None
        }
    });
    match compute_mul {
        Some("schoolbook") => {
            mod_mul_write_into_zero_acc_schoolbook(b, quotient, numerator, inv_raw, p)
        }
        Some("schoolbook_lowq") => {
            mod_mul_write_into_zero_acc_schoolbook_lowq(b, quotient, numerator, inv_raw, p)
        }
        Some("schoolbook_peak_lowq") => {
            mod_mul_write_into_zero_acc_schoolbook_peak_lowq(b, quotient, numerator, inv_raw, p)
        }
        Some("horner") => mod_mul_horner_add_qq(b, quotient, numerator, inv_raw, p),
        Some("karatsuba1") | Some("1") => {
            mod_mul_write_into_zero_acc_karatsuba(b, quotient, numerator, inv_raw, p)
        }
        Some("karatsuba_lowq") | Some("lowq") => {
            mod_mul_write_into_zero_acc_karatsuba_lowq(b, quotient, numerator, inv_raw, p)
        }
        _ => mod_mul_write_into_zero_acc_karatsuba2(b, quotient, numerator, inv_raw, p),
    }
}

pub(crate) fn d1_direct_quotient_unscale_neg_product(
    b: &mut B,
    quotient: &[QubitId],
    p: U256,
    inverse_iters: usize,
) {
    let exact_unscale = std::env::var("D1_DIRECT_QUOTIENT_EXACT_UNSCALE")
        .ok()
        .as_deref()
        == Some("1");
    if exact_unscale {
        mod_neg_inplace(b, quotient, p);
    } else {
        mod_neg_inplace_fast(b, quotient, p);
    }
    for _ in 0..inverse_iters {
        if exact_unscale {
            mod_halve_inplace(b, quotient, p);
        } else {
            mod_halve_inplace_fast(b, quotient, p);
        }
    }
}

pub(crate) fn d1_inplace_hmr_phase_clean_direct_quotient_enabled() -> bool {
    std::env::var("D1_INPLACE_HMR_PHASE_CLEAN_DIRECT_QUOTIENT")
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn d1_direct_quotient_compute_into_zero(
    b: &mut B,
    factor: &[QubitId],
    numerator: &[QubitId],
    quotient: &[QubitId],
    p: U256,
    inverse_iters: usize,
) {
    debug_assert_eq!(factor.len(), N);
    debug_assert_eq!(numerator.len(), N);
    debug_assert_eq!(quotient.len(), N);

    if std::env::var("D1_DIRECT_QUOTIENT_PRESCALED_KAL")
        .ok()
        .as_deref()
        == Some("1")
    {
        let chunked = std::env::var("D1_DIRECT_QUOTIENT_PRESCALED_CHUNKED")
            .ok()
            .as_deref()
            != Some("0");
        let mut compute_from_prescaled_inverse = |b: &mut B, inv_raw: &[QubitId]| {
            b.set_phase("d1_direct_quotient_compute_neg_prescaled_quotient");
            let compute_mul = std::env::var("D1_DIRECT_QUOTIENT_COMPUTE_MUL").ok();
            let compute_mul = compute_mul.as_deref().or_else(|| {
                if d1_phase_corrected_arith_core_enabled() {
                    Some("schoolbook")
                } else {
                    None
                }
            });
            match compute_mul {
                Some("schoolbook") => {
                    mod_mul_write_into_zero_acc_schoolbook(b, quotient, numerator, inv_raw, p)
                }
                Some("schoolbook_lowq") => {
                    mod_mul_write_into_zero_acc_schoolbook_lowq(b, quotient, numerator, inv_raw, p)
                }
                Some("schoolbook_peak_lowq") => mod_mul_write_into_zero_acc_schoolbook_peak_lowq(
                    b, quotient, numerator, inv_raw, p,
                ),
                Some("horner") => mod_mul_horner_add_qq(b, quotient, numerator, inv_raw, p),
                Some("karatsuba1") | Some("1") => {
                    mod_mul_write_into_zero_acc_karatsuba(b, quotient, numerator, inv_raw, p)
                }
                Some("karatsuba_lowq") | Some("lowq") => {
                    mod_mul_write_into_zero_acc_karatsuba_lowq(b, quotient, numerator, inv_raw, p)
                }
                _ => mod_mul_write_into_zero_acc_karatsuba2(b, quotient, numerator, inv_raw, p),
            }
            b.set_phase("d1_direct_quotient_unneg_prescaled_quotient");
            mod_neg_inplace_fast(b, quotient, p);
        };
        b.set_phase("d1_direct_quotient_compute_prescaled_raw_inverse");
        if chunked {
            with_kal_inv_raw_prescaled_chunked(b, factor, p, inverse_iters, |b, inv_raw| {
                compute_from_prescaled_inverse(b, inv_raw)
            });
        } else {
            with_kal_inv_raw_prescaled_mixed(b, factor, p, inverse_iters, |b, inv_raw| {
                compute_from_prescaled_inverse(b, inv_raw)
            });
        }
        return;
    }

    b.set_phase("d1_direct_quotient_compute_raw_inverse");
    if std::env::var("D1_DIRECT_QUOTIENT_BRANCH_INV")
        .ok()
        .as_deref()
        == Some("1")
    {
        with_kal_branch_inv_raw_roll(b, factor, p, inverse_iters, |b, inv_raw| {
            d1_direct_quotient_arithmetic_from_raw_inverse(
                b,
                numerator,
                quotient,
                inv_raw,
                p,
                inverse_iters,
            )
        });
    } else if std::env::var("D1_DIRECT_QUOTIENT_HMR_DISCARD_KALISKI_STATE")
        .ok()
        .as_deref()
        == Some("1")
    {
        with_kal_inv_raw_hmr_discard(b, factor, p, inverse_iters, |b, inv_raw| {
            d1_direct_quotient_arithmetic_from_raw_inverse(
                b,
                numerator,
                quotient,
                inv_raw,
                p,
                inverse_iters,
            )
        });
    } else if std::env::var("D1_DIRECT_QUOTIENT_UNBORROWED_INV")
        .ok()
        .as_deref()
        == Some("1")
    {
        with_kal_inv_raw(b, factor, p, inverse_iters, |b, inv_raw| {
            d1_direct_quotient_arithmetic_from_raw_inverse(
                b,
                numerator,
                quotient,
                inv_raw,
                p,
                inverse_iters,
            )
        });
    } else {
        with_kal_inv_raw_borrowing_v(b, factor, p, inverse_iters, |b, inv_raw| {
            d1_direct_quotient_arithmetic_from_raw_inverse(
                b,
                numerator,
                quotient,
                inv_raw,
                p,
                inverse_iters,
            )
        });
    }
}

pub(crate) fn d1_direct_quotient_arithmetic_from_raw_inverse(
    b: &mut B,
    numerator: &[QubitId],
    quotient: &[QubitId],
    inv_raw: &[QubitId],
    p: U256,
    inverse_iters: usize,
) {
    debug_assert_eq!(numerator.len(), N);
    debug_assert_eq!(quotient.len(), N);
    debug_assert_eq!(inv_raw.len(), N);

    b.set_phase("d1_direct_quotient_compute_neg_scaled_quotient");
    let compute_mul = std::env::var("D1_DIRECT_QUOTIENT_COMPUTE_MUL").ok();
    let compute_mul = compute_mul.as_deref().or_else(|| {
        if d1_phase_corrected_arith_core_enabled() {
            Some("schoolbook")
        } else {
            None
        }
    });
    match compute_mul {
        Some("schoolbook") => {
            mod_mul_write_into_zero_acc_schoolbook(b, quotient, numerator, inv_raw, p)
        }
        Some("schoolbook_lowq") => {
            mod_mul_write_into_zero_acc_schoolbook_lowq(b, quotient, numerator, inv_raw, p)
        }
        Some("schoolbook_peak_lowq") => {
            mod_mul_write_into_zero_acc_schoolbook_peak_lowq(b, quotient, numerator, inv_raw, p)
        }
        Some("horner") => mod_mul_horner_add_qq(b, quotient, numerator, inv_raw, p),
        Some("karatsuba1") | Some("1") => {
            mod_mul_write_into_zero_acc_karatsuba(b, quotient, numerator, inv_raw, p)
        }
        Some("karatsuba_lowq") | Some("lowq") => {
            mod_mul_write_into_zero_acc_karatsuba_lowq(b, quotient, numerator, inv_raw, p)
        }
        _ => mod_mul_write_into_zero_acc_karatsuba2(b, quotient, numerator, inv_raw, p),
    }
    b.set_phase("d1_direct_quotient_unscale_quotient");
    let exact_unscale = std::env::var("D1_DIRECT_QUOTIENT_EXACT_UNSCALE")
        .ok()
        .as_deref()
        == Some("1");
    if exact_unscale {
        mod_neg_inplace(b, quotient, p);
    } else {
        mod_neg_inplace_fast(b, quotient, p);
    }
    for _ in 0..inverse_iters {
        if exact_unscale {
            mod_halve_inplace(b, quotient, p);
        } else {
            mod_halve_inplace_fast(b, quotient, p);
        }
    }
    b.set_phase("d1_direct_quotient_restore_inverse");
}

pub(crate) fn d1_direct_quotient_arithmetic_from_prescaled_inverse(
    b: &mut B,
    numerator: &[QubitId],
    quotient: &[QubitId],
    inv_raw: &[QubitId],
    p: U256,
) {
    debug_assert_eq!(numerator.len(), N);
    debug_assert_eq!(quotient.len(), N);
    debug_assert_eq!(inv_raw.len(), N);

    b.set_phase("d1_direct_quotient_compute_neg_prescaled_quotient");
    let compute_mul = std::env::var("D1_DIRECT_QUOTIENT_COMPUTE_MUL").ok();
    let compute_mul = compute_mul.as_deref().or_else(|| {
        if d1_phase_corrected_arith_core_enabled() {
            Some("schoolbook")
        } else {
            None
        }
    });
    match compute_mul {
        Some("schoolbook") => {
            mod_mul_write_into_zero_acc_schoolbook(b, quotient, numerator, inv_raw, p)
        }
        Some("schoolbook_lowq") => {
            mod_mul_write_into_zero_acc_schoolbook_lowq(b, quotient, numerator, inv_raw, p)
        }
        Some("schoolbook_peak_lowq") => {
            mod_mul_write_into_zero_acc_schoolbook_peak_lowq(b, quotient, numerator, inv_raw, p)
        }
        Some("horner") => mod_mul_horner_add_qq(b, quotient, numerator, inv_raw, p),
        Some("karatsuba1") | Some("1") => {
            mod_mul_write_into_zero_acc_karatsuba(b, quotient, numerator, inv_raw, p)
        }
        Some("karatsuba_lowq") | Some("lowq") => {
            mod_mul_write_into_zero_acc_karatsuba_lowq(b, quotient, numerator, inv_raw, p)
        }
        _ => mod_mul_write_into_zero_acc_karatsuba2(b, quotient, numerator, inv_raw, p),
    }
    b.set_phase("d1_direct_quotient_unneg_prescaled_quotient");
    mod_neg_inplace_fast(b, quotient, p);
}
