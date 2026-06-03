//! `b5::round315_385` — verbatim split of the original `b5` module.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

pub(crate) fn build_round315_b5_hash_history_full_source_stream_scaled_inverse_component_builder() -> B {
    let mut b = B::new();
    let dx = b.alloc_qubits(N);
    b.declare_qubit_register(&dx);
    let v = b.alloc_qubits(N);
    b.declare_qubit_register(&v);
    let r = b.alloc_qubits(N);

    b.x(r[0]);
    round218_b5_transport::emit_round315_b5_hash_history_full_source_stream_scaled_inverse(
        &mut b,
        &dx,
        &v,
        &r,
        SECP256K1_P,
    );
    b.free_vec(&r);
    b
}

pub fn build_round315_b5_hash_history_full_source_stream_scaled_inverse_component() -> Vec<Op> {
    build_round315_b5_hash_history_full_source_stream_scaled_inverse_component_builder().ops
}

pub(crate) fn build_round326_b5_live_l_rank_exact_cover_component_builder(zeta_bits: usize) -> B {
    let mut b = B::new();
    let zeta = b.alloc_qubits(zeta_bits);
    b.declare_qubit_register(&zeta);
    let old_g0 = b.alloc_qubits(round218_b5_program::ROUND218_B5_BLOCK_BITS);
    b.declare_qubit_register(&old_g0);
    let l_rank = b.alloc_qubits(4);
    b.declare_qubit_register(&l_rank);

    round218_b5_transport::emit_round326_b5_live_l_rank_exact_cover(
        &mut b, &zeta, &old_g0, &l_rank,
    );
    b
}

pub fn build_round326_b5_live_l_rank_exact_cover_component(zeta_bits: usize) -> Vec<Op> {
    build_round326_b5_live_l_rank_exact_cover_component_builder(zeta_bits).ops
}

pub(crate) fn build_round326_b5_branch_rank_exact_cover_cleaner_component_builder(zeta_bits: usize) -> B {
    let mut b = B::new();
    let zeta = b.alloc_qubits(zeta_bits);
    b.declare_qubit_register(&zeta);
    let old_g0 = b.alloc_qubits(round218_b5_program::ROUND218_B5_BLOCK_BITS);
    b.declare_qubit_register(&old_g0);
    let l_rank = b.alloc_qubits(4);
    b.declare_qubit_register(&l_rank);
    let branch = b.alloc_qubits(round218_b5_program::ROUND218_B5_BLOCK_BITS);
    b.declare_qubit_register(&branch);

    round218_b5_transport::emit_round326_b5_branch_rank_exact_cover_cleaner(
        &mut b, &zeta, &old_g0, &l_rank, &branch,
    );
    b
}

pub fn build_round326_b5_branch_rank_exact_cover_cleaner_component(zeta_bits: usize) -> Vec<Op> {
    build_round326_b5_branch_rank_exact_cover_cleaner_component_builder(zeta_bits).ops
}

pub(crate) fn build_round379_b5_source_live_cheap_lft_frame_block_component_builder(
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

    round218_b5_transport::emit_round379_b5_source_live_cheap_lft_frame_block(
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

pub fn build_round379_b5_source_live_cheap_lft_frame_block_component(
    zeta_bits: usize,
    window_bits: usize,
) -> Vec<Op> {
    build_round379_b5_source_live_cheap_lft_frame_block_component_builder(zeta_bits, window_bits)
        .ops
}

pub fn build_round379_b5_source_live_cheap_lft_frame_block_component_phase_resources(
    zeta_bits: usize,
    window_bits: usize,
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round379_b5_source_live_cheap_lft_frame_block_component_builder(
        zeta_bits,
        window_bits,
    );
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub(crate) fn build_round380_b5_full_source_stream_cheap_lft_frame_transport_component_builder() -> B {
    let mut b = B::new();
    let dx = b.alloc_qubits(N);
    b.declare_qubit_register(&dx);
    let v = b.alloc_qubits(N);
    b.declare_qubit_register(&v);
    let r = b.alloc_qubits(N);
    b.declare_qubit_register(&r);

    round218_b5_transport::emit_round380_b5_full_source_stream_cheap_lft_frame_transport(
        &mut b,
        &dx,
        &v,
        &r,
        SECP256K1_P,
    );
    b
}

pub fn build_round380_b5_full_source_stream_cheap_lft_frame_transport_component() -> Vec<Op> {
    build_round380_b5_full_source_stream_cheap_lft_frame_transport_component_builder().ops
}

pub(crate) fn build_round381_b5_source_live_branch_only_cheap_lft_frame_block_component_builder(
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

    round218_b5_transport::emit_round381_b5_source_live_branch_only_cheap_lft_frame_block(
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

pub fn build_round381_b5_source_live_branch_only_cheap_lft_frame_block_component(
    zeta_bits: usize,
    window_bits: usize,
) -> Vec<Op> {
    build_round381_b5_source_live_branch_only_cheap_lft_frame_block_component_builder(
        zeta_bits,
        window_bits,
    )
    .ops
}

pub fn build_round381_b5_source_live_branch_only_cheap_lft_frame_block_component_phase_resources(
    zeta_bits: usize,
    window_bits: usize,
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round381_b5_source_live_branch_only_cheap_lft_frame_block_component_builder(
        zeta_bits,
        window_bits,
    );
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub(crate) fn build_round383_b5_current_pattern_ranked_cheap_lft_source_block_component_builder(
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
    let l_rank = b.alloc_qubits(4);
    b.declare_qubit_register(&l_rank);
    let old_g0_word = b.alloc_qubits(round218_b5_program::ROUND218_B5_BLOCK_BITS);
    b.declare_qubit_register(&old_g0_word);

    round218_b5_transport::emit_round383_b5_current_pattern_ranked_cheap_lft_source_block(
        &mut b,
        &zeta,
        &f_window,
        &g_window,
        &v,
        &r,
        &l_rank,
        &old_g0_word,
        SECP256K1_P,
    );
    b
}

pub fn build_round383_b5_current_pattern_ranked_cheap_lft_source_block_component(
    zeta_bits: usize,
    window_bits: usize,
) -> Vec<Op> {
    build_round383_b5_current_pattern_ranked_cheap_lft_source_block_component_builder(
        zeta_bits,
        window_bits,
    )
    .ops
}

pub fn build_round383_b5_current_pattern_ranked_cheap_lft_source_block_component_phase_resources(
    zeta_bits: usize,
    window_bits: usize,
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round383_b5_current_pattern_ranked_cheap_lft_source_block_component_builder(
        zeta_bits,
        window_bits,
    );
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub(crate) fn build_round384_b5_current_pattern_ranked_source_rollback_block_component_builder(
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
    let l_rank = b.alloc_qubits(4);
    b.declare_qubit_register(&l_rank);
    let old_g0_word = b.alloc_qubits(round218_b5_program::ROUND218_B5_BLOCK_BITS);
    b.declare_qubit_register(&old_g0_word);

    round218_b5_transport::emit_round384_b5_current_pattern_ranked_source_rollback_block(
        &mut b,
        &zeta,
        &f_window,
        &g_window,
        &l_rank,
        &old_g0_word,
    );
    b
}

pub fn build_round384_b5_current_pattern_ranked_source_rollback_block_component(
    zeta_bits: usize,
    window_bits: usize,
) -> Vec<Op> {
    build_round384_b5_current_pattern_ranked_source_rollback_block_component_builder(
        zeta_bits,
        window_bits,
    )
    .ops
}

pub fn build_round384_b5_current_pattern_ranked_source_rollback_block_component_phase_resources(
    zeta_bits: usize,
    window_bits: usize,
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round384_b5_current_pattern_ranked_source_rollback_block_component_builder(
        zeta_bits,
        window_bits,
    );
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub(crate) fn build_round385_b5_fused_advance_frame_rollback_block_component_builder(
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

    round218_b5_transport::emit_round385_b5_fused_advance_frame_rollback_block(
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

pub fn build_round385_b5_fused_advance_frame_rollback_block_component(
    zeta_bits: usize,
    window_bits: usize,
) -> Vec<Op> {
    build_round385_b5_fused_advance_frame_rollback_block_component_builder(zeta_bits, window_bits)
        .ops
}

pub fn build_round385_b5_fused_advance_frame_rollback_block_component_phase_resources(
    zeta_bits: usize,
    window_bits: usize,
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round385_b5_fused_advance_frame_rollback_block_component_builder(
        zeta_bits,
        window_bits,
    );
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}
