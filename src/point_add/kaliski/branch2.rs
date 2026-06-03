//! `kaliski::branch2` — verbatim split of the original `kaliski` module.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

pub(crate) fn kaliski_branch_record_backward_term(
    b: &mut B,
    v_in: &[QubitId],
    st: &KaliskiBranchState,
    term_bits: &[QubitId],
    p: U256,
    iters: usize,
) {
    let n = v_in.len();
    for i in (0..iters).rev() {
        kaliski_branch_iteration_backward(
            b,
            &st.u,
            &st.v_w,
            st.m_hist[i],
            st.a_hist[i],
            Some((term_bits, i)),
            st.f_flag,
        );
    }
    b.x(st.f_flag);
    for i in 0..n {
        b.cx(v_in[i], st.v_w[i]);
        if bit(p, i) {
            b.x(st.u[i]);
        }
    }
}

pub(crate) fn with_kal_branch_inv_raw_roll<F: FnOnce(&mut B, &[QubitId])>(
    b: &mut B,
    v_in: &[QubitId],
    p: U256,
    iters: usize,
    body: F,
) {
    let n = v_in.len();
    let mut st = alloc_kaliski_branch_state_no_add(b, n, iters);
    let term_bits = b.alloc_qubits(9);
    kaliski_branch_record_forward_term(b, v_in, &st, &term_bits, p, iters);

    // Final denominator state is known when iters is beyond the convergence
    // tail. Free it so coefficient replay carries only histories + inv coeffs.
    b.x(st.u[0]);
    b.free_vec(&st.u);
    b.free_vec(&st.v_w);
    b.free(st.f_flag);

    let inv_raw = b.alloc_qubits(n);
    let coeff_s = b.alloc_qubits(n);
    b.x(coeff_s[0]);
    apply_coeff_channel_from_term_roll(
        b, p, &inv_raw, &coeff_s, &st.a_hist, &st.m_hist, &term_bits,
    );

    body(b, &inv_raw);

    apply_coeff_channel_from_term_roll_inverse(
        b, p, &inv_raw, &coeff_s, &st.a_hist, &st.m_hist, &term_bits,
    );
    b.x(coeff_s[0]);
    b.free_vec(&coeff_s);
    b.free_vec(&inv_raw);

    st.u = b.alloc_qubits(n);
    st.v_w = b.alloc_qubits(n);
    st.f_flag = b.alloc_qubit();
    b.x(st.u[0]);
    kaliski_branch_record_backward_term(b, v_in, &st, &term_bits, p, iters);
    b.free_vec(&term_bits);
    free_kaliski_branch_state(b, st);
}

pub(crate) fn with_kal_branch_term_roll_tagged_div<F: FnOnce(&mut B)>(
    b: &mut B,
    v_in: &[QubitId],
    p: U256,
    iters: usize,
    coeff: (&[QubitId], &[QubitId]),
    body: F,
) {
    let n = v_in.len();
    let mut st = alloc_kaliski_branch_state_no_add(b, n, iters);
    let term_bits = b.alloc_qubits(9);
    kaliski_branch_record_forward_term(b, v_in, &st, &term_bits, p, iters);

    b.x(st.u[0]);
    b.free_vec(&st.u);
    b.free_vec(&st.v_w);
    b.free(st.f_flag);

    apply_coeff_channel_from_term_roll(b, p, coeff.0, coeff.1, &st.a_hist, &st.m_hist, &term_bits);
    body(b);

    st.u = b.alloc_qubits(n);
    st.v_w = b.alloc_qubits(n);
    st.f_flag = b.alloc_qubit();
    b.x(st.u[0]);
    kaliski_branch_record_backward_term(b, v_in, &st, &term_bits, p, iters);
    b.free_vec(&term_bits);
    free_kaliski_branch_state(b, st);
}

pub(crate) fn with_kal_branch_term_tagged_div<F: FnOnce(&mut B)>(
    b: &mut B,
    v_in: &[QubitId],
    p: U256,
    iters: usize,
    coeff: (&[QubitId], &[QubitId]),
    body: F,
) {
    let n = v_in.len();
    let mut st = alloc_kaliski_branch_state_no_add(b, n, iters);
    let term_bits = b.alloc_qubits(9);
    kaliski_branch_record_forward_term(b, v_in, &st, &term_bits, p, iters);

    b.x(st.u[0]);
    b.free_vec(&st.u);
    b.free_vec(&st.v_w);
    b.free(st.f_flag);

    apply_coeff_channel_from_term_index(b, p, coeff.0, coeff.1, &st.a_hist, &st.m_hist, &term_bits);
    body(b);

    st.u = b.alloc_qubits(n);
    st.v_w = b.alloc_qubits(n);
    st.f_flag = b.alloc_qubit();
    b.x(st.u[0]);
    kaliski_branch_record_backward_term(b, v_in, &st, &term_bits, p, iters);
    b.free_vec(&term_bits);
    free_kaliski_branch_state(b, st);
}

pub(crate) fn with_kal_branch_stream_tagged_div<F: FnOnce(&mut B)>(
    b: &mut B,
    v_in: &[QubitId],
    p: U256,
    iters: usize,
    coeff: (&[QubitId], &[QubitId]),
    body: F,
) {
    let n = v_in.len();
    let mut st = alloc_kaliski_branch_state(b, n, iters);
    kaliski_branch_record_forward(b, v_in, &st, p, iters);

    // At sufficient iteration count the denominator state is known `(u,v,f)=(1,0,0)`.
    // Free it before the coefficient replay so the replay peak is history + coeff,
    // not history + denominator + coeff.
    b.x(st.u[0]);
    b.free_vec(&st.u);
    b.free_vec(&st.v_w);
    b.free(st.f_flag);

    apply_coeff_channel_from_hist(b, p, coeff.0, coeff.1, &st.a_hist, &st.add_hist);
    body(b);

    st.u = b.alloc_qubits(n);
    st.v_w = b.alloc_qubits(n);
    st.f_flag = b.alloc_qubit();
    b.x(st.u[0]);
    kaliski_branch_record_backward(b, v_in, &st, p, iters);
    free_kaliski_branch_state(b, st);
}

pub(crate) fn with_kal_branch_tagged_div_coeff<F: FnOnce(&mut B)>(
    b: &mut B,
    v_in: &[QubitId],
    p: U256,
    iters: usize,
    coeff: (&[QubitId], &[QubitId]),
    body: F,
) {
    let st = alloc_kaliski_branch_state(b, v_in.len(), iters);
    kaliski_branch_forward_with_coeff(b, v_in, &st, p, iters, coeff);
    body(b);
    kaliski_branch_backward(b, v_in, &st, p, iters);
    free_kaliski_branch_state(b, st);
}
