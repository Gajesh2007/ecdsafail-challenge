//! `frontier::sidecar` — verbatim split of the original `frontier` module.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

pub(crate) fn build_round633_frontier_min_sidecar_builder() -> B {
    const FRONTIER_BITS: usize = 9;

    let mut b = B::new();
    let lhs = b.alloc_qubits(FRONTIER_BITS);
    b.declare_qubit_register(&lhs);
    let rhs = b.alloc_qubits(FRONTIER_BITS);
    b.declare_qubit_register(&rhs);
    let min_out = b.alloc_qubits(FRONTIER_BITS);
    b.declare_qubit_register(&min_out);
    let rhs_lt_lhs = b.alloc_qubit();
    b.declare_qubit_register(&[rhs_lt_lhs]);

    round633_emit_min_frontier_compute_uncompute(&mut b, &lhs, &rhs, &min_out, rhs_lt_lhs);
    b
}

pub fn build_round633_frontier_min_sidecar_component() -> Vec<Op> {
    build_round633_frontier_min_sidecar_builder().ops
}

pub fn build_round633_frontier_min_sidecar_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round633_frontier_min_sidecar_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

// Round640 red-team artifact only.  The emitted excluded-target equality clear
// is measured in Rust but rejected by fresh_start_pa Round640 as semantically
// invalid; do not integrate this helper into a PA emitter.
pub(crate) fn round640_emit_xor_diff_sidecar_update(
    b: &mut B,
    dst_high: &[QubitId],
    dst_start: &[QubitId],
    src_start: &[QubitId],
    add_ctrl: QubitId,
    old_start: &[QubitId],
    branch: QubitId,
    update_ctrl: QubitId,
    source_eq: QubitId,
    scratch_eq: QubitId,
    body_scratch: QubitId,
    eq_scratch: &[QubitId],
) {
    const FRONTIER_BITS: usize = 9;
    const PACKED_WIDTH: usize = N + 1;
    const WINDOW: usize = 10;
    debug_assert_eq!(dst_high.len(), PACKED_WIDTH);
    debug_assert_eq!(dst_start.len(), FRONTIER_BITS);
    debug_assert_eq!(src_start.len(), FRONTIER_BITS);
    debug_assert_eq!(old_start.len(), FRONTIER_BITS);
    debug_assert!(eq_scratch.len() >= FRONTIER_BITS - 2);

    b.set_phase("round640_copy_old_dst_start");
    for i in 0..FRONTIER_BITS {
        b.cx(dst_start[i], old_start[i]);
    }

    b.set_phase("round640_compute_old_dst_gt_src");
    cmp_gt_into(b, old_start, src_start, branch);

    b.set_phase("round640_rewrite_dst_start_to_src");
    b.ccx(add_ctrl, branch, update_ctrl);
    for i in 0..FRONTIER_BITS {
        b.ccx(update_ctrl, old_start[i], dst_start[i]);
        b.ccx(update_ctrl, src_start[i], dst_start[i]);
    }
    b.ccx(add_ctrl, branch, update_ctrl);

    b.set_phase("round640_uncompute_old_dst_gt_src");
    cmp_gt_into(b, old_start, src_start, branch);

    b.set_phase("round640_xor_rewritten_dst_start_into_old_start");
    for i in 0..FRONTIER_BITS {
        b.cx(dst_start[i], old_start[i]);
    }

    b.set_phase("round640_xor_diff_scrub_window");
    for src_value in 0..PACKED_WIDTH {
        round631_emit_eq_const_toggle(b, src_start, src_value, source_eq, eq_scratch);
        for delta in 1..=WINDOW {
            let candidate = src_value + delta;
            if candidate <= PACKED_WIDTH - 1 {
                let diff = src_value ^ candidate;
                for bit in 0..FRONTIER_BITS {
                    if ((diff >> bit) & 1) == 0 {
                        continue;
                    }
                    round640_emit_eq_const_except_toggle(
                        b, old_start, bit, diff, scratch_eq, eq_scratch,
                    );
                    mcx3_polar(
                        b,
                        source_eq,
                        true,
                        scratch_eq,
                        true,
                        dst_high[candidate],
                        true,
                        old_start[bit],
                        body_scratch,
                    );
                    round640_emit_eq_const_except_toggle(
                        b, old_start, bit, diff, scratch_eq, eq_scratch,
                    );
                }
            } else if candidate == PACKED_WIDTH {
                let diff = src_value ^ PACKED_WIDTH;
                for bit in 0..FRONTIER_BITS {
                    if ((diff >> bit) & 1) == 0 {
                        continue;
                    }
                    round640_emit_eq_const_except_toggle(
                        b, old_start, bit, diff, scratch_eq, eq_scratch,
                    );
                    mcx2_polar(b, source_eq, true, scratch_eq, true, old_start[bit]);
                    round640_emit_eq_const_except_toggle(
                        b, old_start, bit, diff, scratch_eq, eq_scratch,
                    );
                }
            }
        }
        round631_emit_eq_const_toggle(b, src_start, src_value, source_eq, eq_scratch);
    }
}

pub(crate) fn build_round640_xor_diff_sidecar_scrubber_builder() -> B {
    const FRONTIER_BITS: usize = 9;
    const PACKED_WIDTH: usize = N + 1;

    let mut b = B::new();
    let dst_high = b.alloc_qubits(PACKED_WIDTH);
    b.declare_qubit_register(&dst_high);
    let dst_start = b.alloc_qubits(FRONTIER_BITS);
    b.declare_qubit_register(&dst_start);
    let src_start = b.alloc_qubits(FRONTIER_BITS);
    b.declare_qubit_register(&src_start);
    let add_ctrl = b.alloc_qubit();
    b.declare_qubit_register(&[add_ctrl]);

    let old_start = b.alloc_qubits(FRONTIER_BITS);
    let branch = b.alloc_qubit();
    let update_ctrl = b.alloc_qubit();
    let source_eq = b.alloc_qubit();
    let scratch_eq = b.alloc_qubit();
    let body_scratch = b.alloc_qubit();
    let eq_scratch = b.alloc_qubits(FRONTIER_BITS - 2);

    round640_emit_xor_diff_sidecar_update(
        &mut b,
        &dst_high,
        &dst_start,
        &src_start,
        add_ctrl,
        &old_start,
        branch,
        update_ctrl,
        source_eq,
        scratch_eq,
        body_scratch,
        &eq_scratch,
    );

    b.set_phase("round640_free_xor_diff_scrubber_scratch");
    b.free_vec(&eq_scratch);
    b.free(body_scratch);
    b.free(scratch_eq);
    b.free(source_eq);
    b.free(update_ctrl);
    b.free(branch);
    b.free_vec(&old_start);
    b
}

pub fn build_round640_xor_diff_sidecar_scrubber_component() -> Vec<Op> {
    build_round640_xor_diff_sidecar_scrubber_builder().ops
}

pub fn build_round640_xor_diff_sidecar_scrubber_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round640_xor_diff_sidecar_scrubber_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}
