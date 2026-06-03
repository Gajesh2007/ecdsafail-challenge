//! Source-live cubic product-tail emission helpers.
//!
//! Shared cubic HMR-repair, x/y-tail, and clean-product-tail emitters used by
//! the source-live point-add experiments.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;


pub(crate) fn emit_source_live_cubic_hmr_repair_from_inverse(
    b: &mut B,
    ty: &[QubitId],
    lam: &[QubitId],
    oy: &[BitId],
    inv_raw: &[QubitId],
    lam_masks: &[BitId],
    p: U256,
    inverse_iters: usize,
    prescaled_inverse: bool,
) {
    let repair_start = b.ops.len();
    b.set_phase("source_live_cubic_hmr_repair_add_qy_to_yprod");
    mod_add_qb_phase_clean(b, ty, oy, p);
    b.set_phase("source_live_cubic_hmr_recompute_old_lam");
    if prescaled_inverse {
        d1_direct_quotient_arithmetic_from_prescaled_inverse(b, ty, lam, inv_raw, p);
    } else {
        d1_direct_quotient_arithmetic_from_raw_inverse(b, ty, lam, inv_raw, p, inverse_iters);
    }
    let repair_forward: Vec<_> = b.ops[repair_start..].to_vec();

    b.set_phase("source_live_cubic_hmr_apply_lam_mask");
    for i in 0..N {
        b.z_if(lam[i], lam_masks[i]);
    }

    b.set_phase("source_live_cubic_hmr_uncompute_old_lam");
    emit_inverse_ops_measurement_clean_scoped(
        b,
        &repair_forward,
        "source_live_cubic_hmr_phase_repair",
    );
}

pub(crate) fn emit_source_live_cubic_xtail_ytail(
    b: &mut B,
    tx: &[QubitId],
    ty: &[QubitId],
    lam: &[QubitId],
    ox: &[BitId],
    oy: &[BitId],
    p: U256,
    clean_lam: SourceLiveCubicLamClean,
) {
    debug_assert_eq!(tx.len(), N);
    debug_assert_eq!(ty.len(), N);
    debug_assert_eq!(lam.len(), N);
    debug_assert_eq!(ox.len(), N);
    debug_assert_eq!(oy.len(), N);

    // Entry convention: tx = dx, ty = dy, lam = -lambda. Keeping dy live
    // gives the curve-supported identity
    //   Ry = dy + lam^3 - 3*Qx*lam - Qy.
    // The clean mode then uses the second affine identity
    //   Ry + Qy = lam * (Rx - Qx)
    // to zero the separate slope word with only the pair2 inverse/multiply
    // cleanup. It deliberately does not route through the D1 in-place product
    // lowerer, whose displaced old target is the source of the dirty product
    // shortcut.
    let lam_sq = b.alloc_qubits(N);
    b.set_phase("source_live_cubic_compute_lam_sq");
    squaring_add_to_acc_schoolbook_phase_clean(b, &lam_sq, lam, p);

    b.set_phase("source_live_cubic_xtail_sub_lam_sq");
    mod_sub_qq(b, tx, &lam_sq, p);
    b.set_phase("source_live_cubic_xtail_add_3qx");
    mod_add_double_qb_phase_clean(b, tx, ox, p);
    mod_add_qb_phase_clean(b, tx, ox, p);
    b.set_phase("source_live_cubic_xtail_neg_to_rx_minus_qx");
    mod_neg_inplace(b, tx, p);

    b.set_phase("source_live_cubic_ytail_add_lam_cube");
    if std::env::var("SOURCE_LIVE_CUBIC_LAMCUBE_MUL").is_ok() {
        mod_mul_add_into_acc_selected(b, ty, &lam_sq, lam, p, "SOURCE_LIVE_CUBIC_LAMCUBE_MUL");
    } else {
        mod_mul_add_into_acc_schoolbook_phase_clean(b, ty, &lam_sq, lam, p);
    }

    let three_lam = b.alloc_qubits(N);
    b.set_phase("source_live_cubic_build_three_lam");
    for i in 0..N {
        b.cx(lam[i], three_lam[i]);
    }
    mod_double_inplace(b, &three_lam, p);
    mod_add_qq(b, &three_lam, lam, p);

    b.set_phase("source_live_cubic_ytail_sub_3qx_lam");
    if std::env::var("SOURCE_LIVE_CUBIC_QX_CLASSICAL_MUL")
        .ok()
        .as_deref()
        == Some("1")
    {
        mod_mul_sub_qb(b, ty, &three_lam, ox, p);
    } else {
        let qx = load_bits(b, ox);
        match std::env::var("SOURCE_LIVE_CUBIC_QX_MUL").ok().as_deref() {
            Some("karatsuba1") | Some("1") => {
                mod_mul_sub_into_acc_karatsuba(b, ty, &three_lam, &qx, p)
            }
            Some("schoolbook_peak_lowq") => {
                mod_mul_sub_into_acc_schoolbook_peak_lowq(b, ty, &three_lam, &qx, p)
            }
            Some("schoolbook") => mod_mul_sub_into_acc_schoolbook(b, ty, &three_lam, &qx, p),
            Some(other) => panic!(
                "unsupported SOURCE_LIVE_CUBIC_QX_MUL={other}; expected schoolbook, schoolbook_peak_lowq, or karatsuba1"
            ),
            None => mod_mul_sub_into_acc_schoolbook_phase_clean(b, ty, &three_lam, &qx, p),
        }
        unload_bits(b, &qx, ox);
    }

    b.set_phase("source_live_cubic_uncompute_three_lam");
    mod_sub_qq(b, &three_lam, lam, p);
    mod_halve_inplace(b, &three_lam, p);
    for i in 0..N {
        b.cx(lam[i], three_lam[i]);
    }
    b.free_vec(&three_lam);

    b.set_phase("source_live_cubic_ytail_sub_qy");
    mod_sub_qb_phase_clean(b, ty, oy, p);

    b.set_phase("source_live_cubic_uncompute_lam_sq");
    squaring_sub_from_acc_schoolbook_phase_clean(b, &lam_sq, lam, p);
    b.free_vec(&lam_sq);

    match clean_lam {
        SourceLiveCubicLamClean::Dirty => {}
        SourceLiveCubicLamClean::HmrPhaseRepair { inverse_iters } => {
            b.set_phase("source_live_cubic_hmr_measure_old_lam");
            let lam_masks = b.alloc_bits(N);
            for i in 0..N {
                b.hmr(lam[i], lam_masks[i]);
            }

            b.set_phase("source_live_cubic_hmr_repair_inverse");
            let prescaled_inverse = std::env::var("PA_SOURCE_LIVE_CUBIC_HMR_PRESCALED_KAL")
                .ok()
                .as_deref()
                == Some("1");
            if prescaled_inverse {
                let chunked = std::env::var("PA_SOURCE_LIVE_CUBIC_HMR_PRESCALED_CHUNKED")
                    .ok()
                    .as_deref()
                    != Some("0");
                if chunked {
                    with_kal_inv_raw_prescaled_chunked(b, tx, p, inverse_iters, |b, inv_raw| {
                        emit_source_live_cubic_hmr_repair_from_inverse(
                            b,
                            ty,
                            lam,
                            oy,
                            inv_raw,
                            &lam_masks,
                            p,
                            inverse_iters,
                            true,
                        );
                    });
                } else {
                    with_kal_inv_raw_prescaled_mixed(b, tx, p, inverse_iters, |b, inv_raw| {
                        emit_source_live_cubic_hmr_repair_from_inverse(
                            b,
                            ty,
                            lam,
                            oy,
                            inv_raw,
                            &lam_masks,
                            p,
                            inverse_iters,
                            true,
                        );
                    });
                }
            } else {
                with_kal_inv_raw_borrowing_v(b, tx, p, inverse_iters, |b, inv_raw| {
                    emit_source_live_cubic_hmr_repair_from_inverse(
                        b,
                        ty,
                        lam,
                        oy,
                        inv_raw,
                        &lam_masks,
                        p,
                        inverse_iters,
                        false,
                    );
                });
            }

            b.set_phase("source_live_cubic_hmr_free_zero_lam");
            b.free_vec(lam);
        }
        SourceLiveCubicLamClean::Inverse { inverse_iters } => {
            b.set_phase("source_live_cubic_lambda_clean_add_qy_to_yprod");
            mod_add_qb_phase_clean(b, ty, oy, p);

            b.set_phase("source_live_cubic_lambda_clean_inverse");
            with_kal_inv_raw(b, tx, p, inverse_iters, |b, inv_raw| {
                b.set_phase("source_live_cubic_lambda_clean_double");
                for _ in 0..inverse_iters {
                    mod_double_inplace_fast(b, lam, p);
                }
                b.set_phase("source_live_cubic_lambda_clean_mul");
                mod_mul_add_into_acc_selected(
                    b,
                    lam,
                    inv_raw,
                    ty,
                    p,
                    "SOURCE_LIVE_CUBIC_CLEAN_LAM_MUL",
                );
                b.set_phase("source_live_cubic_lambda_clean_restore_ry");
                mod_sub_qb_phase_clean(b, ty, oy, p);
                b.set_phase("source_live_cubic_lambda_clean_backward");
            });

            b.set_phase("source_live_cubic_lambda_clean_free_lam");
            b.free_vec(lam);
        }
        SourceLiveCubicLamClean::Product { inverse_iters } => {
            b.set_phase("source_live_cubic_product_clean_lam_times_rx_minus_qx");
            d1_inplace_product_lowerer_with_kaliski_clean(b, tx, lam, p, inverse_iters);
            b.set_phase("source_live_cubic_product_clean_sub_ry");
            mod_sub_qq(b, lam, ty, p);
            b.set_phase("source_live_cubic_product_clean_sub_qy");
            mod_sub_qb_phase_clean(b, lam, oy, p);
            b.set_phase("source_live_cubic_product_clean_free_lam");
            b.free_vec(lam);
        }
    }

    b.set_phase("source_live_cubic_finalize_rx");
    mod_add_qb_phase_clean(b, tx, ox, p);
}

pub(crate) fn emit_source_live_clean_product_tail(
    b: &mut B,
    tx: &[QubitId],
    ty: &[QubitId],
    lam: &[QubitId],
    ox: &[BitId],
    oy: &[BitId],
    p: U256,
    inverse_iters: usize,
) {
    debug_assert_eq!(tx.len(), N);
    debug_assert_eq!(ty.len(), N);
    debug_assert_eq!(lam.len(), N);
    debug_assert_eq!(ox.len(), N);
    debug_assert_eq!(oy.len(), N);

    // Entry convention: tx = dx, ty = dy, lam = -lambda.  First consume the
    // source-live invariant dy + lam*dx = 0, then use lam itself as the product
    // target: lam <- lam*(Rx-Qx) = Ry+Qy.  Swapping that word into ty leaves
    // the separate slope register clean without the dirty cubic reset.
    b.set_phase("source_live_product_tail_zero_dy");
    mod_mul_add_into_acc_selected(b, ty, lam, tx, p, "SOURCE_LIVE_ZERO_DY_MUL");
    b.free_vec(ty);

    if std::env::var("PA_SOURCE_LIVE_DIRECT_XTAIL_SQUARE")
        .ok()
        .as_deref()
        == Some("1")
    {
        b.set_phase("source_live_product_tail_direct_xtail_square");
        mod_mul_sub_qq(b, tx, lam, lam, p);
    } else {
        let lam_sq = b.alloc_qubits(N);
        b.set_phase("source_live_product_tail_compute_lam_sq");
        squaring_add_to_acc_schoolbook_phase_clean(b, &lam_sq, lam, p);

        b.set_phase("source_live_product_tail_xtail_sub_lam_sq");
        mod_sub_qq(b, tx, &lam_sq, p);
        b.set_phase("source_live_product_tail_uncompute_lam_sq");
        squaring_sub_from_acc_schoolbook_phase_clean(b, &lam_sq, lam, p);
        b.free_vec(&lam_sq);
    }
    b.set_phase("source_live_product_tail_xtail_add_3qx");
    mod_add_double_qb_phase_clean(b, tx, ox, p);
    mod_add_qb_phase_clean(b, tx, ox, p);
    b.set_phase("source_live_product_tail_xtail_neg_to_rx_minus_qx");
    mod_neg_inplace(b, tx, p);

    if std::env::var("PA_SOURCE_LIVE_PRODUCT_HMR_OVERWRITE")
        .ok()
        .as_deref()
        == Some("1")
    {
        let quotient_phase_repair =
            std::env::var("PA_SOURCE_LIVE_PRODUCT_HMR_QUOTIENT_PHASE_REPAIR")
                .ok()
                .as_deref()
                == Some("1");
        let direct_quotient_phase_repair =
            std::env::var("PA_SOURCE_LIVE_PRODUCT_HMR_DIRECT_QUOTIENT_PHASE_REPAIR")
                .ok()
                .as_deref()
                == Some("1");
        let single_inverse_phase_repair =
            std::env::var("PA_SOURCE_LIVE_PRODUCT_HMR_SINGLE_INVERSE_PHASE_REPAIR")
                .ok()
                .as_deref()
                == Some("1");
        assert!(
            !(quotient_phase_repair || direct_quotient_phase_repair || single_inverse_phase_repair)
                || std::env::var("PA_SOURCE_LIVE_PRODUCT_HMR_REPAIR_UNSAFE_PROBE")
                    .ok()
                    .as_deref()
                    == Some("1"),
            "source-live product HMR phase-repair flags are rejected: Round508 basis-fuzz probes \
             show quotient, direct-quotient, and single-inverse repairs are value-corrupt and over \
             budget. Set PA_SOURCE_LIVE_PRODUCT_HMR_REPAIR_UNSAFE_PROBE=1 only to reproduce the \
             rejected lanes."
        );
        b.set_phase("source_live_product_tail_hmr_reacquire_zero_ty");
        b.reacquire_vec(ty);
        b.set_phase("source_live_product_tail_hmr_product_into_ty");
        mod_mul_write_into_zero_acc_selected(
            b,
            ty,
            lam,
            tx,
            p,
            "SOURCE_LIVE_PRODUCT_HMR_WRITE_MUL",
        );

        b.set_phase("source_live_product_tail_hmr_measure_old_lam");
        let lam_masks = b.alloc_bits(N);
        for i in 0..N {
            b.hmr(lam[i], lam_masks[i]);
        }

        if single_inverse_phase_repair {
            b.set_phase("source_live_product_tail_hmr_single_inv_phase_repair");
            with_kal_inv_raw_borrowing_v(b, tx, p, inverse_iters, |b, inv_raw| {
                let repair_start = b.ops.len();
                d1_direct_quotient_arithmetic_from_raw_inverse(
                    b,
                    ty,
                    lam,
                    inv_raw,
                    p,
                    inverse_iters,
                );
                let repair_forward: Vec<_> = b.ops[repair_start..].to_vec();

                b.set_phase("source_live_product_tail_hmr_single_inv_apply_lam_mask");
                for i in 0..N {
                    b.z_if(lam[i], lam_masks[i]);
                }

                b.set_phase("source_live_product_tail_hmr_single_inv_uncompute_old_lam");
                if source_live_product_hmr_keep_measured_conditions_enabled() {
                    emit_inverse_ops_hmr_safe_keep_conditions(
                        b,
                        &repair_forward,
                        "source_live_product_tail_hmr_single_inverse_phase_repair",
                    );
                } else {
                    emit_inverse_ops_measurement_clean_scoped(
                        b,
                        &repair_forward,
                        "source_live_product_tail_hmr_single_inverse_phase_repair",
                    );
                }
            });
        } else if direct_quotient_phase_repair {
            b.set_phase("source_live_product_tail_hmr_direct_phase_compute_old_lam");
            let direct_start = b.ops.len();
            d1_direct_quotient_compute_into_zero(b, tx, ty, lam, p, inverse_iters);
            let direct_forward: Vec<_> = b.ops[direct_start..].to_vec();

            b.set_phase("source_live_product_tail_hmr_direct_phase_apply_lam_mask");
            for i in 0..N {
                b.z_if(lam[i], lam_masks[i]);
            }

            if source_live_product_hmr_direct_forward_clean_enabled() {
                b.set_phase("source_live_product_tail_hmr_direct_phase_product_clean_lam");
                d1_inplace_product_lowerer_with_kaliski_clean(b, tx, lam, p, inverse_iters);
                b.set_phase("source_live_product_tail_hmr_direct_phase_sub_yprod");
                mod_sub_qq(b, lam, ty, p);
            } else if source_live_product_hmr_keep_measured_conditions_enabled() {
                b.set_phase("source_live_product_tail_hmr_direct_phase_uncompute_old_lam");
                emit_inverse_ops_hmr_safe_keep_conditions(
                    b,
                    &direct_forward,
                    "source_live_product_tail_hmr_direct_quotient_phase_repair",
                );
            } else {
                b.set_phase("source_live_product_tail_hmr_direct_phase_uncompute_old_lam");
                emit_inverse_ops_measurement_clean_scoped(
                    b,
                    &direct_forward,
                    "source_live_product_tail_hmr_direct_quotient_phase_repair",
                );
            }
        } else if quotient_phase_repair {
            b.set_phase("source_live_product_tail_hmr_phase_copy_yprod_to_lam");
            for i in 0..N {
                b.cx(ty[i], lam[i]);
            }

            b.set_phase("source_live_product_tail_hmr_phase_recompute_old_lam");
            d1_inplace_product_hmr_phase_compute_old_target(b, tx, lam, p, inverse_iters);

            b.set_phase("source_live_product_tail_hmr_phase_apply_lam_mask");
            for i in 0..N {
                b.z_if(lam[i], lam_masks[i]);
            }

            b.set_phase("source_live_product_tail_hmr_phase_uncompute_old_lam");
            d1_inplace_product_lowerer_with_kaliski_clean(b, tx, lam, p, inverse_iters);

            b.set_phase("source_live_product_tail_hmr_phase_uncopy_yprod_from_lam");
            for i in (0..N).rev() {
                b.cx(ty[i], lam[i]);
            }
        }
        b.free_vec(lam);

        b.set_phase("source_live_product_tail_sub_qy");
        mod_sub_qb_phase_clean(b, ty, oy, p);

        b.set_phase("source_live_product_tail_finalize_rx");
        mod_add_qb_phase_clean(b, tx, ox, p);
        return;
    }

    if std::env::var("PA_SOURCE_LIVE_PRODUCT_CENTERED_QUOTIENT_CLEAN")
        .ok()
        .as_deref()
        == Some("1")
    {
        b.set_phase("source_live_product_tail_centered_reacquire_zero_ty");
        b.reacquire_vec(ty);
        b.set_phase("source_live_product_tail_centered_product_into_ty");
        mod_mul_write_into_zero_acc_selected(
            b,
            ty,
            lam,
            tx,
            p,
            "SOURCE_LIVE_PRODUCT_CENTERED_WRITE_MUL",
        );

        b.set_phase("source_live_product_tail_centered_clean_lam");
        add_neg_quotient_into_acc_with_centered_by_bench(b, lam, tx, ty, p);
        b.free_vec(lam);

        b.set_phase("source_live_product_tail_sub_qy");
        mod_sub_qb_phase_clean(b, ty, oy, p);

        b.set_phase("source_live_product_tail_finalize_rx");
        mod_add_qb_phase_clean(b, tx, ox, p);
        return;
    }

    b.set_phase("source_live_product_tail_inplace_lam_times_rx_minus_qx");
    d1_inplace_product_lowerer_with_kaliski_clean(b, tx, lam, p, inverse_iters);

    b.set_phase("source_live_product_tail_sub_qy");
    mod_sub_qb_phase_clean(b, lam, oy, p);

    b.set_phase("source_live_product_tail_reacquire_zero_ty");
    b.reacquire_vec(ty);
    b.set_phase("source_live_product_tail_swap_y_into_ty");
    for i in 0..N {
        b.swap(ty[i], lam[i]);
    }
    b.set_phase("source_live_product_tail_free_zero_lam");
    b.free_vec(lam);

    b.set_phase("source_live_product_tail_finalize_rx");
    mod_add_qb_phase_clean(b, tx, ox, p);
}
