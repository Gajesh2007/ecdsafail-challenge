//! `kaliski::util` — verbatim split of the original `kaliski` module.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

/// Classical modular inverse via Fermat's little theorem. Used ONLY at
/// circuit-construction time to compute correction constants.
#[allow(dead_code)]
pub(crate) fn classical_modinv(a: U256, p: U256) -> U256 {
    // a^(p-2) mod p via square-and-multiply.
    let exponent = p.wrapping_sub(U256::from(2));
    let mut result = U256::from(1);
    let mut base = a % p;
    for i in 0..256 {
        if exponent.bit(i) {
            result = mulmod(result, base, p);
        }
        base = mulmod(base, base, p);
    }
    result
}

/// Classical modular multiplication used to compute correction constants
/// at build time.
pub(crate) fn mulmod(a: U256, b: U256, p: U256) -> U256 {
    // Naive (a * b) mod p — both < p < 2^256, so the product may overflow
    // 256 bits. Use U256's widening mul if available; else do it in u512
    // via chunks. alloy's U256 has `mul_mod`.
    a.mul_mod(b, p)
}

pub(crate) fn alloc_kaliski_state(b: &mut B, n: usize, max_iters: usize) -> KaliskiState {
    KaliskiState {
        u: b.alloc_qubits(n),
        v_w: b.alloc_qubits(n),
        r: b.alloc_qubits(n),
        s: b.alloc_qubits(n),
        m_hist: b.alloc_qubits(max_iters),
        f_flag: b.alloc_qubit(),
    }
}

pub(crate) fn alloc_kaliski_state_borrowing_v(b: &mut B, v_w: &[QubitId], max_iters: usize) -> KaliskiState {
    let n = v_w.len();
    KaliskiState {
        u: b.alloc_qubits(n),
        v_w: v_w.to_vec(),
        r: b.alloc_qubits(n),
        s: b.alloc_qubits(n),
        m_hist: b.alloc_qubits(max_iters),
        f_flag: b.alloc_qubit(),
    }
}

pub(crate) fn free_kaliski_state(b: &mut B, st: KaliskiState) {
    b.free(st.f_flag);
    b.free_vec(&st.m_hist);
    b.free_vec(&st.s);
    b.free_vec(&st.r);
    b.free_vec(&st.v_w);
    b.free_vec(&st.u);
}

pub(crate) fn free_kaliski_state_borrowed_v(b: &mut B, st: KaliskiState) {
    b.free(st.f_flag);
    b.free_vec(&st.m_hist);
    b.free_vec(&st.s);
    b.free_vec(&st.r);
    b.free_vec(&st.u);
}

pub(crate) fn kaliski_forward_borrowing_v(b: &mut B, st: &KaliskiState, p: U256, iters: usize) {
    debug_assert!(iters <= st.m_hist.len());
    kaliski_forward_loaded_v(b, st, p, iters, None);
}

pub(crate) fn kaliski_forward_loaded_v(
    b: &mut B,
    st: &KaliskiState,
    p: U256,
    iters: usize,
    coeff: Option<(&[QubitId], &[QubitId])>,
) {
    if let Some((cr, cs)) = coeff {
        assert_eq!(cr.len(), st.u.len());
        assert_eq!(cs.len(), st.u.len());
    }
    // s := 1
    b.x(st.s[0]);
    // f := 1
    b.x(st.f_flag);

    // ─── Iterations ───
    let use_bulk_prefix3 = bulk_prefix_enabled();
    let bulk_prefix_iters = bulk_prefix_safe_iters();
    for i in 0..iters {
        if use_bulk_prefix3 && i < bulk_prefix_iters {
            kaliski_iteration_bulk_prefix3(
                b,
                p,
                &st.u,
                &st.v_w,
                &st.r,
                &st.s,
                st.m_hist[i],
                i,
                coeff,
            );
        } else {
            kaliski_iteration(
                b,
                p,
                &st.u,
                &st.v_w,
                &st.r,
                &st.s,
                st.m_hist[i],
                st.f_flag,
                i,
                coeff,
            );
        }
    }

    // After the loop for nonzero v_in, classical invariants give:
    //   u = 1, v_w = 0, f = 0, a = b = add = 0
    //   r = raw coefficient (the NEGATIVE form: r = -v^{-1} * 2^{2n} mod p)
    //   s = some coefficient
    // We skip the `x(r); add_nbit_const(r, p+1)` negation (~2n CCX per call,
    // 4 calls total ≈ 8n Toffoli saved). Callers compensate by using the
    // negated inv: body multiplications that would normally `mul_add` with
    // +inv become `mul_sub` with -inv, and vice versa.
}

/// Like `with_eq_zero` but uses measurement-based uncomputation for the
/// backward OR chain (0 Toffoli instead of n-1 CCX). NOT safe inside
/// emit_inverse blocks (uses HMR ops).
pub(crate) fn with_eq_zero_fast<F: FnOnce(&mut B)>(b: &mut B, v: &[QubitId], flag: QubitId, body: F) {
    let n = v.len();
    assert!(n > 0);
    if n == 1 {
        b.x(v[0]);
        b.cx(v[0], flag);
        body(b);
        b.cx(v[0], flag);
        b.x(v[0]);
        return;
    }
    let or_chain: Vec<QubitId> = b.alloc_qubits(n - 1);
    // Forward OR chain (n-1 CCX)
    or_step(b, v[0], v[1], or_chain[0]);
    for i in 1..n - 1 {
        or_step(b, or_chain[i - 1], v[i + 1], or_chain[i]);
    }
    b.x(or_chain[n - 2]);
    b.cx(or_chain[n - 2], flag);
    b.x(or_chain[n - 2]);
    body(b);
    b.x(or_chain[n - 2]);
    b.cx(or_chain[n - 2], flag);
    b.x(or_chain[n - 2]);
    // Measurement-based uncompute (0 Toffoli)
    for i in (1..n - 1).rev() {
        or_step_uncompute(b, or_chain[i - 1], v[i + 1], or_chain[i]);
    }
    or_step_uncompute(b, v[0], v[1], or_chain[0]);
    b.free_vec(&or_chain);
}

/// Measurement-based uncompute of one or_step: uncomputes
/// `out = x OR y` using HMR + CZ (0 Toffoli).
/// Precondition: out = x OR y (was computed by or_step(x, y, out)).
/// After this: out = 0.
pub(crate) fn or_step_uncompute(b: &mut B, x: QubitId, y: QubitId, out: QubitId) {
    // out currently holds NOT((NOT x) AND (NOT y)) = x OR y.
    // Flip to get the AND value: (NOT x) AND (NOT y).
    b.x(out);
    // Now match the AND controls: flip x and y.
    b.x(x);
    b.x(y);
    let m = b.alloc_bit();
    b.hmr(out, m); // measure; out → 0
    b.cz_if(x, y, m); // phase correction with (NOT x_orig, NOT y_orig) controls
    b.x(y);
    b.x(x);
}

pub(crate) fn kaliski_backward_borrowing_v(b: &mut B, st: &KaliskiState, p: U256, iters: usize) {
    let n = st.v_w.len();
    debug_assert!(iters <= st.m_hist.len());

    let use_bulk_prefix3 = bulk_prefix_enabled();
    let bulk_prefix_iters = bulk_prefix_safe_iters();
    for i in (0..iters).rev() {
        if use_bulk_prefix3 && i < bulk_prefix_iters {
            kaliski_iteration_bulk_prefix3_backward(
                b,
                p,
                &st.u,
                &st.v_w,
                &st.r,
                &st.s,
                st.m_hist[i],
                i,
                None,
            );
        } else {
            kaliski_iteration_backward(
                b,
                p,
                &st.u,
                &st.v_w,
                &st.r,
                &st.s,
                st.m_hist[i],
                st.f_flag,
                i,
                None,
            );
        }
    }

    b.x(st.f_flag);
    b.x(st.s[0]);
    for i in 0..n {
        if bit(p, i) {
            b.x(st.u[i]);
        }
    }
}

pub(crate) fn with_eq_const_fast<F: FnOnce(&mut B)>(
    b: &mut B,
    bits: &[QubitId],
    c: usize,
    flag: QubitId,
    body: F,
) {
    for (i, &q) in bits.iter().enumerate() {
        if ((c >> i) & 1) != 0 {
            b.x(q);
        }
    }
    with_eq_zero_fast(b, bits, flag, body);
    for (i, &q) in bits.iter().enumerate() {
        if ((c >> i) & 1) != 0 {
            b.x(q);
        }
    }
}

pub(crate) fn hmr_reset_unknown_vec(b: &mut B, qs: &[QubitId]) {
    if round499_zero_condition_hmr_erase_enabled() {
        let zero = b.alloc_bit();
        b.bit_store0(zero);
        for &q in qs {
            b.hmr_if(q, zero, zero);
        }
        b.free_vec(qs);
        return;
    }

    let masks = b.alloc_bits(qs.len());
    for (&q, &m) in qs.iter().zip(masks.iter()) {
        b.hmr(q, m);
    }
    b.free_vec(qs);
}

pub(crate) fn hmr_reset_unknown_vec_with_masks(b: &mut B, qs: &[QubitId]) -> Vec<BitId> {
    let masks = b.alloc_bits(qs.len());
    for (&q, &m) in qs.iter().zip(masks.iter()) {
        b.hmr(q, m);
    }
    b.free_vec(qs);
    masks
}

pub(crate) fn hmr_discard_kaliski_non_inverse_phase_dirty(b: &mut B, st: &KaliskiState, p: U256) {
    b.set_phase("kal_hmr_discard_known_terminal_state");
    b.x(st.u[0]);
    b.free_vec(&st.u);
    b.free_vec(&st.v_w);
    b.free(st.f_flag);

    b.set_phase("kal_hmr_discard_clean_terminal_s");
    for i in 0..st.s.len() {
        if bit(p, i) {
            b.x(st.s[i]);
        }
    }
    b.free_vec(&st.s);

    b.set_phase("kal_hmr_discard_non_inverse_state_phase_dirty");
    if let Some((&m0, rest)) = st.m_hist.split_first() {
        // For nonzero denominators, the first Kaliski branch bit is always 1:
        // u starts at the odd prime p, v is nonzero, and the step1/step2
        // updates force m0 high regardless of v's low bit.  Free it as a
        // known constant instead of paying an avoidable HMR phase term.
        b.x(m0);
        b.free(m0);
        let zero_tail = std::env::var("D1_DIRECT_QUOTIENT_HMR_CLEAR_ZERO_M_TAIL")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0)
            .min(rest.len());
        assert!(
            zero_tail == 0
                || std::env::var("D1_DIRECT_QUOTIENT_HMR_CLEAR_ZERO_M_TAIL_UNSAFE_PROBE")
                    .ok()
                    .as_deref()
                    == Some("1"),
            "D1_DIRECT_QUOTIENT_HMR_CLEAR_ZERO_M_TAIL is not stable under the Google hash harness: \
             changing the opstream changes the Fiat-Shamir samples and the cleared tail can add \
             uncaptured R phase. Set D1_DIRECT_QUOTIENT_HMR_CLEAR_ZERO_M_TAIL_UNSAFE_PROBE=1 only \
             to reproduce the rejected diagnostic."
        );
        if zero_tail == 0 {
            hmr_reset_unknown_vec(b, rest);
        } else {
            let split = rest.len() - zero_tail;
            let (unknown, known_zero) = rest.split_at(split);
            hmr_reset_unknown_vec(b, unknown);
            b.free_vec(known_zero);
        }
    }
}

pub(crate) fn hmr_discard_kaliski_non_inverse_transcript_repair(
    b: &mut B,
    v_in: &[QubitId],
    st: &KaliskiState,
    p: U256,
    iters: usize,
) {
    b.set_phase("kal_hmr_discard_known_terminal_state");
    b.x(st.u[0]);
    b.free_vec(&st.u);
    b.free_vec(&st.v_w);
    b.free(st.f_flag);

    b.set_phase("kal_hmr_discard_clean_terminal_s");
    for i in 0..st.s.len() {
        if bit(p, i) {
            b.x(st.s[i]);
        }
    }
    b.free_vec(&st.s);

    b.set_phase("kal_hmr_discard_non_inverse_state_hmr_masks");
    let masks = hmr_reset_unknown_vec_with_masks(b, &st.m_hist);

    b.set_phase("kal_hmr_discard_non_inverse_transcript_recompute");
    let mut repair = alloc_kaliski_branch_state_no_add(b, v_in.len(), iters);
    let term_bits = b.alloc_qubits(9);
    kaliski_branch_record_forward_term(b, v_in, &repair, &term_bits, p, iters);

    b.set_phase("kal_hmr_discard_non_inverse_transcript_apply_phase");
    for (&q, &m) in repair.m_hist.iter().zip(masks.iter()) {
        b.z_if(q, m);
    }

    b.set_phase("kal_hmr_discard_non_inverse_transcript_uncompute");
    kaliski_branch_record_backward_term(b, v_in, &repair, &term_bits, p, iters);
    b.free_vec(&term_bits);
    free_kaliski_branch_state(b, repair);
}

pub(crate) fn hmr_discard_kaliski_inverse_phase_dirty(b: &mut B, st: KaliskiState) {
    if std::env::var("D1_DIRECT_QUOTIENT_HMR_CLEAR_R_RAW_UNSAFE_PROBE")
        .ok()
        .as_deref()
        == Some("1")
    {
        b.set_phase("kal_hmr_discard_inverse_state_free_raw_r_unsafe_probe");
        b.free_vec(&st.r);
        return;
    }
    b.set_phase("kal_hmr_discard_inverse_state_phase_dirty");
    hmr_reset_unknown_vec(b, &st.r);
}

pub(crate) fn hmr_discard_kaliski_inverse_raw_repair(
    b: &mut B,
    v_in: &[QubitId],
    st: KaliskiState,
    p: U256,
    iters: usize,
) {
    b.set_phase("kal_hmr_discard_inverse_state_hmr_masks");
    let masks = hmr_reset_unknown_vec_with_masks(b, &st.r);

    b.set_phase("kal_hmr_discard_inverse_raw_recompute");
    with_kal_inv_raw(b, v_in, p, iters, |b, inv_raw| {
        b.set_phase("kal_hmr_discard_inverse_raw_apply_phase");
        for (&q, &m) in inv_raw.iter().zip(masks.iter()) {
            b.z_if(q, m);
        }
    });
}

pub(crate) fn kaliski_forward_prescaled_mixed(
    b: &mut B,
    v_in: &[QubitId],
    st: &KaliskiState,
    p: U256,
    iters: usize,
    scale: U256,
) {
    kaliski_forward_prescaled_kind(b, v_in, st, p, iters, scale, false);
}

pub(crate) fn kaliski_forward_prescaled_chunked(
    b: &mut B,
    v_in: &[QubitId],
    st: &KaliskiState,
    p: U256,
    iters: usize,
    scale: U256,
) {
    kaliski_forward_prescaled_kind(b, v_in, st, p, iters, scale, true);
}

pub(crate) fn kaliski_forward_prescaled_kind(
    b: &mut B,
    v_in: &[QubitId],
    st: &KaliskiState,
    p: U256,
    iters: usize,
    scale: U256,
    chunked: bool,
) {
    let n = v_in.len();
    debug_assert!(iters <= st.m_hist.len());

    for i in 0..n {
        if bit(p, i) {
            b.x(st.u[i]);
        }
    }
    if chunked {
        mul_by_const_acc_chunked_shifts_inplace_src(b, v_in, scale, &st.v_w, p, false);
    } else {
        mul_by_const_acc_exact_adds_fast_shifts(b, v_in, scale, &st.v_w, p, false);
    }
    b.x(st.s[0]);
    b.x(st.f_flag);

    let use_bulk_prefix3 = bulk_prefix_enabled();
    let bulk_prefix_iters = bulk_prefix_safe_iters();
    for i in 0..iters {
        if use_bulk_prefix3 && i < bulk_prefix_iters {
            kaliski_iteration_bulk_prefix3(
                b,
                p,
                &st.u,
                &st.v_w,
                &st.r,
                &st.s,
                st.m_hist[i],
                i,
                None,
            );
        } else {
            kaliski_iteration(
                b,
                p,
                &st.u,
                &st.v_w,
                &st.r,
                &st.s,
                st.m_hist[i],
                st.f_flag,
                i,
                None,
            );
        }
    }
}

pub(crate) fn kaliski_backward_prescaled_mixed(
    b: &mut B,
    v_in: &[QubitId],
    st: &KaliskiState,
    p: U256,
    iters: usize,
    scale: U256,
) {
    kaliski_backward_prescaled_kind(b, v_in, st, p, iters, scale, false);
}

pub(crate) fn kaliski_backward_prescaled_chunked(
    b: &mut B,
    v_in: &[QubitId],
    st: &KaliskiState,
    p: U256,
    iters: usize,
    scale: U256,
) {
    kaliski_backward_prescaled_kind(b, v_in, st, p, iters, scale, true);
}

pub(crate) fn kaliski_backward_prescaled_kind(
    b: &mut B,
    v_in: &[QubitId],
    st: &KaliskiState,
    p: U256,
    iters: usize,
    scale: U256,
    chunked: bool,
) {
    let n = v_in.len();
    debug_assert!(iters <= st.m_hist.len());

    let use_bulk_prefix3 = bulk_prefix_enabled();
    let bulk_prefix_iters = bulk_prefix_safe_iters();
    for i in (0..iters).rev() {
        if use_bulk_prefix3 && i < bulk_prefix_iters {
            kaliski_iteration_bulk_prefix3_backward(
                b,
                p,
                &st.u,
                &st.v_w,
                &st.r,
                &st.s,
                st.m_hist[i],
                i,
                None,
            );
        } else {
            kaliski_iteration_backward(
                b,
                p,
                &st.u,
                &st.v_w,
                &st.r,
                &st.s,
                st.m_hist[i],
                st.f_flag,
                i,
                None,
            );
        }
    }

    b.x(st.f_flag);
    b.x(st.s[0]);
    if chunked {
        mul_by_const_acc_chunked_shifts_inplace_src(b, v_in, scale, &st.v_w, p, true);
    } else {
        mul_by_const_acc_exact_adds_fast_shifts(b, v_in, scale, &st.v_w, p, true);
    }
    for i in 0..n {
        if bit(p, i) {
            b.x(st.u[i]);
        }
    }
}
