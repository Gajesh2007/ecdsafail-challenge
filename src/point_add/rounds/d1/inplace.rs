//! `d1::inplace` — verbatim split of the original `d1` module.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

pub(crate) fn d1_inplace_product_lowerer_with_kaliski_clean_impl(
    b: &mut B,
    factor: &[QubitId],
    target: &[QubitId],
    p: U256,
    inverse_iters: usize,
    allow_hmr_phase_clean: bool,
) {
    debug_assert_eq!(factor.len(), N);
    debug_assert_eq!(target.len(), N);

    let displaced_target = b.alloc_qubits(N);
    b.set_phase("d1_product_compute_h_times_n");
    match std::env::var("D1_INPLACE_PRODUCT_MUL").ok().as_deref() {
        Some("walk") => mod_mul_add_qq(b, &displaced_target, factor, target, p),
        Some("schoolbook") => {
            mod_mul_write_into_zero_acc_schoolbook(b, &displaced_target, target, factor, p)
        }
        Some("schoolbook_peak_lowq") => mod_mul_write_into_zero_acc_schoolbook_peak_lowq(
            b,
            &displaced_target,
            target,
            factor,
            p,
        ),
        Some("schoolbook_lowq") => {
            mod_mul_write_into_zero_acc_schoolbook_lowq(b, &displaced_target, target, factor, p)
        }
        Some("karatsuba1") => {
            mod_mul_write_into_zero_acc_karatsuba(b, &displaced_target, target, factor, p)
        }
        Some("karatsuba_lowq") | Some("lowq") => {
            mod_mul_write_into_zero_acc_karatsuba_lowq(b, &displaced_target, target, factor, p)
        }
        _ => mod_mul_write_into_zero_acc_karatsuba2(b, &displaced_target, target, factor, p),
    }

    b.set_phase("d1_product_swap_product_into_target");
    for i in 0..N {
        b.swap(target[i], displaced_target[i]);
    }

    if std::env::var("D1_INPLACE_PRODUCT_CORE_ONLY")
        .ok()
        .as_deref()
        == Some("1")
    {
        if std::env::var("D1_INPLACE_PRODUCT_CORE_HMR_RESET_DISPLACED")
            .ok()
            .as_deref()
            == Some("1")
        {
            if std::env::var("D1_INPLACE_PRODUCT_CORE_FREE_DISPLACED_UNSAFE_PROBE")
                .ok()
                .as_deref()
                == Some("1")
            {
                b.set_phase("d1_product_core_free_displaced_target_unsafe_probe");
                b.free_vec(&displaced_target);
                return;
            }
            b.set_phase("d1_product_core_hmr_reset_displaced_target");
            if round499_zero_condition_hmr_erase_enabled() {
                let zero = b.alloc_bit();
                b.bit_store0(zero);
                for &q in &displaced_target {
                    b.hmr_if(q, zero, zero);
                }
            } else {
                let reset_mask = b.alloc_bits(N);
                for i in 0..N {
                    b.hmr(displaced_target[i], reset_mask[i]);
                }
            }
            b.set_phase("d1_product_core_hmr_free_displaced_target");
            b.free_vec(&displaced_target);
        } else {
            b.set_phase("d1_product_core_only_leave_displaced_target_live");
        }
        return;
    }

    let reset_displaced_after_swap = std::env::var("D1_INPLACE_RESET_DISPLACED_AFTER_SWAP")
        .ok()
        .as_deref()
        == Some("1");

    if allow_hmr_phase_clean
        && std::env::var("D1_INPLACE_HMR_PHASE_CLEAN").ok().as_deref() == Some("1")
    {
        let direct_quotient_phase_repair = d1_inplace_hmr_phase_clean_direct_quotient_enabled();
        let measured_uncompute_phase_repair =
            d1_inplace_hmr_phase_clean_measured_uncompute_enabled();

        assert!(
            !direct_quotient_phase_repair
                || std::env::var("D1_INPLACE_HMR_PHASE_CLEAN_DIRECT_QUOTIENT_UNSAFE_PROBE")
                    .ok()
                    .as_deref()
                    == Some("1"),
            "D1_INPLACE_HMR_PHASE_CLEAN_DIRECT_QUOTIENT=1 is rejected: Round504 probes show \
             the measurement-clean direct quotient repair is value-corrupt and far over budget. \
             Set D1_INPLACE_HMR_PHASE_CLEAN_DIRECT_QUOTIENT_UNSAFE_PROBE=1 only to reproduce \
             the rejected lane."
        );

        b.set_phase("d1_product_hmr_displaced_target");
        let reset_mask = b.alloc_bits(N);
        for i in 0..N {
            b.hmr(displaced_target[i], reset_mask[i]);
        }

        b.set_phase("d1_product_hmr_phase_copy_product_to_scratch");
        for i in 0..N {
            b.cx(target[i], displaced_target[i]);
        }

        if direct_quotient_phase_repair && measured_uncompute_phase_repair {
            panic!(
                "D1 direct-quotient HMR phase repair with measured uncompute is fail-closed: \
                 Round488 direct probes showed value/scratch corruption"
            );
        }

        if direct_quotient_phase_repair {
            b.set_phase("d1_product_hmr_phase_compute_old_target_direct_quotient");
            let repair_start = b.ops.len();
            d1_inplace_product_hmr_phase_compute_old_target(
                b,
                factor,
                &displaced_target,
                p,
                inverse_iters,
            );
            let repair_forward: Vec<_> = b.ops[repair_start..].to_vec();

            b.set_phase("d1_product_hmr_phase_apply_reset_mask");
            for i in 0..N {
                b.z_if(displaced_target[i], reset_mask[i]);
            }

            b.set_phase("d1_product_hmr_phase_uncompute_old_target_direct_quotient");
            emit_inverse_ops_measurement_clean_scoped(
                b,
                &repair_forward,
                "d1_product_hmr_direct_quotient_phase_repair",
            );
        } else {
            b.set_phase("d1_product_hmr_phase_compute_old_target");
            d1_inplace_product_hmr_phase_compute_old_target(
                b,
                factor,
                &displaced_target,
                p,
                inverse_iters,
            );

            b.set_phase("d1_product_hmr_phase_apply_reset_mask");
            for i in 0..N {
                b.z_if(displaced_target[i], reset_mask[i]);
            }

            b.set_phase("d1_product_hmr_phase_uncompute_old_target");
            if measured_uncompute_phase_repair {
                emit_inverse_measurement_clean_scoped(b, |b| {
                    d1_inplace_product_hmr_phase_compute_old_target(
                        b,
                        factor,
                        &displaced_target,
                        p,
                        inverse_iters,
                    )
                });
            } else {
                emit_inverse(b, |b| {
                    d1_inplace_product_hmr_phase_compute_old_target(
                        b,
                        factor,
                        &displaced_target,
                        p,
                        inverse_iters,
                    )
                });
            }
        }

        b.set_phase("d1_product_hmr_phase_uncopy_product_scratch");
        for i in (0..N).rev() {
            b.cx(target[i], displaced_target[i]);
        }

        b.set_phase("d1_product_hmr_phase_free_zero_displaced_target");
        b.free_vec(&displaced_target);
        return;
    }

    fn d1_cleanup_mul(
        b: &mut B,
        acc: &[QubitId],
        inv_raw: &[QubitId],
        target: &[QubitId],
        p: U256,
    ) {
        match std::env::var("D1_INPLACE_CLEANUP_KARATSUBA")
            .ok()
            .as_deref()
        {
            Some("walk") => mod_mul_add_qq(b, acc, inv_raw, target, p),
            Some("2") => mod_mul_add_into_acc_karatsuba2(b, acc, inv_raw, target, p),
            Some("1") => mod_mul_add_into_acc_karatsuba(b, acc, inv_raw, target, p),
            Some("lowq") => mod_mul_add_into_acc_karatsuba_lowq(b, acc, inv_raw, target, p),
            Some("schoolbook_peak_lowq") => {
                mod_mul_add_into_acc_schoolbook_peak_lowq(b, acc, inv_raw, target, p)
            }
            _ => mod_mul_add_into_acc_schoolbook(b, acc, inv_raw, target, p),
        }
    }

    fn d1_scale_displaced_target(
        b: &mut B,
        displaced_target: &[QubitId],
        factor: &[QubitId],
        target: &[QubitId],
        p: U256,
        iters: usize,
    ) {
        let mut dirty: Vec<QubitId> = factor.to_vec();
        dirty.extend_from_slice(target);
        for _ in 0..iters {
            mod_double_inplace_fast_with_dirty(b, displaced_target, p, Some(&dirty));
        }
    }

    if reset_displaced_after_swap {
        // After the swap, `displaced_target` holds the old target, not zero.
        // A raw reset is phase-dirty; the phase-clean replacement is the
        // quotient-based cleanup below.
        b.set_phase("d1_product_reset_displaced_target_after_swap_phase_clean_cleanup");
    } else {
        b.set_phase("d1_product_clean_displaced_target_with_raw_inverse");
    }
    if std::env::var("D1_INPLACE_OPT_COEFF_CLEAN").ok().as_deref() == Some("1") {
        b.set_phase("d1_product_clean_displaced_target_with_optimized_coeff");
        let coeff_r = b.alloc_qubits(N);
        let st = alloc_kaliski_state_borrowing_v(b, factor, inverse_iters);
        for i in 0..N {
            if bit(p, i) {
                b.x(st.u[i]);
            }
        }
        kaliski_forward_loaded_v(b, &st, p, inverse_iters, Some((&coeff_r, target)));
        b.set_phase("d1_product_scale_displaced_target_for_optimized_coeff");
        d1_scale_displaced_target(b, &displaced_target, factor, target, p, inverse_iters);
        b.set_phase("d1_product_zero_displaced_target_with_optimized_coeff");
        mod_add_qq_fast(b, &displaced_target, &coeff_r, p);
        b.set_phase("d1_product_restore_optimized_coeff_transducer");
        kaliski_backward_borrowing_v_with_coeff(b, &st, p, inverse_iters, (&coeff_r, target));
        free_kaliski_state_borrowed_v(b, st);
        b.free_vec(&coeff_r);
    } else if std::env::var("D1_INPLACE_BRANCH_COEFF_CLEAN")
        .ok()
        .as_deref()
        == Some("1")
    {
        b.set_phase("d1_product_clean_displaced_target_with_branch_coeff");
        let coeff_r = b.alloc_qubits(N);
        let st = alloc_kaliski_branch_state_no_add_borrowing_v(b, factor, inverse_iters);
        kaliski_branch_forward_with_coeff_borrowing_v(b, &st, p, inverse_iters, (&coeff_r, target));
        b.set_phase("d1_product_scale_displaced_target_for_branch_coeff");
        d1_scale_displaced_target(b, &displaced_target, factor, target, p, inverse_iters);
        b.set_phase("d1_product_zero_displaced_target_with_branch_coeff");
        mod_add_qq_fast(b, &displaced_target, &coeff_r, p);
        b.set_phase("d1_product_restore_branch_coeff_transducer");
        kaliski_branch_backward_with_coeff_borrowing_v(
            b,
            &st,
            p,
            inverse_iters,
            (&coeff_r, target),
        );
        free_kaliski_branch_state_borrowed_v(b, st);
        b.free_vec(&coeff_r);
    } else if std::env::var("D1_INPLACE_BRANCH_INV").ok().as_deref() == Some("1") {
        b.set_phase("d1_product_clean_displaced_target_with_branch_roll_inverse");
        with_kal_branch_inv_raw_roll(b, factor, p, inverse_iters, |b, inv_raw| {
            b.set_phase("d1_product_scale_displaced_target_for_branch_raw_inverse");
            d1_scale_displaced_target(b, &displaced_target, factor, target, p, inverse_iters);
            b.set_phase("d1_product_zero_displaced_target");
            d1_cleanup_mul(b, &displaced_target, inv_raw, target, p);
        });
    } else if std::env::var("D1_INPLACE_PRESCALED_KAL").ok().as_deref() == Some("1") {
        b.set_phase("d1_product_clean_displaced_target_with_prescaled_raw_inverse");
        if std::env::var("D1_INPLACE_PRESCALED_CHUNKED")
            .ok()
            .as_deref()
            == Some("0")
        {
            with_kal_inv_raw_prescaled_mixed(b, factor, p, inverse_iters, |b, inv_raw| {
                b.set_phase("d1_product_zero_displaced_target");
                d1_cleanup_mul(b, &displaced_target, inv_raw, target, p);
            });
        } else {
            with_kal_inv_raw_prescaled_chunked(b, factor, p, inverse_iters, |b, inv_raw| {
                b.set_phase("d1_product_zero_displaced_target");
                d1_cleanup_mul(b, &displaced_target, inv_raw, target, p);
            });
        }
    } else {
        b.set_phase("d1_product_clean_displaced_target_with_borrowed_factor_inverse");
        with_kal_inv_raw_borrowing_v(b, factor, p, inverse_iters, |b, inv_raw| {
            b.set_phase("d1_product_scale_displaced_target_for_raw_inverse");
            d1_scale_displaced_target(b, &displaced_target, factor, target, p, inverse_iters);
            b.set_phase("d1_product_zero_displaced_target");
            d1_cleanup_mul(b, &displaced_target, inv_raw, target, p);
        });
    }

    b.set_phase("d1_product_free_zero_displaced_target");
    b.free_vec(&displaced_target);
}

pub(crate) fn d1_inplace_hmr_phase_clean_measured_uncompute_enabled() -> bool {
    std::env::var("D1_INPLACE_HMR_PHASE_CLEAN_MEASURED_UNCOMPUTE")
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn d1_inplace_product_hmr_phase_compute_old_target(
    b: &mut B,
    factor: &[QubitId],
    target: &[QubitId],
    p: U256,
    inverse_iters: usize,
) {
    let direct_quotient_enabled = std::env::var("D1_DIRECT_QUOTIENT").ok().as_deref() == Some("1");
    if d1_inplace_hmr_phase_clean_direct_quotient_enabled() && direct_quotient_enabled {
        d1_inplace_quotient_lowerer_with_kaliski_clean(b, factor, target, p, inverse_iters);
    } else {
        d1_inplace_quotient_lowerer_baseline(b, factor, target, p, inverse_iters);
    }
}

pub(crate) fn d1_inplace_product_lowerer_with_kaliski_clean(
    b: &mut B,
    factor: &[QubitId],
    target: &[QubitId],
    p: U256,
    inverse_iters: usize,
) {
    with_d1_phase_corrected_product_core(d1_phase_corrected_arith_core_enabled(), || {
        d1_inplace_product_lowerer_with_kaliski_clean_impl(
            b,
            factor,
            target,
            p,
            inverse_iters,
            true,
        )
    });
}

pub(crate) fn d1_inplace_quotient_lowerer_with_kaliski_clean(
    b: &mut B,
    factor: &[QubitId],
    target: &[QubitId],
    p: U256,
    inverse_iters: usize,
) {
    debug_assert_eq!(factor.len(), N);
    debug_assert_eq!(target.len(), N);

    if d1_phase_corrected_arith_core_enabled()
        || std::env::var("D1_DIRECT_QUOTIENT").ok().as_deref() == Some("1")
    {
        b.set_phase("d1_direct_quotient_alloc");
        let displaced_target = b.alloc_qubits(N);

        d1_direct_quotient_compute_into_zero(
            b,
            factor,
            target,
            &displaced_target,
            p,
            inverse_iters,
        );

        b.set_phase("d1_direct_quotient_swap_into_target");
        for i in 0..N {
            b.swap(target[i], displaced_target[i]);
        }

        b.set_phase("d1_direct_quotient_clean_displaced_target");
        let cleanup_mul = std::env::var("D1_DIRECT_QUOTIENT_CLEANUP_MUL").ok();
        let cleanup_mul = cleanup_mul.as_deref().or_else(|| {
            if d1_phase_corrected_arith_core_enabled() {
                Some("1")
            } else {
                None
            }
        });
        match cleanup_mul {
            Some("walk") => mod_mul_sub_qq(b, &displaced_target, factor, target, p),
            Some("karatsuba1") | Some("1") => {
                mod_mul_sub_into_acc_karatsuba(b, &displaced_target, factor, target, p)
            }
            Some("schoolbook_peak_lowq") => {
                mod_mul_sub_into_acc_schoolbook_peak_lowq(b, &displaced_target, factor, target, p)
            }
            _ => mod_mul_sub_into_acc_schoolbook(b, &displaced_target, factor, target, p),
        }
        b.set_phase("d1_direct_quotient_free_displaced_target");
        b.free_vec(&displaced_target);
        return;
    }

    b.set_phase("d1_quotient_inverse_product_transducer");
    emit_inverse_measurement_clean_scoped(b, |b| {
        d1_inplace_product_lowerer_with_kaliski_clean_impl(
            b,
            factor,
            target,
            p,
            inverse_iters,
            true,
        );
    });
}

pub(crate) fn d1_inplace_quotient_lowerer_baseline(
    b: &mut B,
    factor: &[QubitId],
    target: &[QubitId],
    p: U256,
    inverse_iters: usize,
) {
    emit_inverse_measurement_clean_scoped(b, |b| {
        d1_inplace_product_lowerer_with_kaliski_clean_impl(
            b,
            factor,
            target,
            p,
            inverse_iters,
            false,
        );
    });
}

pub fn build_d1_inplace_product_lowerer_component() -> Vec<Op> {
    let mut b = B::new();
    let h = b.alloc_qubits(N);
    b.declare_qubit_register(&h);
    let n = b.alloc_qubits(N);
    b.declare_qubit_register(&n);
    let inverse_iters = std::env::var("D1_INPLACE_KAL_ITERS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .map(|iters| {
            checked_kaliski_iters(
                "D1 product lowerer",
                "D1_INPLACE_KAL_ITERS",
                iters,
                D1_INPLACE_MIN_SAFE_ITERS,
            )
        })
        .unwrap_or(D1_INPLACE_MIN_SAFE_ITERS);
    d1_inplace_product_lowerer_with_kaliski_clean(&mut b, &h, &n, SECP256K1_P, inverse_iters);
    b.ops
}

pub fn build_d1_inplace_quotient_lowerer_component() -> Vec<Op> {
    let mut b = B::new();
    let h = b.alloc_qubits(N);
    b.declare_qubit_register(&h);
    let n = b.alloc_qubits(N);
    b.declare_qubit_register(&n);
    let inverse_iters = std::env::var("D1_INPLACE_KAL_ITERS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .map(|iters| {
            checked_kaliski_iters(
                "D1 quotient lowerer",
                "D1_INPLACE_KAL_ITERS",
                iters,
                D1_INPLACE_MIN_SAFE_ITERS,
            )
        })
        .unwrap_or(D1_INPLACE_MIN_SAFE_ITERS);
    d1_inplace_quotient_lowerer_with_kaliski_clean(&mut b, &h, &n, SECP256K1_P, inverse_iters);
    b.ops
}

pub fn build_d1_inplace_product_lowerer_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let mut b = B::new();
    let h = b.alloc_qubits(N);
    b.declare_qubit_register(&h);
    let n = b.alloc_qubits(N);
    b.declare_qubit_register(&n);
    let inverse_iters = std::env::var("D1_INPLACE_KAL_ITERS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .map(|iters| {
            checked_kaliski_iters(
                "D1 product lowerer resources",
                "D1_INPLACE_KAL_ITERS",
                iters,
                D1_INPLACE_MIN_SAFE_ITERS,
            )
        })
        .unwrap_or(D1_INPLACE_MIN_SAFE_ITERS);
    d1_inplace_product_lowerer_with_kaliski_clean(&mut b, &h, &n, SECP256K1_P, inverse_iters);
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub fn build_d1_inplace_quotient_lowerer_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let mut b = B::new();
    let h = b.alloc_qubits(N);
    b.declare_qubit_register(&h);
    let n = b.alloc_qubits(N);
    b.declare_qubit_register(&n);
    let inverse_iters = std::env::var("D1_INPLACE_KAL_ITERS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .map(|iters| {
            checked_kaliski_iters(
                "D1 quotient lowerer resources",
                "D1_INPLACE_KAL_ITERS",
                iters,
                D1_INPLACE_MIN_SAFE_ITERS,
            )
        })
        .unwrap_or(D1_INPLACE_MIN_SAFE_ITERS);
    d1_inplace_quotient_lowerer_with_kaliski_clean(&mut b, &h, &n, SECP256K1_P, inverse_iters);
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}
