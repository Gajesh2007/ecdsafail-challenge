//! `kaliski::coeff` — verbatim split of the original `kaliski` module.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

/// Optional side-channel coefficient transform used by the tagged-DIV probe.
/// It applies the same linear Kaliski coefficient update to an external
/// `(cr, cs)` pair while the ordinary inverse state still carries the
/// qrisp sentinel needed to uncompute branch flags.
pub(crate) fn coeff_channel_cswap(b: &mut B, ctrl: QubitId, cr: &[QubitId], cs: &[QubitId]) {
    assert_eq!(cr.len(), cs.len());
    for i in 0..cr.len() {
        cswap(b, ctrl, cr[i], cs[i]);
    }
}

pub(crate) fn coeff_channel_cadd(b: &mut B, p: U256, cr: &[QubitId], cs: &[QubitId], ctrl: QubitId) {
    cmod_add_qq(b, cs, cr, ctrl, p);
}

pub(crate) fn coeff_channel_csub(b: &mut B, p: U256, cr: &[QubitId], cs: &[QubitId], ctrl: QubitId) {
    cmod_sub_qq(b, cs, cr, ctrl, p);
}

pub(crate) fn coeff_channel_double(b: &mut B, p: U256, cr: &[QubitId]) {
    // The data coefficient is an arbitrary field element, not the bounded
    // qrisp inverse coefficient, so the early no-correction shift is invalid.
    mod_double_inplace_fast(b, cr, p);
}

pub(crate) fn kaliski_backward_borrowing_v_with_coeff(
    b: &mut B,
    st: &KaliskiState,
    p: U256,
    iters: usize,
    coeff: (&[QubitId], &[QubitId]),
) {
    let n = st.v_w.len();
    debug_assert!(iters <= st.m_hist.len());
    assert_eq!(coeff.0.len(), n);
    assert_eq!(coeff.1.len(), n);

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
                Some(coeff),
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
                Some(coeff),
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

pub(crate) fn apply_coeff_channel_from_hist(
    b: &mut B,
    p: U256,
    cr: &[QubitId],
    cs: &[QubitId],
    a_hist: &[QubitId],
    add_hist: &[QubitId],
) {
    assert_eq!(a_hist.len(), add_hist.len());
    for i in 0..a_hist.len() {
        b.set_phase("br_stream_coeff_cswap1");
        coeff_channel_cswap(b, a_hist[i], cr, cs);
        b.set_phase("br_stream_coeff_add");
        coeff_channel_cadd(b, p, cr, cs, add_hist[i]);
        b.set_phase("br_stream_coeff_double");
        coeff_channel_double(b, p, cr);
        b.set_phase("br_stream_coeff_cswap2");
        coeff_channel_cswap(b, a_hist[i], cr, cs);
    }
}

pub(crate) fn apply_coeff_channel_from_term_roll(
    b: &mut B,
    p: U256,
    cr: &[QubitId],
    cs: &[QubitId],
    a_hist: &[QubitId],
    m_hist: &[QubitId],
    term_bits: &[QubitId],
) {
    assert_eq!(a_hist.len(), m_hist.len());
    let active = b.alloc_qubit();
    b.x(active); // active before the terminal iteration.
    for i in 0..a_hist.len() {
        b.set_phase("br_roll_term_update");
        let eq_i = b.alloc_qubit();
        with_eq_const_fast(b, term_bits, i, eq_i, |b| {
            b.cx(eq_i, active);
        });
        b.free(eq_i);

        b.set_phase("br_roll_coeff_cswap1");
        coeff_channel_cswap(b, a_hist[i], cr, cs);

        b.set_phase("br_roll_coeff_add");
        let same = b.alloc_qubit();
        b.x(same);
        b.cx(a_hist[i], same);
        b.cx(m_hist[i], same); // same = !(a xor m)
        let add_ctrl = b.alloc_qubit();
        b.ccx(active, same, add_ctrl);
        coeff_channel_cadd(b, p, cr, cs, add_ctrl);
        b.ccx(active, same, add_ctrl);
        b.free(add_ctrl);
        b.cx(m_hist[i], same);
        b.cx(a_hist[i], same);
        b.x(same);
        b.free(same);

        b.set_phase("br_roll_coeff_double");
        coeff_channel_double(b, p, cr);
        b.set_phase("br_roll_coeff_cswap2");
        coeff_channel_cswap(b, a_hist[i], cr, cs);
    }
    b.free(active);
}

pub(crate) fn apply_coeff_channel_from_term_roll_inverse(
    b: &mut B,
    p: U256,
    cr: &[QubitId],
    cs: &[QubitId],
    a_hist: &[QubitId],
    m_hist: &[QubitId],
    term_bits: &[QubitId],
) {
    assert_eq!(a_hist.len(), m_hist.len());
    let active = b.alloc_qubit(); // active after the last forward iteration is 0.
    for i in (0..a_hist.len()).rev() {
        b.set_phase("br_roll_inv_coeff_cswap2");
        coeff_channel_cswap(b, a_hist[i], cr, cs);
        b.set_phase("br_roll_inv_coeff_halve");
        mod_halve_inplace_fast(b, cr, p);

        b.set_phase("br_roll_inv_coeff_sub");
        let same = b.alloc_qubit();
        b.x(same);
        b.cx(a_hist[i], same);
        b.cx(m_hist[i], same); // same = !(a xor m)
        let sub_ctrl = b.alloc_qubit();
        b.ccx(active, same, sub_ctrl);
        coeff_channel_csub(b, p, cr, cs, sub_ctrl);
        b.ccx(active, same, sub_ctrl);
        b.free(sub_ctrl);
        b.cx(m_hist[i], same);
        b.cx(a_hist[i], same);
        b.x(same);
        b.free(same);

        b.set_phase("br_roll_inv_coeff_cswap1");
        coeff_channel_cswap(b, a_hist[i], cr, cs);

        b.set_phase("br_roll_inv_term_update");
        let eq_i = b.alloc_qubit();
        with_eq_const_fast(b, term_bits, i, eq_i, |b| {
            b.cx(eq_i, active);
        });
        b.free(eq_i);
    }
    // We have rewound the rolling flag to its pre-iteration-0 value, 1.
    b.x(active);
    b.free(active);
}

pub(crate) fn apply_coeff_channel_from_term_index(
    b: &mut B,
    p: U256,
    cr: &[QubitId],
    cs: &[QubitId],
    a_hist: &[QubitId],
    m_hist: &[QubitId],
    term_bits: &[QubitId],
) {
    assert_eq!(a_hist.len(), m_hist.len());
    for i in 0..a_hist.len() {
        b.set_phase("br_term_coeff_cswap1");
        coeff_channel_cswap(b, a_hist[i], cr, cs);

        // add is true for UG: (a,m)=(1,1).
        b.set_phase("br_term_coeff_add_ug");
        let ug_ctrl = b.alloc_qubit();
        b.ccx(a_hist[i], m_hist[i], ug_ctrl);
        coeff_channel_cadd(b, p, cr, cs, ug_ctrl);
        {
            let um = b.alloc_bit();
            b.hmr(ug_ctrl, um);
            b.cz_if(a_hist[i], m_hist[i], um);
        }
        b.free(ug_ctrl);

        // add is also true for active VG: (a,m)=(0,0) before the terminal
        // iteration. The terminal index is written once during branch record.
        b.set_phase("br_term_coeff_add_vg");
        let active = b.alloc_qubit();
        let ci = load_const(b, term_bits.len(), U256::from(i as u64));
        cmp_gt_into(b, term_bits, &ci, active); // active = term_idx > i
        let vg_ctrl = b.alloc_qubit();
        let scratch = b.alloc_qubit();
        mcx3_polar(
            b, active, true, a_hist[i], false, m_hist[i], false, vg_ctrl, scratch,
        );
        coeff_channel_cadd(b, p, cr, cs, vg_ctrl);
        mcx3_polar(
            b, active, true, a_hist[i], false, m_hist[i], false, vg_ctrl, scratch,
        );
        b.free(scratch);
        b.free(vg_ctrl);
        cmp_gt_into(b, term_bits, &ci, active);
        unload_const(b, &ci, U256::from(i as u64));
        b.free(active);

        b.set_phase("br_term_coeff_double");
        coeff_channel_double(b, p, cr);
        b.set_phase("br_term_coeff_cswap2");
        coeff_channel_cswap(b, a_hist[i], cr, cs);
    }
}

pub(crate) fn with_kal_inv_raw_coeff<F: FnOnce(&mut B, &[QubitId])>(
    b: &mut B,
    v_in: &[QubitId],
    p: U256,
    iters: usize,
    coeff: Option<(&[QubitId], &[QubitId])>,
    body: F,
) {
    let n = v_in.len();
    let iters = kaliski_effective_iters(n, iters);
    let mut st = alloc_kaliski_state(b, n, iters);
    let keep_full_state = std::env::var("KAL_KEEP_FULL_STATE").ok().as_deref() == Some("1");
    let keep_u = keep_full_state || std::env::var("KAL_KEEP_U").ok().as_deref() == Some("1");
    let keep_v = keep_full_state || std::env::var("KAL_KEEP_V").ok().as_deref() == Some("1");
    let keep_f = keep_full_state || std::env::var("KAL_KEEP_F").ok().as_deref() == Some("1");
    // KAL_FREE_S=1 (default ON in this branch): at end of forward Kaliski,
    // the s register provably equals p (the modulus) when iters >= ~407
    // (verified classically for our specific Kaliski variant). Free s by
    // X-ing the bits of p, then re-load before backward.
    let free_s = !keep_full_state && std::env::var("KAL_FREE_S").ok().as_deref() != Some("0");

    // Forward kaliski. st.r[..n] holds raw = v_in^{-1} * 2^(2n) mod p.
    // If coeff is supplied, the same branch controls also transform that
    // external coefficient pair, but the ordinary qrisp sentinel state remains
    // available for clean branch-flag uncomputation.
    kaliski_forward_with_coeff(b, v_in, &st, p, iters, coeff);

    if !keep_v {
        b.free_vec(&st.v_w);
    }
    if !keep_f {
        b.free(st.f_flag);
    }
    if !keep_u {
        b.x(st.u[0]);
        b.free_vec(&st.u);
    }
    if free_s {
        // s = p at this point. X each bit of p to zero it.
        for i in 0..n {
            if bit(p, i) {
                b.x(st.s[i]);
            }
        }
        b.free_vec(&st.s);
    }

    let r_low: Vec<QubitId> = st.r[..n].to_vec();
    body(b, &r_low);

    if !keep_u {
        // Re-alloc at |0> for the backward pass; restore u[0] = 1.
        st.u = b.alloc_qubits(n);
        b.x(st.u[0]);
    }
    if !keep_f {
        st.f_flag = b.alloc_qubit();
    }
    if !keep_v {
        st.v_w = b.alloc_qubits(n);
    }
    if free_s {
        // Re-allocate s and load p back.
        st.s = b.alloc_qubits(n);
        for i in 0..n {
            if bit(p, i) {
                b.x(st.s[i]);
            }
        }
    }

    // Experimental mode: use the exact reversed forward block shape, but skip
    // HMR/R in the reverse replay. This is heavier than the explicit backward,
    // but it keeps the specialized prefix and its matching global reverse in a
    // single contract. The hope is to eliminate the residual phase mismatch.
    if std::env::var("KAL_BULK3_GENERALIZED_REVERSE").is_ok() {
        emit_inverse_hmr_safe(b, |b| kaliski_forward(b, v_in, &st, p, iters));
    } else {
        // Explicit backward pass (uses measurement-based uncompute, saves
        // ~511 CCX per iteration vs the emit_inverse version).
        kaliski_backward(b, v_in, &st, p, iters);
    }

    free_kaliski_state(b, st);
}
