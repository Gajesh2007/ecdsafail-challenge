//! `mid::round587_592` — verbatim split of the original `mid` module.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

pub(crate) fn build_round587_current_256_raw_splice_builder() -> B {
    let mut b = B::new();
    let numerator = b.alloc_qubits(N);
    b.declare_qubit_register(&numerator);
    let quotient = b.alloc_qubits(N);
    b.declare_qubit_register(&quotient);
    let inv_raw = b.alloc_qubits(N);
    b.declare_qubit_register(&inv_raw);

    b.set_phase("round587_current_256_raw_splice");
    d1_direct_quotient_arithmetic_from_raw_inverse(
        &mut b,
        &numerator,
        &quotient,
        &inv_raw,
        SECP256K1_P,
        400,
    );
    b
}

pub fn build_round587_current_256_raw_splice_component() -> Vec<Op> {
    build_round587_current_256_raw_splice_builder().ops
}

pub fn build_round587_current_256_raw_splice_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round587_current_256_raw_splice_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub(crate) fn round588_folded_solinas_raw_consumer(
    b: &mut B,
    numerator: &[QubitId],
    quotient: &[QubitId],
    raw_r: &[QubitId],
    p: U256,
    inverse_iters: usize,
) {
    debug_assert_eq!(numerator.len(), N);
    debug_assert_eq!(quotient.len(), N);
    debug_assert_eq!(raw_r.len(), 399);
    let (raw_low, raw_high) = raw_r.split_at(N);
    debug_assert_eq!(raw_high.len(), 143);

    b.set_phase("round588_folded_low_256_product");
    d1_direct_quotient_compute_product_from_raw_inverse(b, numerator, quotient, raw_low, p);

    b.set_phase("round588_folded_high_init_c_times_num");
    let high_term = b.alloc_qubits(N);
    let solinas_c = (U256::from(1u64) << 32) + U256::from(977u64);
    mul_by_const_acc(b, numerator, solinas_c, &high_term, p, false);

    b.set_phase("round588_folded_high_143_controls");
    for i in 0..raw_high.len() {
        cmod_add_qq(b, quotient, &high_term, raw_high[i], p);
        if i + 1 < raw_high.len() {
            mod_double_inplace_fast(b, &high_term, p);
        }
    }

    b.set_phase("round588_folded_high_restore_scale");
    for _ in 1..raw_high.len() {
        mod_halve_inplace_fast(b, &high_term, p);
    }

    b.set_phase("round588_folded_high_uncompute_c_times_num");
    mul_by_const_acc(b, numerator, solinas_c, &high_term, p, true);
    b.free_vec(&high_term);

    b.set_phase("round588_folded_unscale");
    d1_direct_quotient_unscale_neg_product(b, quotient, p, inverse_iters);
}

pub(crate) fn build_round588_folded_solinas_raw_splice_builder() -> B {
    let mut b = B::new();
    let numerator = b.alloc_qubits(N);
    b.declare_qubit_register(&numerator);
    let quotient = b.alloc_qubits(N);
    b.declare_qubit_register(&quotient);
    let packed_us = b.alloc_qubits(257);
    b.declare_qubit_register(&packed_us);
    let packed_vr_raw = b.alloc_qubits(399);
    b.declare_qubit_register(&packed_vr_raw);
    let m_hist = b.alloc_qubits(399);
    b.declare_qubit_register(&m_hist);

    round588_folded_solinas_raw_consumer(
        &mut b,
        &numerator,
        &quotient,
        &packed_vr_raw,
        SECP256K1_P,
        400,
    );

    let _ = (packed_us, m_hist);
    b
}

pub fn build_round588_folded_solinas_raw_splice_component() -> Vec<Op> {
    build_round588_folded_solinas_raw_splice_builder().ops
}

pub fn build_round588_folded_solinas_raw_splice_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round588_folded_solinas_raw_splice_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub(crate) fn round589_rectangular_solinas_raw_consumer(
    b: &mut B,
    numerator: &[QubitId],
    quotient: &[QubitId],
    raw_r: &[QubitId],
    p: U256,
    inverse_iters: usize,
) {
    debug_assert_eq!(numerator.len(), N);
    debug_assert_eq!(quotient.len(), N);
    debug_assert_eq!(raw_r.len(), 399);
    let (raw_low, raw_high) = raw_r.split_at(N);
    debug_assert_eq!(raw_high.len(), 143);

    b.set_phase("round589_rect_low_256_product");
    d1_direct_quotient_compute_product_from_raw_inverse(b, numerator, quotient, raw_low, p);

    b.set_phase("round589_rect_high_init_c_times_num");
    let high_term = b.alloc_qubits(N);
    let solinas_c = (U256::from(1u64) << 32) + U256::from(977u64);
    mul_by_const_acc(b, numerator, solinas_c, &high_term, p, false);

    b.set_phase("round589_rect_high_product");
    let tmp_ext = b.alloc_qubits(2 * N);
    schoolbook_rect_mul_into(b, &high_term, raw_high, &tmp_ext);

    b.set_phase("round589_rect_high_reduce");
    mod_add_solinas_ext_product(b, quotient, &tmp_ext, p);

    b.set_phase("round589_rect_high_unproduct");
    schoolbook_rect_mul_into_inverse(b, &high_term, raw_high, &tmp_ext);
    b.free_vec(&tmp_ext);

    b.set_phase("round589_rect_high_uncompute_c_times_num");
    mul_by_const_acc(b, numerator, solinas_c, &high_term, p, true);
    b.free_vec(&high_term);

    b.set_phase("round589_rect_unscale");
    d1_direct_quotient_unscale_neg_product(b, quotient, p, inverse_iters);
}

pub(crate) fn build_round589_rectangular_solinas_raw_splice_builder() -> B {
    let mut b = B::new();
    let numerator = b.alloc_qubits(N);
    b.declare_qubit_register(&numerator);
    let quotient = b.alloc_qubits(N);
    b.declare_qubit_register(&quotient);
    let packed_us = b.alloc_qubits(257);
    b.declare_qubit_register(&packed_us);
    let packed_vr_raw = b.alloc_qubits(399);
    b.declare_qubit_register(&packed_vr_raw);
    let m_hist = b.alloc_qubits(399);
    b.declare_qubit_register(&m_hist);

    round589_rectangular_solinas_raw_consumer(
        &mut b,
        &numerator,
        &quotient,
        &packed_vr_raw,
        SECP256K1_P,
        400,
    );

    let _ = (packed_us, m_hist);
    b
}

pub fn build_round589_rectangular_solinas_raw_splice_component() -> Vec<Op> {
    build_round589_rectangular_solinas_raw_splice_builder().ops
}

pub fn build_round589_rectangular_solinas_raw_splice_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round589_rectangular_solinas_raw_splice_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub(crate) fn round590_rectangular_addsub_solinas_raw_consumer(
    b: &mut B,
    numerator: &[QubitId],
    quotient: &[QubitId],
    raw_r: &[QubitId],
    p: U256,
    inverse_iters: usize,
) {
    debug_assert_eq!(numerator.len(), N);
    debug_assert_eq!(quotient.len(), N);
    debug_assert_eq!(raw_r.len(), 399);
    let (raw_low, raw_high) = raw_r.split_at(N);
    debug_assert_eq!(raw_high.len(), 143);

    b.set_phase("round590_addsub_low_256_product");
    d1_direct_quotient_compute_product_from_raw_inverse(b, numerator, quotient, raw_low, p);

    b.set_phase("round590_addsub_high_init_c_times_num");
    let high_term = b.alloc_qubits(N);
    let solinas_c = (U256::from(1u64) << 32) + U256::from(977u64);
    mul_by_const_acc(b, numerator, solinas_c, &high_term, p, false);

    b.set_phase("round590_addsub_high_product");
    let tmp_ext = b.alloc_qubits(2 * N);
    schoolbook_rect_mul_into_addsub(b, &high_term, raw_high, &tmp_ext);

    b.set_phase("round590_addsub_high_reduce");
    mod_add_solinas_ext_product(b, quotient, &tmp_ext, p);

    b.set_phase("round590_addsub_high_unproduct");
    schoolbook_rect_mul_into_addsub_inverse(b, &high_term, raw_high, &tmp_ext);
    b.free_vec(&tmp_ext);

    b.set_phase("round590_addsub_high_uncompute_c_times_num");
    mul_by_const_acc(b, numerator, solinas_c, &high_term, p, true);
    b.free_vec(&high_term);

    b.set_phase("round590_addsub_unscale");
    d1_direct_quotient_unscale_neg_product(b, quotient, p, inverse_iters);
}

pub(crate) fn build_round590_rectangular_addsub_solinas_raw_splice_builder() -> B {
    let mut b = B::new();
    let numerator = b.alloc_qubits(N);
    b.declare_qubit_register(&numerator);
    let quotient = b.alloc_qubits(N);
    b.declare_qubit_register(&quotient);
    let packed_us = b.alloc_qubits(257);
    b.declare_qubit_register(&packed_us);
    let packed_vr_raw = b.alloc_qubits(399);
    b.declare_qubit_register(&packed_vr_raw);
    let m_hist = b.alloc_qubits(399);
    b.declare_qubit_register(&m_hist);

    round590_rectangular_addsub_solinas_raw_consumer(
        &mut b,
        &numerator,
        &quotient,
        &packed_vr_raw,
        SECP256K1_P,
        400,
    );

    let _ = (packed_us, m_hist);
    b
}

pub fn build_round590_rectangular_addsub_solinas_raw_splice_component() -> Vec<Op> {
    build_round590_rectangular_addsub_solinas_raw_splice_builder().ops
}

pub fn build_round590_rectangular_addsub_solinas_raw_splice_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round590_rectangular_addsub_solinas_raw_splice_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub(crate) fn build_round590_rectangular_addsub_solinas_raw_only_splice_builder() -> B {
    let mut b = B::new();
    let numerator = b.alloc_qubits(N);
    b.declare_qubit_register(&numerator);
    let quotient = b.alloc_qubits(N);
    b.declare_qubit_register(&quotient);
    let raw_r = b.alloc_qubits(399);
    b.declare_qubit_register(&raw_r);

    round590_rectangular_addsub_solinas_raw_consumer(
        &mut b,
        &numerator,
        &quotient,
        &raw_r,
        SECP256K1_P,
        400,
    );

    b
}

pub fn build_round590_rectangular_addsub_solinas_raw_only_splice_component() -> Vec<Op> {
    build_round590_rectangular_addsub_solinas_raw_only_splice_builder().ops
}

pub fn build_round590_rectangular_addsub_solinas_raw_only_splice_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round590_rectangular_addsub_solinas_raw_only_splice_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub(crate) fn round592_residue_solinas_raw_consumer(
    b: &mut B,
    numerator: &[QubitId],
    quotient: &[QubitId],
    raw_r: &[QubitId],
    p: U256,
    inverse_iters: usize,
) {
    debug_assert_eq!(numerator.len(), N);
    debug_assert_eq!(quotient.len(), N);
    debug_assert_eq!(raw_r.len(), 399);
    let (raw_low, raw_high) = raw_r.split_at(N);
    debug_assert_eq!(raw_high.len(), 143);

    let residue = b.alloc_qubits(N);

    b.set_phase("round592_residue_copy_low");
    for i in 0..N {
        b.cx(raw_low[i], residue[i]);
    }

    b.set_phase("round592_residue_high_reduce");
    let solinas_c = (U256::from(1u64) << 32) + U256::from(977u64);
    for i in 0..raw_high.len() {
        cmod_add_qc_fast(b, &residue, solinas_c << i, raw_high[i], p);
    }

    b.set_phase("round592_residue_low_256_product");
    d1_direct_quotient_compute_product_from_raw_inverse(b, numerator, quotient, &residue, p);

    b.set_phase("round592_residue_high_unreduce");
    for i in (0..raw_high.len()).rev() {
        cmod_sub_qc_fast(b, &residue, solinas_c << i, raw_high[i], p);
    }

    b.set_phase("round592_residue_uncopy_low");
    for i in (0..N).rev() {
        b.cx(raw_low[i], residue[i]);
    }
    b.free_vec(&residue);

    b.set_phase("round592_residue_unscale");
    d1_direct_quotient_unscale_neg_product(b, quotient, p, inverse_iters);
}

pub(crate) fn build_round592_residue_solinas_raw_splice_builder() -> B {
    let mut b = B::new();
    let numerator = b.alloc_qubits(N);
    b.declare_qubit_register(&numerator);
    let quotient = b.alloc_qubits(N);
    b.declare_qubit_register(&quotient);
    let packed_us = b.alloc_qubits(257);
    b.declare_qubit_register(&packed_us);
    let packed_vr_raw = b.alloc_qubits(399);
    b.declare_qubit_register(&packed_vr_raw);
    let m_hist = b.alloc_qubits(399);
    b.declare_qubit_register(&m_hist);

    round592_residue_solinas_raw_consumer(
        &mut b,
        &numerator,
        &quotient,
        &packed_vr_raw,
        SECP256K1_P,
        400,
    );

    let _ = (packed_us, m_hist);
    b
}

pub fn build_round592_residue_solinas_raw_splice_component() -> Vec<Op> {
    build_round592_residue_solinas_raw_splice_builder().ops
}

pub fn build_round592_residue_solinas_raw_splice_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round592_residue_solinas_raw_splice_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub(crate) fn build_round592_residue_solinas_raw_only_splice_builder() -> B {
    let mut b = B::new();
    let numerator = b.alloc_qubits(N);
    b.declare_qubit_register(&numerator);
    let quotient = b.alloc_qubits(N);
    b.declare_qubit_register(&quotient);
    let raw_r = b.alloc_qubits(399);
    b.declare_qubit_register(&raw_r);

    round592_residue_solinas_raw_consumer(&mut b, &numerator, &quotient, &raw_r, SECP256K1_P, 400);

    b
}

pub fn build_round592_residue_solinas_raw_only_splice_component() -> Vec<Op> {
    build_round592_residue_solinas_raw_only_splice_builder().ops
}

pub fn build_round592_residue_solinas_raw_only_splice_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round592_residue_solinas_raw_only_splice_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}
