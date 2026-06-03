//! `low::round190_199` — verbatim split of the original `low` module.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

pub(crate) fn emit_round190_selector_fused_source_live_residual_digit(
    b: &mut B,
    predecessor: &[QubitId],
    addend: &[QubitId],
    target: &[QubitId],
) {
    let width = predecessor.len();
    debug_assert_eq!(addend.len(), width);
    debug_assert_eq!(target.len(), width);
    debug_assert!(width >= 2);

    let branch = b.alloc_qubit();
    let op_sign = b.alloc_qubit();
    let carry = b.alloc_qubit();
    let gated = b.alloc_qubits(width);

    let pred0 = predecessor[0];
    let pred1 = predecessor[1];

    b.set_phase("round190_compute_source_live_selector");
    b.cx(pred0, branch);
    b.cx(pred1, branch);
    b.ccx(pred0, pred1, branch);

    b.x(op_sign);
    b.cx(pred1, op_sign);

    b.set_phase("round190_gate_addend_by_selector");
    for i in 0..width {
        b.ccx(branch, addend[i], gated[i]);
    }

    b.set_phase("round190_signed_digit_target_update");
    emit_round190_direct_centered_signed_digit_body(b, op_sign, &gated, target, carry);

    b.set_phase("round190_hmr_ungate_addend_by_selector");
    for i in 0..width {
        let m = b.alloc_bit();
        b.hmr(gated[i], m);
        b.cz_if(branch, addend[i], m);
    }

    b.set_phase("round190_uncompute_source_live_selector");
    b.cx(pred1, op_sign);
    b.x(op_sign);

    b.ccx(pred0, pred1, branch);
    b.cx(pred1, branch);
    b.cx(pred0, branch);

    b.set_phase("round190_free_selector_scratch");
    b.free_vec(&gated);
    b.free(carry);
    b.free(op_sign);
    b.free(branch);
}

pub(crate) fn emit_round190_active_source_live_signed_digit_hmr(
    b: &mut B,
    predecessor: &[QubitId],
    addend: &[QubitId],
    target: &[QubitId],
) {
    let width = predecessor.len();
    debug_assert_eq!(addend.len(), width);
    debug_assert_eq!(target.len(), width);
    debug_assert!(width >= 2);

    let op_sign = b.alloc_qubit();
    let c_in = b.alloc_qubit();
    let carries = b.alloc_qubits(width - 1);
    let pred_sign = predecessor[1];

    b.set_phase("round190_active_compute_source_live_sign");
    b.x(op_sign);
    b.cx(pred_sign, op_sign);

    b.set_phase("round190_active_hmr_signed_digit_update");
    b.x(op_sign);
    for &wire in addend {
        b.cx(op_sign, wire);
    }
    b.cx(op_sign, c_in);
    b.x(op_sign);

    cuccaro_add_fast_borrowed_carries(b, addend, target, c_in, &carries);

    b.x(op_sign);
    b.cx(op_sign, c_in);
    for &wire in addend.iter().rev() {
        b.cx(op_sign, wire);
    }
    b.x(op_sign);

    b.set_phase("round190_active_uncompute_source_live_sign");
    b.cx(pred_sign, op_sign);
    b.x(op_sign);

    b.set_phase("round190_active_free_hmr_scratch");
    b.free_vec(&carries);
    b.free(c_in);
    b.free(op_sign);
}

pub(crate) fn emit_round190_external_active_signed_digit(
    b: &mut B,
    active: QubitId,
    sign: QubitId,
    addend: &[QubitId],
    target: &[QubitId],
) {
    let width = addend.len();
    debug_assert_eq!(target.len(), width);
    debug_assert!(width >= 2);

    let op_sign = b.alloc_qubit();
    let carry = b.alloc_qubit();
    let gated = b.alloc_qubits(width);

    emit_round190_external_active_signed_digit_with_scratch(
        b, active, sign, addend, target, op_sign, carry, &gated,
    );

    b.set_phase("round190_external_active_free_scratch");
    b.free_vec(&gated);
    b.free(carry);
    b.free(op_sign);
}

pub(crate) fn emit_round190_external_active_signed_digit_with_scratch(
    b: &mut B,
    active: QubitId,
    sign: QubitId,
    addend: &[QubitId],
    target: &[QubitId],
    op_sign: QubitId,
    carry: QubitId,
    gated: &[QubitId],
) {
    let width = addend.len();
    debug_assert_eq!(target.len(), width);
    debug_assert_eq!(gated.len(), width);
    debug_assert!(width >= 2);

    b.set_phase("round190_external_active_compute_sign");
    b.x(op_sign);
    b.cx(sign, op_sign);

    b.set_phase("round190_external_active_gate_addend");
    for i in 0..width {
        b.ccx(active, addend[i], gated[i]);
    }

    b.set_phase("round190_external_active_signed_digit_update");
    emit_round190_direct_centered_signed_digit_body(b, op_sign, &gated, target, carry);

    b.set_phase("round190_external_active_hmr_ungate_addend");
    for i in 0..width {
        let m = b.alloc_bit();
        b.hmr(gated[i], m);
        b.cz_if(active, addend[i], m);
    }

    b.set_phase("round190_external_active_uncompute_sign");
    b.cx(sign, op_sign);
    b.x(op_sign);
}

pub(crate) fn emit_round190_shared_active_external_signed_digits(
    b: &mut B,
    predecessor: &[QubitId],
    addends: &[Vec<QubitId>],
    targets: &[Vec<QubitId>],
) {
    let width = predecessor.len();
    debug_assert!(width >= 2);
    debug_assert_eq!(addends.len(), targets.len());
    debug_assert!(!addends.is_empty());
    for (addend, target) in addends.iter().zip(targets.iter()) {
        debug_assert_eq!(addend.len(), width);
        debug_assert_eq!(target.len(), width);
    }

    let branch = b.alloc_qubit();
    let op_sign = b.alloc_qubit();
    let carry = b.alloc_qubit();
    let gated = b.alloc_qubits(width);
    let pred0 = predecessor[0];
    let pred1 = predecessor[1];

    b.set_phase("round190_shared_active_compute_source_live_selector");
    b.cx(pred0, branch);
    b.cx(pred1, branch);
    b.ccx(pred0, pred1, branch);

    for (addend, target) in addends.iter().zip(targets.iter()) {
        emit_round190_external_active_signed_digit_with_scratch(
            b, branch, pred1, addend, target, op_sign, carry, &gated,
        );
    }

    b.set_phase("round190_shared_active_uncompute_source_live_selector");
    b.ccx(pred0, pred1, branch);
    b.cx(pred1, branch);
    b.cx(pred0, branch);

    b.set_phase("round190_shared_active_free_scratch");
    b.free_vec(&gated);
    b.free(carry);
    b.free(op_sign);
    b.free(branch);
}

pub(crate) fn emit_round190_two_slot_exactly_one_active_router(
    b: &mut B,
    predecessor0: &[QubitId],
    addend0: &[QubitId],
    target0: &[QubitId],
    predecessor1: &[QubitId],
    addend1: &[QubitId],
    target1: &[QubitId],
) {
    let width = predecessor0.len();
    debug_assert!(width >= 2);
    debug_assert_eq!(predecessor1.len(), width);
    debug_assert_eq!(addend0.len(), width);
    debug_assert_eq!(addend1.len(), width);
    debug_assert_eq!(target0.len(), width);
    debug_assert_eq!(target1.len(), width);

    let take_second = b.alloc_qubit();
    let pred1_0 = predecessor1[0];
    let pred1_1 = predecessor1[1];

    b.set_phase("round190_two_slot_compute_second_active");
    b.cx(pred1_0, take_second);
    b.cx(pred1_1, take_second);
    b.ccx(pred1_0, pred1_1, take_second);

    b.set_phase("round190_two_slot_route_active_to_slot0");
    for i in 0..width {
        cswap(b, take_second, predecessor0[i], predecessor1[i]);
        cswap(b, take_second, addend0[i], addend1[i]);
        cswap(b, take_second, target0[i], target1[i]);
    }

    b.set_phase("round190_two_slot_active_hmr_update");
    emit_round190_active_source_live_signed_digit_hmr(b, predecessor0, addend0, target0);

    b.set_phase("round190_two_slot_unroute_active_from_slot0");
    for i in (0..width).rev() {
        cswap(b, take_second, target0[i], target1[i]);
        cswap(b, take_second, addend0[i], addend1[i]);
        cswap(b, take_second, predecessor0[i], predecessor1[i]);
    }

    b.set_phase("round190_two_slot_uncompute_second_active");
    b.ccx(pred1_0, pred1_1, take_second);
    b.cx(pred1_1, take_second);
    b.cx(pred1_0, take_second);

    b.set_phase("round190_two_slot_free_router_scratch");
    b.free(take_second);
}

pub(crate) fn build_round190_selector_fused_source_live_residual_builder(width: usize) -> B {
    assert!(
        width >= 2,
        "Round190 selector-fused residual width must be >= 2"
    );
    let mut b = B::new();
    let predecessor = b.alloc_qubits(width);
    b.declare_qubit_register(&predecessor);
    let addend = b.alloc_qubits(width);
    b.declare_qubit_register(&addend);
    let target = b.alloc_qubits(width);
    b.declare_qubit_register(&target);

    b.set_phase("round190_selector_fused_source_live_residual");
    emit_round190_selector_fused_source_live_residual_digit(&mut b, &predecessor, &addend, &target);
    b
}

pub(crate) fn build_round190_active_source_live_signed_digit_hmr_builder(width: usize) -> B {
    assert!(
        width >= 2,
        "Round190 active source-live digit width must be >= 2"
    );
    let mut b = B::new();
    let predecessor = b.alloc_qubits(width);
    b.declare_qubit_register(&predecessor);
    let addend = b.alloc_qubits(width);
    b.declare_qubit_register(&addend);
    let target = b.alloc_qubits(width);
    b.declare_qubit_register(&target);

    b.set_phase("round190_active_source_live_signed_digit_hmr");
    emit_round190_active_source_live_signed_digit_hmr(&mut b, &predecessor, &addend, &target);
    b
}

pub(crate) fn build_round190_external_active_signed_digit_builder(width: usize) -> B {
    assert!(
        width >= 2,
        "Round190 external-active digit width must be >= 2"
    );
    let mut b = B::new();
    let active = b.alloc_qubits(1);
    b.declare_qubit_register(&active);
    let sign = b.alloc_qubits(1);
    b.declare_qubit_register(&sign);
    let addend = b.alloc_qubits(width);
    b.declare_qubit_register(&addend);
    let target = b.alloc_qubits(width);
    b.declare_qubit_register(&target);

    b.set_phase("round190_external_active_signed_digit");
    emit_round190_external_active_signed_digit(&mut b, active[0], sign[0], &addend, &target);
    b
}

pub(crate) fn build_round190_shared_active_external_signed_digits_builder(width: usize, digits: usize) -> B {
    assert!(
        width >= 2,
        "Round190 shared-active digit width must be >= 2"
    );
    assert!(digits >= 1, "Round190 shared-active digits must be nonzero");
    let mut b = B::new();
    let predecessor = b.alloc_qubits(width);
    b.declare_qubit_register(&predecessor);
    let mut addends = Vec::with_capacity(digits);
    let mut targets = Vec::with_capacity(digits);
    for _ in 0..digits {
        let addend = b.alloc_qubits(width);
        b.declare_qubit_register(&addend);
        let target = b.alloc_qubits(width);
        b.declare_qubit_register(&target);
        addends.push(addend);
        targets.push(target);
    }

    b.set_phase("round190_shared_active_external_signed_digits");
    emit_round190_shared_active_external_signed_digits(&mut b, &predecessor, &addends, &targets);
    b
}

pub(crate) fn build_round190_two_slot_exactly_one_active_router_builder(width: usize) -> B {
    assert!(width >= 2, "Round190 two-slot router width must be >= 2");
    let mut b = B::new();
    let predecessor0 = b.alloc_qubits(width);
    b.declare_qubit_register(&predecessor0);
    let addend0 = b.alloc_qubits(width);
    b.declare_qubit_register(&addend0);
    let target0 = b.alloc_qubits(width);
    b.declare_qubit_register(&target0);
    let predecessor1 = b.alloc_qubits(width);
    b.declare_qubit_register(&predecessor1);
    let addend1 = b.alloc_qubits(width);
    b.declare_qubit_register(&addend1);
    let target1 = b.alloc_qubits(width);
    b.declare_qubit_register(&target1);

    b.set_phase("round190_two_slot_exactly_one_active_router");
    emit_round190_two_slot_exactly_one_active_router(
        &mut b,
        &predecessor0,
        &addend0,
        &target0,
        &predecessor1,
        &addend1,
        &target1,
    );
    b
}

pub fn build_round190_selector_fused_source_live_residual_width(width: usize) -> Vec<Op> {
    build_round190_selector_fused_source_live_residual_builder(width).ops
}

pub fn build_round190_active_source_live_signed_digit_hmr_width(width: usize) -> Vec<Op> {
    build_round190_active_source_live_signed_digit_hmr_builder(width).ops
}

pub fn build_round190_external_active_signed_digit_width(width: usize) -> Vec<Op> {
    build_round190_external_active_signed_digit_builder(width).ops
}

pub fn build_round190_shared_active_external_signed_digits_width(
    width: usize,
    digits: usize,
) -> Vec<Op> {
    build_round190_shared_active_external_signed_digits_builder(width, digits).ops
}

pub fn build_round190_two_slot_exactly_one_active_router_width(width: usize) -> Vec<Op> {
    build_round190_two_slot_exactly_one_active_router_builder(width).ops
}

pub fn build_round190_selector_fused_source_live_residual_component() -> Vec<Op> {
    build_round190_selector_fused_source_live_residual_builder(
        round190_selector_fused_width_from_env(),
    )
    .ops
}

pub fn build_round190_active_source_live_signed_digit_hmr_component() -> Vec<Op> {
    build_round190_active_source_live_signed_digit_hmr_builder(
        round190_selector_fused_width_from_env(),
    )
    .ops
}

pub fn build_round190_external_active_signed_digit_component() -> Vec<Op> {
    build_round190_external_active_signed_digit_builder(round190_selector_fused_width_from_env())
        .ops
}

pub fn build_round190_shared_active_external_signed_digits_component(digits: usize) -> Vec<Op> {
    build_round190_shared_active_external_signed_digits_builder(
        round190_selector_fused_width_from_env(),
        digits,
    )
    .ops
}

pub fn build_round190_two_slot_exactly_one_active_router_component() -> Vec<Op> {
    build_round190_two_slot_exactly_one_active_router_builder(
        round190_selector_fused_width_from_env(),
    )
    .ops
}

pub fn build_round190_selector_fused_source_live_residual_phase_resources_width(
    width: usize,
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round190_selector_fused_source_live_residual_builder(width);
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub fn build_round190_active_source_live_signed_digit_hmr_phase_resources_width(
    width: usize,
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round190_active_source_live_signed_digit_hmr_builder(width);
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub fn build_round190_external_active_signed_digit_phase_resources_width(
    width: usize,
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round190_external_active_signed_digit_builder(width);
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub fn build_round190_shared_active_external_signed_digits_phase_resources_width(
    width: usize,
    digits: usize,
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round190_shared_active_external_signed_digits_builder(width, digits);
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub fn build_round190_two_slot_exactly_one_active_router_phase_resources_width(
    width: usize,
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round190_two_slot_exactly_one_active_router_builder(width);
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub fn build_round190_selector_fused_source_live_residual_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    build_round190_selector_fused_source_live_residual_phase_resources_width(
        round190_selector_fused_width_from_env(),
    )
}

pub fn build_round190_active_source_live_signed_digit_hmr_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    build_round190_active_source_live_signed_digit_hmr_phase_resources_width(
        round190_selector_fused_width_from_env(),
    )
}

pub fn build_round190_external_active_signed_digit_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    build_round190_external_active_signed_digit_phase_resources_width(
        round190_selector_fused_width_from_env(),
    )
}

pub fn build_round190_shared_active_external_signed_digits_phase_resources(
    digits: usize,
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    build_round190_shared_active_external_signed_digits_phase_resources_width(
        round190_selector_fused_width_from_env(),
        digits,
    )
}

pub fn build_round190_two_slot_exactly_one_active_router_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    build_round190_two_slot_exactly_one_active_router_phase_resources_width(
        round190_selector_fused_width_from_env(),
    )
}

pub fn round198_semantic_coeff_clean_sequence_register_widths() -> [usize; 5] {
    let (lane_width, coeff_width, q_bits) =
        round158_halfgcd_splice_live::round198_semantic_coeff_clean_sequence_widths();
    [lane_width, lane_width, coeff_width, coeff_width, q_bits]
}

pub(crate) fn build_round198_semantic_coeff_clean_sequence_builder(overflow_aware: bool) -> B {
    let (lane_width, coeff_width, q_bits) =
        round158_halfgcd_splice_live::round198_semantic_coeff_clean_sequence_widths();
    let mut b = B::new();
    let u = b.alloc_qubits(lane_width);
    b.declare_qubit_register(&u);
    let v = b.alloc_qubits(lane_width);
    b.declare_qubit_register(&v);
    let coeff_b = b.alloc_qubits(coeff_width);
    b.declare_qubit_register(&coeff_b);
    let coeff_d = b.alloc_qubits(coeff_width);
    b.declare_qubit_register(&coeff_d);
    let q = b.alloc_qubits(q_bits);
    b.declare_qubit_register(&q);
    b.x(coeff_d[0]);

    round158_halfgcd_splice_live::emit_round198_semantic_coeff_clean_prefix_sequence(
        &mut b,
        &u,
        &v,
        &coeff_b,
        &coeff_d,
        &q,
        overflow_aware,
    );

    b
}

pub fn build_round198_semantic_coeff_clean_sequence_phase_resources(
    overflow_aware: bool,
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round198_semantic_coeff_clean_sequence_builder(overflow_aware);
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub fn build_round198_semantic_coeff_clean_sequence(overflow_aware: bool) -> Vec<Op> {
    build_round198_semantic_coeff_clean_sequence_builder(overflow_aware).ops
}

pub fn round199_semantic_full_gcd_prefix_register_widths() -> [usize; 5] {
    let (lane_width, coeff_width, total_q_bits, _max_q_bits, _steps) =
        round158_halfgcd_splice_live::round199_semantic_full_gcd_prefix_widths();
    [
        lane_width,
        lane_width,
        coeff_width,
        coeff_width,
        total_q_bits,
    ]
}

pub(crate) fn build_round199_semantic_full_gcd_prefix_builder(roundtrip: bool) -> B {
    let (lane_width, coeff_width, total_q_bits, _max_q_bits, _steps) =
        round158_halfgcd_splice_live::round199_semantic_full_gcd_prefix_widths();
    let mut b = B::new();
    let u = b.alloc_qubits(lane_width);
    b.declare_qubit_register(&u);
    for i in 0..lane_width {
        if bit(SECP256K1_P, i) {
            b.x(u[i]);
        }
    }
    let v = b.alloc_qubits(lane_width);
    b.declare_qubit_register(&v);
    let coeff_b = b.alloc_qubits(coeff_width);
    b.declare_qubit_register(&coeff_b);
    let coeff_d = b.alloc_qubits(coeff_width);
    b.declare_qubit_register(&coeff_d);
    let q_tail = b.alloc_qubits(total_q_bits);
    b.declare_qubit_register(&q_tail);
    b.x(coeff_d[0]);

    if roundtrip {
        round158_halfgcd_splice_live::emit_round199_semantic_full_gcd_prefix_roundtrip(
            &mut b, &u, &v, &coeff_b, &coeff_d, &q_tail,
        );
    } else {
        round158_halfgcd_splice_live::emit_round199_semantic_full_gcd_prefix_sequence(
            &mut b, &u, &v, &coeff_b, &coeff_d, &q_tail,
        );
    }

    b
}

pub fn build_round199_semantic_full_gcd_prefix_phase_resources(
    roundtrip: bool,
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round199_semantic_full_gcd_prefix_builder(roundtrip);
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub fn build_round199_semantic_full_gcd_prefix(roundtrip: bool) -> Vec<Op> {
    build_round199_semantic_full_gcd_prefix_builder(roundtrip).ops
}
