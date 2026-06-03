//! `low::round008_190` — verbatim split of the original `low` module.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

pub(crate) fn round8_qtail_output_side_cleanup_enabled() -> bool {
    std::env::var("ROUND8_QTAIL_OUTPUT_SIDE_CLEANUP")
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn round8_qtail_round217_product_reuse_enabled() -> bool {
    std::env::var("ROUND8_QTAIL_ROUND217_PRODUCT_REUSE")
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn round8_qtail_pair2_product_core_only_enabled() -> bool {
    std::env::var("ROUND8_QTAIL_PAIR2_PRODUCT_CORE_ONLY")
        .ok()
        .as_deref()
        == Some("1")
        || round8_qtail_pair2_product_core_hmr_reset_enabled()
}

pub(crate) fn round8_qtail_pair2_product_core_hmr_reset_enabled() -> bool {
    std::env::var("ROUND8_QTAIL_PAIR2_PRODUCT_CORE_HMR_RESET")
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn round8_qtail_round217_product_reuse_forbidden_full_source_enabled() -> bool {
    std::env::var("ROUND8_QTAIL_ROUND217_PRODUCT_REUSE_ALLOW_FORBIDDEN_FULL_SOURCE")
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn round8_qtail_output_side_regular_phase_repair_enabled() -> bool {
    std::env::var("ROUND8_QTAIL_OUTPUT_SIDE_REGULAR_PHASE_REPAIR")
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn round8_qtail_pair2_iters_for_output_side_regular_probe() -> usize {
    std::env::var("KAL_PAIR2_ITERS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .map(|iters| {
            checked_kaliski_iters(
                "round8 qtail output-side regular phase repair",
                "KAL_PAIR2_ITERS",
                iters,
                ROUND8_QTAIL_PAIR2_MIN_SAFE_ITERS,
            )
        })
        .unwrap_or(ROUND8_QTAIL_PAIR2_MIN_SAFE_ITERS)
}

pub(crate) fn round8_emit_output_side_regular_phase_repair_probe(
    b: &mut B,
    tx: &[QubitId],
    ty: &[QubitId],
    p: U256,
) {
    debug_assert_eq!(tx.len(), N);
    debug_assert_eq!(ty.len(), N);

    let inverse_iters = round8_qtail_pair2_iters_for_output_side_regular_probe();
    let yprod = b.alloc_qubits(N);
    b.set_phase("round8_output_side_regular_compute_yprod");
    mod_mul_write_into_zero_acc_karatsuba2(b, &yprod, ty, tx, p);

    b.set_phase("round8_output_side_regular_hmr_old_lambda");
    let lam_masks = b.alloc_bits(N);
    for i in 0..N {
        b.hmr(ty[i], lam_masks[i]);
    }
    b.free_vec(ty);

    b.set_phase("round8_output_side_regular_reacquire_ty");
    b.reacquire_vec(ty);
    b.set_phase("round8_output_side_regular_swap_yprod_into_ty");
    for i in 0..N {
        b.swap(ty[i], yprod[i]);
    }
    b.free_vec(&yprod);

    let lam_repair = b.alloc_qubits(N);
    with_kal_inv_raw_borrowing_v(b, tx, p, inverse_iters, |b, inv_raw| {
        b.set_phase("round8_output_side_regular_compute_lambda_repair");
        let repair_start = b.ops.len();
        b.set_phase("round8_output_side_regular_repair_scaled_lambda");
        mod_mul_add_into_acc_selected(
            b,
            &lam_repair,
            inv_raw,
            ty,
            p,
            "ROUND8_OUTPUT_SIDE_PHASE_REPAIR_MUL",
        );
        b.set_phase("round8_output_side_regular_repair_unscale_lambda");
        mod_neg_inplace_fast(b, &lam_repair, p);
        for _ in 0..inverse_iters {
            mod_halve_inplace_fast(b, &lam_repair, p);
        }
        let repair_forward: Vec<_> = b.ops[repair_start..].to_vec();

        b.set_phase("round8_output_side_regular_apply_lambda_phase");
        for i in 0..N {
            b.z_if(lam_repair[i], lam_masks[i]);
        }

        b.set_phase("round8_output_side_regular_uncompute_lambda_repair");
        emit_inverse_ops_measurement_clean_scoped(
            b,
            &repair_forward,
            "round8_output_side_regular_phase_repair",
        );
    });
    b.set_phase("round8_output_side_regular_free_zero_lambda_repair");
    b.free_vec(&lam_repair);
}

pub(crate) fn round8_emit_low_q_output_side_second_inverse_qtail_pa(
    b: &mut B,
    tx: &[QubitId],
    ty: &[QubitId],
    _ox: &[BitId],
    _oy: &[BitId],
    p: U256,
    inverse_iters: usize,
) {
    debug_assert_eq!(tx.len(), N);
    debug_assert_eq!(ty.len(), N);
    assert_eq!(
        p, SECP256K1_P,
        "low-Q output-side qtail cleanup is secp256k1-only"
    );

    let yprod = b.alloc_qubits(N);
    b.set_phase("low_q_output_side_compute_yprod");
    mod_mul_write_into_zero_acc_karatsuba2(b, &yprod, ty, tx, p);

    let zero_c = b.alloc_qubit();
    b.set_phase("low_q_output_side_patch_zero_output_denominator");
    cmp_eq_zero_into(b, tx, zero_c);
    cadd_nbit_const(b, tx, U256::from(1u64), zero_c);

    let lambda_out = b.alloc_qubits(N);
    b.set_phase("low_q_output_side_compute_lambda_out");
    let lambda_start = b.ops.len();
    d1_direct_quotient_compute_into_zero(b, tx, &yprod, &lambda_out, p, inverse_iters);
    let lambda_forward: Vec<_> = b.ops[lambda_start..].to_vec();

    b.set_phase("low_q_output_side_clean_dirty_lambda");
    mod_sub_qq(b, ty, &lambda_out, p);

    b.set_phase("low_q_output_side_uncompute_lambda_out");
    emit_inverse_ops_measurement_clean_scoped(
        b,
        &lambda_forward,
        "low_q_output_side_second_inverse_lambda_out",
    );
    b.free_vec(&lambda_out);

    b.set_phase("low_q_output_side_unpatch_zero_output_denominator");
    csub_nbit_const(b, tx, U256::from(1u64), zero_c);
    cmp_eq_zero_into(b, tx, zero_c);
    b.free(zero_c);

    b.set_phase("low_q_output_side_swap_yprod_into_ty");
    for i in 0..N {
        b.swap(ty[i], yprod[i]);
    }
    b.free_vec(&yprod);
}

pub(crate) fn round8_emit_output_side_cleanup_or_fail(
    b: &mut B,
    tx: &[QubitId],
    ty: &[QubitId],
    _ox: &[BitId],
    _oy: &[BitId],
    p: U256,
) {
    if round8_qtail_output_side_regular_phase_repair_enabled() {
        round8_emit_output_side_regular_phase_repair_probe(b, tx, ty, p);
        return;
    }
    b.set_phase("round8_fallback_output_side_cleanup_fail_closed");
    panic!(
        "ROUND8_QTAIL_OUTPUT_SIDE_CLEANUP=1 is fail-closed until the regular c=Rx-Qx inverse/lambda/dx/inv(dx) cleanup emitter, Round368 singular R=-Q guard, zero-scratch/zero-phase proof, and 9024 Google fuzz gate are implemented. Set ROUND8_QTAIL_OUTPUT_SIDE_REGULAR_PHASE_REPAIR=1 only for the regular-branch stats probe."
    );
}

pub(crate) fn with_round8_qtail_pair2_product_core_scope<F: FnOnce(&mut B)>(b: &mut B, f: F) {
    if !round8_qtail_pair2_product_core_only_enabled() {
        f(b);
        return;
    }

    let saved_core = std::env::var("D1_INPLACE_PRODUCT_CORE_ONLY").ok();
    let saved_reset = std::env::var("D1_INPLACE_PRODUCT_CORE_HMR_RESET_DISPLACED").ok();
    std::env::set_var("D1_INPLACE_PRODUCT_CORE_ONLY", "1");
    if round8_qtail_pair2_product_core_hmr_reset_enabled() {
        std::env::set_var("D1_INPLACE_PRODUCT_CORE_HMR_RESET_DISPLACED", "1");
    } else {
        std::env::remove_var("D1_INPLACE_PRODUCT_CORE_HMR_RESET_DISPLACED");
    }
    f(b);
    match saved_core {
        Some(value) => std::env::set_var("D1_INPLACE_PRODUCT_CORE_ONLY", value),
        None => std::env::remove_var("D1_INPLACE_PRODUCT_CORE_ONLY"),
    }
    match saved_reset {
        Some(value) => std::env::set_var("D1_INPLACE_PRODUCT_CORE_HMR_RESET_DISPLACED", value),
        None => std::env::remove_var("D1_INPLACE_PRODUCT_CORE_HMR_RESET_DISPLACED"),
    }
}

pub(crate) fn round8_emit_qtail_round217_product_reuse_or_fail(
    b: &mut B,
    tx: &[QubitId],
    ty: &[QubitId],
    p: U256,
) {
    debug_assert_eq!(tx.len(), N);
    debug_assert_eq!(ty.len(), N);
    assert_eq!(p, SECP256K1_P, "Round217 qtail reuse is secp256k1-only");
    if round8_qtail_round217_product_reuse_forbidden_full_source_enabled() {
        b.set_phase("round8_qtail_round217_forbidden_full_source_product_probe");
        round218_b5_transport::emit_round218_b5_full_source_stream_product_lowerer(b, tx, ty, p);
        return;
    }
    b.set_phase("round8_qtail_round217_source_live_product_splice_enter");
    // ROUND8_QTAIL_ROUND217_PRODUCT_REUSE=1 remains fail-closed until the
    // typed qtail/Round217 PA splice exists.  The source-live product transport must
    // clean source controls without materialized full-source history, endpoint
    // replay, product tape, nonzero phase, or hidden scratch, then pass
    // same-artifact stats and 9024 Google exact PA fuzz.
    round218_b5_transport::emit_round218_b5_source_live_stream_product_lowerer(b, tx, ty, p);
}

pub(crate) fn round8_pair1_checkpoint_qtail_second_inverse_fallback(
    b: &mut B,
    tx: &[QubitId],
    ty: &[QubitId],
    ox: &[BitId],
    oy: &[BitId],
    p: U256,
) {
    let pair2_iters = std::env::var("KAL_PAIR2_ITERS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .map(|iters| {
            checked_kaliski_iters(
                "round8 qtail D1/product pair2",
                "KAL_PAIR2_ITERS",
                iters,
                ROUND8_QTAIL_PAIR2_MIN_SAFE_ITERS,
            )
        })
        .unwrap_or(ROUND8_QTAIL_PAIR2_MIN_SAFE_ITERS);
    let scaled_by_qtail_product = std::env::var("ROUND8_QTAIL_SCALED_PAIR2_PRODUCT")
        .ok()
        .as_deref()
        == Some("1");
    let centered_by_qtail_product = std::env::var("ROUND8_QTAIL_CENTERED_PAIR2_PRODUCT")
        .ok()
        .as_deref()
        == Some("1")
        || std::env::var("BY_CENTERED_PAIR2_REPLACE").ok().as_deref() == Some("1");
    let output_side_qtail_cleanup = round8_qtail_output_side_cleanup_enabled();
    let low_q_output_side_second_inverse = low_q_output_side_second_inverse_qtail_pa_enabled();
    let round217_product_reuse = round8_qtail_round217_product_reuse_enabled();
    assert!(
        !(scaled_by_qtail_product && centered_by_qtail_product),
        "ROUND8 qtail can use either scaled-BY or centered-BY pair2 product-clean, not both"
    );
    assert!(
        !(output_side_qtail_cleanup
            && (scaled_by_qtail_product
                || centered_by_qtail_product
                || low_q_output_side_second_inverse)),
        "ROUND8_QTAIL_OUTPUT_SIDE_CLEANUP replaces the D1 product cleaner and cannot be combined with scaled/centered/low-Q output-side pair2 product hooks"
    );
    assert!(
        !(low_q_output_side_second_inverse
            && (scaled_by_qtail_product || centered_by_qtail_product)),
        "LOW_Q_OUTPUT_SIDE_SECOND_INVERSE_QTAIL_PA replaces the D1 product cleaner and cannot be combined with scaled/centered pair2 product hooks"
    );
    assert!(
        !(round217_product_reuse
            && (output_side_qtail_cleanup
                || low_q_output_side_second_inverse
                || scaled_by_qtail_product
                || centered_by_qtail_product)),
        "ROUND8_QTAIL_ROUND217_PRODUCT_REUSE replaces the D1 product cleaner and cannot be combined with output-side/scaled/centered pair2 product hooks"
    );

    if std::env::var("ROUND8_QTAIL_ROUND84_XTAIL").ok().as_deref() == Some("1") {
        round84_emit_fused_square_xtail(b, tx, ty, ox, p);
    } else {
        b.set_phase("round8_fallback_xtail_square");
        mod_mul_sub_qq(b, tx, ty, ty, p);
        b.set_phase("round8_fallback_xtail_add_2ox");
        mod_add_double_qb(b, tx, ox, p);
        b.set_phase("round8_fallback_xtail_to_rx");
        mod_neg_inplace_fast(b, tx, p);
    }

    b.set_phase("round8_fallback_c_ox_minus_rx");
    mod_sub_qb(b, tx, ox, p);
    mod_neg_inplace_fast(b, tx, p);

    if scaled_by_qtail_product {
        let yprod = b.alloc_qubits(N);
        b.set_phase("round8_fallback_scaled_by_qtail_product_clean");
        write_pair2_product_and_clean_lam_with_scaled_by_bench(b, ty, tx, &yprod, p);
        b.set_phase("round8_fallback_scaled_by_swap_product_into_ty");
        for i in 0..N {
            b.swap(ty[i], yprod[i]);
        }
        b.free_vec(&yprod);
    } else if centered_by_qtail_product {
        let yprod = b.alloc_qubits(N);
        b.set_phase("round8_fallback_centered_by_qtail_product");
        mod_mul_write_into_zero_acc_karatsuba2(b, &yprod, ty, tx, p);
        b.set_phase("round8_fallback_centered_by_qtail_clean_lam");
        add_neg_quotient_into_acc_with_centered_by_bench(b, ty, tx, &yprod, p);
        b.set_phase("round8_fallback_centered_by_swap_product_into_ty");
        for i in 0..N {
            b.swap(ty[i], yprod[i]);
        }
        b.free_vec(&yprod);
    } else if output_side_qtail_cleanup {
        round8_emit_output_side_cleanup_or_fail(b, tx, ty, ox, oy, p);
    } else if low_q_output_side_second_inverse {
        round8_emit_low_q_output_side_second_inverse_qtail_pa(b, tx, ty, ox, oy, p, pair2_iters);
    } else if round217_product_reuse {
        round8_emit_qtail_round217_product_reuse_or_fail(b, tx, ty, p);
    } else {
        with_round8_qtail_pair2_product_core_scope(b, |b| {
            d1_inplace_product_lowerer_with_kaliski_clean(b, tx, ty, p, pair2_iters);
        });
    }

    b.set_phase("round8_fallback_y_output");
    mod_sub_qb(b, ty, oy, p);
    b.set_phase("round8_fallback_x_restore");
    mod_neg_inplace_fast(b, tx, p);
    mod_add_qb(b, tx, ox, p);
}

pub(crate) fn build_round8_fused_source_live_qtail_child_builder() -> B {
    if std::env::var("ROUND8_QTAIL_BY_NAF_FREEPOS").ok().as_deref() == Some("1") {
        let b = &mut B::new();
        let tx = b.alloc_qubits(N);
        b.declare_qubit_register(&tx);
        let ty = b.alloc_qubits(N);
        b.declare_qubit_register(&ty);
        let ox = b.alloc_bits(N);
        b.declare_bit_register(&ox);
        let oy = b.alloc_bits(N);
        b.declare_bit_register(&oy);

        b.set_phase("round8_by_naf_freepos_scaffold_entry");
        by::emit_round8_qtail_by_naf_freepos_scaffold(b);
        return std::mem::replace(b, B::new());
    }

    if std::env::var("ROUND8_QTAIL_COMPLETE_FALLBACK")
        .ok()
        .as_deref()
        == Some("1")
    {
        let b = &mut B::new();
        let tx = b.alloc_qubits(N);
        b.declare_qubit_register(&tx);
        let ty = b.alloc_qubits(N);
        b.declare_qubit_register(&ty);
        let ox = b.alloc_bits(N);
        b.declare_bit_register(&ox);
        let oy = b.alloc_bits(N);
        b.declare_bit_register(&oy);

        let p = SECP256K1_P;
        b.set_phase("round8_fallback_google_abi_dx");
        mod_sub_qb(b, &tx, &ox, p);
        b.set_phase("round8_fallback_google_abi_dy");
        mod_sub_qb(b, &ty, &oy, p);
        if round218_b5_source_live_transport_pa_enabled() {
            round218_b5_transport::emit_round218_b5_source_live_transport_pa_or_fail(
                b, &tx, &ty, &ox, &oy, p,
            );
        } else if round218_b5_history_stream_pa_enabled() {
            emit_round218_b5_history_stream_pa(b, &tx, &ty, &ox, &oy, p);
        } else if round181_d1_pair1_pair2_pa_enabled() {
            emit_round181_d1_pair1_pair2_pa(b, &tx, &ty, &ox, &oy, p);
        } else {
            round24_emit_two_bank_pair1_checkpoint(b, &tx, &ty, p);
            round8_pair1_checkpoint_qtail_second_inverse_fallback(b, &tx, &ty, &ox, &oy, p);
        }
        return std::mem::replace(b, B::new());
    }

    // Source-owned emission surface for the surviving source-live/register-
    // teleport qtail lane.  V0 deliberately reuses the Round24 pair1
    // checkpoint so the Rust/KMX plumbing is wire-addressed and testable while
    // the real fused qtail body is built in this file.
    let b = &mut B::new();
    let tx = b.alloc_qubits(N);
    b.declare_qubit_register(&tx);
    let ty = b.alloc_qubits(N);
    b.declare_qubit_register(&ty);
    let ox = b.alloc_bits(N);
    b.declare_bit_register(&ox);
    let oy = b.alloc_bits(N);
    b.declare_bit_register(&oy);
    let p = SECP256K1_P;
    round24_emit_two_bank_pair1_checkpoint(b, &tx, &ty, p);
    std::mem::replace(b, B::new())
}

pub fn build_round8_fused_source_live_qtail_child() -> Vec<Op> {
    build_round8_fused_source_live_qtail_child_builder().ops
}

pub fn build_round8_fused_source_live_qtail_child_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round8_fused_source_live_qtail_child_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub(crate) fn round24_pair1_iters() -> usize {
    let (env_name, value) = if let Ok(s) = std::env::var("ROUND24_PAIR1_ITERS") {
        ("ROUND24_PAIR1_ITERS", s)
    } else if let Ok(s) = std::env::var("KAL_PAIR1_ITERS") {
        ("KAL_PAIR1_ITERS", s)
    } else {
        return ROUND24_PAIR1_MIN_SAFE_ITERS;
    };
    let iters = value
        .parse::<usize>()
        .unwrap_or_else(|_| panic!("{env_name} must be a usize, got {value:?}"));
    checked_kaliski_iters(
        "round24 pair1 checkpoint",
        env_name,
        iters,
        ROUND24_PAIR1_MIN_SAFE_ITERS,
    )
}

pub(crate) fn round24_swap_clean_lam_into_ty(b: &mut B, ty: &[QubitId], lam: &[QubitId], p: U256) {
    debug_assert_eq!(ty.len(), lam.len());
    b.set_phase("round24_pair1_negate_lam_to_positive_slope");
    mod_neg_inplace_fast(b, lam, p);
    b.set_phase("round24_pair1_swap_lam_into_zero_ty");
    for i in 0..ty.len() {
        b.swap(ty[i], lam[i]);
    }
    b.set_phase("round24_pair1_free_zero_lam_scratch");
    b.free_vec(lam);
}

pub(crate) fn round24_mul_lam_into_zero(
    b: &mut B,
    lam: &[QubitId],
    numerator: &[QubitId],
    inv_raw: &[QubitId],
    p: U256,
) {
    match std::env::var("ROUND24_PAIR1_LAM_MUL").ok().as_deref() {
        Some("walk") => mod_mul_add_qq(b, lam, inv_raw, numerator, p),
        Some("schoolbook_lowq") => {
            mod_mul_write_into_zero_acc_schoolbook_lowq(b, lam, numerator, inv_raw, p)
        }
        Some("schoolbook_peak_lowq") => {
            mod_mul_write_into_zero_acc_schoolbook_peak_lowq(b, lam, numerator, inv_raw, p)
        }
        Some("karatsuba1") => mod_mul_write_into_zero_acc_karatsuba(b, lam, numerator, inv_raw, p),
        Some("karatsuba_lowq") | Some("lowq") => {
            mod_mul_write_into_zero_acc_karatsuba_lowq(b, lam, numerator, inv_raw, p)
        }
        Some("karatsuba2") => mod_mul_write_into_zero_acc_karatsuba2(b, lam, numerator, inv_raw, p),
        _ => mod_mul_write_into_zero_acc_schoolbook(b, lam, numerator, inv_raw, p),
    }
}

pub(crate) fn round24_emit_two_bank_pair1_checkpoint(b: &mut B, tx: &[QubitId], ty: &[QubitId], p: U256) {
    let pair1_iters = round24_pair1_iters();
    let mode = std::env::var("ROUND24_PAIR1_MODE").unwrap_or_else(|_| "folded_chunked".to_string());

    match mode.as_str() {
        "raw" => {
            b.set_phase("round24_pair1_raw_kaliski_forward");
            with_kal_inv_raw(b, tx, p, pair1_iters, |b, inv_raw| {
                let lam = b.alloc_qubits(N);
                b.set_phase("round24_pair1_raw_mul_lam_scratch");
                round24_mul_lam_into_zero(b, &lam, ty, inv_raw, p);
                b.set_phase("round24_pair1_raw_halve_lam_scratch");
                for _ in 0..pair1_iters {
                    mod_halve_inplace_fast(b, &lam, p);
                }
                b.set_phase("round24_pair1_raw_zero_ty");
                mod_mul_add_into_acc_selected(b, ty, &lam, tx, p, "ROUND24_PAIR1_ZERO_TY_MUL");
                round24_swap_clean_lam_into_ty(b, ty, &lam, p);
                b.set_phase("round24_pair1_raw_kaliski_backward");
            });
        }
        "folded_chunked" => {
            b.set_phase("round24_pair1_folded_chunked_kaliski_forward");
            with_kal_inv_raw_prescaled_chunked(b, tx, p, pair1_iters, |b, inv_raw| {
                let lam = b.alloc_qubits(N);
                b.set_phase("round24_pair1_folded_chunked_mul_lam_scratch");
                round24_mul_lam_into_zero(b, &lam, ty, inv_raw, p);
                b.set_phase("round24_pair1_folded_chunked_zero_ty");
                mod_mul_add_into_acc_selected(b, ty, &lam, tx, p, "ROUND24_PAIR1_ZERO_TY_MUL");
                round24_swap_clean_lam_into_ty(b, ty, &lam, p);
                b.set_phase("round24_pair1_folded_chunked_kaliski_backward");
            });
        }
        "raw_borrow_v" => {
            let lam_cell: std::cell::RefCell<Option<Vec<QubitId>>> = std::cell::RefCell::new(None);
            b.set_phase("round24_pair1_raw_borrow_v_kaliski_forward");
            with_kal_inv_raw_borrowing_v(b, tx, p, pair1_iters, |b, inv_raw| {
                let lam = b.alloc_qubits(N);
                b.set_phase("round24_pair1_raw_borrow_v_mul_lam_scratch");
                round24_mul_lam_into_zero(b, &lam, ty, inv_raw, p);
                b.set_phase("round24_pair1_raw_borrow_v_halve_lam_scratch");
                for _ in 0..pair1_iters {
                    mod_halve_inplace_fast(b, &lam, p);
                }
                *lam_cell.borrow_mut() = Some(lam);
                b.set_phase("round24_pair1_raw_borrow_v_kaliski_backward");
            });
            let lam = lam_cell.into_inner().expect("round24 raw_borrow_v lam set");
            b.set_phase("round24_pair1_raw_borrow_v_zero_ty_after_restore");
            mod_mul_add_into_acc_selected(b, ty, &lam, tx, p, "ROUND24_PAIR1_ZERO_TY_MUL");
            round24_swap_clean_lam_into_ty(b, ty, &lam, p);
        }
        other => panic!(
            "unsupported ROUND24_PAIR1_MODE={other}; expected raw, folded_chunked, or raw_borrow_v"
        ),
    }
}

pub fn build_round24_two_bank_pair1_checkpoint() -> Vec<Op> {
    let b = &mut B::new();
    let tx = b.alloc_qubits(N);
    b.declare_qubit_register(&tx);
    let ty = b.alloc_qubits(N);
    b.declare_qubit_register(&ty);
    let ox = b.alloc_bits(N);
    b.declare_bit_register(&ox);
    let oy = b.alloc_bits(N);
    b.declare_bit_register(&oy);

    let p = SECP256K1_P;
    b.set_phase("round24_google_abi_dx");
    mod_sub_qb(b, &tx, &ox, p);
    b.set_phase("round24_google_abi_dy");
    mod_sub_qb(b, &ty, &oy, p);
    round24_emit_two_bank_pair1_checkpoint(b, &tx, &ty, p);

    // This guarded hook intentionally stops at (h, lambda), so the production
    // point-add semantic alt-seed check is not applicable here.
    b.ops.clone()
}

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

pub fn build_round84_fused_square_xtail_component() -> Vec<Op> {
    build_round84_fused_square_xtail_component_builder().ops
}

pub(crate) fn build_round84_fused_square_xtail_component_builder() -> B {
    let mut b = B::new();
    let tx = b.alloc_qubits(N);
    b.declare_qubit_register(&tx);
    let lam = b.alloc_qubits(N);
    b.declare_qubit_register(&lam);
    let ox = b.alloc_bits(N);
    b.declare_bit_register(&ox);
    let oy = b.alloc_bits(N);
    b.declare_bit_register(&oy);

    round84_emit_fused_square_xtail(&mut b, &tx, &lam, &ox, SECP256K1_P);
    b
}

pub fn build_round84_fused_square_xtail_component_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round84_fused_square_xtail_component_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub(crate) fn emit_round146_decoder_roundtrip(
    b: &mut B,
    numerator: &[QubitId],
    denominator: &[QubitId],
    quotient: &[QubitId],
) {
    halfgcd_coeff_decoder::emit_halfgcd_coeff_quotient_decoder(b, numerator, denominator, quotient);
    emit_inverse(b, |b| {
        halfgcd_coeff_decoder::emit_halfgcd_coeff_quotient_decoder(
            b,
            numerator,
            denominator,
            quotient,
        );
    });
}

pub(crate) fn round146_semantic_max_divisor() -> U256 {
    U256::from_str_radix(
        "82302208564988718744202673340416757137332630777895436281211408153252062596056",
        10,
    )
    .unwrap()
}

pub(crate) fn round181_d1_pair1_pair2_pa_enabled() -> bool {
    std::env::var("ROUND181_D1_PAIR1_PAIR2_PA").ok().as_deref() == Some("1")
}

pub(crate) fn round181_d1_pair1_iters() -> usize {
    if let Ok(s) = std::env::var("ROUND181_D1_PAIR1_ITERS") {
        let iters = s
            .parse::<usize>()
            .unwrap_or_else(|_| panic!("ROUND181_D1_PAIR1_ITERS must be a usize, got {s:?}"));
        return checked_kaliski_iters(
            "round181 D1 pair1 quotient",
            "ROUND181_D1_PAIR1_ITERS",
            iters,
            D1_INPLACE_MIN_SAFE_ITERS,
        );
    }
    round24_pair1_iters()
}

pub(crate) fn with_round181_d1_pair1_cleanup_scope<F: FnOnce(&mut B)>(b: &mut B, f: F) {
    let override_mode = std::env::var("ROUND181_D1_PAIR1_CLEANUP_KARATSUBA").ok();
    let shield_global_pair2_mode = std::env::var("ROUND181_D1_PAIR1_SHIELD_PAIR2_CLEANUP")
        .ok()
        .as_deref()
        == Some("1");
    let Some(mode) = override_mode.or_else(|| {
        if shield_global_pair2_mode {
            Some("unset".to_string())
        } else {
            None
        }
    }) else {
        f(b);
        return;
    };

    let saved = std::env::var("D1_INPLACE_CLEANUP_KARATSUBA").ok();
    match mode.as_str() {
        "unset" | "default" | "schoolbook" => std::env::remove_var("D1_INPLACE_CLEANUP_KARATSUBA"),
        other => std::env::set_var("D1_INPLACE_CLEANUP_KARATSUBA", other),
    }
    f(b);
    match saved {
        Some(value) => std::env::set_var("D1_INPLACE_CLEANUP_KARATSUBA", value),
        None => std::env::remove_var("D1_INPLACE_CLEANUP_KARATSUBA"),
    }
}

pub(crate) fn emit_round181_d1_pair1_pair2_pa(
    b: &mut B,
    tx: &[QubitId],
    ty: &[QubitId],
    ox: &[BitId],
    oy: &[BitId],
    p: U256,
) {
    let pair1_iters = round181_d1_pair1_iters();
    if std::env::var("TRACE_ROUND181_D1_PA_ACTIVE").ok().as_deref() == Some("1") {
        eprintln!(
            "ROUND181_D1_PA entry active={} free={} next_q={} phase={}",
            b.active_qubits,
            b.free_qubits.len(),
            b.next_qubit,
            b.phase
        );
    }
    b.set_phase("round181_d1_pair1_quotient");
    with_round181_d1_pair1_cleanup_scope(b, |b| {
        d1_inplace_quotient_lowerer_with_kaliski_clean(b, tx, ty, p, pair1_iters);
    });
    if std::env::var("TRACE_ROUND181_D1_PA_ACTIVE").ok().as_deref() == Some("1") {
        eprintln!(
            "ROUND181_D1_PA after_pair1 active={} free={} next_q={} phase={}",
            b.active_qubits,
            b.free_qubits.len(),
            b.next_qubit,
            b.phase
        );
    }
    round8_pair1_checkpoint_qtail_second_inverse_fallback(b, tx, ty, ox, oy, p);
}

pub fn build_round185_halfgcd_fixed_depth64_google_abi_pa() -> Vec<Op> {
    round185_halfgcd_fixed_depth64_pa::build_round185_halfgcd_fixed_depth64_google_abi_pa()
}

pub(crate) fn round190_selector_fused_width_from_env() -> usize {
    std::env::var(ROUND190_SELECTOR_FUSED_SOURCE_LIVE_RESIDUAL_WIDTH_ENV)
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|&width| width >= 2)
        .unwrap_or(N)
}
