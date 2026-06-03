//! `frontier::cswap` — verbatim split of the original `frontier` module.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

pub(crate) fn round666_emit_lane_cswap(b: &mut B, swap_ctrl: QubitId, lane_a: &[QubitId], lane_b: &[QubitId]) {
    debug_assert_eq!(lane_a.len(), lane_b.len());
    b.set_phase("round666_borrow_recovery_lane_swap");
    for i in 0..lane_a.len() {
        cswap(b, swap_ctrl, lane_a[i], lane_b[i]);
    }
}

pub(crate) fn round634_emit_step3_cswap(
    b: &mut B,
    swap_ctrl: QubitId,
    lane_a: &[QubitId],
    lane_b: &[QubitId],
    start_a: &[QubitId],
    start_b: &[QubitId],
) {
    debug_assert_eq!(lane_a.len(), lane_b.len());
    debug_assert_eq!(start_a.len(), start_b.len());

    b.set_phase("round634_step3_cswap_packed_lanes");
    for i in 0..lane_a.len() {
        cswap(b, swap_ctrl, lane_a[i], lane_b[i]);
    }

    b.set_phase("round634_step3_cswap_frontiers");
    for i in 0..start_a.len() {
        cswap(b, swap_ctrl, start_a[i], start_b[i]);
    }
}

pub(crate) fn round648_emit_step9_cswap(
    b: &mut B,
    swap_ctrl: QubitId,
    lane_a: &[QubitId],
    lane_b: &[QubitId],
    start_a: &[QubitId],
    start_b: &[QubitId],
) {
    debug_assert_eq!(lane_a.len(), lane_b.len());
    debug_assert_eq!(start_a.len(), start_b.len());

    b.set_phase("round648_step9_cswap_packed_lanes");
    for i in 0..lane_a.len() {
        cswap(b, swap_ctrl, lane_a[i], lane_b[i]);
    }

    b.set_phase("round648_step9_cswap_frontiers");
    for i in 0..start_a.len() {
        cswap(b, swap_ctrl, start_a[i], start_b[i]);
    }
}

pub(crate) fn round654_emit_step2_gt_bundle(
    b: &mut B,
    lane_a: &[QubitId],
    lane_b: &[QubitId],
    start_a: &[QubitId],
    start_b: &[QubitId],
    step_ctrl: QubitId,
    common_gt: QubitId,
    gt_flag: QubitId,
) {
    const FRONTIER_BITS: usize = 9;
    const MAX_DELTA: usize = 12;

    let min_start = b.alloc_qubits(FRONTIER_BITS);
    let rhs_lt_lhs = b.alloc_qubit();
    let active = b.alloc_qubit();
    let body_ctrl = b.alloc_qubit();
    let c_in = b.alloc_qubit();
    let body_scratch = b.alloc_qubit();
    let eq_scratch = b.alloc_qubits(FRONTIER_BITS - 2);

    round634_emit_min_frontier_compute(b, start_a, start_b, &min_start, rhs_lt_lhs);
    round651_compute_common_frontier_gt(
        b,
        lane_a,
        lane_b,
        &min_start,
        step_ctrl,
        common_gt,
        active,
        body_ctrl,
        c_in,
        body_scratch,
        &eq_scratch,
    );
    round634_emit_min_frontier_uncompute(b, start_a, start_b, &min_start, rhs_lt_lhs);

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
        b,
        lane_a,
        start_a,
        start_b,
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
        b,
        lane_b,
        start_a,
        start_b,
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
        b,
        start_a,
        start_b,
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
        b,
        start_a,
        start_b,
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
}
