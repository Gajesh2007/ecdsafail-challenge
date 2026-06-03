
#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

/// Specialized real forward primitive for the first few guaranteed-bulk
/// Kaliski iterations where `f = 1` and `v_w != 0` are known a priori.
///
/// This keeps the same persistent-state interface as `kaliski_iteration`
/// (notably `m_i` ends in the same value that the generic step would have
/// produced), but drops STEP 0 / `f` handling entirely.
///
/// Not wired into the live inversion path yet: a direct forward-only swap-in
/// attempt did not preserve full point-add correctness, so this remains an
/// experimental helper while the history/backward compatibility conditions are
/// worked out.
pub(crate) fn kaliski_iteration_bulk_prefix3(
    b: &mut B,
    p: U256,
    u: &[QubitId],
    v_w: &[QubitId],
    r: &[QubitId],
    s: &[QubitId],
    m_i: QubitId,
    iter_idx: usize,
    coeff: Option<(&[QubitId], &[QubitId])>,
) {
    let a_f = b.alloc_qubit();
    let b_f = b.alloc_qubit();
    let add_f = b.alloc_qubit();
    let l_gt = b.alloc_qubit();

    let _kal_saved_phase = b.phase;

    // STEP 0 is a no-op on the guaranteed-bulk prefix (v_w != 0 so the
    // is_zero flag is always 0). The forward measurement-uncompute phases of
    // the OR chain are self-cancelling within with_eq_zero_fast, so dropping
    // the call entirely on both forward and backward is consistent.
    let _ = iter_idx;
    b.set_phase("kal_bulk_step1");
    // Specialized STEP 1 for f=1; the generic z HMR scaffold is a self-
    // cancelling noop (alloc-0 + ccx + hmr + matching cz_if) so we skip it.
    b.x(a_f);
    b.cx(u[0], a_f); // a_f = !u0
    b.x(v_w[0]);
    b.ccx(u[0], v_w[0], m_i); // m_i = u0 & !v0
    b.x(v_w[0]);
    b.cx(a_f, b_f);
    b.cx(m_i, b_f); // b_f = a_f xor m_i

    b.set_phase("kal_bulk_step2");
    // Late-iter comparator truncation: bitlen(u)+bitlen(v_w) ≤ 2n-iter_idx so
    // high bits are 0 and don't affect u > v_w.
    let cmp_width = if iter_idx < u.len() {
        u.len()
    } else {
        2 * u.len() - iter_idx
    };
    kal_step2_with_gt(b, &u[..cmp_width], &v_w[..cmp_width], l_gt, |b| {
        b.x(b_f);
        let t = b.alloc_qubit();
        b.ccx(l_gt, b_f, t);
        b.cx(t, a_f);
        b.cx(t, m_i);
        {
            let tm = b.alloc_bit();
            b.hmr(t, tm);
            b.cz_if(l_gt, b_f, tm);
        }
        b.free(t);
        // add_dummy scaffold (self-cancelling noop) skipped.
        b.x(b_f);
    });
    b.free(l_gt);

    b.set_phase("kal_bulk_step3_cswap");
    // Late-iter truncation: bitlen(u)+bitlen(v_w) ≤ 2n-iter_idx (Kaliski invariant).
    let uv_width_step3 = if iter_idx < u.len() {
        u.len()
    } else {
        2 * u.len() - iter_idx
    };
    for j in 0..uv_width_step3 {
        cswap(b, a_f, u[j], v_w[j]);
    }
    let rs_width_step3 = if iter_idx + 1 < u.len() {
        iter_idx + 1
    } else {
        u.len()
    };
    for j in 0..rs_width_step3 {
        cswap(b, a_f, r[j], s[j]);
    }
    if let Some((cr, cs)) = coeff {
        b.set_phase("kal_bulk_coeff_step3_cswap");
        coeff_channel_cswap(b, a_f, cr, cs);
    }

    b.set_phase("kal_bulk_step4");
    // Specialized STEP 4 with add_f = !b_f.
    b.x(add_f);
    b.cx(b_f, add_f);
    {
        let n = u.len();
        // Narrow load/sub width to the late-iter bound (same formula as sub_width).
        // Before this fix: load_width = n, sub_width = max(2n-k, n) → load too wide.
        // After: load_width = sub_width = max(2n-iter_idx, n). Saves n CCX/qubits per iter.
        let load_width = if iter_idx < n { n } else { 2 * n - iter_idx };
        let transform_width = if iter_idx + 1 < n { iter_idx + 1 } else { n };
        let add_width = if iter_idx + 2 < n { iter_idx + 2 } else { n };
        if std::env::var("KAL_STEP4_CTRL_LOWQ").ok().as_deref() == Some("1") {
            cucc_sub_ctrl_lowq(b, &u[..load_width], &v_w[..load_width], add_f);
            let mut r_slice: Vec<QubitId> = r[0..transform_width].to_vec();
            let r_pad = if add_width > transform_width {
                let q = b.alloc_qubit();
                r_slice.push(q);
                Some(q)
            } else {
                None
            };
            let s_slice: Vec<QubitId> = s[0..add_width].to_vec();
            cucc_add_ctrl_lowq(b, &r_slice, &s_slice, add_f);
            if let Some(q) = r_pad {
                b.free(q);
            }
        } else {
            let tmp = b.alloc_qubits(n);
            for i in 0..load_width {
                b.ccx(add_f, u[i], tmp[i]);
            }
            // Narrow load/sub width to the late-iter bound.
            // Both tmp and v_w are 256 qubits. Use slice [0..load_width] for each.
            kal_step4_sub_nbit_qq_with_scratch(
                b,
                &tmp[..load_width],
                &v_w[..load_width],
                &tmp[load_width..],
            );
            for i in 0..transform_width {
                b.cx(r[i], u[i]);
            }
            for i in 0..transform_width {
                b.ccx(add_f, u[i], tmp[i]);
            }
            for i in 0..transform_width {
                b.cx(r[i], u[i]);
            }
            let add_width = if iter_idx + 2 < n { iter_idx + 2 } else { n };
            let mut tmp_slice: Vec<QubitId> = tmp[0..transform_width].to_vec();
            let tmp_pad = if add_width > transform_width {
                let q = b.alloc_qubit();
                tmp_slice.push(q);
                Some(q)
            } else {
                None
            };
            let s_slice: Vec<QubitId> = s[0..add_width].to_vec();
            kal_step4_add_nbit_qq(b, &tmp_slice, &s_slice);
            if let Some(q) = tmp_pad {
                b.free(q);
            }
            for i in 0..n {
                let m = b.alloc_bit();
                b.hmr(tmp[i], m);
                if i < transform_width {
                    b.cz_if(add_f, r[i], m);
                } else {
                    b.cz_if(add_f, u[i], m);
                }
            }
            b.free_vec(&tmp);
        }
    }
    if let Some((cr, cs)) = coeff {
        b.set_phase("kal_bulk_coeff_step4_add");
        coeff_channel_cadd(b, p, cr, cs, add_f);
    }

    b.set_phase("kal_bulk_step5");
    b.cx(b_f, add_f);
    b.x(add_f);
    b.cx(m_i, b_f);
    b.cx(a_f, b_f);
    b.free(add_f);
    b.free(b_f);

    b.set_phase("kal_bulk_step6_7_8");
    for i in 0..(u.len() - 1) {
        b.swap(v_w[i], v_w[i + 1]);
    }
    if iter_idx < r_small_threshold() {
        mod_double_no_corr(b, r);
    } else {
        let mut dirty: Vec<QubitId> = u.to_vec();
        dirty.extend_from_slice(v_w);
        mod_double_inplace_fast_with_dirty(b, r, p, Some(&dirty));
    }
    if let Some((cr, _cs)) = coeff {
        b.set_phase("kal_bulk_coeff_step8_double");
        coeff_channel_double(b, p, cr);
    }

    b.set_phase("kal_bulk_step9_cswap");
    // Late-iter truncation: same uv-width bound as step3.
    let uv_width_step9 = if iter_idx < u.len() {
        u.len()
    } else {
        2 * u.len() - iter_idx
    };
    for j in 0..uv_width_step9 {
        cswap(b, a_f, u[j], v_w[j]);
    }
    let rs_width_step9 = if iter_idx + 2 < u.len() {
        iter_idx + 2
    } else {
        u.len()
    };
    for j in 0..rs_width_step9 {
        cswap(b, a_f, r[j], s[j]);
    }
    if let Some((cr, cs)) = coeff {
        b.set_phase("kal_bulk_coeff_step9_cswap");
        coeff_channel_cswap(b, a_f, cr, cs);
    }

    b.x(s[0]);
    b.cx(s[0], a_f);
    b.x(s[0]);

    b.free(a_f);
    b.set_phase(_kal_saved_phase);
}

pub(crate) fn kaliski_iteration(
    b: &mut B,
    p: U256,
    u: &[QubitId],
    v_w: &[QubitId],
    r: &[QubitId],
    s: &[QubitId],
    m_i: QubitId,
    f: QubitId,
    iter_idx: usize,
    coeff: Option<(&[QubitId], &[QubitId])>,
) {
    let n = u.len();
    // Iter-local flags (zero at iter start and iter end): alloc fresh here
    // so they don't live during body (which sees lower peak by -3 qubits).
    let a_f = b.alloc_qubit();
    let b_f = b.alloc_qubit();
    let add_f = b.alloc_qubit();

    let _kal_saved_phase = b.phase;
    b.set_phase("kal_step0_eqzero");
    // ─── STEP 0: is_zero = (v_w == 0);  m[i] ^= (f AND is_zero);  f ^= m[i] ───
    // Truncated OR chain for late iter: v_w's bits [2n-iter..n-1] are 0
    // (Kaliski invariant), so OR only of low 2n-iter bits suffices.
    let or_width = if iter_idx < n { n } else { 2 * n - iter_idx };
    with_eq_zero_fast(b, &v_w[0..or_width], add_f, |b| {
        b.ccx(f, add_f, m_i);
    });
    b.cx(m_i, f);

    b.set_phase("kal_step1");
    // ─── STEP 1 ───
    //   a ^= (f=1 AND u[0]=0)
    //   m[i] ^= (f=1 AND a=0 AND v_w[0]=0)  [= f AND u[0] AND NOT v_w[0]]
    //   b ^= a; b ^= m[i]
    //
    // Shared-intermediate trick: compute z = f AND u[0] once into b_f
    // (known 0 here), then derive a_f = f XOR z = f AND NOT u[0] via CX,
    // and update m_i via ccx(z, NOT v_w[0], m_i). Uncompute z, then set
    // b_f to a_f XOR m_i as before. Saves 1 CCX per iter vs mcx2+mcx3.
    b.ccx(f, u[0], b_f); // b_f = f AND u[0] (z)
    b.cx(f, a_f);
    b.cx(b_f, a_f); // a_f = f XOR z = f AND NOT u[0]
    b.x(v_w[0]);
    b.ccx(b_f, v_w[0], m_i); // m_i ^= z AND NOT v_w[0]
    b.x(v_w[0]);
    // Measurement-uncompute z (= f AND u[0]) from b_f: 0 CCX.
    {
        let zm = b.alloc_bit();
        b.hmr(b_f, zm);
        b.cz_if(f, u[0], zm);
    }
    b.cx(a_f, b_f);
    b.cx(m_i, b_f); // b_f = a_f XOR m_i

    // ─── STEP 2: with l = u > v_w: a ^= (f AND l AND ¬b); m_i ^= same.
    // Late-iter: u and v_w have bitlen ≤ 2n-iter, so only compare low 2n-iter bits.
    let cmp_width = if iter_idx < n { n } else { 2 * n - iter_idx };
    let l_gt = b.alloc_qubit();
    kal_step2_with_gt(b, &u[0..cmp_width], &v_w[0..cmp_width], l_gt, |b| {
        b.x(b_f); // negate polarity of b_f
        b.ccx(f, l_gt, add_f); // add_f = f AND l_gt
                               // Fuse two CCX with same (add_f, b_f) controls: compute once into
                               // a fresh ancilla, fan out via CX, measurement-uncompute. Saves 1 CCX.
        let t = b.alloc_qubit();
        b.ccx(add_f, b_f, t); // t = add_f AND ¬b_f_orig
        b.cx(t, a_f); // a_f ^= t
        b.cx(t, m_i); // m_i ^= t
        {
            let tm = b.alloc_bit();
            b.hmr(t, tm);
            b.cz_if(add_f, b_f, tm);
        }
        b.free(t);
        // Measurement-uncompute add_f (= f AND l_gt): 0 CCX.
        {
            let am = b.alloc_bit();
            b.hmr(add_f, am);
            b.cz_if(f, l_gt, am);
        }
        b.x(b_f);
    });
    b.free(l_gt);

    b.set_phase("kal_step3_cswap");
    // ─── STEP 3: with control(a): swap(u, v_w); swap(r, s) ───
    // Late-iter truncation: Kaliski invariant: bitlen(u) + bitlen(v_w) ≤ 2n-iter,
    // so u[j]=v_w[j]=0 for j >= 2n-iter_idx. Truncate (u,v_w) cswap.
    // Small-iter truncation: max(r,s) ≤ 2^iter_idx, so r[j]=s[j]=0 for j >= iter_idx+1.
    let uv_width = if iter_idx < n { n } else { 2 * n - iter_idx };
    for j in 0..uv_width {
        cswap(b, a_f, u[j], v_w[j]);
    }
    let rs_width_step3 = if iter_idx + 1 < n { iter_idx + 1 } else { n };
    for j in 0..rs_width_step3 {
        cswap(b, a_f, r[j], s[j]);
    }
    if let Some((cr, cs)) = coeff {
        b.set_phase("kal_coeff_step3_cswap");
        coeff_channel_cswap(b, a_f, cr, cs);
    }

    b.set_phase("kal_step4");
    // ─── STEP 4 ───
    //   add ^= (f=1 AND b=0)
    //   with control(add): v_w -= u; s += r
    //
    // Fused dual controlled sub+add: reuse one tmp register across both ops.
    // Load tmp = add_f AND u, do sub on v_w, then transform tmp to
    // add_f AND r in place (without unloading + reloading) by temporarily
    // XOR'ing r into u and re-applying ccx(add_f, u, tmp), then add tmp to
    // s and unload. Saves n CCX/iter.
    mcx2_polar(b, f, true, b_f, false, add_f);
    {
        // Load tmp = add_f AND u. Late-iter bound: u[i]=0 for i >= 2n-iter.
        let load_width = if iter_idx < n { n } else { 2 * n - iter_idx };
        let transform_width = if iter_idx + 1 < n { iter_idx + 1 } else { n };
        let add_width = if iter_idx + 2 < n { iter_idx + 2 } else { n };
        if std::env::var("KAL_STEP4_CTRL_LOWQ").ok().as_deref() == Some("1") {
            cucc_sub_ctrl_lowq(b, &u[..load_width], &v_w[..load_width], add_f);
            let mut r_slice: Vec<QubitId> = r[0..transform_width].to_vec();
            let r_pad = if add_width > transform_width {
                let q = b.alloc_qubit();
                r_slice.push(q);
                Some(q)
            } else {
                None
            };
            let s_slice: Vec<QubitId> = s[0..add_width].to_vec();
            cucc_add_ctrl_lowq(b, &r_slice, &s_slice, add_f);
            if let Some(q) = r_pad {
                b.free(q);
            }
        } else {
            let tmp = b.alloc_qubits(n);
            for i in 0..load_width {
                b.ccx(add_f, u[i], tmp[i]);
            }
            // Sub v_w -= tmp. Late-iter: both high bits 0, truncate to load_width.
            let tmp_sub_slice: Vec<QubitId> = tmp[0..load_width].to_vec();
            let v_w_sub_slice: Vec<QubitId> = v_w[0..load_width].to_vec();
            kal_step4_sub_nbit_qq_with_scratch(
                b,
                &tmp_sub_slice,
                &v_w_sub_slice,
                &tmp[load_width..],
            );
            // Transform tmp from "add_f AND u" to "add_f AND r".
            // Small-iter: only the low iter+1 bits of r can be nonzero; the
            // carry slot for s += r is handled by an explicit 0 pad instead of a
            // useless extra CCX on a known-zero r bit.
            // Late-iter: full transform (r unbounded but u high bits 0 so CCX at
            // high bits effectively produces add_f AND r from tmp=0).
            for i in 0..transform_width {
                b.cx(r[i], u[i]);
            }
            for i in 0..transform_width {
                b.ccx(add_f, u[i], tmp[i]);
            }
            for i in 0..transform_width {
                b.cx(r[i], u[i]);
            }
            // Add s += tmp. Small-iter still needs one extra carry slot above the
            // live r bits, but that top input bit is known 0.
            let mut tmp_slice: Vec<QubitId> = tmp[0..transform_width].to_vec();
            let tmp_pad = if add_width > transform_width {
                let q = b.alloc_qubit();
                tmp_slice.push(q);
                Some(q)
            } else {
                None
            };
            let s_slice: Vec<QubitId> = s[0..add_width].to_vec();
            kal_step4_add_nbit_qq(b, &tmp_slice, &s_slice);
            if let Some(q) = tmp_pad {
                b.free(q);
            }
            // Unload: bits < transform_width have tmp = add_f AND r;
            // bits [transform_width..load_width) have tmp = add_f AND u (transform skipped, load done);
            // bits >= load_width have tmp = 0 (load skipped).
            for i in 0..n {
                let m = b.alloc_bit();
                b.hmr(tmp[i], m);
                if i < transform_width {
                    b.cz_if(add_f, r[i], m);
                } else if i < load_width {
                    b.cz_if(add_f, u[i], m);
                }
                // else: tmp[i]=0, no phase correction needed.
            }
            b.free_vec(&tmp);
        }
    }
    if let Some((cr, cs)) = coeff {
        b.set_phase("kal_coeff_step4_add");
        coeff_channel_cadd(b, p, cr, cs, add_f);
    }

    b.set_phase("kal_step5");
    // ─── STEP 5: uncompute add; uncompute b ───
    // Measurement-uncompute add_f = f AND (NOT b_f): 0 CCX.
    b.x(b_f);
    {
        let sm = b.alloc_bit();
        b.hmr(add_f, sm);
        b.cz_if(f, b_f, sm);
    }
    b.x(b_f);
    b.cx(m_i, b_f);
    b.cx(a_f, b_f);
    b.free(add_f);
    b.free(b_f);

    b.set_phase("kal_step6_7_8");
    // ─── STEP 6: v_w := v_w / 2 (shift right by 1). Unconditional swap chain.
    // Invariant: v_w[0]=0 before this step whether f=1 (STEP 4 made v_w even)
    // or f=0 (algorithm terminated with v_w=0). Unconditional shift of 0 is 0.
    // Saves 255 CCX per iter vs cswap-controlled version.
    let _ = f;
    for i in 0..(n - 1) {
        b.swap(v_w[i], v_w[i + 1]);
    }

    // ─── STEP 7 + 8: r := 2*r mod p ───────────────────────────────────
    // For iter_idx < r_small_threshold(), r's top bit is guaranteed 0 (since
    // max(r,s) ≤ 2^iter_idx by induction). mod_double's Solinas correction
    // is identity; a plain shift suffices. Saves ~255 CCX per small iter.
    if iter_idx < r_small_threshold() {
        mod_double_no_corr(b, r);
    } else {
        let mut dirty: Vec<QubitId> = u.to_vec();
        dirty.extend_from_slice(v_w);
        mod_double_inplace_fast_with_dirty(b, r, p, Some(&dirty));
    }
    if let Some((cr, _cs)) = coeff {
        b.set_phase("kal_coeff_step8_double");
        coeff_channel_double(b, p, cr);
    }

    b.set_phase("kal_step9_cswap");
    // ─── STEP 9: with control(a): swap(u, v_w); swap(r, s) (again) ───
    // Late-iter (u,v_w) truncation per Kaliski invariant (same as STEP 3).
    // Small-iter (r,s) truncation: after STEP 4 s ≤ 2^{iter+1}, after STEP 7+8 r ≤ 2^{iter+1}.
    let uv_width = if iter_idx < n { n } else { 2 * n - iter_idx };
    for j in 0..uv_width {
        cswap(b, a_f, u[j], v_w[j]);
    }
    let rs_width_step9 = if iter_idx + 2 < n { iter_idx + 2 } else { n };
    for j in 0..rs_width_step9 {
        cswap(b, a_f, r[j], s[j]);
    }
    if let Some((cr, cs)) = coeff {
        b.set_phase("kal_coeff_step9_cswap");
        coeff_channel_cswap(b, a_f, cr, cs);
    }

    // ─── STEP 10: uncompute a via `a ^= NOT s[0]` ───
    // After STEP 9's swap, the invariant (from qrisp) is that
    //   a == NOT s[0]
    // Hence `cx(NOT s[0], a)` zeros a.
    b.x(s[0]);
    b.cx(s[0], a_f);
    b.x(s[0]);

    // Free iter-local flags (all at 0 now).
    b.free(a_f);
    b.set_phase(_kal_saved_phase);
}

/// Forward-only Kaliski computation. Reads `v_in` (never writes), populates
/// `st.*` with the algorithm's intermediate state. After this returns:
///   - `v_in` is unchanged
///   - `st.r[..n]` holds the RAW Kaliski inverse `v^{-1} * 2^{2n} mod p`
///   - everything else in `st` is populated with deterministic iteration history
///
/// The caller is responsible for applying the classical correction factor
/// `K = 2^{-2n} mod p` and for calling `emit_inverse(kaliski_forward)` to
/// restore `st.*` to all zero.
pub(crate) fn kaliski_forward(b: &mut B, v_in: &[QubitId], st: &KaliskiState, p: U256, iters: usize) {
    kaliski_forward_with_coeff(b, v_in, st, p, iters, None);
}

pub(crate) fn kaliski_forward_with_coeff(
    b: &mut B,
    v_in: &[QubitId],
    st: &KaliskiState,
    p: U256,
    iters: usize,
    coeff: Option<(&[QubitId], &[QubitId])>,
) {
    let n = v_in.len();
    debug_assert!(iters <= st.m_hist.len());
    if let Some((cr, cs)) = coeff {
        assert_eq!(cr.len(), n);
        assert_eq!(cs.len(), n);
    }

    // ─── Init ───
    // u := p (classical load)
    for i in 0..n {
        if bit(p, i) {
            b.x(st.u[i]);
        }
    }
    // v_w := v_in  (CX-copy; v_in unchanged)
    for i in 0..n {
        b.cx(v_in[i], st.v_w[i]);
    }
    kaliski_forward_loaded_v(b, st, p, iters, coeff);
}
