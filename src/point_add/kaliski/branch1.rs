//! `kaliski::branch1` — verbatim split of the original `kaliski` module.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

pub(crate) fn alloc_kaliski_branch_state(b: &mut B, n: usize, max_iters: usize) -> KaliskiBranchState {
    KaliskiBranchState {
        u: b.alloc_qubits(n),
        v_w: b.alloc_qubits(n),
        m_hist: b.alloc_qubits(max_iters),
        a_hist: b.alloc_qubits(max_iters),
        add_hist: b.alloc_qubits(max_iters),
        f_flag: b.alloc_qubit(),
    }
}

pub(crate) fn alloc_kaliski_branch_state_no_add(b: &mut B, n: usize, max_iters: usize) -> KaliskiBranchState {
    KaliskiBranchState {
        u: b.alloc_qubits(n),
        v_w: b.alloc_qubits(n),
        m_hist: b.alloc_qubits(max_iters),
        a_hist: b.alloc_qubits(max_iters),
        add_hist: Vec::new(),
        f_flag: b.alloc_qubit(),
    }
}

pub(crate) fn alloc_kaliski_branch_state_no_add_borrowing_v(
    b: &mut B,
    v_w: &[QubitId],
    max_iters: usize,
) -> KaliskiBranchState {
    KaliskiBranchState {
        u: b.alloc_qubits(v_w.len()),
        v_w: v_w.to_vec(),
        m_hist: b.alloc_qubits(max_iters),
        a_hist: b.alloc_qubits(max_iters),
        add_hist: Vec::new(),
        f_flag: b.alloc_qubit(),
    }
}

pub(crate) fn free_kaliski_branch_state(b: &mut B, st: KaliskiBranchState) {
    b.free(st.f_flag);
    b.free_vec(&st.add_hist);
    b.free_vec(&st.a_hist);
    b.free_vec(&st.m_hist);
    b.free_vec(&st.v_w);
    b.free_vec(&st.u);
}

pub(crate) fn free_kaliski_branch_state_borrowed_v(b: &mut B, st: KaliskiBranchState) {
    b.free(st.f_flag);
    b.free_vec(&st.add_hist);
    b.free_vec(&st.a_hist);
    b.free_vec(&st.m_hist);
    b.free_vec(&st.u);
}

pub(crate) fn kaliski_branch_iteration_with_coeff(
    b: &mut B,
    p: U256,
    u: &[QubitId],
    v_w: &[QubitId],
    m_i: QubitId,
    a_i: QubitId,
    f: QubitId,
    coeff: (&[QubitId], &[QubitId]),
) {
    let n = u.len();
    let b_f = b.alloc_qubit();
    let add_f = b.alloc_qubit();
    let _kal_saved_phase = b.phase;

    b.set_phase("br_step0_eqzero");
    with_eq_zero_fast(b, v_w, add_f, |b| {
        b.ccx(f, add_f, m_i);
    });
    b.cx(m_i, f);

    b.set_phase("br_step1");
    b.ccx(f, u[0], b_f);
    b.cx(f, a_i);
    b.cx(b_f, a_i);
    b.x(v_w[0]);
    b.ccx(b_f, v_w[0], m_i);
    b.x(v_w[0]);
    {
        let zm = b.alloc_bit();
        b.hmr(b_f, zm);
        b.cz_if(f, u[0], zm);
    }
    b.cx(a_i, b_f);
    b.cx(m_i, b_f);

    b.set_phase("br_step2");
    let l_gt = b.alloc_qubit();
    kal_step2_with_gt(b, u, v_w, l_gt, |b| {
        b.x(b_f);
        b.ccx(f, l_gt, add_f);
        let t = b.alloc_qubit();
        b.ccx(add_f, b_f, t);
        b.cx(t, a_i);
        b.cx(t, m_i);
        {
            let tm = b.alloc_bit();
            b.hmr(t, tm);
            b.cz_if(add_f, b_f, tm);
        }
        b.free(t);
        {
            let am = b.alloc_bit();
            b.hmr(add_f, am);
            b.cz_if(f, l_gt, am);
        }
        b.x(b_f);
    });
    b.free(l_gt);

    b.set_phase("br_step3_cswap");
    for j in 0..n {
        cswap(b, a_i, u[j], v_w[j]);
    }
    coeff_channel_cswap(b, a_i, coeff.0, coeff.1);

    b.set_phase("br_step4");
    mcx2_polar(b, f, true, b_f, false, add_f);
    cucc_sub_ctrl(b, u, v_w, add_f);
    b.set_phase("br_coeff_step4_add");
    coeff_channel_cadd(b, p, coeff.0, coeff.1, add_f);

    b.set_phase("br_step5");
    b.x(b_f);
    {
        let sm = b.alloc_bit();
        b.hmr(add_f, sm);
        b.cz_if(f, b_f, sm);
    }
    b.x(b_f);
    b.cx(m_i, b_f);
    b.cx(a_i, b_f);
    b.free(add_f);
    b.free(b_f);

    b.set_phase("br_step6_8");
    for i in 0..(n - 1) {
        b.swap(v_w[i], v_w[i + 1]);
    }
    coeff_channel_double(b, p, coeff.0);

    b.set_phase("br_step9_cswap");
    for j in 0..n {
        cswap(b, a_i, u[j], v_w[j]);
    }
    coeff_channel_cswap(b, a_i, coeff.0, coeff.1);

    b.set_phase(_kal_saved_phase);
}

pub(crate) fn kaliski_branch_iteration_backward_with_coeff(
    b: &mut B,
    p: U256,
    u: &[QubitId],
    v_w: &[QubitId],
    m_i: QubitId,
    a_i: QubitId,
    f: QubitId,
    coeff: (&[QubitId], &[QubitId]),
) {
    let n = u.len();
    let b_f = b.alloc_qubit();
    let add_f = b.alloc_qubit();
    let _kal_saved_phase = b.phase;

    b.cx(a_i, b_f);
    b.cx(m_i, b_f);
    mcx2_polar(b, f, true, b_f, false, add_f);

    b.set_phase("br_coeff_bk_step9_cswap");
    coeff_channel_cswap(b, a_i, coeff.0, coeff.1);
    b.set_phase("br_bk_step9_cswap");
    for j in (0..n).rev() {
        cswap(b, a_i, u[j], v_w[j]);
    }

    b.set_phase("br_coeff_bk_step6_halve");
    mod_halve_inplace_fast(b, coeff.0, p);
    b.set_phase("br_bk_step6");
    for i in (0..(n - 1)).rev() {
        b.swap(v_w[i], v_w[i + 1]);
    }

    b.set_phase("br_coeff_bk_step4_sub");
    coeff_channel_csub(b, p, coeff.0, coeff.1, add_f);
    b.set_phase("br_bk_step4");
    cucc_add_ctrl(b, u, v_w, add_f);

    b.set_phase("br_bk_step5_unadd");
    b.x(b_f);
    {
        let sm = b.alloc_bit();
        b.hmr(add_f, sm);
        b.cz_if(f, b_f, sm);
    }
    b.x(b_f);

    b.set_phase("br_coeff_bk_step3_cswap");
    coeff_channel_cswap(b, a_i, coeff.0, coeff.1);
    b.set_phase("br_bk_step3_cswap");
    for j in (0..n).rev() {
        cswap(b, a_i, u[j], v_w[j]);
    }

    b.set_phase("br_bk_step2");
    let l_gt = b.alloc_qubit();
    kal_step2_with_gt(b, u, v_w, l_gt, |b| {
        b.x(b_f);
        b.ccx(f, l_gt, add_f);
        let t = b.alloc_qubit();
        b.ccx(add_f, b_f, t);
        b.cx(t, m_i);
        b.cx(t, a_i);
        {
            let tm = b.alloc_bit();
            b.hmr(t, tm);
            b.cz_if(add_f, b_f, tm);
        }
        b.free(t);
        {
            let am = b.alloc_bit();
            b.hmr(add_f, am);
            b.cz_if(f, l_gt, am);
        }
        b.x(b_f);
    });
    b.free(l_gt);

    b.set_phase("br_bk_step1");
    b.cx(m_i, b_f);
    b.cx(a_i, b_f);
    b.ccx(f, u[0], b_f);
    b.x(v_w[0]);
    b.ccx(b_f, v_w[0], m_i);
    b.x(v_w[0]);
    b.cx(b_f, a_i);
    b.cx(f, a_i);
    {
        let zm = b.alloc_bit();
        b.hmr(b_f, zm);
        b.cz_if(f, u[0], zm);
    }

    b.set_phase("br_bk_step0_eqzero");
    b.cx(m_i, f);
    with_eq_zero_fast(b, v_w, add_f, |b| {
        b.ccx(f, add_f, m_i);
    });

    b.free(add_f);
    b.free(b_f);
    b.set_phase(_kal_saved_phase);
}

pub(crate) fn kaliski_branch_iteration_record(
    b: &mut B,
    u: &[QubitId],
    v_w: &[QubitId],
    m_i: QubitId,
    a_i: QubitId,
    add_i: Option<QubitId>,
    term_bits: Option<(&[QubitId], usize)>,
    f: QubitId,
) {
    let n = u.len();
    let b_f = b.alloc_qubit();
    let add_f = b.alloc_qubit();
    let _kal_saved_phase = b.phase;

    b.set_phase("br_rec_step0_eqzero");
    with_eq_zero_fast(b, v_w, add_f, |b| {
        b.ccx(f, add_f, m_i);
        if let Some((term_bits, iter_idx)) = term_bits {
            for (j, &q) in term_bits.iter().enumerate() {
                if ((iter_idx >> j) & 1) != 0 {
                    b.cx(m_i, q);
                }
            }
        }
    });
    b.cx(m_i, f);

    b.set_phase("br_rec_step1");
    b.ccx(f, u[0], b_f);
    b.cx(f, a_i);
    b.cx(b_f, a_i);
    b.x(v_w[0]);
    b.ccx(b_f, v_w[0], m_i);
    b.x(v_w[0]);
    {
        let zm = b.alloc_bit();
        b.hmr(b_f, zm);
        b.cz_if(f, u[0], zm);
    }
    b.cx(a_i, b_f);
    b.cx(m_i, b_f);

    b.set_phase("br_rec_step2");
    let l_gt = b.alloc_qubit();
    kal_step2_with_gt(b, u, v_w, l_gt, |b| {
        b.x(b_f);
        b.ccx(f, l_gt, add_f);
        let t = b.alloc_qubit();
        b.ccx(add_f, b_f, t);
        b.cx(t, a_i);
        b.cx(t, m_i);
        {
            let tm = b.alloc_bit();
            b.hmr(t, tm);
            b.cz_if(add_f, b_f, tm);
        }
        b.free(t);
        {
            let am = b.alloc_bit();
            b.hmr(add_f, am);
            b.cz_if(f, l_gt, am);
        }
        b.x(b_f);
    });
    b.free(l_gt);

    b.set_phase("br_rec_step3_cswap");
    for j in 0..n {
        cswap(b, a_i, u[j], v_w[j]);
    }

    b.set_phase("br_rec_step4");
    mcx2_polar(b, f, true, b_f, false, add_f);
    if let Some(add_i) = add_i {
        b.cx(add_f, add_i);
    }
    cucc_sub_ctrl(b, u, v_w, add_f);

    b.set_phase("br_rec_step5");
    b.x(b_f);
    {
        let sm = b.alloc_bit();
        b.hmr(add_f, sm);
        b.cz_if(f, b_f, sm);
    }
    b.x(b_f);
    b.cx(m_i, b_f);
    b.cx(a_i, b_f);
    b.free(add_f);
    b.free(b_f);

    b.set_phase("br_rec_step6");
    for i in 0..(n - 1) {
        b.swap(v_w[i], v_w[i + 1]);
    }

    b.set_phase("br_rec_step9_cswap");
    for j in 0..n {
        cswap(b, a_i, u[j], v_w[j]);
    }

    b.set_phase(_kal_saved_phase);
}

pub(crate) fn kaliski_branch_iteration_backward_recorded(
    b: &mut B,
    u: &[QubitId],
    v_w: &[QubitId],
    m_i: QubitId,
    a_i: QubitId,
    add_i: QubitId,
    f: QubitId,
) {
    let n = u.len();
    let b_f = b.alloc_qubit();
    let add_f = b.alloc_qubit();
    let _kal_saved_phase = b.phase;

    b.cx(a_i, b_f);
    b.cx(m_i, b_f);
    mcx2_polar(b, f, true, b_f, false, add_f);

    b.set_phase("br_rec_bk_step9_cswap");
    for j in (0..n).rev() {
        cswap(b, a_i, u[j], v_w[j]);
    }

    b.set_phase("br_rec_bk_step6");
    for i in (0..(n - 1)).rev() {
        b.swap(v_w[i], v_w[i + 1]);
    }

    b.set_phase("br_rec_bk_step4");
    cucc_add_ctrl(b, u, v_w, add_f);
    b.cx(add_f, add_i);

    b.set_phase("br_rec_bk_step5_unadd");
    b.x(b_f);
    {
        let sm = b.alloc_bit();
        b.hmr(add_f, sm);
        b.cz_if(f, b_f, sm);
    }
    b.x(b_f);

    b.set_phase("br_rec_bk_step3_cswap");
    for j in (0..n).rev() {
        cswap(b, a_i, u[j], v_w[j]);
    }

    b.set_phase("br_rec_bk_step2");
    let l_gt = b.alloc_qubit();
    kal_step2_with_gt(b, u, v_w, l_gt, |b| {
        b.x(b_f);
        b.ccx(f, l_gt, add_f);
        let t = b.alloc_qubit();
        b.ccx(add_f, b_f, t);
        b.cx(t, m_i);
        b.cx(t, a_i);
        {
            let tm = b.alloc_bit();
            b.hmr(t, tm);
            b.cz_if(add_f, b_f, tm);
        }
        b.free(t);
        {
            let am = b.alloc_bit();
            b.hmr(add_f, am);
            b.cz_if(f, l_gt, am);
        }
        b.x(b_f);
    });
    b.free(l_gt);

    b.set_phase("br_rec_bk_step1");
    b.cx(m_i, b_f);
    b.cx(a_i, b_f);
    b.ccx(f, u[0], b_f);
    b.x(v_w[0]);
    b.ccx(b_f, v_w[0], m_i);
    b.x(v_w[0]);
    b.cx(b_f, a_i);
    b.cx(f, a_i);
    {
        let zm = b.alloc_bit();
        b.hmr(b_f, zm);
        b.cz_if(f, u[0], zm);
    }

    b.set_phase("br_rec_bk_step0_eqzero");
    b.cx(m_i, f);
    with_eq_zero_fast(b, v_w, add_f, |b| {
        b.ccx(f, add_f, m_i);
    });

    b.free(add_f);
    b.free(b_f);
    b.set_phase(_kal_saved_phase);
}

pub(crate) fn kaliski_branch_iteration_backward(
    b: &mut B,
    u: &[QubitId],
    v_w: &[QubitId],
    m_i: QubitId,
    a_i: QubitId,
    term_bits: Option<(&[QubitId], usize)>,
    f: QubitId,
) {
    let n = u.len();
    let b_f = b.alloc_qubit();
    let add_f = b.alloc_qubit();
    let _kal_saved_phase = b.phase;

    b.cx(a_i, b_f);
    b.cx(m_i, b_f);
    mcx2_polar(b, f, true, b_f, false, add_f);

    b.set_phase("br_bk_step9_cswap");
    for j in (0..n).rev() {
        cswap(b, a_i, u[j], v_w[j]);
    }

    b.set_phase("br_bk_step6");
    for i in (0..(n - 1)).rev() {
        b.swap(v_w[i], v_w[i + 1]);
    }

    b.set_phase("br_bk_step4");
    cucc_add_ctrl(b, u, v_w, add_f);

    b.set_phase("br_bk_step5_unadd");
    b.x(b_f);
    {
        let sm = b.alloc_bit();
        b.hmr(add_f, sm);
        b.cz_if(f, b_f, sm);
    }
    b.x(b_f);

    b.set_phase("br_bk_step3_cswap");
    for j in (0..n).rev() {
        cswap(b, a_i, u[j], v_w[j]);
    }

    b.set_phase("br_bk_step2");
    let l_gt = b.alloc_qubit();
    kal_step2_with_gt(b, u, v_w, l_gt, |b| {
        b.x(b_f);
        b.ccx(f, l_gt, add_f);
        let t = b.alloc_qubit();
        b.ccx(add_f, b_f, t);
        b.cx(t, m_i);
        b.cx(t, a_i);
        {
            let tm = b.alloc_bit();
            b.hmr(t, tm);
            b.cz_if(add_f, b_f, tm);
        }
        b.free(t);
        {
            let am = b.alloc_bit();
            b.hmr(add_f, am);
            b.cz_if(f, l_gt, am);
        }
        b.x(b_f);
    });
    b.free(l_gt);

    b.set_phase("br_bk_step1");
    b.cx(m_i, b_f);
    b.cx(a_i, b_f);
    b.ccx(f, u[0], b_f);
    b.x(v_w[0]);
    b.ccx(b_f, v_w[0], m_i);
    b.x(v_w[0]);
    b.cx(b_f, a_i);
    b.cx(f, a_i);
    {
        let zm = b.alloc_bit();
        b.hmr(b_f, zm);
        b.cz_if(f, u[0], zm);
    }

    b.set_phase("br_bk_step0_eqzero");
    if let Some((term_bits, iter_idx)) = term_bits {
        for (j, &q) in term_bits.iter().enumerate() {
            if ((iter_idx >> j) & 1) != 0 {
                b.cx(m_i, q);
            }
        }
    }
    b.cx(m_i, f);
    with_eq_zero_fast(b, v_w, add_f, |b| {
        b.ccx(f, add_f, m_i);
    });

    b.free(add_f);
    b.free(b_f);
    b.set_phase(_kal_saved_phase);
}

pub(crate) fn kaliski_branch_forward_with_coeff(
    b: &mut B,
    v_in: &[QubitId],
    st: &KaliskiBranchState,
    p: U256,
    iters: usize,
    coeff: (&[QubitId], &[QubitId]),
) {
    let n = v_in.len();
    for i in 0..n {
        if bit(p, i) {
            b.x(st.u[i]);
        }
        b.cx(v_in[i], st.v_w[i]);
    }
    b.x(st.f_flag);
    for i in 0..iters {
        kaliski_branch_iteration_with_coeff(
            b,
            p,
            &st.u,
            &st.v_w,
            st.m_hist[i],
            st.a_hist[i],
            st.f_flag,
            coeff,
        );
    }
}

pub(crate) fn kaliski_branch_forward_with_coeff_borrowing_v(
    b: &mut B,
    st: &KaliskiBranchState,
    p: U256,
    iters: usize,
    coeff: (&[QubitId], &[QubitId]),
) {
    let n = st.v_w.len();
    for i in 0..n {
        if bit(p, i) {
            b.x(st.u[i]);
        }
    }
    b.x(st.f_flag);
    for i in 0..iters {
        kaliski_branch_iteration_with_coeff(
            b,
            p,
            &st.u,
            &st.v_w,
            st.m_hist[i],
            st.a_hist[i],
            st.f_flag,
            coeff,
        );
    }
}

pub(crate) fn kaliski_branch_backward(
    b: &mut B,
    v_in: &[QubitId],
    st: &KaliskiBranchState,
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
            None,
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

pub(crate) fn kaliski_branch_backward_with_coeff_borrowing_v(
    b: &mut B,
    st: &KaliskiBranchState,
    p: U256,
    iters: usize,
    coeff: (&[QubitId], &[QubitId]),
) {
    let n = st.v_w.len();
    for i in (0..iters).rev() {
        kaliski_branch_iteration_backward_with_coeff(
            b,
            p,
            &st.u,
            &st.v_w,
            st.m_hist[i],
            st.a_hist[i],
            st.f_flag,
            coeff,
        );
    }
    b.x(st.f_flag);
    for i in 0..n {
        if bit(p, i) {
            b.x(st.u[i]);
        }
    }
}

pub(crate) fn kaliski_branch_record_forward(
    b: &mut B,
    v_in: &[QubitId],
    st: &KaliskiBranchState,
    p: U256,
    iters: usize,
) {
    let n = v_in.len();
    for i in 0..n {
        if bit(p, i) {
            b.x(st.u[i]);
        }
        b.cx(v_in[i], st.v_w[i]);
    }
    b.x(st.f_flag);
    for i in 0..iters {
        kaliski_branch_iteration_record(
            b,
            &st.u,
            &st.v_w,
            st.m_hist[i],
            st.a_hist[i],
            Some(st.add_hist[i]),
            None,
            st.f_flag,
        );
    }
}

pub(crate) fn kaliski_branch_record_backward(
    b: &mut B,
    v_in: &[QubitId],
    st: &KaliskiBranchState,
    p: U256,
    iters: usize,
) {
    let n = v_in.len();
    for i in (0..iters).rev() {
        kaliski_branch_iteration_backward_recorded(
            b,
            &st.u,
            &st.v_w,
            st.m_hist[i],
            st.a_hist[i],
            st.add_hist[i],
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

pub(crate) fn kaliski_branch_record_forward_term(
    b: &mut B,
    v_in: &[QubitId],
    st: &KaliskiBranchState,
    term_bits: &[QubitId],
    p: U256,
    iters: usize,
) {
    let n = v_in.len();
    for i in 0..n {
        if bit(p, i) {
            b.x(st.u[i]);
        }
        b.cx(v_in[i], st.v_w[i]);
    }
    b.x(st.f_flag);
    for i in 0..iters {
        kaliski_branch_iteration_record(
            b,
            &st.u,
            &st.v_w,
            st.m_hist[i],
            st.a_hist[i],
            None,
            Some((term_bits, i)),
            st.f_flag,
        );
    }
}
