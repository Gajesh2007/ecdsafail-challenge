//! `b5::round200_315` — verbatim split of the original `b5` module.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

pub(crate) fn emit_round200_signed_coeff_lambda_horner(
    b: &mut B,
    lam: &[QubitId],
    dy: &[QubitId],
    coeff: &[QubitId],
    p: U256,
) {
    assert_eq!(lam.len(), N);
    assert_eq!(dy.len(), N);
    assert!(coeff.len() > N);

    let lowq_cmod = std::env::var("ROUND200_LOWQ_CMOD").ok().as_deref() == Some("1")
        || std::env::var("ROUND200_LOWQ_LAM_CMOD").ok().as_deref() == Some("1");

    b.set_phase("round200_lam_horner_low_coeff");
    if lowq_cmod {
        for i in (0..N).rev() {
            if i < N - 1 {
                mod_double_inplace_fast(b, lam, p);
            }
            cmod_add_qq_lowq(b, lam, dy, coeff[i], p);
        }
    } else {
        mod_mul_horner_add_qq(b, lam, dy, &coeff[..N], p);
    }

    // A negative 257-bit two's-complement coefficient has low 256 bits equal
    // to 2^256 - |c|.  Since 2^256 = p + (2^32+977), subtract
    // (2^32+977)*dy under the sign bit to obtain c*dy mod p.
    let sign = coeff[N];
    let secp_fold = U256::from(4_294_968_273u64);
    b.set_phase("round200_lam_sign_fold_correction");
    for shift in 0..=32 {
        if bit(secp_fold, shift) {
            if lowq_cmod {
                cmod_sub_qq_lowq(b, lam, dy, sign, p);
            } else {
                cmod_sub_qq(b, lam, dy, sign, p);
            }
        }
        if shift < 32 {
            mod_double_inplace_fast(b, dy, p);
        }
    }
    b.set_phase("round200_lam_sign_fold_restore_dy");
    for _ in 0..32 {
        mod_halve_inplace_fast(b, dy, p);
    }
}

pub(crate) fn round200_horner_unadd(b: &mut B, acc: &[QubitId], x: &[QubitId], y: &[QubitId], p: U256) {
    let lowq_cmod = std::env::var("ROUND200_LOWQ_CMOD").ok().as_deref() == Some("1")
        || std::env::var("ROUND200_LOWQ_ZERO_CMOD").ok().as_deref() == Some("1");
    if !lowq_cmod {
        mod_mul_horner_unadd_qq(b, acc, x, y, p);
        return;
    }

    assert_eq!(acc.len(), N);
    assert_eq!(x.len(), N);
    assert_eq!(y.len(), N);
    if x[0] == y[0] {
        for i in 0..N {
            cmod_sub_qq_lowq(b, acc, x, y[i], p);
            if i < N - 1 {
                mod_halve_inplace_fast(b, acc, p);
            }
        }
    } else {
        mod_neg_inplace_fast(b, x, p);
        for i in 0..N {
            cmod_add_qq_lowq(b, acc, x, y[i], p);
            if i < N - 1 {
                mod_halve_inplace_fast(b, acc, p);
            }
        }
        mod_neg_inplace_fast(b, x, p);
    }
}

pub fn round200_semantic_full_gcd_pair1_checkpoint_register_widths() -> [usize; 3] {
    [N, N, N]
}

pub(crate) fn emit_round200_semantic_full_gcd_pair1_checkpoint_in_place(
    b: &mut B,
    v: &[QubitId],
    ty: &[QubitId],
    p: U256,
) -> Vec<QubitId> {
    emit_round200_semantic_full_gcd_pair1_checkpoint_in_place_with_options(b, v, ty, p, false)
}

pub(crate) fn emit_round200_semantic_full_gcd_pair1_checkpoint_in_place_with_options(
    b: &mut B,
    v: &[QubitId],
    ty: &[QubitId],
    p: U256,
    preserve_dy: bool,
) -> Vec<QubitId> {
    assert_eq!(p, SECP256K1_P);
    let (lane_width, coeff_width, total_q_bits, _max_q_bits, _steps) =
        round158_halfgcd_splice_live::round199_semantic_full_gcd_prefix_widths();
    assert_eq!(lane_width, N);

    let u = b.alloc_qubits(lane_width);
    for i in 0..lane_width {
        if bit(p, i) {
            b.x(u[i]);
        }
    }
    let coeff_b = b.alloc_qubits(coeff_width);
    let coeff_d = b.alloc_qubits(coeff_width);
    let q_tail = b.alloc_qubits(total_q_bits);
    b.x(coeff_d[0]);

    let prefix_start = b.ops.len();
    round158_halfgcd_splice_live::emit_round199_semantic_full_gcd_prefix_sequence(
        b, &u, &v, &coeff_b, &coeff_d, &q_tail,
    );
    let prefix_ops = b.ops[prefix_start..].to_vec();

    let lam = b.alloc_qubits(N);
    emit_round200_signed_coeff_lambda_horner(b, &lam, ty, &coeff_b, p);

    round158_halfgcd_splice_live::replay_round199_semantic_full_gcd_prefix_inverse_from_ops(
        b,
        &prefix_ops,
    );

    b.set_phase("round200_free_full_gcd_prefix_scratch");
    unload_const(b, &u, p);
    b.free_vec(&u);
    b.free_vec(&coeff_b);
    b.x(coeff_d[0]);
    b.free_vec(&coeff_d);
    b.free_vec(&q_tail);

    if preserve_dy {
        b.set_phase("round200_preserve_dy_for_source_live_cubic");
    } else {
        b.set_phase("round200_zero_dy_horner_unadd");
        round200_horner_unadd(b, ty, &lam, v, p);
    }

    lam
}

pub(crate) fn build_round200_semantic_full_gcd_pair1_checkpoint_builder() -> B {
    let mut b = B::new();
    let v = b.alloc_qubits(N);
    b.declare_qubit_register(&v);
    let ty = b.alloc_qubits(N);
    b.declare_qubit_register(&ty);
    let lam =
        emit_round200_semantic_full_gcd_pair1_checkpoint_in_place(&mut b, &v, &ty, SECP256K1_P);
    b.declare_qubit_register(&lam);
    b
}

pub fn build_round200_semantic_full_gcd_pair1_checkpoint_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round200_semantic_full_gcd_pair1_checkpoint_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub fn build_round200_semantic_full_gcd_pair1_checkpoint() -> Vec<Op> {
    build_round200_semantic_full_gcd_pair1_checkpoint_builder().ops
}

pub(crate) fn round218_b5_source_live_transport_pa_enabled() -> bool {
    std::env::var("ROUND218_B5_SOURCE_LIVE_TRANSPORT_PA")
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn round218_b5_history_stream_pa_enabled() -> bool {
    std::env::var("ROUND218_B5_HISTORY_STREAM_PA")
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn emit_round218_b5_history_stream_pa(
    b: &mut B,
    tx: &[QubitId],
    ty: &[QubitId],
    ox: &[BitId],
    oy: &[BitId],
    p: U256,
) {
    b.set_phase("round218_b5_history_stream_pair1_quotient");
    round218_b5_transport::emit_round218_b5_full_source_stream_quotient_lowerer(b, tx, ty, p);

    b.set_phase("round8_fallback_xtail_square");
    mod_mul_sub_qq(b, tx, ty, ty, p);
    b.set_phase("round8_fallback_xtail_add_2ox");
    mod_add_double_qb(b, tx, ox, p);
    b.set_phase("round8_fallback_xtail_to_rx");
    mod_neg_inplace_fast(b, tx, p);

    b.set_phase("round8_fallback_c_ox_minus_rx");
    mod_sub_qb(b, tx, ox, p);
    mod_neg_inplace_fast(b, tx, p);

    b.set_phase("round218_b5_history_stream_pair2_product");
    round218_b5_transport::emit_round218_b5_full_source_stream_product_lowerer(b, tx, ty, p);

    b.set_phase("round8_fallback_y_output");
    mod_sub_qb(b, ty, oy, p);
    b.set_phase("round8_fallback_x_restore");
    mod_neg_inplace_fast(b, tx, p);
    mod_add_qb(b, tx, ox, p);
}

pub(crate) fn build_round218_b5_transport_coeff_step_component_builder() -> B {
    let mut b = B::new();
    let v = b.alloc_qubits(N);
    b.declare_qubit_register(&v);
    let r = b.alloc_qubits(N);
    b.declare_qubit_register(&r);
    let controls = b.alloc_qubits(2);
    b.declare_qubit_register(&controls);

    b.set_phase("round218_b5_transport_coeff_step_component");
    round218_b5_transport::emit_round218_scaled_coeff_step_selected(
        &mut b,
        &v,
        &r,
        controls[0],
        controls[1],
        SECP256K1_P,
    );
    b
}

pub fn build_round218_b5_transport_coeff_step_component() -> Vec<Op> {
    build_round218_b5_transport_coeff_step_component_builder().ops
}

pub(crate) fn build_round218_b5_transport_coeff_block_component_builder() -> B {
    let mut b = B::new();
    let v = b.alloc_qubits(N);
    b.declare_qubit_register(&v);
    let r = b.alloc_qubits(N);
    b.declare_qubit_register(&r);
    let branch_word = b.alloc_qubits(round218_b5_program::ROUND218_B5_BLOCK_BITS);
    b.declare_qubit_register(&branch_word);
    let old_g0_word = b.alloc_qubits(round218_b5_program::ROUND218_B5_BLOCK_BITS);
    b.declare_qubit_register(&old_g0_word);

    b.set_phase("round218_b5_transport_coeff_b5_block_component");
    round218_b5_transport::emit_round218_scaled_coeff_b5_block_selected(
        &mut b,
        &v,
        &r,
        &branch_word,
        &old_g0_word,
        SECP256K1_P,
    );
    b
}

pub fn build_round218_b5_transport_coeff_block_component() -> Vec<Op> {
    build_round218_b5_transport_coeff_block_component_builder().ops
}

pub(crate) fn build_round218_b5_transport_coeff_fixed_block_component_builder(
    block_index: usize,
    zeta_start: i128,
    f_low: u8,
    g_low: u8,
) -> B {
    let mut b = B::new();
    let v = b.alloc_qubits(N);
    b.declare_qubit_register(&v);
    let r = b.alloc_qubits(N);
    b.declare_qubit_register(&r);
    let row = round218_b5_program::block_row(
        block_index,
        round218_b5_program::BlockSelector {
            zeta_start,
            f_low,
            g_low,
            width: round218_b5_program::ROUND218_B5_BLOCK_BITS as u8,
        },
    );

    b.set_phase("round218_b5_transport_coeff_fixed_block_component");
    round218_b5_transport::emit_round218_scaled_coeff_block_fixed(
        &mut b,
        &v,
        &r,
        &row,
        SECP256K1_P,
    );
    b
}

pub fn build_round218_b5_transport_coeff_fixed_block_component(
    block_index: usize,
    zeta_start: i128,
    f_low: u8,
    g_low: u8,
) -> Vec<Op> {
    build_round218_b5_transport_coeff_fixed_block_component_builder(
        block_index,
        zeta_start,
        f_low,
        g_low,
    )
    .ops
}

pub(crate) fn build_round218_b5_source_live_transport_block_component_builder(zeta_start: i128) -> B {
    let mut b = B::new();
    let f_low = b.alloc_qubits(round218_b5_program::ROUND218_B5_BLOCK_BITS);
    b.declare_qubit_register(&f_low);
    let g_low = b.alloc_qubits(round218_b5_program::ROUND218_B5_BLOCK_BITS);
    b.declare_qubit_register(&g_low);
    let v = b.alloc_qubits(N);
    b.declare_qubit_register(&v);
    let r = b.alloc_qubits(N);
    b.declare_qubit_register(&r);
    let branch_word = b.alloc_qubits(round218_b5_program::ROUND218_B5_BLOCK_BITS);
    b.declare_qubit_register(&branch_word);
    let old_g0_word = b.alloc_qubits(round218_b5_program::ROUND218_B5_BLOCK_BITS);
    b.declare_qubit_register(&old_g0_word);

    round218_b5_transport::emit_round218_b5_source_live_transport_block(
        &mut b,
        &f_low,
        &g_low,
        &v,
        &r,
        zeta_start,
        &branch_word,
        &old_g0_word,
        SECP256K1_P,
    );
    b
}

pub fn build_round218_b5_source_live_transport_block_component(zeta_start: i128) -> Vec<Op> {
    build_round218_b5_source_live_transport_block_component_builder(zeta_start).ops
}

pub(crate) fn build_round218_b5_source_window_transport_block_component_builder(zeta_start: i128) -> B {
    let mut b = B::new();
    let f_window = b.alloc_qubits(round218_b5_selector::ROUND218_B5_LOW_WINDOW_BITS);
    b.declare_qubit_register(&f_window);
    let g_window = b.alloc_qubits(round218_b5_selector::ROUND218_B5_LOW_WINDOW_BITS);
    b.declare_qubit_register(&g_window);
    let v = b.alloc_qubits(N);
    b.declare_qubit_register(&v);
    let r = b.alloc_qubits(N);
    b.declare_qubit_register(&r);
    let branch_word = b.alloc_qubits(round218_b5_program::ROUND218_B5_BLOCK_BITS);
    b.declare_qubit_register(&branch_word);
    let old_g0_word = b.alloc_qubits(round218_b5_program::ROUND218_B5_BLOCK_BITS);
    b.declare_qubit_register(&old_g0_word);
    let next_f_low = b.alloc_qubits(round218_b5_program::ROUND218_B5_BLOCK_BITS);
    b.declare_qubit_register(&next_f_low);
    let next_g_low = b.alloc_qubits(round218_b5_program::ROUND218_B5_BLOCK_BITS);
    b.declare_qubit_register(&next_g_low);

    round218_b5_transport::emit_round218_b5_source_window_transport_block(
        &mut b,
        &f_window,
        &g_window,
        &v,
        &r,
        zeta_start,
        &branch_word,
        &old_g0_word,
        &next_f_low,
        &next_g_low,
        SECP256K1_P,
    );
    b
}

pub fn build_round218_b5_source_window_transport_block_component(zeta_start: i128) -> Vec<Op> {
    build_round218_b5_source_window_transport_block_component_builder(zeta_start).ops
}

pub(crate) fn build_round218_b5_dynamic_source_window_transport_block_component_builder(
    zeta_min: i128,
    zeta_max: i128,
    window_bits: usize,
) -> B {
    assert!(
        window_bits >= round218_b5_program::ROUND218_B5_BLOCK_BITS,
        "dynamic source-window component needs at least B=5 source bits"
    );
    let spec = round218_b5_selector::Round218B5DynamicZetaTransducerSpec::new(zeta_min, zeta_max);
    let mut b = B::new();
    let zeta_start = b.alloc_qubits(spec.start_zeta_bits());
    if !zeta_start.is_empty() {
        b.declare_qubit_register(&zeta_start);
    }
    let f_window = b.alloc_qubits(window_bits);
    b.declare_qubit_register(&f_window);
    let g_window = b.alloc_qubits(window_bits);
    b.declare_qubit_register(&g_window);
    let v = b.alloc_qubits(N);
    b.declare_qubit_register(&v);
    let r = b.alloc_qubits(N);
    b.declare_qubit_register(&r);
    let branch_word = b.alloc_qubits(round218_b5_program::ROUND218_B5_BLOCK_BITS);
    b.declare_qubit_register(&branch_word);
    let old_g0_word = b.alloc_qubits(round218_b5_program::ROUND218_B5_BLOCK_BITS);
    b.declare_qubit_register(&old_g0_word);
    let end_zeta = b.alloc_qubits(spec.end_zeta_bits());
    if !end_zeta.is_empty() {
        b.declare_qubit_register(&end_zeta);
    }
    let next_bits = window_bits - round218_b5_program::ROUND218_B5_BLOCK_BITS;
    let next_f = b.alloc_qubits(next_bits);
    if !next_f.is_empty() {
        b.declare_qubit_register(&next_f);
    }
    let next_g = b.alloc_qubits(next_bits);
    if !next_g.is_empty() {
        b.declare_qubit_register(&next_g);
    }

    round218_b5_transport::emit_round218_b5_dynamic_source_window_transport_block(
        &mut b,
        spec,
        &zeta_start,
        &f_window,
        &g_window,
        &v,
        &r,
        &branch_word,
        &old_g0_word,
        &end_zeta,
        &next_f,
        &next_g,
        SECP256K1_P,
    );
    b
}

pub fn build_round218_b5_dynamic_source_window_transport_block_component(
    zeta_min: i128,
    zeta_max: i128,
    window_bits: usize,
) -> Vec<Op> {
    build_round218_b5_dynamic_source_window_transport_block_component_builder(
        zeta_min,
        zeta_max,
        window_bits,
    )
    .ops
}

pub(crate) fn build_round218_b5_twos_zeta_source_window_transport_block_component_builder(
    zeta_bits: usize,
    window_bits: usize,
) -> B {
    assert!(
        zeta_bits >= 3,
        "two's-complement zeta component needs at least 3 signed bits"
    );
    assert!(
        window_bits >= round218_b5_program::ROUND218_B5_BLOCK_BITS,
        "two's-complement source-window component needs at least B=5 source bits"
    );
    let mut b = B::new();
    let zeta = b.alloc_qubits(zeta_bits);
    b.declare_qubit_register(&zeta);
    let f_window = b.alloc_qubits(window_bits);
    b.declare_qubit_register(&f_window);
    let g_window = b.alloc_qubits(window_bits);
    b.declare_qubit_register(&g_window);
    let v = b.alloc_qubits(N);
    b.declare_qubit_register(&v);
    let r = b.alloc_qubits(N);
    b.declare_qubit_register(&r);
    let branch_word = b.alloc_qubits(round218_b5_program::ROUND218_B5_BLOCK_BITS);
    b.declare_qubit_register(&branch_word);
    let old_g0_word = b.alloc_qubits(round218_b5_program::ROUND218_B5_BLOCK_BITS);
    b.declare_qubit_register(&old_g0_word);
    let next_bits = window_bits - round218_b5_program::ROUND218_B5_BLOCK_BITS;
    let next_f = b.alloc_qubits(next_bits);
    if !next_f.is_empty() {
        b.declare_qubit_register(&next_f);
    }
    let next_g = b.alloc_qubits(next_bits);
    if !next_g.is_empty() {
        b.declare_qubit_register(&next_g);
    }

    round218_b5_transport::emit_round218_b5_twos_zeta_source_window_transport_block(
        &mut b,
        &zeta,
        &f_window,
        &g_window,
        &v,
        &r,
        &branch_word,
        &old_g0_word,
        &next_f,
        &next_g,
        SECP256K1_P,
    );
    b
}

pub fn build_round218_b5_twos_zeta_source_window_transport_block_component(
    zeta_bits: usize,
    window_bits: usize,
) -> Vec<Op> {
    build_round218_b5_twos_zeta_source_window_transport_block_component_builder(
        zeta_bits,
        window_bits,
    )
    .ops
}

pub(crate) fn build_round218_b5_source_live_projective_scalar_transport_block_component_builder(
    zeta_bits: usize,
    window_bits: usize,
) -> B {
    let mut b = B::new();
    let zeta = b.alloc_qubits(zeta_bits);
    b.declare_qubit_register(&zeta);
    let f_window = b.alloc_qubits(window_bits);
    b.declare_qubit_register(&f_window);
    let g_window = b.alloc_qubits(window_bits);
    b.declare_qubit_register(&g_window);
    let v = b.alloc_qubits(N);
    b.declare_qubit_register(&v);
    let r = b.alloc_qubits(N);
    b.declare_qubit_register(&r);

    round218_b5_transport::emit_round218_b5_source_live_projective_scalar_transport_block(
        &mut b,
        &zeta,
        &f_window,
        &g_window,
        &v,
        &r,
        SECP256K1_P,
    );
    b
}

pub fn build_round218_b5_source_live_projective_scalar_transport_block_component(
    zeta_bits: usize,
    window_bits: usize,
) -> Vec<Op> {
    build_round218_b5_source_live_projective_scalar_transport_block_component_builder(
        zeta_bits,
        window_bits,
    )
    .ops
}

pub fn build_round218_b5_source_live_projective_scalar_transport_block_component_phase_resources(
    zeta_bits: usize,
    window_bits: usize,
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round218_b5_source_live_projective_scalar_transport_block_component_builder(
        zeta_bits,
        window_bits,
    );
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub(crate) fn build_round218_b5_twos_zeta_control_transport_block_component_builder(
    zeta_bits: usize,
    window_bits: usize,
) -> B {
    build_round218_b5_source_live_projective_scalar_transport_block_component_builder(
        zeta_bits,
        window_bits,
    )
}

pub fn build_round218_b5_twos_zeta_control_transport_block_component(
    zeta_bits: usize,
    window_bits: usize,
) -> Vec<Op> {
    build_round218_b5_twos_zeta_control_transport_block_component_builder(zeta_bits, window_bits)
        .ops
}

pub(crate) fn build_round218_b5_full_source_stream_transport_component_builder() -> B {
    let mut b = B::new();
    let dx = b.alloc_qubits(N);
    b.declare_qubit_register(&dx);
    let v = b.alloc_qubits(N);
    b.declare_qubit_register(&v);
    let r = b.alloc_qubits(N);
    b.declare_qubit_register(&r);

    round218_b5_transport::emit_round218_b5_full_source_stream_transport(
        &mut b,
        &dx,
        &v,
        &r,
        SECP256K1_P,
    );
    b
}

pub fn build_round218_b5_full_source_stream_transport_component() -> Vec<Op> {
    build_round218_b5_full_source_stream_transport_component_builder().ops
}

pub(crate) fn build_round218_b5_full_source_stream_scaled_inverse_component_builder() -> B {
    let mut b = B::new();
    let dx = b.alloc_qubits(N);
    b.declare_qubit_register(&dx);
    let v = b.alloc_qubits(N);
    b.declare_qubit_register(&v);

    round218_b5_transport::emit_round218_b5_full_source_stream_scaled_inverse_from_zero(
        &mut b,
        &dx,
        &v,
        SECP256K1_P,
    );
    b
}

pub fn build_round218_b5_full_source_stream_scaled_inverse_component() -> Vec<Op> {
    build_round218_b5_full_source_stream_scaled_inverse_component_builder().ops
}

pub(crate) fn build_round218_b5_selector_component_builder(zeta_start: i128) -> B {
    let mut b = B::new();
    let f_low = b.alloc_qubits(round218_b5_selector::ROUND218_B5_LOW_STATE_BITS);
    b.declare_qubit_register(&f_low);
    let g_low = b.alloc_qubits(round218_b5_selector::ROUND218_B5_LOW_STATE_BITS);
    b.declare_qubit_register(&g_low);
    let branch_word = b.alloc_qubits(round218_b5_selector::ROUND218_B5_LOW_STATE_BITS);
    b.declare_qubit_register(&branch_word);
    let old_g0_word = b.alloc_qubits(round218_b5_selector::ROUND218_B5_LOW_STATE_BITS);
    b.declare_qubit_register(&old_g0_word);
    let scratch = b.alloc_qubits(
        round218_b5_selector::round218_b5_low_state_selector_scratch_qubits(zeta_start),
    );

    b.set_phase("round218_b5_selector_component");
    round218_b5_selector::emit_round218_b5_low_state_selector_with_scratch(
        &mut b,
        &f_low,
        &g_low,
        zeta_start,
        &branch_word,
        &old_g0_word,
        &scratch,
    );
    b.free_vec(&scratch);
    b
}

pub fn build_round218_b5_selector_component(zeta_start: i128) -> Vec<Op> {
    build_round218_b5_selector_component_builder(zeta_start).ops
}

pub(crate) fn build_round218_b5_dynamic_zeta_selector_component_builder(zeta_min: i128, zeta_max: i128) -> B {
    let spec = round218_b5_selector::Round218B5DynamicZetaTransducerSpec::new(zeta_min, zeta_max);
    let mut b = B::new();
    let zeta_start = b.alloc_qubits(spec.start_zeta_bits());
    if !zeta_start.is_empty() {
        b.declare_qubit_register(&zeta_start);
    }
    let f_low = b.alloc_qubits(round218_b5_selector::ROUND218_B5_LOW_STATE_BITS);
    b.declare_qubit_register(&f_low);
    let g_low = b.alloc_qubits(round218_b5_selector::ROUND218_B5_LOW_STATE_BITS);
    b.declare_qubit_register(&g_low);
    let branch_word = b.alloc_qubits(round218_b5_selector::ROUND218_B5_LOW_STATE_BITS);
    b.declare_qubit_register(&branch_word);
    let old_g0_word = b.alloc_qubits(round218_b5_selector::ROUND218_B5_LOW_STATE_BITS);
    b.declare_qubit_register(&old_g0_word);
    let end_zeta = b.alloc_qubits(spec.end_zeta_bits());
    if !end_zeta.is_empty() {
        b.declare_qubit_register(&end_zeta);
    }
    let scratch = b.alloc_qubits(
        round218_b5_selector::round218_b5_dynamic_zeta_transducer_scratch_qubits(spec),
    );

    b.set_phase("round218_b5_dynamic_zeta_selector_component");
    round218_b5_selector::emit_round218_b5_dynamic_zeta_transducer_with_scratch(
        &mut b,
        spec,
        &zeta_start,
        &f_low,
        &g_low,
        &branch_word,
        &old_g0_word,
        &end_zeta,
        &scratch,
    );
    b.free_vec(&scratch);
    b
}

pub fn build_round218_b5_dynamic_zeta_selector_component(
    zeta_min: i128,
    zeta_max: i128,
) -> Vec<Op> {
    build_round218_b5_dynamic_zeta_selector_component_builder(zeta_min, zeta_max).ops
}

pub(crate) fn build_round314_b5_source_live_hash_transport_window_block_component_builder(
    zeta_bits: usize,
    window_bits: usize,
) -> B {
    let mut b = B::new();
    let zeta = b.alloc_qubits(zeta_bits);
    b.declare_qubit_register(&zeta);
    let f_window = b.alloc_qubits(window_bits);
    b.declare_qubit_register(&f_window);
    let g_window = b.alloc_qubits(window_bits);
    b.declare_qubit_register(&g_window);
    let v = b.alloc_qubits(N);
    b.declare_qubit_register(&v);
    let r = b.alloc_qubits(N);
    b.declare_qubit_register(&r);
    let l_hash = b.alloc_qubits(8);
    b.declare_qubit_register(&l_hash);
    let next_bits = window_bits - round218_b5_program::ROUND218_B5_BLOCK_BITS;
    let next_f = b.alloc_qubits(next_bits);
    if !next_f.is_empty() {
        b.declare_qubit_register(&next_f);
    }
    let next_g = b.alloc_qubits(next_bits);
    if !next_g.is_empty() {
        b.declare_qubit_register(&next_g);
    }

    round218_b5_transport::emit_round314_b5_source_live_hash_transport_window_block(
        &mut b,
        &zeta,
        &f_window,
        &g_window,
        &v,
        &r,
        &l_hash,
        &next_f,
        &next_g,
        SECP256K1_P,
    );
    b
}

pub fn build_round314_b5_source_live_hash_transport_window_block_component(
    zeta_bits: usize,
    window_bits: usize,
) -> Vec<Op> {
    build_round314_b5_source_live_hash_transport_window_block_component_builder(
        zeta_bits,
        window_bits,
    )
    .ops
}

pub(crate) fn build_round315_b5_hash_history_full_source_stream_transport_component_builder() -> B {
    let mut b = B::new();
    let dx = b.alloc_qubits(N);
    b.declare_qubit_register(&dx);
    let v = b.alloc_qubits(N);
    b.declare_qubit_register(&v);
    let r = b.alloc_qubits(N);
    b.declare_qubit_register(&r);

    round218_b5_transport::emit_round315_b5_hash_history_full_source_stream_transport(
        &mut b,
        &dx,
        &v,
        &r,
        SECP256K1_P,
    );
    b
}

pub fn build_round315_b5_hash_history_full_source_stream_transport_component() -> Vec<Op> {
    build_round315_b5_hash_history_full_source_stream_transport_component_builder().ops
}
