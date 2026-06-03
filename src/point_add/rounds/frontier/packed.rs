//! `frontier::packed` — verbatim split of the original `frontier` module.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

pub fn build_round668_unified_packed_iteration_skeleton_component() -> Vec<Op> {
    build_round667_unified_active_high_refresh_builder_with_tail(true).ops
}

pub fn build_round668_unified_packed_iteration_skeleton_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round667_unified_active_high_refresh_builder_with_tail(true);
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub(crate) fn round671_k_code(symbol: usize) -> &'static [bool] {
    match symbol {
        1 => &[false],
        0 => &[true, true],
        2 => &[true, false, true],
        3 => &[true, false, false, true],
        4 => &[true, false, false, false, true],
        5 => &[true, false, false, false, false, true],
        6 => &[true, false, false, false, false, false, true],
        7 => &[true, false, false, false, false, false, false, true],
        8 => &[true, false, false, false, false, false, false, false, true],
        9 => &[
            true, false, false, false, false, false, false, false, false, true,
        ],
        10 => &[
            true, false, false, false, false, false, false, false, false, false, true,
        ],
        11 => &[
            true, false, false, false, false, false, false, false, false, false, false, true,
        ],
        _ => panic!("Round671 k symbol out of range"),
    }
}

pub(crate) fn build_round632_packed_step4_equality_body_builder() -> B {
    const FRONTIER_BITS: usize = 9;
    const PACKED_WIDTH: usize = N + 1;

    let mut b = B::new();
    let src = b.alloc_qubits(PACKED_WIDTH);
    b.declare_qubit_register(&src);
    let dst = b.alloc_qubits(PACKED_WIDTH);
    b.declare_qubit_register(&dst);
    let frontier = b.alloc_qubits(FRONTIER_BITS);
    b.declare_qubit_register(&frontier);
    let step_ctrl = b.alloc_qubit();
    b.declare_qubit_register(&[step_ctrl]);

    let active = b.alloc_qubit();
    let boundary = b.alloc_qubit();
    let body_ctrl = b.alloc_qubit();
    let c_in = b.alloc_qubit();
    let body_scratch = b.alloc_qubit();
    let eq_scratch = b.alloc_qubits(FRONTIER_BITS - 2);

    round632_emit_low_sub_body(
        &mut b,
        &src,
        &dst,
        &frontier,
        step_ctrl,
        active,
        boundary,
        body_ctrl,
        c_in,
        body_scratch,
        &eq_scratch,
    );
    round632_emit_high_add_body(
        &mut b,
        &src,
        &dst,
        &frontier,
        step_ctrl,
        active,
        boundary,
        body_ctrl,
        c_in,
        body_scratch,
        &eq_scratch,
    );

    b.set_phase("round632_free_body_scratch");
    b.free_vec(&eq_scratch);
    b.free(body_scratch);
    b.free(c_in);
    b.free(body_ctrl);
    b.free(boundary);
    b.free(active);
    b
}

pub fn build_round632_packed_step4_equality_body_component() -> Vec<Op> {
    build_round632_packed_step4_equality_body_builder().ops
}

pub fn build_round632_packed_step4_equality_body_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round632_packed_step4_equality_body_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub(crate) fn build_round634_packed_step3_step4_shell_builder() -> B {
    const FRONTIER_BITS: usize = 9;
    const PACKED_WIDTH: usize = N + 1;

    let mut b = B::new();
    let lane_a = b.alloc_qubits(PACKED_WIDTH);
    b.declare_qubit_register(&lane_a);
    let lane_b = b.alloc_qubits(PACKED_WIDTH);
    b.declare_qubit_register(&lane_b);
    let start_a = b.alloc_qubits(FRONTIER_BITS);
    b.declare_qubit_register(&start_a);
    let start_b = b.alloc_qubits(FRONTIER_BITS);
    b.declare_qubit_register(&start_b);
    let swap_ctrl = b.alloc_qubit();
    let add_ctrl = b.alloc_qubit();
    b.declare_qubit_register(&[swap_ctrl, add_ctrl]);

    let active = b.alloc_qubit();
    let boundary = b.alloc_qubit();
    let body_ctrl = b.alloc_qubit();
    let c_in = b.alloc_qubit();
    let body_scratch = b.alloc_qubit();
    let eq_scratch = b.alloc_qubits(FRONTIER_BITS - 2);
    let min_start = b.alloc_qubits(FRONTIER_BITS);
    let rhs_lt_lhs = b.alloc_qubit();

    round634_emit_step3_cswap(&mut b, swap_ctrl, &lane_a, &lane_b, &start_a, &start_b);

    round632_emit_low_sub_body(
        &mut b,
        &lane_a,
        &lane_b,
        &start_b,
        add_ctrl,
        active,
        boundary,
        body_ctrl,
        c_in,
        body_scratch,
        &eq_scratch,
    );

    round634_emit_min_frontier_compute(&mut b, &start_a, &start_b, &min_start, rhs_lt_lhs);
    round632_emit_high_add_body(
        &mut b,
        &lane_b,
        &lane_a,
        &min_start,
        add_ctrl,
        active,
        boundary,
        body_ctrl,
        c_in,
        body_scratch,
        &eq_scratch,
    );
    round634_emit_min_frontier_uncompute(&mut b, &start_a, &start_b, &min_start, rhs_lt_lhs);

    b.set_phase("round634_free_step4_shell_scratch");
    b.free(rhs_lt_lhs);
    b.free_vec(&min_start);
    b.free_vec(&eq_scratch);
    b.free(body_scratch);
    b.free(c_in);
    b.free(body_ctrl);
    b.free(boundary);
    b.free(active);
    b
}

pub fn build_round634_packed_step3_step4_shell_component() -> Vec<Op> {
    build_round634_packed_step3_step4_shell_builder().ops
}

pub fn build_round634_packed_step3_step4_shell_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round634_packed_step3_step4_shell_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub(crate) fn round647_high_bits_from_packed_lane(lane: &[QubitId]) -> Vec<QubitId> {
    const PACKED_WIDTH: usize = N + 1;
    debug_assert_eq!(lane.len(), PACKED_WIDTH);
    (0..N).map(|idx| lane[PACKED_WIDTH - 1 - idx]).collect()
}

pub(crate) fn round647_emit_packed_shift_double_frontier(
    b: &mut B,
    lane: &[QubitId],
    start: &[QubitId],
    flag: QubitId,
    eq_scratch: &[QubitId],
) {
    const PACKED_WIDTH: usize = N + 1;
    debug_assert_eq!(lane.len(), PACKED_WIDTH);

    b.set_phase("round647_packed_v_shift_r_double");
    for idx in 0..PACKED_WIDTH - 1 {
        b.swap(lane[idx], lane[idx + 1]);
    }

    b.set_phase("round647_packed_solinas_fold_spill");
    let high = round647_high_bits_from_packed_lane(lane);
    let spill = lane[0];
    let solinas_c = U256::MAX
        .wrapping_sub(SECP256K1_P)
        .wrapping_add(U256::from(1u64));
    cadd_nbit_const_direct_fast(b, &high, solinas_c, spill);
    b.cx(high[0], spill);

    b.set_phase("round647_packed_high_start_decrement");
    round647_emit_decrement_start_if_high_nonzero(b, start, flag, eq_scratch);
}

pub(crate) fn round648_emit_packed_shift_double_frontier_dirty(
    b: &mut B,
    lane: &[QubitId],
    start: &[QubitId],
    dirty: &[QubitId],
    q_clean2: &[QubitId; 2],
    flag: QubitId,
    eq_scratch: &[QubitId],
) {
    const PACKED_WIDTH: usize = N + 1;
    debug_assert_eq!(lane.len(), PACKED_WIDTH);
    debug_assert!(dirty.len() >= N - 2);

    b.set_phase("round648_packed_v_shift_r_double");
    for idx in 0..PACKED_WIDTH - 1 {
        b.swap(lane[idx], lane[idx + 1]);
    }

    b.set_phase("round648_packed_solinas_fold_spill_dirty_vented");
    let high = round647_high_bits_from_packed_lane(lane);
    let spill = lane[0];
    let solinas_c = U256::MAX
        .wrapping_sub(SECP256K1_P)
        .wrapping_add(U256::from(1u64));
    venting::ciadd_dirty_2clean_classical(
        b,
        &high,
        &dirty[..N - 2],
        q_clean2,
        solinas_c.as_limbs()[0],
        spill,
        false,
    );
    b.cx(high[0], spill);

    b.set_phase("round648_packed_high_start_decrement");
    round647_emit_decrement_start_if_high_nonzero(b, start, flag, eq_scratch);
}

pub(crate) fn build_round648_packed_iteration_refresh_skeleton_builder() -> B {
    const FRONTIER_BITS: usize = 9;
    const PACKED_WIDTH: usize = N + 1;

    let mut b = B::new();
    let lane_a = b.alloc_qubits(PACKED_WIDTH);
    b.declare_qubit_register(&lane_a);
    let lane_b = b.alloc_qubits(PACKED_WIDTH);
    b.declare_qubit_register(&lane_b);
    let start_a = b.alloc_qubits(FRONTIER_BITS);
    b.declare_qubit_register(&start_a);
    let start_b = b.alloc_qubits(FRONTIER_BITS);
    b.declare_qubit_register(&start_b);
    let refresh_start = b.alloc_qubits(FRONTIER_BITS);
    b.declare_qubit_register(&refresh_start);
    let swap_ctrl = b.alloc_qubit();
    let add_ctrl = b.alloc_qubit();
    b.declare_qubit_register(&[swap_ctrl, add_ctrl]);

    let active = b.alloc_qubit();
    let boundary = b.alloc_qubit();
    let body_ctrl = b.alloc_qubit();
    let c_in = b.alloc_qubit();
    let body_scratch = b.alloc_qubit();
    let eq_scratch = b.alloc_qubits(FRONTIER_BITS - 2);
    let min_start = b.alloc_qubits(FRONTIER_BITS);
    let rhs_lt_lhs = b.alloc_qubit();

    round634_emit_step3_cswap(&mut b, swap_ctrl, &lane_a, &lane_b, &start_a, &start_b);

    round632_emit_low_sub_body(
        &mut b,
        &lane_a,
        &lane_b,
        &start_b,
        add_ctrl,
        active,
        boundary,
        body_ctrl,
        c_in,
        body_scratch,
        &eq_scratch,
    );

    round634_emit_min_frontier_compute(&mut b, &start_a, &start_b, &min_start, rhs_lt_lhs);
    round632_emit_high_add_body(
        &mut b,
        &lane_b,
        &lane_a,
        &min_start,
        add_ctrl,
        active,
        boundary,
        body_ctrl,
        c_in,
        body_scratch,
        &eq_scratch,
    );

    b.set_phase("round648_inverse_high_add_for_old_frontier_refresh");
    emit_inverse(&mut b, |b| {
        round632_emit_high_add_body(
            b,
            &lane_b,
            &lane_a,
            &min_start,
            add_ctrl,
            active,
            boundary,
            body_ctrl,
            c_in,
            body_scratch,
            &eq_scratch,
        );
    });

    round632_emit_high_add_body(
        &mut b,
        &lane_b,
        &lane_a,
        &min_start,
        add_ctrl,
        active,
        boundary,
        body_ctrl,
        c_in,
        body_scratch,
        &eq_scratch,
    );
    round634_emit_min_frontier_uncompute(&mut b, &start_a, &start_b, &min_start, rhs_lt_lhs);

    b.set_phase("round648_free_refresh_only_scratch_before_shift");
    b.free(rhs_lt_lhs);
    b.free_vec(&min_start);
    b.free(body_scratch);
    b.free(c_in);

    let clean2 = [active, boundary];
    round648_emit_packed_shift_double_frontier_dirty(
        &mut b,
        &lane_b,
        &start_b,
        &lane_a,
        &clean2,
        body_ctrl,
        &eq_scratch,
    );
    round648_emit_step9_cswap(&mut b, swap_ctrl, &lane_a, &lane_b, &start_a, &start_b);

    b.set_phase("round648_free_packed_iteration_scratch");
    b.free_vec(&eq_scratch);
    b.free(body_ctrl);
    b.free(boundary);
    b.free(active);
    b.free_vec(&refresh_start);
    b
}

pub fn build_round648_packed_iteration_refresh_skeleton_component() -> Vec<Op> {
    build_round648_packed_iteration_refresh_skeleton_builder().ops
}

pub fn build_round648_packed_iteration_refresh_skeleton_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round648_packed_iteration_refresh_skeleton_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub(crate) fn build_round649_packed_iteration_trimmed_skeleton_builder() -> B {
    const FRONTIER_BITS: usize = 9;
    const PACKED_WIDTH: usize = N + 1;

    let mut b = B::new();
    let lane_a = b.alloc_qubits(PACKED_WIDTH);
    b.declare_qubit_register(&lane_a);
    let lane_b = b.alloc_qubits(PACKED_WIDTH);
    b.declare_qubit_register(&lane_b);
    let start_a = b.alloc_qubits(FRONTIER_BITS);
    b.declare_qubit_register(&start_a);
    let start_b = b.alloc_qubits(FRONTIER_BITS);
    b.declare_qubit_register(&start_b);
    let swap_ctrl = b.alloc_qubit();
    let add_ctrl = b.alloc_qubit();
    b.declare_qubit_register(&[swap_ctrl, add_ctrl]);

    let active = b.alloc_qubit();
    let boundary = b.alloc_qubit();
    let body_ctrl = b.alloc_qubit();
    let c_in = b.alloc_qubit();
    let body_scratch = b.alloc_qubit();
    let eq_scratch = b.alloc_qubits(FRONTIER_BITS - 2);
    let min_start = b.alloc_qubits(FRONTIER_BITS);
    let rhs_lt_lhs = b.alloc_qubit();

    round634_emit_step3_cswap(&mut b, swap_ctrl, &lane_a, &lane_b, &start_a, &start_b);

    round632_emit_low_sub_body(
        &mut b,
        &lane_a,
        &lane_b,
        &start_b,
        add_ctrl,
        active,
        boundary,
        body_ctrl,
        c_in,
        body_scratch,
        &eq_scratch,
    );

    round634_emit_min_frontier_compute(&mut b, &start_a, &start_b, &min_start, rhs_lt_lhs);
    round632_emit_high_add_body(
        &mut b,
        &lane_b,
        &lane_a,
        &min_start,
        add_ctrl,
        active,
        boundary,
        body_ctrl,
        c_in,
        body_scratch,
        &eq_scratch,
    );

    b.set_phase("round649_inverse_high_add_for_old_frontier_refresh");
    emit_inverse(&mut b, |b| {
        round632_emit_high_add_body(
            b,
            &lane_b,
            &lane_a,
            &min_start,
            add_ctrl,
            active,
            boundary,
            body_ctrl,
            c_in,
            body_scratch,
            &eq_scratch,
        );
    });

    round632_emit_high_add_body(
        &mut b,
        &lane_b,
        &lane_a,
        &min_start,
        add_ctrl,
        active,
        boundary,
        body_ctrl,
        c_in,
        body_scratch,
        &eq_scratch,
    );
    round634_emit_min_frontier_uncompute(&mut b, &start_a, &start_b, &min_start, rhs_lt_lhs);

    b.set_phase("round649_free_refresh_only_scratch_before_shift");
    b.free(rhs_lt_lhs);
    b.free_vec(&min_start);
    b.free(body_scratch);
    b.free(c_in);

    let clean2 = [active, boundary];
    round648_emit_packed_shift_double_frontier_dirty(
        &mut b,
        &lane_b,
        &start_b,
        &lane_a,
        &clean2,
        body_ctrl,
        &eq_scratch,
    );
    round648_emit_step9_cswap(&mut b, swap_ctrl, &lane_a, &lane_b, &start_a, &start_b);

    b.set_phase("round649_free_packed_iteration_scratch");
    b.free_vec(&eq_scratch);
    b.free(body_ctrl);
    b.free(boundary);
    b.free(active);
    b
}

pub fn build_round649_packed_iteration_trimmed_skeleton_component() -> Vec<Op> {
    build_round649_packed_iteration_trimmed_skeleton_builder().ops
}

pub fn build_round649_packed_iteration_trimmed_skeleton_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round649_packed_iteration_trimmed_skeleton_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub(crate) fn build_round647_packed_shift_double_frontier_builder() -> B {
    const FRONTIER_BITS: usize = 9;
    const PACKED_WIDTH: usize = N + 1;

    let mut b = B::new();
    let lane = b.alloc_qubits(PACKED_WIDTH);
    b.declare_qubit_register(&lane);
    let start = b.alloc_qubits(FRONTIER_BITS);
    b.declare_qubit_register(&start);
    let flag = b.alloc_qubit();
    b.declare_qubit_register(&[flag]);
    let eq_scratch = b.alloc_qubits(FRONTIER_BITS - 2);

    round647_emit_packed_shift_double_frontier(&mut b, &lane, &start, flag, &eq_scratch);

    b.set_phase("round647_free_shift_double_scratch");
    b.free_vec(&eq_scratch);
    b
}

pub fn build_round647_packed_shift_double_frontier_component() -> Vec<Op> {
    build_round647_packed_shift_double_frontier_builder().ops
}

pub fn build_round647_packed_shift_double_frontier_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round647_packed_shift_double_frontier_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}
