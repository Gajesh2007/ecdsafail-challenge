//! `bench::by_ops` — verbatim split of the original `bench` module.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

pub(crate) fn by_cmod_neg_inplace_fast(b: &mut B, v: &[QubitId], ctrl: QubitId, p: U256) {
    // ctrl ? (p-v) : v.  Like the BY structural tests, this maps v=0 to the
    // noncanonical representative p when ctrl=1; the benchmark scaffold below
    // keeps controls at zero and uses this only to exercise the actual gate
    // body/cost inside the point-add harness.
    for &q in v {
        b.cx(ctrl, q);
    }
    cadd_nbit_const_fast(b, v, p.wrapping_add(U256::from(1u64)), ctrl);
}

pub(crate) fn by_cmod_neg_inplace_canonical_for_bench(b: &mut B, v: &[QubitId], ctrl: QubitId, p: U256) {
    // ctrl ? (-v mod p) : v, preserving the canonical zero representative.  The
    // fast BY negation maps 0 -> p; that is fine inside replay scaffolds but not
    // when the pair2 product-clean path wants to free the slope register after
    // inverse replay.  Nonzeroness is invariant under v -> p-v, so the flag can
    // be uncomputed after the controlled negation.
    let nz = b.alloc_qubit();
    let do_neg = b.alloc_qubit();
    cmp_neq_zero_into(b, v, nz);
    b.ccx(ctrl, nz, do_neg);
    for &q in v {
        b.cx(do_neg, q);
    }
    cadd_nbit_const_fast(b, v, p.wrapping_add(U256::from(1u64)), do_neg);
    b.ccx(ctrl, nz, do_neg);
    cmp_neq_zero_into(b, v, nz);
    b.free(do_neg);
    b.free(nz);
}

pub(crate) fn by_signed_controlled_add_for_bench(b: &mut B, acc: &[QubitId], a: &[QubitId], ctrl: QubitId) {
    let f = b.alloc_qubits(acc.len());
    for i in 0..acc.len() {
        b.ccx(ctrl, a[i], f[i]);
    }
    add_nbit_qq_fast(b, &f, acc);
    for i in 0..acc.len() {
        let m = b.alloc_bit();
        b.hmr(f[i], m);
        b.cz_if(ctrl, a[i], m);
    }
    b.free_vec(&f);
}

pub(crate) fn by_signed_controlled_sub_for_bench(b: &mut B, acc: &[QubitId], a: &[QubitId], ctrl: QubitId) {
    let f = b.alloc_qubits(acc.len());
    for i in 0..acc.len() {
        b.ccx(ctrl, a[i], f[i]);
    }
    sub_nbit_qq_fast(b, &f, acc);
    for i in 0..acc.len() {
        let m = b.alloc_bit();
        b.hmr(f[i], m);
        b.cz_if(ctrl, a[i], m);
    }
    b.free_vec(&f);
}

pub(crate) fn by_twos_cneg_for_bench(b: &mut B, v: &[QubitId], ctrl: QubitId) {
    if std::env::var("BY_CENTERED_REPLAY_DIRECTFAST_CNEG")
        .ok()
        .as_deref()
        == Some("1")
    {
        for &q in v {
            b.cx(ctrl, q);
        }
        cadd_nbit_const_direct_fast(b, v, U256::from(1u64), ctrl);
        return;
    }
    for &q in v {
        b.cx(ctrl, q);
    }
    cadd_nbit_const_fast(b, v, U256::from(1u64), ctrl);
}

pub(crate) fn by_arithmetic_shift_right_even_for_bench(b: &mut B, v: &[QubitId]) {
    for i in 0..v.len() - 1 {
        b.swap(v[i], v[i + 1]);
    }
    b.cx(v[v.len() - 2], v[v.len() - 1]);
}

pub(crate) fn by_signed_controlled_add_exact_for_bench(
    b: &mut B,
    acc: &[QubitId],
    a: &[QubitId],
    ctrl: QubitId,
) {
    let f = b.alloc_qubits(acc.len());
    for i in 0..acc.len() {
        b.ccx(ctrl, a[i], f[i]);
    }
    add_nbit_qq(b, &f, acc);
    for i in 0..acc.len() {
        b.ccx(ctrl, a[i], f[i]);
    }
    b.free_vec(&f);
}

pub(crate) fn by_signed_controlled_sub_exact_for_bench(
    b: &mut B,
    acc: &[QubitId],
    a: &[QubitId],
    ctrl: QubitId,
) {
    let f = b.alloc_qubits(acc.len());
    for i in 0..acc.len() {
        b.ccx(ctrl, a[i], f[i]);
    }
    sub_nbit_qq(b, &f, acc);
    for i in 0..acc.len() {
        b.ccx(ctrl, a[i], f[i]);
    }
    b.free_vec(&f);
}

pub(crate) fn by_twos_cneg_exact_for_bench(b: &mut B, v: &[QubitId], ctrl: QubitId) {
    for &q in v {
        b.cx(ctrl, q);
    }
    cadd_nbit_const(b, v, U256::from(1u64), ctrl);
}

pub(crate) fn by_arithmetic_shift_left_even_inverse_for_bench(b: &mut B, v: &[QubitId]) {
    b.cx(v[v.len() - 2], v[v.len() - 1]);
    for i in (0..v.len() - 1).rev() {
        b.swap(v[i], v[i + 1]);
    }
}

pub(crate) fn by_logical_shift_right_even_for_bench(b: &mut B, v: &[QubitId]) {
    for i in 0..v.len() - 1 {
        b.swap(v[i], v[i + 1]);
    }
}

pub(crate) fn by_logical_shift_left_even_inverse_for_bench(b: &mut B, v: &[QubitId]) {
    for i in (0..v.len() - 1).rev() {
        b.swap(v[i], v[i + 1]);
    }
}

pub(crate) fn by_delta_positive_into_for_bench(b: &mut B, delta: &[QubitId], flag: QubitId) {
    let nz = b.alloc_qubit();
    cmp_neq_zero_into(b, delta, nz);
    let sign = delta[delta.len() - 1];
    b.x(sign);
    b.ccx(nz, sign, flag);
    b.x(sign);
    cmp_neq_zero_into(b, delta, nz);
    b.free(nz);
}

pub(crate) fn by_2adic_branch_step_for_bench(
    b: &mut B,
    f: &[QubitId],
    g: &[QubitId],
    delta: &[QubitId],
    odd_out: QubitId,
    a_out: QubitId,
) {
    b.cx(g[0], odd_out);
    let positive = b.alloc_qubit();
    by_delta_positive_into_for_bench(b, delta, positive);
    b.ccx(odd_out, positive, a_out);
    by_delta_positive_into_for_bench(b, delta, positive);
    b.free(positive);

    for i in 0..f.len() {
        cswap(b, a_out, f[i], g[i]);
    }
    by_twos_cneg_for_bench(b, g, a_out);
    cucc_add_ctrl(b, f, g, odd_out);
    by_logical_shift_right_even_for_bench(b, g);

    by_twos_cneg_for_bench(b, delta, a_out);
    add_nbit_const_fast(b, delta, U256::from(1u64));
}

pub(crate) fn by_2adic_branch_step_reverse_for_bench(
    b: &mut B,
    f: &[QubitId],
    g: &[QubitId],
    delta: &[QubitId],
    odd_hist: QubitId,
    a_hist: QubitId,
) {
    sub_nbit_const_fast(b, delta, U256::from(1u64));
    by_twos_cneg_for_bench(b, delta, a_hist);
    by_logical_shift_left_even_inverse_for_bench(b, g);
    cucc_sub_ctrl(b, f, g, odd_hist);
    by_twos_cneg_for_bench(b, g, a_hist);
    for i in 0..f.len() {
        cswap(b, a_hist, f[i], g[i]);
    }

    let positive = b.alloc_qubit();
    by_delta_positive_into_for_bench(b, delta, positive);
    b.ccx(odd_hist, positive, a_hist);
    by_delta_positive_into_for_bench(b, delta, positive);
    b.free(positive);
    b.cx(g[0], odd_hist);
}

pub(crate) fn by_signed_branch_step_for_bench(
    b: &mut B,
    f: &[QubitId],
    g: &[QubitId],
    delta: &[QubitId],
    odd_out: QubitId,
    a_out: QubitId,
) {
    b.cx(g[0], odd_out);
    let positive = b.alloc_qubit();
    by_delta_positive_into_for_bench(b, delta, positive);
    b.ccx(odd_out, positive, a_out);
    by_delta_positive_into_for_bench(b, delta, positive);
    b.free(positive);

    for i in 0..f.len() {
        cswap(b, a_out, f[i], g[i]);
    }
    by_twos_cneg_for_bench(b, g, a_out);
    cucc_add_ctrl(b, f, g, odd_out);
    by_arithmetic_shift_right_even_for_bench(b, g);

    by_twos_cneg_for_bench(b, delta, a_out);
    add_nbit_const_fast(b, delta, U256::from(1u64));
}

pub(crate) fn by_signed_branch_step_reverse_for_bench(
    b: &mut B,
    f: &[QubitId],
    g: &[QubitId],
    delta: &[QubitId],
    odd_hist: QubitId,
    a_hist: QubitId,
) {
    sub_nbit_const_fast(b, delta, U256::from(1u64));
    by_twos_cneg_for_bench(b, delta, a_hist);
    by_arithmetic_shift_left_even_inverse_for_bench(b, g);
    cucc_sub_ctrl(b, f, g, odd_hist);
    by_twos_cneg_for_bench(b, g, a_hist);
    for i in 0..f.len() {
        cswap(b, a_hist, f[i], g[i]);
    }

    let positive = b.alloc_qubit();
    by_delta_positive_into_for_bench(b, delta, positive);
    b.ccx(odd_hist, positive, a_hist);
    by_delta_positive_into_for_bench(b, delta, positive);
    b.free(positive);
    b.cx(g[0], odd_hist);
}

pub(crate) fn by_signed_branch_apply_step_for_bench(
    b: &mut B,
    f: &[QubitId],
    g: &[QubitId],
    delta: &[QubitId],
    odd: QubitId,
    a: QubitId,
) {
    for i in 0..f.len() {
        cswap(b, a, f[i], g[i]);
    }
    by_twos_cneg_for_bench(b, g, a);
    cucc_add_ctrl(b, f, g, odd);
    by_arithmetic_shift_right_even_for_bench(b, g);

    by_twos_cneg_for_bench(b, delta, a);
    add_nbit_const_fast(b, delta, U256::from(1u64));
}

pub(crate) fn by_signed_branch_apply_step_reverse_for_bench(
    b: &mut B,
    f: &[QubitId],
    g: &[QubitId],
    delta: &[QubitId],
    odd: QubitId,
    a: QubitId,
) {
    sub_nbit_const_fast(b, delta, U256::from(1u64));
    by_twos_cneg_for_bench(b, delta, a);
    by_arithmetic_shift_left_even_inverse_for_bench(b, g);
    cucc_sub_ctrl(b, f, g, odd);
    by_twos_cneg_for_bench(b, g, a);
    for i in 0..f.len() {
        cswap(b, a, f[i], g[i]);
    }
}

pub(crate) fn by_copy_lowword_sign_extended_for_bench(
    b: &mut B,
    src: &[QubitId],
    dst: &[QubitId],
    low_bits: usize,
) {
    assert!(dst.len() >= low_bits);
    assert!(src.len() >= low_bits);
    for i in 0..low_bits {
        b.cx(src[i], dst[i]);
    }
    for i in low_bits..dst.len() {
        b.cx(src[low_bits - 1], dst[i]);
    }
}

pub(crate) fn by_xor_signed_lowword_const_for_bench(b: &mut B, dst: &[QubitId], c: U256, low_bits: usize) {
    assert!(dst.len() >= low_bits);
    for i in 0..low_bits {
        if bit(c, i) {
            b.x(dst[i]);
        }
    }
    if bit(c, low_bits - 1) {
        for i in low_bits..dst.len() {
            b.x(dst[i]);
        }
    }
}

pub(crate) fn by_signed_lowword_window_xor_controls_for_bench(
    b: &mut B,
    f_full: &[QubitId],
    g_full: &[QubitId],
    delta_full: &[QubitId],
    odd_hist: &[QubitId],
    a_hist: &[QubitId],
    q_hist: Option<(&[QubitId], &[QubitId])>,
    start: usize,
) {
    // Window selector primitive for the centered-BY denominator path.  The next
    // 16 BY branch decisions depend only on the low 16 bits of the current
    // signed denominator pair plus delta.  Compute them in a narrow local
    // 2-adic simulator, xor them into the persistent odd/A histories, and then
    // reverse the simulator.  The full-width denominator state is updated by a
    // separate selected-control application below; this first hook deliberately
    // wires the lowword-window control source into the real pair replacement.
    const W: usize = 16;
    const QBITS: usize = 34;
    let f = b.alloc_qubits(QBITS);
    let g = b.alloc_qubits(QBITS);
    let delta = b.alloc_qubits(delta_full.len());
    let odd_tmp = b.alloc_qubits(W);
    let a_tmp = b.alloc_qubits(W);

    by_copy_lowword_sign_extended_for_bench(b, f_full, &f, W);
    by_copy_lowword_sign_extended_for_bench(b, g_full, &g, W);
    for i in 0..delta_full.len() {
        b.cx(delta_full[i], delta[i]);
    }

    for j in 0..W {
        by_signed_branch_step_for_bench(b, &f, &g, &delta, odd_tmp[j], a_tmp[j]);
    }
    for j in 0..W {
        b.cx(odd_tmp[j], odd_hist[start + j]);
        b.cx(a_tmp[j], a_hist[start + j]);
    }
    if let Some((q0_hist, q1_hist)) = q_hist {
        let windows = odd_hist.len() / W;
        assert_eq!(q0_hist.len(), q1_hist.len());
        assert_eq!(q0_hist.len() % windows, 0);
        let qhist_bits = q0_hist.len() / windows;
        assert!(qhist_bits <= QBITS);
        let q_start = (start / W) * qhist_bits;
        // After the local signed divsteps, these narrow rows are exactly the
        // lowword quotient corrections q=(P·low)/2^16.  Persist only the
        // bounded signed payload bits (18); the local simulator still uses 34
        // bits to make the signed divsteps reversible.  The same helper is
        // called in reverse to xor the payload clean again.
        for i in 0..qhist_bits {
            b.cx(f[i], q0_hist[q_start + i]);
            b.cx(g[i], q1_hist[q_start + i]);
        }
    }
    for j in (0..W).rev() {
        by_signed_branch_step_reverse_for_bench(b, &f, &g, &delta, odd_tmp[j], a_tmp[j]);
    }

    for i in (0..delta_full.len()).rev() {
        b.cx(delta_full[i], delta[i]);
    }
    by_copy_lowword_sign_extended_for_bench(b, g_full, &g, W);
    by_copy_lowword_sign_extended_for_bench(b, f_full, &f, W);
    b.free_vec(&a_tmp);
    b.free_vec(&odd_tmp);
    b.free_vec(&delta);
    b.free_vec(&g);
    b.free_vec(&f);
}

pub(crate) fn by_window_controls_enabled_for_bench() -> bool {
    std::env::var("BY_CENTERED_WINDOW_DENOM_REPLACE")
        .ok()
        .as_deref()
        == Some("1")
        || by_window_q_payload_enabled_for_bench()
}

pub(crate) fn by_window_q_payload_enabled_for_bench() -> bool {
    std::env::var("BY_CENTERED_WINDOW_Q_DENOM_REPLACE")
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn by_generate_signed_controls_for_bench(
    b: &mut B,
    f: &[QubitId],
    g: &[QubitId],
    delta: &[QubitId],
    odd: &[QubitId],
    a_ctrl: &[QubitId],
    q_hist: Option<(&[QubitId], &[QubitId])>,
) {
    if by_window_controls_enabled_for_bench() {
        const W: usize = 16;
        assert_eq!(odd.len() % W, 0);
        for start in (0..odd.len()).step_by(W) {
            by_signed_lowword_window_xor_controls_for_bench(
                b, f, g, delta, odd, a_ctrl, q_hist, start,
            );
            for j in 0..W {
                by_signed_branch_apply_step_for_bench(
                    b,
                    f,
                    g,
                    delta,
                    odd[start + j],
                    a_ctrl[start + j],
                );
            }
        }
    } else {
        for i in 0..odd.len() {
            by_signed_branch_step_for_bench(b, f, g, delta, odd[i], a_ctrl[i]);
        }
    }
}

pub(crate) fn by_reverse_signed_controls_for_bench(
    b: &mut B,
    f: &[QubitId],
    g: &[QubitId],
    delta: &[QubitId],
    odd: &[QubitId],
    a_ctrl: &[QubitId],
    q_hist: Option<(&[QubitId], &[QubitId])>,
) {
    if by_window_controls_enabled_for_bench() {
        const W: usize = 16;
        assert_eq!(odd.len() % W, 0);
        for start in (0..odd.len()).step_by(W).rev() {
            for j in (0..W).rev() {
                by_signed_branch_apply_step_reverse_for_bench(
                    b,
                    f,
                    g,
                    delta,
                    odd[start + j],
                    a_ctrl[start + j],
                );
            }
            by_signed_lowword_window_xor_controls_for_bench(
                b, f, g, delta, odd, a_ctrl, q_hist, start,
            );
        }
    } else {
        for i in (0..odd.len()).rev() {
            by_signed_branch_step_reverse_for_bench(b, f, g, delta, odd[i], a_ctrl[i]);
        }
    }
}

pub(crate) fn by_copy_signed_mod_p_for_bench(b: &mut B, signed: &[QubitId], out: &[QubitId], p: U256) {
    assert!(signed.len() > out.len());
    for i in 0..out.len() {
        b.cx(signed[i], out[i]);
    }
    let c = U256::MAX.wrapping_sub(p).wrapping_add(U256::from(1u64));
    csub_nbit_const(b, out, c, signed[signed.len() - 1]);
}

pub(crate) fn by_uncopy_signed_mod_p_for_bench(b: &mut B, signed: &[QubitId], out: &[QubitId], p: U256) {
    assert!(signed.len() > out.len());
    let c = U256::MAX.wrapping_sub(p).wrapping_add(U256::from(1u64));
    cadd_nbit_const(b, out, c, signed[signed.len() - 1]);
    for i in 0..out.len() {
        b.cx(signed[i], out[i]);
    }
}

pub(crate) fn by_cmod_add_qq_exact_for_bench(
    b: &mut B,
    acc: &[QubitId],
    a: &[QubitId],
    ctrl: QubitId,
    p: U256,
) {
    let f = b.alloc_qubits(acc.len());
    for i in 0..acc.len() {
        b.ccx(ctrl, a[i], f[i]);
    }
    mod_add_qq(b, acc, &f, p);
    for i in 0..acc.len() {
        b.ccx(ctrl, a[i], f[i]);
    }
    b.free_vec(&f);
}

pub(crate) fn by_cmod_sub_qq_exact_for_bench(
    b: &mut B,
    acc: &[QubitId],
    a: &[QubitId],
    ctrl: QubitId,
    p: U256,
) {
    let f = b.alloc_qubits(acc.len());
    for i in 0..acc.len() {
        b.ccx(ctrl, a[i], f[i]);
    }
    mod_sub_qq(b, acc, &f, p);
    for i in 0..acc.len() {
        b.ccx(ctrl, a[i], f[i]);
    }
    b.free_vec(&f);
}
