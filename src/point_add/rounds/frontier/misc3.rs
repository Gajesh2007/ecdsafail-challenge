//! `frontier::misc3` — verbatim split of the original `frontier` module.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

pub(crate) fn round691_polarized_generic_scale_p_pa_enabled() -> bool {
    std::env::var("ROUND691_POLARIZED_GENERIC_SCALE_P_PA")
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn round691_emit_odd_mod2n_lowword_scale_p(b: &mut B, factor: &[QubitId], target: &[QubitId]) {
    debug_assert_eq!(factor.len(), N);
    debug_assert_eq!(target.len(), N);

    b.set_phase("round691_odd_mod2n_lowword_scale_p");
    for column in (0..N - 1).rev() {
        let width = N - column - 1;
        let addend = &factor[1..1 + width];
        let acc = &target[column + 1..N];
        cucc_add_ctrl_lowq(b, addend, acc, target[column]);
    }
}

pub(crate) fn round691_emit_scale_p(b: &mut B, factor: &[QubitId], target: &[QubitId], p: U256) {
    match std::env::var("ROUND691_SCALE_P_MODE")
        .unwrap_or_else(|_| "generic".to_string())
        .as_str()
    {
        "generic" => {
            b.set_phase("round691_generic_scale_p");
            d1_inplace_product_lowerer_with_kaliski_clean(
                b,
                factor,
                target,
                p,
                round495_d1_source_live_pair2_iters(),
            );
        }
        "odd_mod2n_lowword" => round691_emit_odd_mod2n_lowword_scale_p(b, factor, target),
        other => panic!(
            "unsupported ROUND691_SCALE_P_MODE={other}; expected generic or odd_mod2n_lowword"
        ),
    }
}

pub(crate) fn emit_round691_polarized_generic_scale_p_pa(
    b: &mut B,
    tx: &[QubitId],
    ty: &[QubitId],
    ox: &[BitId],
    oy: &[BitId],
    p: U256,
) {
    debug_assert_eq!(tx.len(), N);
    debug_assert_eq!(ty.len(), N);
    debug_assert_eq!(ox.len(), N);
    debug_assert_eq!(oy.len(), N);

    b.set_phase("round691_pair1_checkpoint");
    round24_emit_two_bank_pair1_checkpoint(b, tx, ty, p);

    round84_emit_fused_square_xtail(b, tx, ty, ox, p);

    if std::env::var("ROUND691_SKIP_EQ_DERIVATIVE").ok().as_deref() == Some("1") {
        b.set_phase("round691_noeq_compute_d");
        let d = load_bits(b, ox);
        mod_sub_qq_fast(b, &d, tx, p);

        round691_emit_scale_p(b, &d, ty, p);

        b.set_phase("round691_noeq_y_sub_offset_y");
        mod_sub_qb(b, ty, oy, p);

        b.set_phase("round691_noeq_uncompute_d");
        mod_add_qq_fast(b, &d, tx, p);
        unload_bits(b, &d, ox);
        return;
    }

    b.set_phase("round691_eq_diff");
    let eq_diff = load_bits(b, ox);
    mod_sub_qq_fast(b, &eq_diff, tx, p);
    let eq = b.alloc_qubit();
    toggle_eq_zero_flag_fast(b, &eq_diff, eq);

    b.set_phase("round691_compute_polarized_d");
    let d = round564_compute_polarized_d(b, tx, ox, oy, eq, p);

    round691_emit_scale_p(b, &d, ty, p);

    b.set_phase("round691_y_sub_offset_y");
    mod_sub_qb(b, ty, oy, p);

    b.set_phase("round691_compute_derivative_square");
    let ox_q = load_bits(b, ox);
    let derivative = b.alloc_qubits(N);
    round564_square_add_selected(b, &derivative, &ox_q, p);
    b.set_phase("round691_sub_eq_derivative_x3");
    for _ in 0..3 {
        cmod_sub_qq(b, ty, &derivative, eq, p);
    }
    b.set_phase("round691_uncompute_derivative_square");
    round564_square_sub_selected(b, &derivative, &ox_q, p);
    b.free_vec(&derivative);
    unload_bits(b, &ox_q, ox);

    b.set_phase("round691_uncompute_polarized_d");
    round564_uncompute_polarized_d(b, &d, tx, ox, oy, eq, p);

    b.set_phase("round691_uncompute_eq");
    toggle_eq_zero_flag_fast(b, &eq_diff, eq);
    b.free(eq);
    mod_add_qq_fast(b, &eq_diff, tx, p);
    unload_bits(b, &eq_diff, ox);
}
