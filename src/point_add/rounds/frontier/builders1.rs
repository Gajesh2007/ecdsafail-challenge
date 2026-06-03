//! `frontier::builders1` — verbatim split of the original `frontier` module.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

pub(crate) fn build_round629_frontier_predicate_stream_builder() -> B {
    const FRONTIER_BITS: usize = 9;
    const PACKED_WIDTH: usize = N + 1;

    let mut b = B::new();
    let frontier = b.alloc_qubits(FRONTIER_BITS);
    b.declare_qubit_register(&frontier);
    let flag = b.alloc_qubit();
    b.declare_qubit_register(&[flag]);

    b.set_phase("round629_low_threshold_predicates");
    for i in 0..PACKED_WIDTH {
        let c = U256::from(i as u64);
        cmp_gt_const_n1(&mut b, &frontier, c, flag);
        cmp_gt_const_n1(&mut b, &frontier, c, flag);
    }

    b.set_phase("round629_high_threshold_predicates");
    for i in 0..PACKED_WIDTH {
        let c = U256::from(i as u64);
        cmp_gt_const_n1(&mut b, &frontier, c, flag);
        cmp_gt_const_n1(&mut b, &frontier, c, flag);
    }

    b
}

pub fn build_round629_frontier_predicate_stream_component() -> Vec<Op> {
    build_round629_frontier_predicate_stream_builder().ops
}

pub fn build_round629_frontier_predicate_stream_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round629_frontier_predicate_stream_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub(crate) fn build_round630_variable_cuccaro_predicate_schedule_builder() -> B {
    const FRONTIER_BITS: usize = 9;
    const PACKED_WIDTH: usize = N + 1;

    let mut b = B::new();
    let frontier = b.alloc_qubits(FRONTIER_BITS);
    b.declare_qubit_register(&frontier);
    let flag_a = b.alloc_qubit();
    let flag_b = b.alloc_qubit();
    b.declare_qubit_register(&[flag_a, flag_b]);

    b.set_phase("round630_low_forward_active");
    for i in 0..PACKED_WIDTH {
        round630_emit_threshold_compute_uncompute(&mut b, &frontier, flag_a, i);
    }

    b.set_phase("round630_low_top_boundary");
    for i in 0..PACKED_WIDTH {
        round630_emit_boundary_pair_compute_uncompute(&mut b, &frontier, flag_a, flag_b, i);
    }

    b.set_phase("round630_low_backward_active");
    for i in 0..PACKED_WIDTH {
        round630_emit_threshold_compute_uncompute(&mut b, &frontier, flag_a, i);
    }

    b.set_phase("round630_high_forward_active");
    for i in 0..PACKED_WIDTH {
        round630_emit_threshold_compute_uncompute(&mut b, &frontier, flag_a, i);
    }

    b.set_phase("round630_high_top_boundary");
    for i in 0..PACKED_WIDTH {
        round630_emit_boundary_pair_compute_uncompute(&mut b, &frontier, flag_a, flag_b, i);
    }

    b.set_phase("round630_high_backward_active");
    for i in 0..PACKED_WIDTH {
        round630_emit_threshold_compute_uncompute(&mut b, &frontier, flag_a, i);
    }

    b
}

pub fn build_round630_variable_cuccaro_predicate_schedule_component() -> Vec<Op> {
    build_round630_variable_cuccaro_predicate_schedule_builder().ops
}

pub fn build_round630_variable_cuccaro_predicate_schedule_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round630_variable_cuccaro_predicate_schedule_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub(crate) fn build_round631_frontier_equality_stream_schedule_builder() -> B {
    const FRONTIER_BITS: usize = 9;
    const PACKED_WIDTH: usize = N + 1;

    let mut b = B::new();
    let frontier = b.alloc_qubits(FRONTIER_BITS);
    b.declare_qubit_register(&frontier);
    let active = b.alloc_qubit();
    let boundary = b.alloc_qubit();
    b.declare_qubit_register(&[active, boundary]);
    let scratch = b.alloc_qubits(FRONTIER_BITS - 2);

    b.set_phase("round631_low_forward_active_stream");
    round631_emit_low_active_stream(&mut b, &frontier, active, &scratch, PACKED_WIDTH);

    b.set_phase("round631_low_boundary_eq_stream");
    round631_emit_boundary_stream(&mut b, &frontier, boundary, &scratch, 1..=PACKED_WIDTH);

    b.set_phase("round631_low_backward_active_stream");
    round631_emit_low_active_stream(&mut b, &frontier, active, &scratch, PACKED_WIDTH);

    b.set_phase("round631_high_forward_active_stream");
    round631_emit_high_active_stream(&mut b, &frontier, active, &scratch, PACKED_WIDTH);

    b.set_phase("round631_high_boundary_eq_stream");
    round631_emit_boundary_stream(&mut b, &frontier, boundary, &scratch, 0..PACKED_WIDTH);

    b.set_phase("round631_high_backward_active_stream");
    round631_emit_high_active_stream(&mut b, &frontier, active, &scratch, PACKED_WIDTH);

    b.set_phase("round631_free_stream_scratch");
    b.free_vec(&scratch);
    b
}

pub fn build_round631_frontier_equality_stream_schedule_component() -> Vec<Op> {
    build_round631_frontier_equality_stream_schedule_builder().ops
}

pub fn build_round631_frontier_equality_stream_schedule_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round631_frontier_equality_stream_schedule_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub(crate) fn build_round658_low_sub_borrow_tap_builder() -> B {
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
    let borrow_flag = b.alloc_qubit();
    b.declare_qubit_register(&[step_ctrl, borrow_flag]);

    let active = b.alloc_qubit();
    let boundary = b.alloc_qubit();
    let body_ctrl = b.alloc_qubit();
    let c_in = b.alloc_qubit();
    let body_scratch = b.alloc_qubit();
    let eq_scratch = b.alloc_qubits(FRONTIER_BITS - 2);

    round658_emit_low_sub_body_with_borrow(
        &mut b,
        &src,
        &dst,
        &frontier,
        step_ctrl,
        borrow_flag,
        active,
        boundary,
        body_ctrl,
        c_in,
        body_scratch,
        &eq_scratch,
    );

    b.set_phase("round658_free_low_sub_borrow_scratch");
    b.free_vec(&eq_scratch);
    b.free(body_scratch);
    b.free(c_in);
    b.free(body_ctrl);
    b.free(boundary);
    b.free(active);
    b
}

pub fn build_round658_low_sub_borrow_tap_component() -> Vec<Op> {
    build_round658_low_sub_borrow_tap_builder().ops
}

pub fn build_round658_low_sub_borrow_tap_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round658_low_sub_borrow_tap_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub(crate) fn build_round659_source_over_frontier_or_builder() -> B {
    const FRONTIER_BITS: usize = 9;
    const PACKED_WIDTH: usize = N + 1;
    const MAX_DELTA: usize = 19;

    let mut b = B::new();
    let source_lane = b.alloc_qubits(PACKED_WIDTH);
    b.declare_qubit_register(&source_lane);
    let source_start = b.alloc_qubits(FRONTIER_BITS);
    b.declare_qubit_register(&source_start);
    let dst_start = b.alloc_qubits(FRONTIER_BITS);
    b.declare_qubit_register(&dst_start);
    let step_ctrl = b.alloc_qubit();
    let overflow_flag = b.alloc_qubit();
    b.declare_qubit_register(&[step_ctrl, overflow_flag]);

    let eq_a = b.alloc_qubit();
    let eq_b = b.alloc_qubit();
    let pair_ctrl = b.alloc_qubit();
    let body_scratch = b.alloc_qubit();
    let eq_scratch = b.alloc_qubits(FRONTIER_BITS - 2);
    let or_chain = b.alloc_qubits(MAX_DELTA - 1);

    b.set_phase("round659_source_over_frontier_range_or");
    round659_emit_source_over_frontier_pairs(
        &mut b,
        &source_lane,
        &source_start,
        &dst_start,
        step_ctrl,
        overflow_flag,
        eq_a,
        eq_b,
        pair_ctrl,
        body_scratch,
        &eq_scratch,
        &or_chain,
    );

    b.set_phase("round659_free_source_over_frontier_scratch");
    b.free_vec(&or_chain);
    b.free_vec(&eq_scratch);
    b.free(body_scratch);
    b.free(pair_ctrl);
    b.free(eq_b);
    b.free(eq_a);
    b
}

pub fn build_round659_source_over_frontier_or_component() -> Vec<Op> {
    build_round659_source_over_frontier_or_builder().ops
}

pub fn build_round659_source_over_frontier_or_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round659_source_over_frontier_or_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub(crate) fn build_round660_low_add_carry_retap_builder() -> B {
    const FRONTIER_BITS: usize = 9;
    const PACKED_WIDTH: usize = N + 1;

    let mut b = B::new();
    let src = b.alloc_qubits(PACKED_WIDTH);
    b.declare_qubit_register(&src);
    let dst_after_sub = b.alloc_qubits(PACKED_WIDTH);
    b.declare_qubit_register(&dst_after_sub);
    let frontier = b.alloc_qubits(FRONTIER_BITS);
    b.declare_qubit_register(&frontier);
    let step_ctrl = b.alloc_qubit();
    let carry_flag = b.alloc_qubit();
    b.declare_qubit_register(&[step_ctrl, carry_flag]);

    let active = b.alloc_qubit();
    let boundary = b.alloc_qubit();
    let body_ctrl = b.alloc_qubit();
    let c_in = b.alloc_qubit();
    let body_scratch = b.alloc_qubit();
    let eq_scratch = b.alloc_qubits(FRONTIER_BITS - 2);

    round660_emit_low_add_carry_retap(
        &mut b,
        &src,
        &dst_after_sub,
        &frontier,
        step_ctrl,
        carry_flag,
        active,
        boundary,
        body_ctrl,
        c_in,
        body_scratch,
        &eq_scratch,
    );

    b.set_phase("round660_free_low_add_carry_retap_scratch");
    b.free_vec(&eq_scratch);
    b.free(body_scratch);
    b.free(c_in);
    b.free(body_ctrl);
    b.free(boundary);
    b.free(active);
    b
}

pub fn build_round660_low_add_carry_retap_component() -> Vec<Op> {
    build_round660_low_add_carry_retap_builder().ops
}

pub fn build_round660_low_add_carry_retap_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round660_low_add_carry_retap_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub(crate) fn build_round661_window_preswap_low_sub_control_builder() -> B {
    const FRONTIER_BITS: usize = 9;
    const PACKED_WIDTH: usize = N + 1;
    const MAX_DELTA: usize = 19;

    let mut b = B::new();
    let lane_a = b.alloc_qubits(PACKED_WIDTH);
    b.declare_qubit_register(&lane_a);
    let lane_b = b.alloc_qubits(PACKED_WIDTH);
    b.declare_qubit_register(&lane_b);
    let start_a = b.alloc_qubits(FRONTIER_BITS);
    b.declare_qubit_register(&start_a);
    let start_b = b.alloc_qubits(FRONTIER_BITS);
    b.declare_qubit_register(&start_b);
    let step_ctrl = b.alloc_qubit();
    let a_flag = b.alloc_qubit();
    b.declare_qubit_register(&[step_ctrl, a_flag]);

    let eq_a = b.alloc_qubit();
    let eq_b = b.alloc_qubit();
    let pair_ctrl = b.alloc_qubit();
    let body_scratch = b.alloc_qubit();
    let eq_scratch = b.alloc_qubits(FRONTIER_BITS - 2);
    let or_chain = b.alloc_qubits(MAX_DELTA - 1);

    b.set_phase("round661_seed_a_from_source_window");
    round659_emit_source_over_frontier_pairs(
        &mut b,
        &lane_a,
        &start_a,
        &start_b,
        step_ctrl,
        a_flag,
        eq_a,
        eq_b,
        pair_ctrl,
        body_scratch,
        &eq_scratch,
        &or_chain,
    );

    b.set_phase("round661_free_source_window_scratch");
    b.free_vec(&or_chain);
    b.free_vec(&eq_scratch);
    b.free(body_scratch);
    b.free(pair_ctrl);
    b.free(eq_b);
    b.free(eq_a);

    round634_emit_step3_cswap(&mut b, a_flag, &lane_a, &lane_b, &start_a, &start_b);

    let active = b.alloc_qubit();
    let boundary = b.alloc_qubit();
    let body_ctrl = b.alloc_qubit();
    let c_in = b.alloc_qubit();
    let body_scratch = b.alloc_qubit();
    let eq_scratch = b.alloc_qubits(FRONTIER_BITS - 2);

    round658_emit_low_sub_body_with_borrow(
        &mut b,
        &lane_a,
        &lane_b,
        &start_b,
        step_ctrl,
        a_flag,
        active,
        boundary,
        body_ctrl,
        c_in,
        body_scratch,
        &eq_scratch,
    );

    b.set_phase("round661_free_low_sub_scratch");
    b.free_vec(&eq_scratch);
    b.free(body_scratch);
    b.free(c_in);
    b.free(body_ctrl);
    b.free(boundary);
    b.free(active);
    b
}

pub fn build_round661_window_preswap_low_sub_control_component() -> Vec<Op> {
    build_round661_window_preswap_low_sub_control_builder().ops
}

pub fn build_round661_window_preswap_low_sub_control_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round661_window_preswap_low_sub_control_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub(crate) fn build_round662_borrow_branch_recovery_arithmetic_builder() -> B {
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
    let borrow_ctrl = b.alloc_qubit();
    b.declare_qubit_register(&[borrow_ctrl]);

    let active = b.alloc_qubit();
    let boundary = b.alloc_qubit();
    let body_ctrl = b.alloc_qubit();
    let c_in = b.alloc_qubit();
    let body_scratch = b.alloc_qubit();
    let eq_scratch = b.alloc_qubits(FRONTIER_BITS - 2);
    let min_start = b.alloc_qubits(FRONTIER_BITS);
    let rhs_lt_lhs = b.alloc_qubit();

    round662_emit_low_add_body(
        &mut b,
        &lane_a,
        &lane_b,
        &start_b,
        borrow_ctrl,
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
        &lane_a,
        &lane_b,
        &min_start,
        borrow_ctrl,
        active,
        boundary,
        body_ctrl,
        c_in,
        body_scratch,
        &eq_scratch,
    );
    round634_emit_min_frontier_uncompute(&mut b, &start_a, &start_b, &min_start, rhs_lt_lhs);

    round632_emit_low_sub_body(
        &mut b,
        &lane_b,
        &lane_a,
        &start_b,
        borrow_ctrl,
        active,
        boundary,
        body_ctrl,
        c_in,
        body_scratch,
        &eq_scratch,
    );

    round648_emit_step9_cswap(&mut b, borrow_ctrl, &lane_a, &lane_b, &start_a, &start_b);

    b.set_phase("round662_free_recovery_scratch");
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

pub fn build_round662_borrow_branch_recovery_arithmetic_component() -> Vec<Op> {
    build_round662_borrow_branch_recovery_arithmetic_builder().ops
}

pub fn build_round662_borrow_branch_recovery_arithmetic_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round662_borrow_branch_recovery_arithmetic_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub(crate) fn build_round663_borrow_start_delta_le10_builder() -> B {
    const FRONTIER_BITS: usize = 9;
    const DELTA_BITS: usize = 4;

    let mut b = B::new();
    let start_a = b.alloc_qubits(FRONTIER_BITS);
    b.declare_qubit_register(&start_a);
    let start_b = b.alloc_qubits(FRONTIER_BITS);
    b.declare_qubit_register(&start_b);
    let borrow_ctrl = b.alloc_qubit();
    b.declare_qubit_register(&[borrow_ctrl]);
    let delta_bits = b.alloc_qubits(DELTA_BITS);
    b.declare_qubit_register(&delta_bits);

    let eq_a = b.alloc_qubit();
    let eq_b = b.alloc_qubit();
    let pair_ctrl = b.alloc_qubit();
    let body_scratch = b.alloc_qubit();
    let eq_scratch = b.alloc_qubits(FRONTIER_BITS - 2);

    round663_emit_borrow_start_delta_le10(
        &mut b,
        &start_a,
        &start_b,
        borrow_ctrl,
        &delta_bits,
        eq_a,
        eq_b,
        pair_ctrl,
        body_scratch,
        &eq_scratch,
    );

    b.set_phase("round663_free_delta_scratch");
    b.free_vec(&eq_scratch);
    b.free(body_scratch);
    b.free(pair_ctrl);
    b.free(eq_b);
    b.free(eq_a);
    b
}

pub fn build_round663_borrow_start_delta_le10_component() -> Vec<Op> {
    build_round663_borrow_start_delta_le10_builder().ops
}

pub fn build_round663_borrow_start_delta_le10_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round663_borrow_start_delta_le10_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub(crate) fn build_round664_delta_growth_frontier_update_builder() -> B {
    const FRONTIER_BITS: usize = 9;
    const DELTA_BITS: usize = 4;

    let mut b = B::new();
    let start_a = b.alloc_qubits(FRONTIER_BITS);
    b.declare_qubit_register(&start_a);
    let start_b = b.alloc_qubits(FRONTIER_BITS);
    b.declare_qubit_register(&start_b);
    let delta_bits = b.alloc_qubits(DELTA_BITS);
    b.declare_qubit_register(&delta_bits);
    let growth_ctrl = b.alloc_qubit();
    let swap_ctrl = b.alloc_qubit();
    b.declare_qubit_register(&[growth_ctrl, swap_ctrl]);

    round664_emit_delta_growth_frontier_update(
        &mut b,
        &start_a,
        &start_b,
        &delta_bits,
        growth_ctrl,
        swap_ctrl,
    );

    b
}

pub fn build_round664_delta_growth_frontier_update_component() -> Vec<Op> {
    build_round664_delta_growth_frontier_update_builder().ops
}

pub fn build_round664_delta_growth_frontier_update_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round664_delta_growth_frontier_update_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub(crate) fn build_round665_high_add_growth_tap_builder() -> B {
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
    let growth_flag = b.alloc_qubit();
    b.declare_qubit_register(&[step_ctrl, growth_flag]);

    let active = b.alloc_qubit();
    let boundary = b.alloc_qubit();
    let body_ctrl = b.alloc_qubit();
    let c_in = b.alloc_qubit();
    let body_scratch = b.alloc_qubit();
    let eq_scratch = b.alloc_qubits(FRONTIER_BITS - 2);

    round665_emit_high_add_body_with_growth_tap(
        &mut b,
        &src,
        &dst,
        &frontier,
        step_ctrl,
        growth_flag,
        active,
        boundary,
        body_ctrl,
        c_in,
        body_scratch,
        &eq_scratch,
    );

    b.set_phase("round665_free_growth_tap_scratch");
    b.free_vec(&eq_scratch);
    b.free(body_scratch);
    b.free(c_in);
    b.free(body_ctrl);
    b.free(boundary);
    b.free(active);
    b
}

pub fn build_round665_high_add_growth_tap_component() -> Vec<Op> {
    build_round665_high_add_growth_tap_builder().ops
}

pub fn build_round665_high_add_growth_tap_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round665_high_add_growth_tap_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}
