//! `frontier::builders3` — verbatim split of the original `frontier` module.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

pub(crate) fn build_round654_step2_gt_bundle_builder() -> B {
    const FRONTIER_BITS: usize = 9;
    const PACKED_WIDTH: usize = N + 1;
    const MAX_DELTA: usize = 12;

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
    let common_gt = b.alloc_qubit();
    let gt_flag = b.alloc_qubit();
    b.declare_qubit_register(&[step_ctrl, common_gt, gt_flag]);

    let min_start = b.alloc_qubits(FRONTIER_BITS);
    let rhs_lt_lhs = b.alloc_qubit();
    let active = b.alloc_qubit();
    let body_ctrl = b.alloc_qubit();
    let c_in = b.alloc_qubit();
    let body_scratch = b.alloc_qubit();
    let eq_scratch = b.alloc_qubits(FRONTIER_BITS - 2);

    round634_emit_min_frontier_compute(&mut b, &start_a, &start_b, &min_start, rhs_lt_lhs);
    round651_compute_common_frontier_gt(
        &mut b,
        &lane_a,
        &lane_b,
        &min_start,
        step_ctrl,
        common_gt,
        active,
        body_ctrl,
        c_in,
        body_scratch,
        &eq_scratch,
    );
    round634_emit_min_frontier_uncompute(&mut b, &start_a, &start_b, &min_start, rhs_lt_lhs);

    b.set_phase("round654_free_common_compare_scratch");
    b.free_vec(&eq_scratch);
    b.free(body_scratch);
    b.free(c_in);
    b.free(body_ctrl);
    b.free(active);
    b.free(rhs_lt_lhs);
    b.free_vec(&min_start);

    b.set_phase("round654_seed_gt_from_common");
    b.cx(common_gt, gt_flag);

    let eq_a = b.alloc_qubit();
    let eq_b = b.alloc_qubit();
    let pair_ctrl = b.alloc_qubit();
    let range_tmp = b.alloc_qubit();
    let body_scratch = b.alloc_qubit();
    let eq_scratch = b.alloc_qubits(FRONTIER_BITS - 2);
    let or_chain = b.alloc_qubits(MAX_DELTA - 1);

    b.set_phase("round654_a_upper_bounded_gt_correction");
    round654_emit_bounded_final_pairs(
        &mut b,
        &lane_a,
        &start_a,
        &start_b,
        step_ctrl,
        common_gt,
        gt_flag,
        true,
        eq_a,
        eq_b,
        pair_ctrl,
        range_tmp,
        body_scratch,
        &eq_scratch,
        &or_chain,
    );

    b.set_phase("round654_b_upper_bounded_gt_correction");
    round654_emit_bounded_final_pairs(
        &mut b,
        &lane_b,
        &start_a,
        &start_b,
        step_ctrl,
        common_gt,
        gt_flag,
        false,
        eq_a,
        eq_b,
        pair_ctrl,
        range_tmp,
        body_scratch,
        &eq_scratch,
        &or_chain,
    );

    b.set_phase("round654_a_upper_overcutoff_gt_correction");
    round654_emit_overcutoff_final_pairs(
        &mut b,
        &start_a,
        &start_b,
        step_ctrl,
        common_gt,
        gt_flag,
        true,
        eq_a,
        eq_b,
        pair_ctrl,
        body_scratch,
        &eq_scratch,
    );

    b.set_phase("round654_b_upper_overcutoff_gt_correction");
    round654_emit_overcutoff_final_pairs(
        &mut b,
        &start_a,
        &start_b,
        step_ctrl,
        common_gt,
        gt_flag,
        false,
        eq_a,
        eq_b,
        pair_ctrl,
        body_scratch,
        &eq_scratch,
    );

    b.set_phase("round654_free_step2_gt_bundle_scratch");
    b.free_vec(&or_chain);
    b.free_vec(&eq_scratch);
    b.free(body_scratch);
    b.free(range_tmp);
    b.free(pair_ctrl);
    b.free(eq_b);
    b.free(eq_a);
    b
}

pub fn build_round654_step2_gt_bundle_component() -> Vec<Op> {
    build_round654_step2_gt_bundle_builder().ops
}

pub fn build_round654_step2_gt_bundle_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round654_step2_gt_bundle_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub(crate) fn build_round655_step1_step2_control_front_builder() -> B {
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
    let f_flag = b.alloc_qubit();
    let m_i = b.alloc_qubit();
    let a_f = b.alloc_qubit();
    let b_f = b.alloc_qubit();
    let add_f = b.alloc_qubit();
    b.declare_qubit_register(&[f_flag, m_i, a_f, b_f, add_f]);

    b.set_phase("round655_step1_flags");
    b.ccx(f_flag, lane_a[0], b_f);
    b.cx(f_flag, a_f);
    b.cx(b_f, a_f);
    b.x(lane_b[0]);
    b.ccx(b_f, lane_b[0], m_i);
    b.x(lane_b[0]);
    {
        let m = b.alloc_bit();
        b.hmr(b_f, m);
        b.cz_if(f_flag, lane_a[0], m);
    }
    b.cx(a_f, b_f);
    b.cx(m_i, b_f);

    let common_gt = b.alloc_qubit();
    let gt_flag = b.alloc_qubit();
    round654_emit_step2_gt_bundle(
        &mut b, &lane_a, &lane_b, &start_a, &start_b, f_flag, common_gt, gt_flag,
    );

    b.set_phase("round655_step2_apply_gt_delta");
    b.x(b_f);
    let delta = b.alloc_qubit();
    b.ccx(gt_flag, b_f, delta);
    b.cx(delta, a_f);
    b.cx(delta, m_i);
    {
        let m = b.alloc_bit();
        b.hmr(delta, m);
        b.cz_if(gt_flag, b_f, m);
    }
    b.x(b_f);
    b.free(delta);

    b.set_phase("round655_uncompute_step2_gt_bundle");
    emit_inverse(&mut b, |b| {
        round654_emit_step2_gt_bundle(
            b, &lane_a, &lane_b, &start_a, &start_b, f_flag, common_gt, gt_flag,
        );
    });
    b.free(gt_flag);
    b.free(common_gt);

    b.set_phase("round655_compute_step4_add_ctrl");
    mcx2_polar(&mut b, f_flag, true, b_f, false, add_f);

    b
}

pub fn build_round655_step1_step2_control_front_component() -> Vec<Op> {
    build_round655_step1_step2_control_front_builder().ops
}

pub fn build_round655_step1_step2_control_front_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round655_step1_step2_control_front_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}
