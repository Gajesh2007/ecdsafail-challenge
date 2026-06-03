//! `bench::centered1` — verbatim split of the original `bench` module.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

pub(crate) fn by_centered_halve_live_parity_for_bench(b: &mut B, v: &[QubitId], parity: QubitId, p: U256) {
    let directfast = std::env::var("BY_CENTERED_REPLAY_DIRECTFAST_HALVE")
        .ok()
        .as_deref()
        == Some("1");
    let sign_hist = b.alloc_qubit();
    let add_ctrl = b.alloc_qubit();
    let sub_ctrl = b.alloc_qubit();
    b.cx(v[0], parity);
    b.cx(v[v.len() - 1], sign_hist);
    b.ccx(parity, sign_hist, add_ctrl);
    b.x(sign_hist);
    b.ccx(parity, sign_hist, sub_ctrl);
    b.x(sign_hist);
    if directfast {
        cadd_nbit_const_direct_fast(b, v, p, add_ctrl);
        csub_nbit_const_direct_fast(b, v, p, sub_ctrl);
    } else {
        cadd_nbit_const_fast(b, v, p, add_ctrl);
        csub_nbit_const_fast(b, v, p, sub_ctrl);
    }
    b.x(sign_hist);
    b.ccx(parity, sign_hist, sub_ctrl);
    b.x(sign_hist);
    b.ccx(parity, sign_hist, add_ctrl);
    b.free(sub_ctrl);
    b.free(add_ctrl);
    by_arithmetic_shift_right_even_for_bench(b, v);
    b.cx(v[v.len() - 1], sign_hist);
    b.cx(parity, sign_hist);
    b.free(sign_hist);
}

pub(crate) fn centered_signed_by_microstep_for_bench(
    b: &mut B,
    r: &[QubitId],
    s: &[QubitId],
    odd: QubitId,
    a: QubitId,
    parity: QubitId,
    p: U256,
) {
    let exact_cneg = std::env::var("BY_CENTERED_REPLAY_EXACT_CNEG")
        .ok()
        .as_deref()
        == Some("1");
    let exact_add = std::env::var("BY_CENTERED_REPLAY_EXACT_ADD")
        .ok()
        .as_deref()
        == Some("1");
    let exact_halve = std::env::var("BY_CENTERED_REPLAY_EXACT_HALVE")
        .ok()
        .as_deref()
        == Some("1");
    for i in 0..r.len() {
        cswap(b, a, r[i], s[i]);
    }
    if exact_cneg {
        by_twos_cneg_exact_for_bench(b, s, a);
    } else {
        by_twos_cneg_for_bench(b, s, a);
    }
    if exact_add {
        by_signed_controlled_add_exact_for_bench(b, s, r, odd);
    } else {
        by_signed_controlled_add_for_bench(b, s, r, odd);
    }
    if exact_halve {
        by_centered_halve_live_parity_exact_for_bench(b, s, parity, p);
    } else {
        by_centered_halve_live_parity_for_bench(b, s, parity, p);
    }
}

pub(crate) fn by_centered_halve_live_parity_exact_for_bench(
    b: &mut B,
    v: &[QubitId],
    parity: QubitId,
    p: U256,
) {
    let sign_hist = b.alloc_qubit();
    let add_ctrl = b.alloc_qubit();
    let sub_ctrl = b.alloc_qubit();
    b.cx(v[0], parity);
    b.cx(v[v.len() - 1], sign_hist);
    b.ccx(parity, sign_hist, add_ctrl);
    b.x(sign_hist);
    b.ccx(parity, sign_hist, sub_ctrl);
    b.x(sign_hist);
    cadd_nbit_const(b, v, p, add_ctrl);
    csub_nbit_const(b, v, p, sub_ctrl);
    b.x(sign_hist);
    b.ccx(parity, sign_hist, sub_ctrl);
    b.x(sign_hist);
    b.ccx(parity, sign_hist, add_ctrl);
    b.free(sub_ctrl);
    b.free(add_ctrl);
    by_arithmetic_shift_right_even_for_bench(b, v);
    b.cx(v[v.len() - 1], sign_hist);
    b.cx(parity, sign_hist);
    b.free(sign_hist);
}

pub(crate) fn by_centered_unhalve_with_parity_for_bench(b: &mut B, v: &[QubitId], parity: QubitId, p: U256) {
    by_arithmetic_shift_left_even_inverse_for_bench(b, v);
    let sign_hist = b.alloc_qubit();
    let add_ctrl = b.alloc_qubit();
    let sub_ctrl = b.alloc_qubit();
    let sign = v[v.len() - 1];
    b.cx(sign, sign_hist);
    b.ccx(parity, sign_hist, add_ctrl);
    b.x(sign_hist);
    b.ccx(parity, sign_hist, sub_ctrl);
    b.x(sign_hist);
    cadd_nbit_const_fast(b, v, p, add_ctrl);
    csub_nbit_const_fast(b, v, p, sub_ctrl);
    b.x(sign_hist);
    b.ccx(parity, sign_hist, sub_ctrl);
    b.x(sign_hist);
    b.ccx(parity, sign_hist, add_ctrl);
    b.free(sub_ctrl);
    b.free(add_ctrl);
    b.cx(sign, sign_hist);
    b.cx(parity, sign_hist);
    b.free(sign_hist);
}

pub(crate) fn by_centered_unhalve_with_parity_exact_for_bench(
    b: &mut B,
    v: &[QubitId],
    parity: QubitId,
    p: U256,
) {
    by_arithmetic_shift_left_even_inverse_for_bench(b, v);
    let sign_hist = b.alloc_qubit();
    let add_ctrl = b.alloc_qubit();
    let sub_ctrl = b.alloc_qubit();
    let sign = v[v.len() - 1];
    // The correction direction is determined by the sign of the doubled value
    // before undoing the ±p correction.  Keep that sign live; the correction
    // flips it when parity=1, so recomputing controls from the post-correction
    // sign leaves dirty controls and R-phase garbage.
    b.cx(sign, sign_hist);
    b.ccx(parity, sign_hist, add_ctrl);
    b.x(sign_hist);
    b.ccx(parity, sign_hist, sub_ctrl);
    b.x(sign_hist);
    cadd_nbit_const(b, v, p, add_ctrl);
    csub_nbit_const(b, v, p, sub_ctrl);
    b.x(sign_hist);
    b.ccx(parity, sign_hist, sub_ctrl);
    b.x(sign_hist);
    b.ccx(parity, sign_hist, add_ctrl);
    b.free(sub_ctrl);
    b.free(add_ctrl);
    b.cx(sign, sign_hist);
    b.cx(parity, sign_hist);
    b.free(sign_hist);
}

pub(crate) fn centered_signed_by_microstep_inverse_for_bench(
    b: &mut B,
    r: &[QubitId],
    s: &[QubitId],
    odd: QubitId,
    a: QubitId,
    parity: QubitId,
    p: U256,
) {
    by_centered_unhalve_with_parity_for_bench(b, s, parity, p);
    by_signed_controlled_sub_for_bench(b, s, r, odd);
    by_twos_cneg_for_bench(b, s, a);
    for i in 0..r.len() {
        cswap(b, a, r[i], s[i]);
    }
}

pub(crate) fn centered_signed_by_microstep_all_exact_for_bench(
    b: &mut B,
    r: &[QubitId],
    s: &[QubitId],
    odd: QubitId,
    a: QubitId,
    parity: QubitId,
    p: U256,
) {
    for i in 0..r.len() {
        cswap(b, a, r[i], s[i]);
    }
    by_twos_cneg_exact_for_bench(b, s, a);
    by_signed_controlled_add_exact_for_bench(b, s, r, odd);
    by_centered_halve_live_parity_exact_for_bench(b, s, parity, p);
}

pub(crate) fn centered_signed_by_microstep_inverse_all_exact_for_bench(
    b: &mut B,
    r: &[QubitId],
    s: &[QubitId],
    odd: QubitId,
    a: QubitId,
    parity: QubitId,
    p: U256,
) {
    by_centered_unhalve_with_parity_exact_for_bench(b, s, parity, p);
    by_signed_controlled_sub_exact_for_bench(b, s, r, odd);
    by_twos_cneg_exact_for_bench(b, s, a);
    for i in 0..r.len() {
        cswap(b, a, r[i], s[i]);
    }
}

pub(crate) fn centered_signed_by_clear_parity_after_inverse_for_bench(
    b: &mut B,
    r: &[QubitId],
    s: &[QubitId],
    odd: QubitId,
    parity: QubitId,
) {
    b.cx(s[0], parity);
    b.ccx(odd, r[0], parity);
}

pub(crate) fn by_add_neg_quotient_from_centered_r_for_bench(
    b: &mut B,
    acc: &[QubitId],
    r: &[QubitId],
    f_neg: QubitId,
    p: U256,
) {
    // Tagged recovery is q = sign(f)*r - 1.  Add -q = 1 - sign(f)*r to acc.
    mod_add_qc(b, acc, U256::from(1u64), p);
    let r_mod = b.alloc_qubits(acc.len());
    by_copy_signed_mod_p_for_bench(b, r, &r_mod, p);
    let f_pos = b.alloc_qubit();
    b.x(f_pos);
    b.cx(f_neg, f_pos);
    cmod_sub_qq(b, acc, &r_mod, f_pos, p);
    cmod_add_qq(b, acc, &r_mod, f_neg, p);
    b.cx(f_neg, f_pos);
    b.x(f_pos);
    b.free(f_pos);
    by_uncopy_signed_mod_p_for_bench(b, r, &r_mod, p);
    b.free_vec(&r_mod);
}

pub(crate) fn by_write_neg_quotient_from_centered_r_for_bench(
    b: &mut B,
    lam: &[QubitId],
    r: &[QubitId],
    f_neg: QubitId,
    p: U256,
) {
    by_add_neg_quotient_from_centered_r_for_bench(b, lam, r, f_neg, p);
}

pub(crate) fn by_load_centered_copy_for_bench(
    b: &mut B,
    src: &[QubitId],
    dst: &[QubitId],
    p: U256,
) -> QubitId {
    assert!(dst.len() >= src.len());
    for i in 0..src.len() {
        b.cx(src[i], dst[i]);
    }
    let center_flag = b.alloc_qubit();
    let half_p = p >> 1usize;
    let half = load_const(b, src.len(), half_p);
    cmp_lt_into(b, &half, &dst[..src.len()], center_flag);
    unload_const(b, &half, half_p);
    csub_nbit_const(b, dst, p, center_flag);
    center_flag
}

pub(crate) fn by_unload_centered_copy_for_bench(
    b: &mut B,
    src: &[QubitId],
    dst: &[QubitId],
    p: U256,
    center_flag: QubitId,
) {
    assert!(dst.len() >= src.len());
    cadd_nbit_const(b, dst, p, center_flag);
    let half_p = p >> 1usize;
    let half = load_const(b, src.len(), half_p);
    cmp_lt_into(b, &half, &dst[..src.len()], center_flag);
    unload_const(b, &half, half_p);
    for i in 0..src.len() {
        b.cx(src[i], dst[i]);
    }
    b.free(center_flag);
}

pub(crate) fn compute_pair1_lam_with_centered_by_bench(
    b: &mut B,
    tx: &[QubitId],
    ty: &[QubitId],
    p: U256,
) -> Vec<QubitId> {
    // Functional pair1 experiment: compute lam=-dy/dx using denominator-derived
    // BY controls and centered tagged numerator replay.  This is Bennett-style:
    // copy the recovered lam, then reverse replay/control generation so only lam
    // remains.  The caller can use the ordinary mul2 cleanup to zero ty.
    const STEPS: usize = 576;
    const DBITS: usize = 12;
    const WIDE: usize = N + 4;
    // Lowword q corrections are bounded below 2^17 in the sampled window
    // algebra, so 18 signed bits are enough for the raw payload history. The
    // local simulator remains 34 bits wide for reversible signed divsteps.
    const WINDOW_QBITS: usize = 18;
    b.set_phase("pair1_by_centered_alloc");
    let f = b.alloc_qubits(STEPS);
    let g = b.alloc_qubits(STEPS);
    let delta = b.alloc_qubits(DBITS);
    let odd = b.alloc_qubits(STEPS);
    let a_ctrl = b.alloc_qubits(STEPS);
    let parity = b.alloc_qubits(STEPS);
    let q_hist = if by_window_q_payload_enabled_for_bench() {
        Some((
            b.alloc_qubits((STEPS / 16) * WINDOW_QBITS),
            b.alloc_qubits((STEPS / 16) * WINDOW_QBITS),
        ))
    } else {
        None
    };
    let r = b.alloc_qubits(WIDE);
    let s = b.alloc_qubits(WIDE);
    let num = b.alloc_qubits(N);
    let lam = b.alloc_qubits(N);

    for i in 0..N {
        if bit(p, i) {
            b.x(f[i]);
        }
        b.cx(tx[i], g[i]);
        b.cx(ty[i], num[i]);
    }
    b.x(delta[0]);
    mod_add_qq_fast(b, &num, tx, p); // tagged numerator: dy + dx
    let center_flag = by_load_centered_copy_for_bench(b, &num, &s, p);

    b.set_phase("pair1_by_centered_generate");
    // Full-width denominator evolution preserves the final f sign needed by
    // tagged quotient recovery.  With BY_CENTERED_WINDOW_DENOM_REPLACE=1 the
    // branch decisions are sourced from 16-step lowword window oracles, then
    // applied to this full-width state; otherwise this is the original direct
    // per-step generator.
    let q_hist_slices = q_hist
        .as_ref()
        .map(|(q0, q1)| (q0.as_slice(), q1.as_slice()));
    by_generate_signed_controls_for_bench(b, &f, &g, &delta, &odd, &a_ctrl, q_hist_slices);

    b.set_phase("pair1_by_centered_forward");
    for i in 0..STEPS {
        centered_signed_by_microstep_for_bench(b, &r, &s, odd[i], a_ctrl[i], parity[i], p);
    }

    b.set_phase("pair1_by_centered_copy_lam");
    by_write_neg_quotient_from_centered_r_for_bench(b, &lam, &r, f[STEPS - 1], p);

    b.set_phase("pair1_by_centered_inverse_replay");
    for i in (0..STEPS).rev() {
        centered_signed_by_microstep_inverse_for_bench(b, &r, &s, odd[i], a_ctrl[i], parity[i], p);
        centered_signed_by_clear_parity_after_inverse_for_bench(b, &r, &s, odd[i], parity[i]);
    }

    b.set_phase("pair1_by_centered_reverse_den");
    let q_hist_slices = q_hist
        .as_ref()
        .map(|(q0, q1)| (q0.as_slice(), q1.as_slice()));
    by_reverse_signed_controls_for_bench(b, &f, &g, &delta, &odd, &a_ctrl, q_hist_slices);

    b.set_phase("pair1_by_centered_clear");
    by_unload_centered_copy_for_bench(b, &num, &s, p, center_flag);
    mod_sub_qq_fast(b, &num, tx, p);
    for i in 0..N {
        b.cx(ty[i], num[i]);
        b.cx(tx[i], g[i]);
        if bit(p, i) {
            b.x(f[i]);
        }
    }
    b.x(delta[0]);
    b.free_vec(&num);
    b.free_vec(&s);
    b.free_vec(&r);
    b.free_vec(&parity);
    if let Some((q0_hist, q1_hist)) = q_hist {
        b.free_vec(&q1_hist);
        b.free_vec(&q0_hist);
    }
    b.free_vec(&a_ctrl);
    b.free_vec(&odd);
    b.free_vec(&delta);
    b.free_vec(&g);
    b.free_vec(&f);
    lam
}

pub(crate) fn add_neg_quotient_into_acc_with_centered_by_bench(
    b: &mut B,
    acc: &[QubitId],
    denom: &[QubitId],
    numer: &[QubitId],
    p: U256,
) {
    // Functional pair2-style experiment: add -(numer/denom) into an existing
    // accumulator, then Bennett-clean the BY denominator/replay scratch.  For
    // pair2, acc is lam and numer = lam*denom, so this zeros lam without a
    // separate quotient output register that would need uncomputation.
    const STEPS: usize = 576;
    const DBITS: usize = 12;
    const WIDE: usize = N + 4;
    const WINDOW_QBITS: usize = 18;
    b.set_phase("by_centered_accquot_alloc");
    let f = b.alloc_qubits(STEPS);
    let g = b.alloc_qubits(STEPS);
    let delta = b.alloc_qubits(DBITS);
    let odd = b.alloc_qubits(STEPS);
    let a_ctrl = b.alloc_qubits(STEPS);
    let parity = b.alloc_qubits(STEPS);
    let q_hist = if by_window_q_payload_enabled_for_bench() {
        Some((
            b.alloc_qubits((STEPS / 16) * WINDOW_QBITS),
            b.alloc_qubits((STEPS / 16) * WINDOW_QBITS),
        ))
    } else {
        None
    };
    let r = b.alloc_qubits(WIDE);
    let s = b.alloc_qubits(WIDE);
    let num = b.alloc_qubits(N);

    for i in 0..N {
        if bit(p, i) {
            b.x(f[i]);
        }
        b.cx(denom[i], g[i]);
        b.cx(numer[i], num[i]);
    }
    b.x(delta[0]);
    mod_add_qq_fast(b, &num, denom, p);
    let center_flag = by_load_centered_copy_for_bench(b, &num, &s, p);

    b.set_phase("by_centered_accquot_generate");
    let q_hist_slices = q_hist
        .as_ref()
        .map(|(q0, q1)| (q0.as_slice(), q1.as_slice()));
    by_generate_signed_controls_for_bench(b, &f, &g, &delta, &odd, &a_ctrl, q_hist_slices);

    b.set_phase("by_centered_accquot_forward");
    for i in 0..STEPS {
        centered_signed_by_microstep_for_bench(b, &r, &s, odd[i], a_ctrl[i], parity[i], p);
    }

    b.set_phase("by_centered_accquot_add");
    by_add_neg_quotient_from_centered_r_for_bench(b, acc, &r, f[STEPS - 1], p);

    b.set_phase("by_centered_accquot_inverse_replay");
    for i in (0..STEPS).rev() {
        centered_signed_by_microstep_inverse_for_bench(b, &r, &s, odd[i], a_ctrl[i], parity[i], p);
        centered_signed_by_clear_parity_after_inverse_for_bench(b, &r, &s, odd[i], parity[i]);
    }

    b.set_phase("by_centered_accquot_reverse_den");
    let q_hist_slices = q_hist
        .as_ref()
        .map(|(q0, q1)| (q0.as_slice(), q1.as_slice()));
    by_reverse_signed_controls_for_bench(b, &f, &g, &delta, &odd, &a_ctrl, q_hist_slices);

    b.set_phase("by_centered_accquot_clear");
    by_unload_centered_copy_for_bench(b, &num, &s, p, center_flag);
    mod_sub_qq_fast(b, &num, denom, p);
    for i in 0..N {
        b.cx(numer[i], num[i]);
        b.cx(denom[i], g[i]);
        if bit(p, i) {
            b.x(f[i]);
        }
    }
    b.x(delta[0]);
    b.free_vec(&num);
    b.free_vec(&s);
    b.free_vec(&r);
    b.free_vec(&parity);
    if let Some((q0_hist, q1_hist)) = q_hist {
        b.free_vec(&q1_hist);
        b.free_vec(&q0_hist);
    }
    b.free_vec(&a_ctrl);
    b.free_vec(&odd);
    b.free_vec(&delta);
    b.free_vec(&g);
    b.free_vec(&f);
}

pub(crate) fn build_direct_centered_binary_trie_qrom_bench_builder(
    row_count: usize,
    address_bits: usize,
    target_bits: usize,
) -> B {
    assert!((1..=N).contains(&address_bits));
    assert!((1..=N).contains(&target_bits));
    assert!(row_count <= (1usize << address_bits));

    let mut b = B::new();
    let tx = b.alloc_qubits(N);
    b.declare_qubit_register(&tx);
    let ty = b.alloc_qubits(N);
    b.declare_qubit_register(&ty);
    let ox = b.alloc_bits(N);
    b.declare_bit_register(&ox);
    let oy = b.alloc_bits(N);
    b.declare_bit_register(&oy);

    b.set_phase("direct_centered_binary_trie_qrom_google_abi");
    emit_direct_centered_binary_trie_qrom_xor(
        &mut b,
        &tx[..address_bits],
        &ty[..target_bits],
        row_count,
    );
    let _ = (ox, oy);
    b
}

pub(crate) fn build_direct_centered_binary_trie_qrom_roundtrip_bench_builder(
    row_count: usize,
    address_bits: usize,
    target_bits: usize,
) -> B {
    assert!((1..=N).contains(&address_bits));
    assert!((1..=N).contains(&target_bits));
    assert!(row_count <= (1usize << address_bits));

    let mut b = B::new();
    let tx = b.alloc_qubits(N);
    b.declare_qubit_register(&tx);
    let ty = b.alloc_qubits(N);
    b.declare_qubit_register(&ty);
    let ox = b.alloc_bits(N);
    b.declare_bit_register(&ox);
    let oy = b.alloc_bits(N);
    b.declare_bit_register(&oy);

    let table_words = direct_centered_binary_trie_qrom_table_words(row_count, target_bits);
    emit_direct_centered_binary_trie_qrom_xor_table_phased(
        &mut b,
        &tx[..address_bits],
        &ty[..target_bits],
        row_count,
        &table_words,
        "direct_centered_binary_trie_qrom_roundtrip_load_walk",
        "direct_centered_binary_trie_qrom_roundtrip_load_clear_root",
    );
    emit_direct_centered_binary_trie_qrom_xor_table_phased(
        &mut b,
        &tx[..address_bits],
        &ty[..target_bits],
        row_count,
        &table_words,
        "direct_centered_binary_trie_qrom_roundtrip_clear_walk",
        "direct_centered_binary_trie_qrom_roundtrip_clear_root",
    );
    let _ = (ox, oy);
    b
}

pub fn build_direct_centered_binary_trie_qrom_bench_phase_resources(
    row_count: usize,
    address_bits: usize,
    target_bits: usize,
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b =
        build_direct_centered_binary_trie_qrom_bench_builder(row_count, address_bits, target_bits);
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub fn build_direct_centered_binary_trie_qrom_bench(
    row_count: usize,
    address_bits: usize,
    target_bits: usize,
) -> Vec<Op> {
    build_direct_centered_binary_trie_qrom_bench_builder(row_count, address_bits, target_bits).ops
}

pub fn build_direct_centered_binary_trie_qrom_roundtrip_bench_phase_resources(
    row_count: usize,
    address_bits: usize,
    target_bits: usize,
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_direct_centered_binary_trie_qrom_roundtrip_bench_builder(
        row_count,
        address_bits,
        target_bits,
    );
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub fn build_direct_centered_binary_trie_qrom_roundtrip_bench(
    row_count: usize,
    address_bits: usize,
    target_bits: usize,
) -> Vec<Op> {
    build_direct_centered_binary_trie_qrom_roundtrip_bench_builder(
        row_count,
        address_bits,
        target_bits,
    )
    .ops
}

pub(crate) fn build_direct_centered_branch_sidecar_bench_builder() -> B {
    let mut b = B::new();
    let tx = b.alloc_qubits(N);
    b.declare_qubit_register(&tx);
    let ty = b.alloc_qubits(N);
    b.declare_qubit_register(&ty);
    let ox = b.alloc_bits(N);
    b.declare_bit_register(&ox);
    let oy = b.alloc_bits(N);
    b.declare_bit_register(&oy);

    b.set_phase("direct_centered_sidecar_google_abi");
    emit_direct_centered_branch_sidecar_component(&mut b, &tx, &ty);
    let _ = (ox, oy);
    b
}

pub fn build_direct_centered_branch_sidecar_bench_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_direct_centered_branch_sidecar_bench_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub fn build_direct_centered_branch_sidecar_bench() -> Vec<Op> {
    build_direct_centered_branch_sidecar_bench_builder().ops
}

pub(crate) fn build_direct_centered_branch_retained_finalizer_bench_builder() -> B {
    let mut b = B::new();
    let tx = b.alloc_qubits(N);
    b.declare_qubit_register(&tx);
    let ty = b.alloc_qubits(N);
    b.declare_qubit_register(&ty);
    let ox = b.alloc_bits(N);
    b.declare_bit_register(&ox);
    let oy = b.alloc_bits(N);
    b.declare_bit_register(&oy);

    b.set_phase("direct_centered_branch_retained_finalizer_google_abi");
    let branch = b.alloc_qubit();
    let gated_divisor = b.alloc_qubits(N);
    let carry = b.alloc_qubit();

    emit_direct_centered_branch_retained_finalizer(&mut b, &tx, &ty, branch, &gated_divisor, carry);
    b.set_phase("direct_centered_branch_retained_finalizer_free");
    b.free(carry);
    b.free_vec(&gated_divisor);
    b.free(branch);
    let _ = (ox, oy);
    b
}

pub fn build_direct_centered_branch_retained_finalizer_bench_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_direct_centered_branch_retained_finalizer_bench_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub fn build_direct_centered_branch_retained_finalizer_bench() -> Vec<Op> {
    build_direct_centered_branch_retained_finalizer_bench_builder().ops
}

pub(crate) fn build_direct_centered_branch_digit_clean_fit_bench_builder() -> B {
    let mut b = B::new();
    let tx = b.alloc_qubits(N);
    b.declare_qubit_register(&tx);
    let ty = b.alloc_qubits(N);
    b.declare_qubit_register(&ty);
    let ox = b.alloc_bits(N);
    b.declare_bit_register(&ox);
    let oy = b.alloc_bits(N);
    b.declare_bit_register(&oy);

    b.set_phase("direct_centered_branch_digit_clean_alloc_envelope");
    let digit_lane = b.alloc_qubits(DIRECT_CENTERED_LOW_BRANCH_DIGIT_LANE_BITS);
    let meta = b.alloc_qubits(DIRECT_CENTERED_LOW_BRANCH_META_BITS);
    let prefix = b.alloc_qubits(DIRECT_CENTERED_LOW_BRANCH_PREFIX_BITS);
    let branch = b.alloc_qubits(DIRECT_CENTERED_EXPLICIT_BRANCH_HISTORY_BITS);
    let touch = b.alloc_qubits(DIRECT_CENTERED_BRANCH_SIDECAR_TOUCH_BITS);

    b.set_phase("direct_centered_branch_digit_clean_reuse_prefix_gated");
    emit_direct_centered_branch_digit_update_clean(
        &mut b,
        &digit_lane,
        &ty,
        branch[0],
        touch[0],
        &prefix[..N],
        touch[1],
    );

    b.set_phase("direct_centered_branch_digit_clean_free_envelope");
    b.free_vec(&touch);
    b.free_vec(&branch);
    b.free_vec(&prefix);
    b.free_vec(&meta);
    b.free_vec(&digit_lane);
    let _ = (tx, ox, oy);
    b
}

pub fn build_direct_centered_branch_digit_clean_fit_bench_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_direct_centered_branch_digit_clean_fit_bench_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub fn build_direct_centered_branch_digit_clean_fit_bench() -> Vec<Op> {
    build_direct_centered_branch_digit_clean_fit_bench_builder().ops
}
