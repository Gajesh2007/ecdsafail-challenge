//! `kaliski::iteration2` — verbatim split of the original `kaliski` module.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

/// Reverse of a single kaliski_iteration. Uses measurement-based
/// uncomputation for the OR chain (with_eq_zero) and the step-4 tmp
/// unload, saving ~511 CCX per iteration vs the gate-reversed version.
pub(crate) fn kaliski_iteration_backward(
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
    // Iter-local flags alloc'd fresh (zero at iter start in the backward
    // direction). They are zeroed and freed at iter end to match forward.
    let a_f = b.alloc_qubit();
    let b_f = b.alloc_qubit();
    let add_f = b.alloc_qubit();

    let _kal_saved_phase = b.phase;
    b.set_phase("bk_step10");
    // Reverse STEP 10
    // Matches forward's gated update.
    b.x(s[0]);
    b.ccx(f, s[0], a_f);
    b.x(s[0]);

    // ── Reverse STEP 9 ─────────────────────────────────────────────────
    let rs_width_step9 = if iter_idx + 2 < n { iter_idx + 2 } else { n };
    let uv_width = if iter_idx < n { n } else { 2 * n - iter_idx };
    if let Some((cr, cs)) = coeff {
        b.set_phase("bk_coeff_step9_cswap");
        coeff_channel_cswap(b, a_f, cr, cs);
    }
    b.set_phase("bk_step9_cswap");
    for j in (0..rs_width_step9).rev() {
        cswap(b, a_f, r[j], s[j]);
    }
    for j in (0..uv_width).rev() {
        cswap(b, a_f, u[j], v_w[j]);
    }

    b.set_phase("bk_step6_7_8");
    // Reverse STEP 8 + 7 ─────────────────────────────────────────────
    if let Some((cr, _cs)) = coeff {
        b.set_phase("bk_coeff_step8_halve");
        mod_halve_inplace_fast(b, cr, p);
    }
    // For iter_idx < r_small_threshold(), forward used mod_double_no_corr —
    // r is guaranteed even (bit 0 = 0), so a plain shift-right inverts it.
    if iter_idx < r_small_threshold() {
        mod_halve_no_corr(b, r);
    } else {
        let mut dirty: Vec<QubitId> = u.to_vec();
        dirty.extend_from_slice(v_w);
        mod_halve_inplace_fast_with_dirty(b, r, p, Some(&dirty));
    }

    // ── Reverse STEP 6 (unconditional shift-left) ───────────
    let _ = f;
    for i in (0..(n - 1)).rev() {
        b.swap(v_w[i], v_w[i + 1]);
    }

    b.set_phase("bk_step5");
    // Reverse STEP 5 ─────────────────────────────────────────────────
    b.cx(a_f, b_f);
    b.cx(m_i, b_f);
    mcx2_polar(b, f, true, b_f, false, add_f);

    // Reverse STEP 4 (with measurement uncompute for unload) ─────────
    if let Some((cr, cs)) = coeff {
        b.set_phase("bk_coeff_step4_sub");
        coeff_channel_csub(b, p, cr, cs, add_f);
    }
    b.set_phase("bk_step4");
    {
        let load_width = if iter_idx + 1 < n { iter_idx + 1 } else { n };
        let sub_width = if iter_idx + 2 < n { iter_idx + 2 } else { n };
        let transform_width = if iter_idx < n { n } else { 2 * n - iter_idx };
        let add_width = transform_width;
        if std::env::var("KAL_STEP4_CTRL_LOWQ").ok().as_deref() == Some("1") {
            let mut r_slice: Vec<QubitId> = r[0..load_width].to_vec();
            let r_pad = if sub_width > load_width {
                let q = b.alloc_qubit();
                r_slice.push(q);
                Some(q)
            } else {
                None
            };
            let s_slice: Vec<QubitId> = s[0..sub_width].to_vec();
            cucc_sub_ctrl_lowq(b, &r_slice, &s_slice, add_f);
            if let Some(q) = r_pad {
                b.free(q);
            }
            cucc_add_ctrl_lowq(b, &u[0..add_width], &v_w[0..add_width], add_f);
        } else {
            let tmp = b.alloc_qubits(n);
            // Load tmp = AND(add_f, r). Small-iter: r[i]=0 for i >= iter+1.
            for i in 0..load_width {
                b.ccx(add_f, r[i], tmp[i]);
            }
            // Reversed (F): sub tmp from s. Small-iter width iter+2.
            let tmp_sub_slice: Vec<QubitId> = tmp[0..sub_width].to_vec();
            let s_slice: Vec<QubitId> = s[0..sub_width].to_vec();
            kal_step4_sub_nbit_qq_with_scratch(b, &tmp_sub_slice, &s_slice, &tmp[sub_width..]);
            // Reversed (E): transform tmp from AND(add_f,r) → AND(add_f,u).
            // Late-iter: u high bits 0, so transform at those bits: cx(r,u=0)→u=r,
            //   ccx(add_f, u=r, tmp) flips tmp. tmp goes 0 → add_f AND r. Not what we
            //   want (need add_f AND u=0). For late iter, truncate transform to uv_width.
            for i in 0..transform_width {
                b.cx(r[i], u[i]);
            }
            for i in 0..transform_width {
                b.ccx(add_f, u[i], tmp[i]);
            }
            for i in 0..transform_width {
                b.cx(r[i], u[i]);
            }
            // Reversed (D): add tmp to v_w. Truncated to uv_width (late iter bound).
            let tmp_add_slice: Vec<QubitId> = tmp[0..add_width].to_vec();
            let v_w_slice: Vec<QubitId> = v_w[0..add_width].to_vec();
            kal_step4_add_nbit_qq(b, &tmp_add_slice, &v_w_slice);
            // Unload: bits < min(load_width, transform_width) both apply (tmp = add_f AND u after transform).
            // For bits where transform was applied, tmp = add_f AND u. For bits where transform skipped
            // (i >= transform_width), tmp stays at whatever load left it (either add_f AND r or 0).
            for i in 0..n {
                let m = b.alloc_bit();
                b.hmr(tmp[i], m);
                if i < transform_width {
                    // Transform applied: tmp = add_f AND u.
                    b.cz_if(add_f, u[i], m);
                } else if i < load_width {
                    // Load done but transform skipped: tmp = add_f AND r.
                    b.cz_if(add_f, r[i], m);
                }
                // else: tmp = 0, no phase.
            }
            b.free_vec(&tmp);
        }
    }
    // Reversed (A): measurement-uncompute add_f = f AND (NOT b_f)
    b.x(b_f);
    {
        let sm = b.alloc_bit();
        b.hmr(add_f, sm);
        b.cz_if(f, b_f, sm);
    }
    b.x(b_f);

    // Reverse STEP 3 ─────────────────────────────────────────────────
    if let Some((cr, cs)) = coeff {
        b.set_phase("bk_coeff_step3_cswap");
        coeff_channel_cswap(b, a_f, cr, cs);
    }
    b.set_phase("bk_step3_cswap");
    let rs_width_step3 = if iter_idx + 1 < n { iter_idx + 1 } else { n };
    let uv_width = if iter_idx < n { n } else { 2 * n - iter_idx };
    for j in (0..rs_width_step3).rev() {
        cswap(b, a_f, r[j], s[j]);
    }
    for j in (0..uv_width).rev() {
        cswap(b, a_f, u[j], v_w[j]);
    }

    b.set_phase("bk_step2");
    // Reverse STEP 2 (with_gt body is self-inverse) ──────────────────
    let cmp_width = if iter_idx < n { n } else { 2 * n - iter_idx };
    let l_gt = b.alloc_qubit();
    kal_step2_with_gt(b, &u[0..cmp_width], &v_w[0..cmp_width], l_gt, |b| {
        b.x(b_f);
        b.ccx(f, l_gt, add_f);
        // Fuse two CCX with same (add_f, b_f) controls into one CCX + two CX
        // + measurement uncompute. Saves 1 CCX per backward iter.
        let t = b.alloc_qubit();
        b.ccx(add_f, b_f, t);
        b.cx(t, m_i);
        b.cx(t, a_f);
        {
            let tm = b.alloc_bit();
            b.hmr(t, tm);
            b.cz_if(add_f, b_f, tm);
        }
        b.free(t);
        // Measurement-uncompute add_f = f AND l_gt: 0 CCX.
        {
            let am = b.alloc_bit();
            b.hmr(add_f, am);
            b.cz_if(f, l_gt, am);
        }
        b.x(b_f);
    });
    b.free(l_gt);

    b.set_phase("bk_step1");
    // Reverse STEP 1 ─────────────────────────────────────────────────
    b.cx(m_i, b_f);
    b.cx(a_f, b_f);
    b.ccx(f, u[0], b_f);
    b.x(v_w[0]);
    b.ccx(b_f, v_w[0], m_i);
    b.x(v_w[0]);
    b.cx(b_f, a_f);
    b.cx(f, a_f);
    // Measurement-uncompute z = f AND u[0] from b_f: 0 CCX.
    {
        let zm = b.alloc_bit();
        b.hmr(b_f, zm);
        b.cz_if(f, u[0], zm);
    }

    b.set_phase("bk_step0_eqzero");
    // Reverse STEP 0 (with measurement uncompute of OR chain) ────────
    // Truncated for late iter: only low 2n-iter bits of v_w are possibly nonzero.
    b.cx(m_i, f);
    {
        let or_width = if iter_idx < n { n } else { 2 * n - iter_idx };
        let nv = or_width;
        if nv == 1 {
            b.x(v_w[0]);
            b.cx(v_w[0], add_f);
            b.ccx(f, add_f, m_i);
            b.cx(v_w[0], add_f);
            b.x(v_w[0]);
        } else {
            let or_chain: Vec<QubitId> = b.alloc_qubits(nv - 1);
            or_step(b, v_w[0], v_w[1], or_chain[0]);
            for i in 1..nv - 1 {
                or_step(b, or_chain[i - 1], v_w[i + 1], or_chain[i]);
            }
            b.x(or_chain[nv - 2]);
            b.cx(or_chain[nv - 2], add_f);
            b.x(or_chain[nv - 2]);
            // Body
            b.ccx(f, add_f, m_i);
            // Uncompute flag
            b.x(or_chain[nv - 2]);
            b.cx(or_chain[nv - 2], add_f);
            b.x(or_chain[nv - 2]);
            // Measurement-based uncompute of OR chain (0 Toffoli)
            for i in (1..nv - 1).rev() {
                or_step_uncompute(b, or_chain[i - 1], v_w[i + 1], or_chain[i]);
            }
            or_step_uncompute(b, v_w[0], v_w[1], or_chain[0]);
            b.free_vec(&or_chain);
        }
    }

    // Free iter-local flags (all at 0 now after backward steps).
    b.free(add_f);
    b.free(b_f);
    b.free(a_f);
    b.set_phase(_kal_saved_phase);
}

/// Explicit backward pass for kaliski_forward. Uses measurement-based
/// uncomputation to save ~511 CCX per iteration vs emit_inverse.
pub(crate) fn kaliski_backward(b: &mut B, v_in: &[QubitId], st: &KaliskiState, p: U256, iters: usize) {
    let n = v_in.len();
    debug_assert!(iters <= st.m_hist.len());

    let use_bulk_prefix3 = bulk_prefix_enabled();
    let bulk_prefix_iters = bulk_prefix_safe_iters();
    // ─── Reverse iterations (in reverse order) ───
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

    // ─── Reverse Init ───
    b.x(st.f_flag);
    b.x(st.s[0]);
    for i in 0..n {
        b.cx(v_in[i], st.v_w[i]);
    }
    for i in 0..n {
        if bit(p, i) {
            b.x(st.u[i]);
        }
    }
}

pub(crate) fn kaliski_effective_iters(n: usize, iters: usize) -> usize {
    iters.min(2 * n)
}

pub(crate) fn kaliski_inv_inplace(b: &mut B, v_in: &[QubitId], p: U256) {
    let n = v_in.len();
    let iters = 2 * n - 114;

    // Bennett compute-copy-uncompute pattern. Each call of
    // `kaliski_inv_inplace` maps v_in ↔ v_in^{-1} (involution), with
    // internal scratch fully zeroed by function end.
    let st = alloc_kaliski_state(b, n, iters);
    let output = b.alloc_qubits(n);

    // ─── Phase 1: compute inverse of v_in into output ───
    kaliski_forward(b, v_in, &st, p, iters);
    // st.r[..n] now holds raw inverse (in mod 2p, low n bits).
    // Apply classical correction: st.r[..n] *= K mod p, where K = 2^{-2n} mod p.
    let two_2n = pow_mod_2_k(p, 2 * n);
    let k_const = classical_modinv(two_2n, p);
    in_place_mul_const(b, &st.r[..n], k_const, p);
    // Copy exact inverse into output.
    for i in 0..n {
        b.cx(st.r[i], output[i]);
    }
    // Undo the correction: st.r[..n] *= K^{-1} mod p.
    in_place_mul_const(b, &st.r[..n], two_2n, p);
    // Now st is back to its post-kaliski_forward state. Reverse the forward.
    emit_inverse(b, |b| kaliski_forward(b, v_in, &st, p, iters));
    // st is all 0 again. v_in unchanged. output = v_in^{-1}.

    // Swap v_in and output.
    for i in 0..n {
        b.swap(v_in[i], output[i]);
    }
    // v_in = inverse, output = v_orig.

    // ─── Phase 2: zero output via a second Bennett pass ───
    // Compute inverse of current v_in (which is v_orig^{-1}), = v_orig,
    // and XOR it into output. Since output currently = v_orig, the XOR
    // zeroes output.
    kaliski_forward(b, v_in, &st, p, iters);
    in_place_mul_const(b, &st.r[..n], k_const, p);
    for i in 0..n {
        b.cx(st.r[i], output[i]);
    } // output ^= v_orig = 0
    in_place_mul_const(b, &st.r[..n], two_2n, p);
    emit_inverse(b, |b| kaliski_forward(b, v_in, &st, p, iters));
    // st all 0, output all 0 (hopefully), v_in = inverse.

    b.free_vec(&output);
    free_kaliski_state(b, st);
}
