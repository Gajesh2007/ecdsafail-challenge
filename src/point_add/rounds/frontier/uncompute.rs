//! `frontier::uncompute` — verbatim split of the original `frontier` module.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

pub(crate) fn round630_emit_threshold_compute_uncompute(
    b: &mut B,
    frontier: &[QubitId],
    flag: QubitId,
    c: usize,
) {
    let c = U256::from(c as u64);
    cmp_gt_const_n1(b, frontier, c, flag);
    cmp_gt_const_n1(b, frontier, c, flag);
}

pub(crate) fn round630_emit_boundary_pair_compute_uncompute(
    b: &mut B,
    frontier: &[QubitId],
    lower_flag: QubitId,
    upper_flag: QubitId,
    c: usize,
) {
    cmp_gt_const_n1(b, frontier, U256::from(c as u64), lower_flag);
    cmp_gt_const_n1(b, frontier, U256::from((c + 1) as u64), upper_flag);
    cmp_gt_const_n1(b, frontier, U256::from((c + 1) as u64), upper_flag);
    cmp_gt_const_n1(b, frontier, U256::from(c as u64), lower_flag);
}

pub(crate) fn round632_uncompute_body_ctrl(b: &mut B, step_ctrl: QubitId, local: QubitId, body_ctrl: QubitId) {
    b.ccx(step_ctrl, local, body_ctrl);
}

pub(crate) fn round633_emit_min_frontier_compute_uncompute(
    b: &mut B,
    lhs: &[QubitId],
    rhs: &[QubitId],
    min_out: &[QubitId],
    rhs_lt_lhs: QubitId,
) {
    debug_assert_eq!(lhs.len(), rhs.len());
    debug_assert_eq!(lhs.len(), min_out.len());

    b.set_phase("round633_copy_lhs_frontier_to_min");
    for i in 0..lhs.len() {
        b.cx(lhs[i], min_out[i]);
    }

    b.set_phase("round633_compute_rhs_lt_lhs");
    cmp_lt_into(b, rhs, lhs, rhs_lt_lhs);

    b.set_phase("round633_mux_rhs_into_min");
    for i in 0..lhs.len() {
        b.ccx(rhs_lt_lhs, lhs[i], min_out[i]);
        b.ccx(rhs_lt_lhs, rhs[i], min_out[i]);
    }

    b.set_phase("round633_unmux_rhs_from_min");
    for i in (0..lhs.len()).rev() {
        b.ccx(rhs_lt_lhs, rhs[i], min_out[i]);
        b.ccx(rhs_lt_lhs, lhs[i], min_out[i]);
    }

    b.set_phase("round633_uncompute_rhs_lt_lhs");
    cmp_lt_into(b, rhs, lhs, rhs_lt_lhs);

    b.set_phase("round633_clear_min");
    for i in (0..lhs.len()).rev() {
        b.cx(lhs[i], min_out[i]);
    }
}

pub(crate) fn round634_emit_min_frontier_uncompute(
    b: &mut B,
    lhs: &[QubitId],
    rhs: &[QubitId],
    min_out: &[QubitId],
    rhs_lt_lhs: QubitId,
) {
    debug_assert_eq!(lhs.len(), rhs.len());
    debug_assert_eq!(lhs.len(), min_out.len());

    b.set_phase("round634_unmux_rhs_from_min");
    for i in (0..lhs.len()).rev() {
        b.ccx(rhs_lt_lhs, rhs[i], min_out[i]);
        b.ccx(rhs_lt_lhs, lhs[i], min_out[i]);
    }

    b.set_phase("round634_uncompute_rhs_lt_lhs");
    cmp_lt_into(b, rhs, lhs, rhs_lt_lhs);

    b.set_phase("round634_clear_min");
    for i in (0..lhs.len()).rev() {
        b.cx(lhs[i], min_out[i]);
    }
}

pub(crate) fn round653_uncompute_pair_ctrl(
    b: &mut B,
    start_a: &[QubitId],
    start_b: &[QubitId],
    value_a: usize,
    value_b: usize,
    step_ctrl: QubitId,
    eq_a: QubitId,
    eq_b: QubitId,
    pair_ctrl: QubitId,
    body_scratch: QubitId,
    eq_scratch: &[QubitId],
) {
    mcx3_polar(
        b,
        step_ctrl,
        true,
        eq_a,
        true,
        eq_b,
        true,
        pair_ctrl,
        body_scratch,
    );
    round631_emit_eq_const_toggle(b, start_b, value_b, eq_b, eq_scratch);
    round631_emit_eq_const_toggle(b, start_a, value_a, eq_a, eq_scratch);
}
