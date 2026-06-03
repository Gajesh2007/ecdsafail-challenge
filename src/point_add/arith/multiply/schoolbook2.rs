//! `multiply::schoolbook2` — verbatim split of the original `multiply` module.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

pub(crate) fn schoolbook_square_symmetric_inverse(b: &mut B, x: &[QubitId], tmp_ext: &[QubitId]) {
    let n = x.len();
    for i in (0..n).rev() {
        let width = if i == n - 1 { 1 } else { n - i + 1 };
        let num_cross = if i + 1 < n { n - i - 1 } else { 0 };
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
        cuccaro_sub_fast(b, &row_padded, &slice, c_in);
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

pub(crate) fn schoolbook_square_symmetric_nohmr(b: &mut B, x: &[QubitId], tmp_ext: &[QubitId]) {
    let n = x.len();
    debug_assert_eq!(tmp_ext.len(), 2 * n);
    for i in 0..n {
        let width = if i == n - 1 { 1 } else { n - i + 1 };
        let num_cross = if i + 1 < n { n - i - 1 } else { 0 };
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
        cuccaro_add(b, &row_padded, &slice, c_in);
        b.free(c_in);
        b.free(pad);
        for k in (0..num_cross).rev() {
            b.ccx(x[i], x[i + 1 + k], row[k + 2]);
        }
        b.cx(x[i], row[0]);
        b.free_vec(&row);
    }
}

pub(crate) fn schoolbook_square_symmetric_nohmr_inverse(b: &mut B, x: &[QubitId], tmp_ext: &[QubitId]) {
    let n = x.len();
    for i in (0..n).rev() {
        let width = if i == n - 1 { 1 } else { n - i + 1 };
        let num_cross = if i + 1 < n { n - i - 1 } else { 0 };
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
        cuccaro_sub(b, &row_padded, &slice, c_in);
        b.free(c_in);
        b.free(pad);
        for k in (0..num_cross).rev() {
            b.ccx(x[i], x[i + 1 + k], row[k + 2]);
        }
        b.cx(x[i], row[0]);
        b.free_vec(&row);
    }
}

pub(crate) fn schoolbook_square_symmetric_lowq(b: &mut B, x: &[QubitId], tmp_ext: &[QubitId]) {
    let n = x.len();
    debug_assert_eq!(tmp_ext.len(), 2 * n);
    for i in 0..n {
        let width = if i == n - 1 { 1 } else { n - i + 1 };
        let num_cross = if i + 1 < n { n - i - 1 } else { 0 };
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
        cuccaro_add(b, &row_padded, &slice, c_in);
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

pub(crate) fn schoolbook_square_symmetric_lowq_inverse(b: &mut B, x: &[QubitId], tmp_ext: &[QubitId]) {
    let n = x.len();
    for i in (0..n).rev() {
        let width = if i == n - 1 { 1 } else { n - i + 1 };
        let num_cross = if i + 1 < n { n - i - 1 } else { 0 };
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
        cuccaro_sub(b, &row_padded, &slice, c_in);
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

/// Like `schoolbook_square_symmetric` (fast, measurement UMA) but the per-row
/// Cuccaro carry lane is hosted on a caller-supplied clean register `host`
/// (returned clean) instead of a fresh allocation. Toffoli-identical to the
/// fast square, peak-identical to the lowq square — used for the z0 lobe of the
/// round84 Karatsuba square, where the not-yet-written z2 slice is clean scratch.
pub(crate) fn schoolbook_square_symmetric_hosted(b: &mut B, x: &[QubitId], tmp_ext: &[QubitId], host: &[QubitId]) {
    let n = x.len();
    debug_assert_eq!(tmp_ext.len(), 2 * n);
    for i in 0..n {
        let width = if i == n - 1 { 1 } else { n - i + 1 };
        let num_cross = if i + 1 < n { n - i - 1 } else { 0 };
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
        cuccaro_add_fast_borrowed_carries(b, &row_padded, &slice, c_in, &host[..row_padded.len() - 1]);
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

pub(crate) fn schoolbook_square_symmetric_hosted_inverse(
    b: &mut B,
    x: &[QubitId],
    tmp_ext: &[QubitId],
    host: &[QubitId],
) {
    let n = x.len();
    for i in (0..n).rev() {
        let width = if i == n - 1 { 1 } else { n - i + 1 };
        let num_cross = if i + 1 < n { n - i - 1 } else { 0 };
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
        cuccaro_sub_fast_borrowed_carries(b, &row_padded, &slice, c_in, &host[..row_padded.len() - 1]);
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

/// Schoolbook squarer with Bennett uncompute. For squaring `tmp_ext = x*x`
/// (2n bits, no mod reduction), then ADD with Solinas reduction to acc,
/// then uncompute tmp_ext via gate-level inverse.
pub(crate) fn squaring_add_to_acc_schoolbook(b: &mut B, acc: &[QubitId], x: &[QubitId], p: U256) {
    let n = acc.len();
    debug_assert_eq!(n, 256);
    debug_assert_eq!(x.len(), n);

    let tmp_ext = b.alloc_qubits(2 * n);
    schoolbook_square_symmetric_lowq(b, x, &tmp_ext);

    let lo: Vec<QubitId> = tmp_ext[0..n].to_vec();
    let hi: Vec<QubitId> = tmp_ext[n..2 * n].to_vec();
    mod_add_qq_fast(b, acc, &lo, p);
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

    schoolbook_square_symmetric_lowq_inverse(b, x, &tmp_ext);
    b.free_vec(&tmp_ext);
}

pub(crate) fn squaring_add_to_acc_schoolbook_phase_clean(b: &mut B, acc: &[QubitId], x: &[QubitId], p: U256) {
    let n = acc.len();
    debug_assert_eq!(n, 256);
    debug_assert_eq!(x.len(), n);

    let tmp_ext = b.alloc_qubits(2 * n);
    schoolbook_square_symmetric_nohmr(b, x, &tmp_ext);

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

    schoolbook_square_symmetric_nohmr_inverse(b, x, &tmp_ext);
    b.free_vec(&tmp_ext);
}

pub(crate) fn squaring_sub_from_acc_schoolbook_phase_clean(
    b: &mut B,
    acc: &[QubitId],
    x: &[QubitId],
    p: U256,
) {
    let n = acc.len();
    debug_assert_eq!(n, 256);
    debug_assert_eq!(x.len(), n);

    let tmp_ext = b.alloc_qubits(2 * n);
    schoolbook_square_symmetric_nohmr(b, x, &tmp_ext);

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

    schoolbook_square_symmetric_nohmr_inverse(b, x, &tmp_ext);
    b.free_vec(&tmp_ext);
}

/// Schoolbook squarer with Bennett uncompute. For squaring `tmp_ext = x*x`
/// (2n bits, no mod reduction), then sub from acc with on-the-fly Solinas
/// reduction, then uncompute tmp_ext via gate-level inverse. Saves ~170k
/// CCX vs walk-x squaring (459k → 289k) by avoiding 256 expensive
/// cmod_add_qq calls (each 5n) in favor of 2n²=131k of cheap AND+Cuccaro.
pub(crate) fn squaring_sub_from_acc_schoolbook(b: &mut B, acc: &[QubitId], x: &[QubitId], p: U256) {
    let n = acc.len();
    debug_assert_eq!(n, 256);
    debug_assert_eq!(x.len(), n);
    let c = U256::MAX.wrapping_sub(p).wrapping_add(U256::from(1));

    // Wide accumulator (2n bits) starts at 0.
    let tmp_ext = b.alloc_qubits(2 * n);

    // Phase 1: symmetric schoolbook tmp_ext = x*x (~half the CCX of full).
    schoolbook_square_symmetric(b, x, &tmp_ext);

    // Phase 2: subtract (lo + hi*c mod p) from acc.
    // For each set bit k of c, sub (hi shifted by k mod p) from acc, by
    // walking hi via mod_double in place. Sub lo first.
    let lo: Vec<QubitId> = tmp_ext[0..n].to_vec();
    let hi: Vec<QubitId> = tmp_ext[n..2 * n].to_vec();
    mod_sub_qq_fast(b, acc, &lo, p);
    let _ = c;
    // 977 consolidation: c = {+2^0, +2^4, -2^6, +2^10, +2^32}. For acc-=hi·c, signs flip:
    // acc -= hi·2^0, acc -= hi·2^4, acc += hi·2^6, acc -= hi·2^10, acc -= hi·2^32.
    mod_sub_qq_fast(b, acc, &hi, p);
    for _ in 0..4 {
        mod_double_inplace_fast(b, &hi, p);
    }
    mod_sub_qq_fast(b, acc, &hi, p);
    for _ in 0..2 {
        mod_double_inplace_fast(b, &hi, p);
    }
    mod_add_qq_fast(b, acc, &hi, p); // sign flipped
    for _ in 0..4 {
        mod_double_inplace_fast(b, &hi, p);
    }
    mod_sub_qq_fast(b, acc, &hi, p);
    let (spill, flag_inv, ovf) = mod_shift_left_by_k(b, &hi, p, 22);
    mod_sub_qq(b, acc, &hi, p);
    mod_shift_right_by_k(b, &hi, p, 22, spill, flag_inv, ovf);
    for _ in 0..10 {
        mod_halve_inplace_fast(b, &hi, p);
    }

    // Phase 3: uncompute tmp_ext via symmetric schoolbook inverse.
    schoolbook_square_symmetric_inverse(b, x, &tmp_ext);

    b.free_vec(&tmp_ext);
}

pub(crate) fn squaring_sub_from_acc_schoolbook_lowq_shift22(
    b: &mut B,
    acc: &[QubitId],
    x: &[QubitId],
    p: U256,
) {
    let n = acc.len();
    debug_assert_eq!(n, 256);
    debug_assert_eq!(x.len(), n);
    let c = U256::MAX.wrapping_sub(p).wrapping_add(U256::from(1));

    let tmp_ext = b.alloc_qubits(2 * n);
    schoolbook_square_symmetric_lowq(b, x, &tmp_ext);

    let lo: Vec<QubitId> = tmp_ext[0..n].to_vec();
    let hi: Vec<QubitId> = tmp_ext[n..2 * n].to_vec();
    mod_sub_qq(b, acc, &lo, p);
    let _ = c;
    mod_sub_qq(b, acc, &hi, p);
    for _ in 0..4 {
        mod_double_inplace_direct_const_fast(b, &hi, p);
    }
    mod_sub_qq(b, acc, &hi, p);
    for _ in 0..2 {
        mod_double_inplace_direct_const_fast(b, &hi, p);
    }
    mod_add_qq(b, acc, &hi, p);
    for _ in 0..4 {
        mod_double_inplace_direct_const_fast(b, &hi, p);
    }
    mod_sub_qq(b, acc, &hi, p);
    let (spill, flag_inv, ovf) = mod_shift_left_by_k_lowq(b, &hi, p, 22);
    mod_sub_qq(b, acc, &hi, p);
    mod_shift_right_by_k_lowq(b, &hi, p, 22, spill, flag_inv, ovf);
    for _ in 0..10 {
        mod_halve_inplace_direct_const_fast(b, &hi, p);
    }

    schoolbook_square_symmetric_lowq_inverse(b, x, &tmp_ext);
    b.free_vec(&tmp_ext);
}

/// Schoolbook: tmp_ext (2n bits) += x * x. Each row i adds (x[i] AND x)
/// shifted by i, captured in n+1 bits to absorb carry into position i+n.
pub(crate) fn schoolbook_square_into(b: &mut B, x: &[QubitId], tmp_ext: &[QubitId]) {
    let n = x.len();
    debug_assert_eq!(tmp_ext.len(), 2 * n);
    for i in 0..n {
        let row = b.alloc_qubits(n);
        for k in 0..n {
            b.ccx(x[i], x[k], row[k]);
        }
        let pad = b.alloc_qubit();
        let mut row_padded = row.clone();
        row_padded.push(pad);
        let slice: Vec<QubitId> = tmp_ext[i..i + n + 1].to_vec();
        let c_in = b.alloc_qubit();
        cuccaro_add_fast(b, &row_padded, &slice, c_in);
        b.free(c_in);
        b.free(pad);
        // Unload row via measurement-based AND uncompute.
        for k in 0..n {
            let m = b.alloc_bit();
            b.hmr(row[k], m);
            b.cz_if(x[i], x[k], m);
        }
        b.free_vec(&row);
    }
}


pub(crate) fn schoolbook_rect_mul_into(b: &mut B, x: &[QubitId], y: &[QubitId], tmp_ext: &[QubitId]) {
    let n = x.len();
    let m = y.len();
    debug_assert!(tmp_ext.len() >= n + m);
    for i in 0..m {
        let row = b.alloc_qubits(n);
        for k in 0..n {
            b.ccx(y[i], x[k], row[k]);
        }
        let pad = b.alloc_qubit();
        let mut row_padded = row.clone();
        row_padded.push(pad);
        let slice: Vec<QubitId> = tmp_ext[i..i + n + 1].to_vec();
        let c_in = b.alloc_qubit();
        cuccaro_add_fast(b, &row_padded, &slice, c_in);
        b.free(c_in);
        b.free(pad);
        for k in 0..n {
            let m = b.alloc_bit();
            b.hmr(row[k], m);
            b.cz_if(y[i], x[k], m);
        }
        b.free_vec(&row);
    }
}

pub(crate) fn schoolbook_rect_mul_into_inverse(b: &mut B, x: &[QubitId], y: &[QubitId], tmp_ext: &[QubitId]) {
    let n = x.len();
    let m = y.len();
    debug_assert!(tmp_ext.len() >= n + m);
    for i in (0..m).rev() {
        let row = b.alloc_qubits(n);
        for k in 0..n {
            b.ccx(y[i], x[k], row[k]);
        }
        let pad = b.alloc_qubit();
        let mut row_padded = row.clone();
        row_padded.push(pad);
        let slice: Vec<QubitId> = tmp_ext[i..i + n + 1].to_vec();
        let c_in = b.alloc_qubit();
        cuccaro_sub_fast(b, &row_padded, &slice, c_in);
        b.free(c_in);
        b.free(pad);
        for k in 0..n {
            let m = b.alloc_bit();
            b.hmr(row[k], m);
            b.cz_if(y[i], x[k], m);
        }
        b.free_vec(&row);
    }
}

pub(crate) fn schoolbook_rect_mul_into_addsub(b: &mut B, x: &[QubitId], y: &[QubitId], tmp_ext: &[QubitId]) {
    let n = x.len();
    let m = y.len();
    debug_assert!(tmp_ext.len() >= n + m);

    let low = b.alloc_qubit();
    let mut wide: Vec<QubitId> = Vec::with_capacity(n + m + 1);
    wide.push(low);
    wide.extend_from_slice(&tmp_ext[..n + m]);

    for k in 0..m {
        let slice: Vec<QubitId> = wide[k..k + n + 1].to_vec();
        controlled_add_subtract_fast(b, x, &slice, y[k]);
    }

    // Rectangular Litinski correction:
    // intermediate = 2xy + 2^(n+m) - 2^m*x - 2^n*(y+1) + x.
    // Apply +2^n*(y+1) + 2^m*x - 2^(n+m) - x.
    {
        let pad = b.alloc_qubit();
        let mut y_ext = y.to_vec();
        y_ext.push(pad);
        let slice: Vec<QubitId> = wide[n..n + m + 1].to_vec();
        let c_in = b.alloc_qubit();
        b.x(c_in);
        cuccaro_add_fast(b, &y_ext, &slice, c_in);
        b.x(c_in);
        b.free(c_in);
        b.free(pad);
    }

    b.x(wide[n + m]);

    {
        let mut x_ext: Vec<QubitId> = x.to_vec();
        while x_ext.len() < n + m + 1 {
            x_ext.push(b.alloc_qubit());
        }
        let c_in = b.alloc_qubit();
        cuccaro_sub(b, &x_ext, &wide, c_in);
        b.free(c_in);
        for _ in n..n + m + 1 {
            let q = x_ext.pop().unwrap();
            b.free(q);
        }
    }

    {
        let pad = b.alloc_qubit();
        let mut x_ext = x.to_vec();
        x_ext.push(pad);
        let slice: Vec<QubitId> = wide[m..m + n + 1].to_vec();
        let c_in = b.alloc_qubit();
        cuccaro_add_fast(b, &x_ext, &slice, c_in);
        b.free(c_in);
        b.free(pad);
    }

    b.free(low);
}

pub(crate) fn schoolbook_rect_mul_into_addsub_inverse(
    b: &mut B,
    x: &[QubitId],
    y: &[QubitId],
    tmp_ext: &[QubitId],
) {
    let n = x.len();
    let m = y.len();
    debug_assert!(tmp_ext.len() >= n + m);

    let low = b.alloc_qubit();
    let mut wide: Vec<QubitId> = Vec::with_capacity(n + m + 1);
    wide.push(low);
    wide.extend_from_slice(&tmp_ext[..n + m]);

    {
        let pad = b.alloc_qubit();
        let mut x_ext = x.to_vec();
        x_ext.push(pad);
        let slice: Vec<QubitId> = wide[m..m + n + 1].to_vec();
        let c_in = b.alloc_qubit();
        cuccaro_sub_fast(b, &x_ext, &slice, c_in);
        b.free(c_in);
        b.free(pad);
    }

    {
        let mut x_ext: Vec<QubitId> = x.to_vec();
        while x_ext.len() < n + m + 1 {
            x_ext.push(b.alloc_qubit());
        }
        let c_in = b.alloc_qubit();
        cuccaro_add(b, &x_ext, &wide, c_in);
        b.free(c_in);
        for _ in n..n + m + 1 {
            let q = x_ext.pop().unwrap();
            b.free(q);
        }
    }

    b.x(wide[n + m]);

    {
        let pad = b.alloc_qubit();
        let mut y_ext = y.to_vec();
        y_ext.push(pad);
        let slice: Vec<QubitId> = wide[n..n + m + 1].to_vec();
        let c_in = b.alloc_qubit();
        b.x(c_in);
        cuccaro_sub_fast(b, &y_ext, &slice, c_in);
        b.x(c_in);
        b.free(c_in);
        b.free(pad);
    }

    for k in (0..m).rev() {
        let slice: Vec<QubitId> = wide[k..k + n + 1].to_vec();
        controlled_add_subtract_fast_inverse(b, x, &slice, y[k]);
    }

    b.free(low);
}
