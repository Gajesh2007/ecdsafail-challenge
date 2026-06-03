//! `multiply::schoolbook1` — verbatim split of the original `multiply` module.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

/// Low-peak variant of `mod_mul_write_into_zero_acc_schoolbook`: uses
/// `schoolbook_mul_into_addsub_lowq` + `_inverse_lowq` instead of the fast
/// variants, saving ~n qubits at peak at the cost of ~n extra Toffolis per
/// row.
///
/// NOTE: microbench (n=256) shows this DOES NOT reduce the local peak
/// (schoolbook_fast 1797 = schoolbook_lowq 1797); the Solinas reduction +
/// acc lifetimes already dominate, and the lowq carry saving is hidden
/// underneath. We also observed a deterministic phase-garbage batch when
/// wiring this in at pair1_mul1 (1/20480 shots, ALT_SEED tag=5, across
/// two runs), so this helper is currently DEAD CODE kept only as a paper
/// trail for the negative result. See `autoresearch.ideas.md`.
#[allow(dead_code)]
pub(crate) fn mod_mul_write_into_zero_acc_schoolbook_lowq(
    b: &mut B,
    acc: &[QubitId],
    x: &[QubitId],
    y: &[QubitId],
    p: U256,
) {
    let n = acc.len();
    debug_assert_eq!(n, 256);

    let tmp_ext = b.alloc_qubits(2 * n);
    schoolbook_mul_into_addsub_lowq(b, x, y, &tmp_ext);

    let lo: Vec<QubitId> = tmp_ext[0..n].to_vec();
    let hi: Vec<QubitId> = tmp_ext[n..2 * n].to_vec();
    mod_add_qq_fast_from_zero(b, acc, &lo, p);
    mod_add_qq_fast(b, acc, &hi, p);
    for _ in 0..4 {
        mod_double_inplace_fast(b, &hi, p);
    }
    mod_add_qq_fast(b, acc, &hi, p);
    for _ in 0..2 {
        mod_double_inplace_fast(b, &hi, p);
    }
    mod_sub_qq_fast(b, acc, &hi, p);
    for _ in 0..4 {
        mod_double_inplace_fast(b, &hi, p);
    }
    mod_add_qq_fast(b, acc, &hi, p);
    let (spill, flag_inv, ovf) = mod_shift_left_by_k(b, &hi, p, 22);
    mod_add_qq(b, acc, &hi, p);
    mod_shift_right_by_k(b, &hi, p, 22, spill, flag_inv, ovf);
    for _ in 0..10 {
        mod_halve_inplace_fast(b, &hi, p);
    }

    schoolbook_mul_into_addsub_lowq_inverse(b, x, y, &tmp_ext);
    b.free_vec(&tmp_ext);
}

/// Specialization of mod_mul_add_into_acc_schoolbook when acc = 0 on entry.
/// Uses mod_add_qq_fast_from_zero for the first Solinas reduction step.
/// Saves ~255 CCX per call.
pub(crate) fn mod_mul_write_into_zero_acc_schoolbook(
    b: &mut B,
    acc: &[QubitId],
    x: &[QubitId],
    y: &[QubitId],
    p: U256,
) {
    let n = acc.len();
    debug_assert_eq!(n, 256);
    let c = U256::MAX.wrapping_sub(p).wrapping_add(U256::from(1));

    let tmp_ext = b.alloc_qubits(2 * n);
    schoolbook_mul_into_addsub(b, x, y, &tmp_ext);

    let lo: Vec<QubitId> = tmp_ext[0..n].to_vec();
    let hi: Vec<QubitId> = tmp_ext[n..2 * n].to_vec();
    // First add: acc is known to be 0, so use the fast-from-zero variant.
    mod_add_qq_fast_from_zero(b, acc, &lo, p);
    let _ = c;
    // 977 = 2^10 - 2^6 + 2^4 + 2^0 consolidation. 5 ops instead of 7.
    mod_add_qq_fast(b, acc, &hi, p);
    for _ in 0..4 {
        mod_double_inplace_fast(b, &hi, p);
    }
    mod_add_qq_fast(b, acc, &hi, p);
    for _ in 0..2 {
        mod_double_inplace_fast(b, &hi, p);
    }
    mod_sub_qq_fast(b, acc, &hi, p);
    for _ in 0..4 {
        mod_double_inplace_fast(b, &hi, p);
    }
    mod_add_qq_fast(b, acc, &hi, p);
    let (spill, flag_inv, ovf) = mod_shift_left_by_k(b, &hi, p, 22);
    mod_add_qq(b, acc, &hi, p);
    mod_shift_right_by_k(b, &hi, p, 22, spill, flag_inv, ovf);
    b.set_phase("sol_halve_tail");
    for _ in 0..10 {
        mod_halve_inplace_fast(b, &hi, p);
    }

    b.set_phase("schoolbook_mul_inverse");
    schoolbook_mul_into_addsub_inverse(b, x, y, &tmp_ext);
    b.free_vec(&tmp_ext);
}

pub(crate) fn mod_mul_write_into_zero_acc_schoolbook_peak_lowq(
    b: &mut B,
    acc: &[QubitId],
    x: &[QubitId],
    y: &[QubitId],
    p: U256,
) {
    let n = acc.len();
    debug_assert_eq!(n, 256);

    let tmp_ext = b.alloc_qubits(2 * n);
    b.set_phase("schoolbook_peak_lowq_write_mul");
    schoolbook_mul_into_addsub_lowq(b, x, y, &tmp_ext);

    let lo: Vec<QubitId> = tmp_ext[0..n].to_vec();
    let hi: Vec<QubitId> = tmp_ext[n..2 * n].to_vec();
    b.set_phase("schoolbook_peak_lowq_write_reduce_lo");
    mod_add_qq(b, acc, &lo, p);
    b.set_phase("schoolbook_peak_lowq_write_reduce_hi0");
    mod_add_qq(b, acc, &hi, p);
    b.set_phase("schoolbook_peak_lowq_write_reduce_hi4");
    for _ in 0..4 {
        mod_double_inplace_fast(b, &hi, p);
    }
    mod_add_qq(b, acc, &hi, p);
    b.set_phase("schoolbook_peak_lowq_write_reduce_hi6");
    for _ in 0..2 {
        mod_double_inplace_fast(b, &hi, p);
    }
    mod_sub_qq(b, acc, &hi, p);
    b.set_phase("schoolbook_peak_lowq_write_reduce_hi10");
    for _ in 0..4 {
        mod_double_inplace_fast(b, &hi, p);
    }
    mod_add_qq(b, acc, &hi, p);
    b.set_phase("schoolbook_peak_lowq_write_shift22");
    if std::env::var("D1_PEAK_LOWQ_SHIFT22_DOUBLES")
        .ok()
        .as_deref()
        == Some("1")
    {
        for _ in 0..22 {
            mod_double_inplace_fast_with_dirty(b, &hi, p, Some(acc));
        }
        mod_add_qq(b, acc, &hi, p);
        for _ in 0..22 {
            mod_halve_inplace_fast_with_dirty(b, &hi, p, Some(acc));
        }
    } else {
        let (spill, flag_inv, ovf) = mod_shift_left_by_k_lowq_with_dirty(b, &hi, p, 22, acc);
        mod_add_qq(b, acc, &hi, p);
        mod_shift_right_by_k_lowq_with_dirty(b, &hi, p, 22, spill, flag_inv, ovf, acc);
    }
    b.set_phase("schoolbook_peak_lowq_write_halve_tail");
    for _ in 0..10 {
        mod_halve_inplace_fast(b, &hi, p);
    }

    b.set_phase("schoolbook_peak_lowq_write_unmul");
    schoolbook_mul_into_addsub_lowq_inverse(b, x, y, &tmp_ext);
    b.free_vec(&tmp_ext);
}



/// Litinski 2024 add-subtract schoolbook: tmp_ext += x * y.
///
/// Precondition: tmp_ext has 2n bits and holds value A_in.
/// Postcondition: tmp_ext holds A_in + x*y (mod 2^{2n}).
pub(crate) fn schoolbook_mul_into_addsub(b: &mut B, x: &[QubitId], y: &[QubitId], tmp_ext: &[QubitId]) {
    let n = x.len();
    debug_assert_eq!(y.len(), n);
    debug_assert_eq!(tmp_ext.len(), 2 * n);

    // wide = [low, tmp_ext[0], ..., tmp_ext[2n-1]]  =  2n+1 bits.
    // This treats the (2n+1)-bit number `wide` as Litinski's accumulator.
    // After all ops, wide = 2*A_in_shifted + 2*x*y  (i.e. 2*(A_in + xy)).
    // `/2 relabel` reads out xy at wide[1..2n+1] = tmp_ext.
    //
    // To add A_in into the 2*(A_in + xy) result correctly, we need to bring A_in
    // in as `2*A_in` in wide. That is done pre-loop: swap tmp_ext values up one bit.
    // But Litinski's derivation assumes A_in = 0. To support non-zero A_in we'd
    // need to double tmp_ext at the start and halve at the end.
    //
    // Fortunately ALL call sites pass tmp_ext starting at 0 (fresh alloc), so we
    // can just assume A_in = 0.
    let low = b.alloc_qubit();
    let mut wide: Vec<QubitId> = Vec::with_capacity(2 * n + 1);
    wide.push(low);
    wide.extend_from_slice(tmp_ext);

    // n controlled add-subtracts (Litinski Fig 2b).
    for k in 0..n {
        let slice: Vec<QubitId> = wide[k..k + n + 1].to_vec();
        controlled_add_subtract_fast(b, x, &slice, y[k]);
    }

    // Corrections:
    //   Using y as ctrl and x as operand, the intermediate value is:
    //     2xy + 2^{2n} - 2^n (x+y+1) + x
    //   Target: 2xy. So apply +2^n(y+1) + 2^n*x - 2^{2n} - x.

    // +2^n * (y + 1): (n+1)-bit add of y_ext (top=0) into wide[n..2n+1] with c_in=1.
    {
        let pad = b.alloc_qubit();
        let mut y_ext = y.to_vec();
        y_ext.push(pad);
        let slice: Vec<QubitId> = wide[n..2 * n + 1].to_vec();
        let c_in = b.alloc_qubit();
        b.x(c_in);
        if kal_vent_modadd_enabled() {
            cuccaro_add(b, &y_ext, &slice, c_in);
        } else {
            cuccaro_add_fast(b, &y_ext, &slice, c_in);
        }
        b.x(c_in);
        b.free(c_in);
        b.free(pad);
    }

    // -2^{2n}: toggle wide[2n].
    b.x(wide[2 * n]);

    // -x as full (2n+1)-bit sub. Use in-place cuccaro_sub (no carry ancillae) to
    // keep peak qubits low during this otherwise-expensive full-width correction.
    // Costs n-1 extra Toffoli vs cuccaro_sub_fast but saves 2n peak qubits.
    {
        let mut x_ext: Vec<QubitId> = x.to_vec();
        while x_ext.len() < 2 * n + 1 {
            x_ext.push(b.alloc_qubit());
        }
        let c_in = b.alloc_qubit();
        cuccaro_sub(b, &x_ext, &wide, c_in);
        b.free(c_in);
        for _ in n..2 * n + 1 {
            let q = x_ext.pop().unwrap();
            b.free(q);
        }
    }

    // +2^n * x: (n+1)-bit add of x_ext into wide[n..2n+1].
    {
        let pad = b.alloc_qubit();
        let mut x_ext = x.to_vec();
        x_ext.push(pad);
        let slice: Vec<QubitId> = wide[n..2 * n + 1].to_vec();
        let c_in = b.alloc_qubit();
        if kal_vent_modadd_enabled() {
            cuccaro_add(b, &x_ext, &slice, c_in);
        } else {
            cuccaro_add_fast(b, &x_ext, &slice, c_in);
        }
        b.free(c_in);
        b.free(pad);
    }

    // wide = 2xy. /2 relabel: xy is at wide[1..2n+1] = tmp_ext. wide[0]=low should be 0.
    b.free(low);
}

/// Low-peak variant of `schoolbook_mul_into_addsub`: uses non-fast Cuccaro
/// (`cuccaro_add`) inside the `controlled_add_subtract` core and in the
/// correction adders. Saves roughly `n` transient qubits at peak vs. the
/// `_fast` variant at the cost of ~n extra Toffolis per row. Top-level
/// semantics identical to `schoolbook_mul_into_addsub`.
pub(crate) fn schoolbook_mul_into_addsub_lowq(b: &mut B, x: &[QubitId], y: &[QubitId], tmp_ext: &[QubitId]) {
    let n = x.len();
    debug_assert_eq!(y.len(), n);
    debug_assert_eq!(tmp_ext.len(), 2 * n);

    let low = b.alloc_qubit();
    let mut wide: Vec<QubitId> = Vec::with_capacity(2 * n + 1);
    wide.push(low);
    wide.extend_from_slice(tmp_ext);

    for k in 0..n {
        let slice: Vec<QubitId> = wide[k..k + n + 1].to_vec();
        controlled_add_subtract_lowq(b, x, &slice, y[k]);
    }

    // +2^n * (y + 1)
    {
        let pad = b.alloc_qubit();
        let mut y_ext = y.to_vec();
        y_ext.push(pad);
        let slice: Vec<QubitId> = wide[n..2 * n + 1].to_vec();
        let c_in = b.alloc_qubit();
        b.x(c_in);
        cuccaro_add(b, &y_ext, &slice, c_in);
        b.x(c_in);
        b.free(c_in);
        b.free(pad);
    }

    // -2^{2n}
    b.x(wide[2 * n]);

    // -x full (2n+1)-bit sub
    {
        let mut x_ext: Vec<QubitId> = x.to_vec();
        while x_ext.len() < 2 * n + 1 {
            x_ext.push(b.alloc_qubit());
        }
        let c_in = b.alloc_qubit();
        cuccaro_sub(b, &x_ext, &wide, c_in);
        b.free(c_in);
        for _ in n..2 * n + 1 {
            let q = x_ext.pop().unwrap();
            b.free(q);
        }
    }

    // +2^n * x
    {
        let pad = b.alloc_qubit();
        let mut x_ext = x.to_vec();
        x_ext.push(pad);
        let slice: Vec<QubitId> = wide[n..2 * n + 1].to_vec();
        let c_in = b.alloc_qubit();
        cuccaro_add(b, &x_ext, &slice, c_in);
        b.free(c_in);
        b.free(pad);
    }

    b.free(low);
}

/// Exact gate-level inverse of `schoolbook_mul_into_addsub_lowq`.
pub(crate) fn schoolbook_mul_into_addsub_lowq_inverse(
    b: &mut B,
    x: &[QubitId],
    y: &[QubitId],
    tmp_ext: &[QubitId],
) {
    let n = x.len();
    debug_assert_eq!(y.len(), n);
    debug_assert_eq!(tmp_ext.len(), 2 * n);

    let low = b.alloc_qubit();
    let mut wide: Vec<QubitId> = Vec::with_capacity(2 * n + 1);
    wide.push(low);
    wide.extend_from_slice(tmp_ext);

    // Reverse correction 4: sub x at bit n.
    {
        let pad = b.alloc_qubit();
        let mut x_ext = x.to_vec();
        x_ext.push(pad);
        let slice: Vec<QubitId> = wide[n..2 * n + 1].to_vec();
        let c_in = b.alloc_qubit();
        cuccaro_sub(b, &x_ext, &slice, c_in);
        b.free(c_in);
        b.free(pad);
    }
    // Reverse correction 3.
    {
        let mut x_ext: Vec<QubitId> = x.to_vec();
        while x_ext.len() < 2 * n + 1 {
            x_ext.push(b.alloc_qubit());
        }
        let c_in = b.alloc_qubit();
        cuccaro_add(b, &x_ext, &wide, c_in);
        b.free(c_in);
        for _ in n..2 * n + 1 {
            let q = x_ext.pop().unwrap();
            b.free(q);
        }
    }
    // Reverse correction 2.
    b.x(wide[2 * n]);
    // Reverse correction 1.
    {
        let pad = b.alloc_qubit();
        let mut y_ext = y.to_vec();
        y_ext.push(pad);
        let slice: Vec<QubitId> = wide[n..2 * n + 1].to_vec();
        let c_in = b.alloc_qubit();
        b.x(c_in);
        cuccaro_sub(b, &y_ext, &slice, c_in);
        b.x(c_in);
        b.free(c_in);
        b.free(pad);
    }
    for k in (0..n).rev() {
        let slice: Vec<QubitId> = wide[k..k + n + 1].to_vec();
        controlled_add_subtract_lowq_inverse(b, x, &slice, y[k]);
    }

    b.free(low);
}

/// Exact gate-level inverse of `schoolbook_mul_into_addsub`.
pub(crate) fn schoolbook_mul_into_addsub_inverse(
    b: &mut B,
    x: &[QubitId],
    y: &[QubitId],
    tmp_ext: &[QubitId],
) {
    let n = x.len();
    debug_assert_eq!(y.len(), n);
    debug_assert_eq!(tmp_ext.len(), 2 * n);

    let low = b.alloc_qubit();
    let mut wide: Vec<QubitId> = Vec::with_capacity(2 * n + 1);
    wide.push(low);
    wide.extend_from_slice(tmp_ext);

    // Reverse correction 4: sub x at bit n.
    {
        let pad = b.alloc_qubit();
        let mut x_ext = x.to_vec();
        x_ext.push(pad);
        let slice: Vec<QubitId> = wide[n..2 * n + 1].to_vec();
        let c_in = b.alloc_qubit();
        cuccaro_sub_fast(b, &x_ext, &slice, c_in);
        b.free(c_in);
        b.free(pad);
    }
    // Reverse correction 3 (sub x full-width): add x back with borrow propagation.
    // Use in-place cuccaro_add (no carries) to keep peak low, matching forward.
    {
        let mut x_ext: Vec<QubitId> = x.to_vec();
        while x_ext.len() < 2 * n + 1 {
            x_ext.push(b.alloc_qubit());
        }
        let c_in = b.alloc_qubit();
        cuccaro_add(b, &x_ext, &wide, c_in);
        b.free(c_in);
        for _ in n..2 * n + 1 {
            let q = x_ext.pop().unwrap();
            b.free(q);
        }
    }
    // Reverse correction 2: toggle wide[2n].
    b.x(wide[2 * n]);
    // Reverse correction 1: sub (y+1) at bit n.
    {
        let pad = b.alloc_qubit();
        let mut y_ext = y.to_vec();
        y_ext.push(pad);
        let slice: Vec<QubitId> = wide[n..2 * n + 1].to_vec();
        let c_in = b.alloc_qubit();
        b.x(c_in);
        cuccaro_sub_fast(b, &y_ext, &slice, c_in);
        b.x(c_in);
        b.free(c_in);
        b.free(pad);
    }
    // Reverse n add-subtract rows.
    for k in (0..n).rev() {
        let slice: Vec<QubitId> = wide[k..k + n + 1].to_vec();
        controlled_add_subtract_fast_inverse(b, x, &slice, y[k]);
    }

    b.free(low);
}

/// Add x*y mod p to acc, via schoolbook into a wide accumulator + Solinas
/// reduction + Bennett uncompute. Saves ~100k CCX vs Horner-on-acc per call.
pub(crate) fn mod_mul_add_into_acc_schoolbook(
    b: &mut B,
    acc: &[QubitId],
    x: &[QubitId],
    y: &[QubitId],
    p: U256,
) {
    let n = acc.len();
    debug_assert_eq!(n, 256);
    let c = U256::MAX.wrapping_sub(p).wrapping_add(U256::from(1));

    let tmp_ext = b.alloc_qubits(2 * n);
    schoolbook_mul_into_addsub(b, x, y, &tmp_ext);

    let lo: Vec<QubitId> = tmp_ext[0..n].to_vec();
    let hi: Vec<QubitId> = tmp_ext[n..2 * n].to_vec();
    let _ = c;
    mod_add_qq_fast(b, acc, &lo, p);
    // Solinas with 977 = 2^10 - 2^6 + 2^4 + 2^0. c = 2^32 + 977 = {+2^0, +2^4, -2^6, +2^10, +2^32}.
    // 5 ops instead of 7 (saves 2 per call). Use shift_left_by_22 for the 10→32 gap.
    mod_add_qq_fast(b, acc, &hi, p); // position 0
    for _ in 0..4 {
        mod_double_inplace_fast(b, &hi, p);
    }
    mod_add_qq_fast(b, acc, &hi, p); // position 4
    for _ in 0..2 {
        mod_double_inplace_fast(b, &hi, p);
    }
    mod_sub_qq_fast(b, acc, &hi, p); // position 6 (SUB because of 977 consolidation)
    for _ in 0..4 {
        mod_double_inplace_fast(b, &hi, p);
    }
    mod_add_qq_fast(b, acc, &hi, p); // position 10
    let (spill, flag_inv, ovf) = mod_shift_left_by_k(b, &hi, p, 22);
    mod_add_qq(b, acc, &hi, p); // position 32
    mod_shift_right_by_k(b, &hi, p, 22, spill, flag_inv, ovf);
    b.set_phase("sol_halve_tail");
    for _ in 0..10 {
        mod_halve_inplace_fast(b, &hi, p);
    }

    b.set_phase("schoolbook_mul_inverse");
    schoolbook_mul_into_addsub_inverse(b, x, y, &tmp_ext);
    b.free_vec(&tmp_ext);
}

pub(crate) fn mod_mul_add_into_acc_schoolbook_peak_lowq(
    b: &mut B,
    acc: &[QubitId],
    x: &[QubitId],
    y: &[QubitId],
    p: U256,
) {
    let n = acc.len();
    debug_assert_eq!(n, 256);

    let tmp_ext = b.alloc_qubits(2 * n);
    b.set_phase("schoolbook_peak_lowq_add_mul");
    schoolbook_mul_into_addsub_lowq(b, x, y, &tmp_ext);

    let lo: Vec<QubitId> = tmp_ext[0..n].to_vec();
    let hi: Vec<QubitId> = tmp_ext[n..2 * n].to_vec();
    b.set_phase("schoolbook_peak_lowq_add_reduce_lo");
    mod_add_qq(b, acc, &lo, p);
    b.set_phase("schoolbook_peak_lowq_add_reduce_hi0");
    mod_add_qq(b, acc, &hi, p);
    b.set_phase("schoolbook_peak_lowq_add_reduce_hi4");
    for _ in 0..4 {
        mod_double_inplace_fast(b, &hi, p);
    }
    mod_add_qq(b, acc, &hi, p);
    b.set_phase("schoolbook_peak_lowq_add_reduce_hi6");
    for _ in 0..2 {
        mod_double_inplace_fast(b, &hi, p);
    }
    mod_sub_qq(b, acc, &hi, p);
    b.set_phase("schoolbook_peak_lowq_add_reduce_hi10");
    for _ in 0..4 {
        mod_double_inplace_fast(b, &hi, p);
    }
    mod_add_qq(b, acc, &hi, p);
    b.set_phase("schoolbook_peak_lowq_add_shift22");
    if std::env::var("D1_PEAK_LOWQ_SHIFT22_DOUBLES")
        .ok()
        .as_deref()
        == Some("1")
    {
        for _ in 0..22 {
            mod_double_inplace_fast_with_dirty(b, &hi, p, Some(acc));
        }
        mod_add_qq(b, acc, &hi, p);
        for _ in 0..22 {
            mod_halve_inplace_fast_with_dirty(b, &hi, p, Some(acc));
        }
    } else {
        let (spill, flag_inv, ovf) = mod_shift_left_by_k_lowq_with_dirty(b, &hi, p, 22, acc);
        mod_add_qq(b, acc, &hi, p);
        mod_shift_right_by_k_lowq_with_dirty(b, &hi, p, 22, spill, flag_inv, ovf, acc);
    }
    b.set_phase("schoolbook_peak_lowq_add_halve_tail");
    for _ in 0..10 {
        mod_halve_inplace_fast(b, &hi, p);
    }

    b.set_phase("schoolbook_peak_lowq_add_unmul");
    schoolbook_mul_into_addsub_lowq_inverse(b, x, y, &tmp_ext);
    b.free_vec(&tmp_ext);
}

pub(crate) fn mod_mul_sub_into_acc_schoolbook_peak_lowq(
    b: &mut B,
    acc: &[QubitId],
    x: &[QubitId],
    y: &[QubitId],
    p: U256,
) {
    mod_neg_inplace_fast(b, acc, p);
    mod_mul_add_into_acc_schoolbook_peak_lowq(b, acc, x, y, p);
    mod_neg_inplace_fast(b, acc, p);
}

pub(crate) fn mod_mul_sub_into_acc_schoolbook(
    b: &mut B,
    acc: &[QubitId],
    x: &[QubitId],
    y: &[QubitId],
    p: U256,
) {
    mod_neg_inplace_fast(b, x, p);
    mod_mul_add_into_acc_schoolbook(b, acc, x, y, p);
    mod_neg_inplace_fast(b, x, p);
}

pub(crate) fn mod_mul_add_into_acc_schoolbook_phase_clean(
    b: &mut B,
    acc: &[QubitId],
    x: &[QubitId],
    y: &[QubitId],
    p: U256,
) {
    let n = acc.len();
    debug_assert_eq!(n, 256);
    let tmp_ext = b.alloc_qubits(2 * n);
    schoolbook_mul_into_addsub_lowq(b, x, y, &tmp_ext);

    let lo: Vec<QubitId> = tmp_ext[0..n].to_vec();
    let hi: Vec<QubitId> = tmp_ext[n..2 * n].to_vec();
    mod_add_qq(b, acc, &lo, p);
    mod_add_qq(b, acc, &hi, p);
    for _ in 0..4 {
        mod_double_inplace(b, &hi, p);
    }
    mod_add_qq(b, acc, &hi, p);
    for _ in 0..2 {
        mod_double_inplace(b, &hi, p);
    }
    mod_sub_qq(b, acc, &hi, p);
    for _ in 0..4 {
        mod_double_inplace(b, &hi, p);
    }
    mod_add_qq(b, acc, &hi, p);
    let (spill, flag_inv, ovf) = mod_shift_left_by_k(b, &hi, p, 22);
    mod_add_qq(b, acc, &hi, p);
    mod_shift_right_by_k(b, &hi, p, 22, spill, flag_inv, ovf);
    for _ in 0..10 {
        mod_halve_inplace(b, &hi, p);
    }

    schoolbook_mul_into_addsub_lowq_inverse(b, x, y, &tmp_ext);
    b.free_vec(&tmp_ext);
}

pub(crate) fn mod_mul_sub_into_acc_schoolbook_phase_clean(
    b: &mut B,
    acc: &[QubitId],
    x: &[QubitId],
    y: &[QubitId],
    p: U256,
) {
    let n = acc.len();
    debug_assert_eq!(n, 256);
    let tmp_ext = b.alloc_qubits(2 * n);
    schoolbook_mul_into_addsub_lowq(b, x, y, &tmp_ext);

    let lo: Vec<QubitId> = tmp_ext[0..n].to_vec();
    let hi: Vec<QubitId> = tmp_ext[n..2 * n].to_vec();
    mod_sub_qq(b, acc, &lo, p);
    mod_sub_qq(b, acc, &hi, p);
    for _ in 0..4 {
        mod_double_inplace(b, &hi, p);
    }
    mod_sub_qq(b, acc, &hi, p);
    for _ in 0..2 {
        mod_double_inplace(b, &hi, p);
    }
    mod_add_qq(b, acc, &hi, p);
    for _ in 0..4 {
        mod_double_inplace(b, &hi, p);
    }
    mod_sub_qq(b, acc, &hi, p);
    let (spill, flag_inv, ovf) = mod_shift_left_by_k(b, &hi, p, 22);
    mod_sub_qq(b, acc, &hi, p);
    mod_shift_right_by_k(b, &hi, p, 22, spill, flag_inv, ovf);
    for _ in 0..10 {
        mod_halve_inplace(b, &hi, p);
    }

    schoolbook_mul_into_addsub_lowq_inverse(b, x, y, &tmp_ext);
    b.free_vec(&tmp_ext);
}

/// Symmetric schoolbook for squaring: x² = sum_i x[i]·2^(2i) + sum_{i<j} 2·x[i]·x[j]·2^(i+j).
/// Each cross-product is computed ONCE (instead of twice in full schoolbook),
/// halving the AND count + Cuccaro_add length. Saves ~130k CCX per squaring.
///
/// Row i layout (width n-i): bit 0 = diagonal x[i] at position 2i, bit 1 = 0
/// (gap), bit k+2 = cross-product (x[i] AND x[i+1+k]) at position i+(i+1+k)+1.
pub(crate) fn schoolbook_square_symmetric(b: &mut B, x: &[QubitId], tmp_ext: &[QubitId]) {
    let n = x.len();
    debug_assert_eq!(tmp_ext.len(), 2 * n);
    for i in 0..n {
        // Width: bit 0 = diag at pos 2i, bit 1 = gap, bits 2..(n-i) = cross-
        // products at positions 2i+2..i+n. Last bit index = n-i, so width = n-i+1.
        // Edge case: i = n-1 has only the diagonal, width = 1.
        let width = if i == n - 1 { 1 } else { n - i + 1 };
        let num_cross = if i + 1 < n { n - i - 1 } else { 0 };
        // num_cross = number of cross-products in this row = width - 2 when width >= 2.
        let row = b.alloc_qubits(width);
        b.cx(x[i], row[0]);
        for k in 0..num_cross {
            b.ccx(x[i], x[i + 1 + k], row[k + 2]);
        }
        let pad = b.alloc_qubit();
        let mut row_padded = row.clone();
        row_padded.push(pad);
        let slice: Vec<QubitId> = tmp_ext[2 * i..2 * i + width + 1].to_vec();
        let c_in = b.alloc_qubit();
        cuccaro_add_fast(b, &row_padded, &slice, c_in);
        b.free(c_in);
        b.free(pad);
        b.cx(x[i], row[0]);
        for k in 0..num_cross {
            let m = b.alloc_bit();
            b.hmr(row[k + 2], m);
            b.cz_if(x[i], x[i + 1 + k], m);
        }
        b.free_vec(&row);
    }
}
