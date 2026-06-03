//! `kaliski::raw` — verbatim split of the original `kaliski` module.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

pub(crate) fn with_kal_inv_raw<F: FnOnce(&mut B, &[QubitId])>(
    b: &mut B,
    v_in: &[QubitId],
    p: U256,
    iters: usize,
    body: F,
) {
    let iters = kaliski_effective_iters(v_in.len(), iters);
    with_kal_inv_raw_coeff(b, v_in, p, iters, None, body);
}

pub(crate) fn with_kal_inv_raw_hmr_discard<F: FnOnce(&mut B, &[QubitId])>(
    b: &mut B,
    v_in: &[QubitId],
    p: U256,
    iters: usize,
    body: F,
) {
    let n = v_in.len();
    let iters = kaliski_effective_iters(n, iters);
    let st = alloc_kaliski_state(b, n, iters);
    kaliski_forward(b, v_in, &st, p, iters);
    if std::env::var("D1_DIRECT_QUOTIENT_HMR_REPAIR_M_HIST")
        .ok()
        .as_deref()
        == Some("1")
    {
        hmr_discard_kaliski_non_inverse_transcript_repair(b, v_in, &st, p, iters);
    } else {
        hmr_discard_kaliski_non_inverse_phase_dirty(b, &st, p);
    }
    let r_low: Vec<QubitId> = st.r[..n].to_vec();
    body(b, &r_low);
    if std::env::var("D1_DIRECT_QUOTIENT_HMR_REPAIR_R_RAW")
        .ok()
        .as_deref()
        == Some("1")
    {
        hmr_discard_kaliski_inverse_raw_repair(b, v_in, st, p, iters);
    } else {
        hmr_discard_kaliski_inverse_phase_dirty(b, st);
    }
}

pub(crate) fn with_kal_inv_raw_borrowing_v<F: FnOnce(&mut B, &[QubitId])>(
    b: &mut B,
    v_in_out: &[QubitId],
    p: U256,
    iters: usize,
    body: F,
) {
    let n = v_in_out.len();
    let iters = kaliski_effective_iters(n, iters);
    let mut st = alloc_kaliski_state_borrowing_v(b, v_in_out, iters);
    let keep_full_state = std::env::var("KAL_KEEP_FULL_STATE").ok().as_deref() == Some("1");
    let keep_u = keep_full_state || std::env::var("KAL_KEEP_U").ok().as_deref() == Some("1");
    let unsafe_free_f = std::env::var("KAL_FREE_F").ok().as_deref() == Some("1");
    assert!(
        !unsafe_free_f || std::env::var("KAL_FREE_F_UNSAFE_PROBE").ok().as_deref() == Some("1"),
        "KAL_FREE_F=1 is forbidden for borrowed-v Kaliski PA backends: the f_flag \
         sentinel is needed by the explicit backward pass. The 9024 Google PA \
         harness rejects this one-qubit shave with a value/phase failure. Set \
         KAL_FREE_F_UNSAFE_PROBE=1 only for reproducing that rejected lane."
    );
    // In the borrowed-v path the body is allowed to keep long-lived outputs
    // that can reuse freed Kaliski qubits.  Resetting/reallocating f_flag then
    // breaks the explicit backward contract on the raw-borrow PA path; keep the
    // one-bit sentinel live unless an experiment explicitly asks to free it.
    let keep_f = keep_full_state
        || std::env::var("KAL_KEEP_F").ok().as_deref() == Some("1")
        || !unsafe_free_f;
    let free_s = !keep_full_state && std::env::var("KAL_FREE_S").ok().as_deref() != Some("0");
    let relocate_r_to_terminal_v = std::env::var("KAL_RELOCATE_R_TO_BORROWED_V")
        .ok()
        .as_deref()
        == Some("1");

    for i in 0..n {
        if bit(p, i) {
            b.x(st.u[i]);
        }
    }
    kaliski_forward_borrowing_v(b, &st, p, iters);

    if !keep_f {
        b.free(st.f_flag);
    }
    if !keep_u {
        b.x(st.u[0]);
        b.free_vec(&st.u);
    }
    if free_s {
        for i in 0..n {
            if bit(p, i) {
                b.x(st.s[i]);
            }
        }
        b.free_vec(&st.s);
    }

    if relocate_r_to_terminal_v {
        b.set_phase("kal_relocate_r_to_terminal_borrowed_v");
        for i in 0..n {
            b.swap(st.r[i], st.v_w[i]);
        }
        b.free_vec(&st.r);
        st.r = Vec::new();
        let r_low: Vec<QubitId> = st.v_w[..n].to_vec();
        body(b, &r_low);
        st.r = b.alloc_qubits(n);
        b.set_phase("kal_restore_r_from_terminal_borrowed_v");
        for i in (0..n).rev() {
            b.swap(st.r[i], st.v_w[i]);
        }
    } else {
        let r_low: Vec<QubitId> = st.r[..n].to_vec();
        body(b, &r_low);
    }

    if !keep_u {
        st.u = b.alloc_qubits(n);
        b.x(st.u[0]);
    }
    if !keep_f {
        st.f_flag = b.alloc_qubit();
    }
    if free_s {
        st.s = b.alloc_qubits(n);
        for i in 0..n {
            if bit(p, i) {
                b.x(st.s[i]);
            }
        }
    }

    kaliski_backward_borrowing_v(b, &st, p, iters);
    free_kaliski_state_borrowed_v(b, st);
}

pub(crate) fn with_kal_inv_raw_prescaled_mixed<F: FnOnce(&mut B, &[QubitId])>(
    b: &mut B,
    v_in: &[QubitId],
    p: U256,
    iters: usize,
    body: F,
) {
    with_kal_inv_raw_prescaled_kind(b, v_in, p, iters, false, body);
}

pub(crate) fn with_kal_inv_raw_prescaled_chunked<F: FnOnce(&mut B, &[QubitId])>(
    b: &mut B,
    v_in: &[QubitId],
    p: U256,
    iters: usize,
    body: F,
) {
    with_kal_inv_raw_prescaled_kind(b, v_in, p, iters, true, body);
}

pub(crate) fn with_kal_inv_raw_prescaled_kind<F: FnOnce(&mut B, &[QubitId])>(
    b: &mut B,
    v_in: &[QubitId],
    p: U256,
    iters: usize,
    chunked: bool,
    body: F,
) {
    let n = v_in.len();
    let iters = kaliski_effective_iters(n, iters);
    let mut st = alloc_kaliski_state(b, n, iters);
    let scale = pow_mod_2_k(p, iters);
    let keep_full_state = std::env::var("KAL_KEEP_FULL_STATE").ok().as_deref() == Some("1");
    let keep_u = keep_full_state || std::env::var("KAL_KEEP_U").ok().as_deref() == Some("1");
    let keep_v = keep_full_state || std::env::var("KAL_KEEP_V").ok().as_deref() == Some("1");
    let keep_f = keep_full_state || std::env::var("KAL_KEEP_F").ok().as_deref() == Some("1");
    let free_s = !keep_full_state && std::env::var("KAL_FREE_S").ok().as_deref() != Some("0");

    if chunked {
        kaliski_forward_prescaled_chunked(b, v_in, &st, p, iters, scale);
    } else {
        kaliski_forward_prescaled_mixed(b, v_in, &st, p, iters, scale);
    }

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
        st.s = b.alloc_qubits(n);
        for i in 0..n {
            if bit(p, i) {
                b.x(st.s[i]);
            }
        }
    }

    if chunked {
        kaliski_backward_prescaled_chunked(b, v_in, &st, p, iters, scale);
    } else {
        kaliski_backward_prescaled_mixed(b, v_in, &st, p, iters, scale);
    }
    free_kaliski_state(b, st);
}

