
#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

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
