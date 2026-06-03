//! Logical shifts and controlled Cuccaro lanes.
//!
//! Single-bit shifts (`shift_left_1`, `shift_right_1`, `c_shift_right_1`) and
//! the controlled adder lane primitives (`ctrl_maj` / `ctrl_uma`,
//! `cucc_*_ctrl`).

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;


/// Controlled n-bit subtract mod 2^n: if ctrl, acc -= a. Both are n-wide
/// qubit slices. Not a mod-p operation.
pub(crate) fn cucc_sub_ctrl(b: &mut B, a: &[QubitId], acc: &[QubitId], ctrl: QubitId) {
    let n = a.len();
    let tmp = b.alloc_qubits(n);
    for i in 0..n {
        b.ccx(ctrl, a[i], tmp[i]);
    }
    sub_nbit_qq(b, &tmp, acc);
    for i in 0..n {
        b.ccx(ctrl, a[i], tmp[i]);
    }
    b.free_vec(&tmp);
}

pub(crate) fn ctrl_maj(b: &mut B, ctrl: QubitId, x: QubitId, y: QubitId, w: QubitId, scratch: QubitId) {
    b.ccx(ctrl, w, y);
    b.ccx(ctrl, w, x);
    mcx3_polar(b, ctrl, true, x, true, y, true, w, scratch);
}

pub(crate) fn ctrl_uma(b: &mut B, ctrl: QubitId, x: QubitId, y: QubitId, w: QubitId, scratch: QubitId) {
    mcx3_polar(b, ctrl, true, x, true, y, true, w, scratch);
    b.ccx(ctrl, w, x);
    b.ccx(ctrl, x, y);
}

pub(crate) fn ctrl_inv_maj(b: &mut B, ctrl: QubitId, x: QubitId, y: QubitId, w: QubitId, scratch: QubitId) {
    mcx3_polar(b, ctrl, true, x, true, y, true, w, scratch);
    b.ccx(ctrl, w, x);
    b.ccx(ctrl, w, y);
}

pub(crate) fn ctrl_inv_uma(b: &mut B, ctrl: QubitId, x: QubitId, y: QubitId, w: QubitId, scratch: QubitId) {
    b.ccx(ctrl, x, y);
    b.ccx(ctrl, w, x);
    mcx3_polar(b, ctrl, true, x, true, y, true, w, scratch);
}

pub(crate) fn cucc_add_ctrl_lowq(b: &mut B, a: &[QubitId], acc: &[QubitId], ctrl: QubitId) {
    let c_in = b.alloc_qubit();
    let scratch = b.alloc_qubit();
    cuccaro_add_ctrl_lowq(b, a, acc, ctrl, c_in, scratch);
    b.free(scratch);
    b.free(c_in);
}

pub(crate) fn cucc_sub_ctrl_lowq(b: &mut B, a: &[QubitId], acc: &[QubitId], ctrl: QubitId) {
    let c_in = b.alloc_qubit();
    let scratch = b.alloc_qubit();
    cuccaro_sub_ctrl_lowq(b, a, acc, ctrl, c_in, scratch);
    b.free(scratch);
    b.free(c_in);
}

/// Controlled n-bit add mod 2^n: if ctrl, acc += a.
pub(crate) fn cucc_add_ctrl(b: &mut B, a: &[QubitId], acc: &[QubitId], ctrl: QubitId) {
    let n = a.len();
    let tmp = b.alloc_qubits(n);
    for i in 0..n {
        b.ccx(ctrl, a[i], tmp[i]);
    }
    add_nbit_qq(b, &tmp, acc);
    for i in 0..n {
        b.ccx(ctrl, a[i], tmp[i]);
    }
    b.free_vec(&tmp);
}

/// Controlled shift-right by 1 of an n-bit register. ASSUMES v[0]=0 when
/// ctrl=1 (so no information is lost). Implemented as a controlled swap
/// cascade: if ctrl=1, new v[i] = old v[i+1] for i < n-1, new v[n-1] = 0.
pub(crate) fn c_shift_right_1(b: &mut B, v: &[QubitId], ctrl: QubitId) {
    let n = v.len();
    for i in 0..(n - 1) {
        cswap(b, ctrl, v[i], v[i + 1]);
    }
}

/// Unconditional shift-left by 1 of an (n+1)-bit register. ASSUMES r[n]=0
/// before the shift. After the shift: r[0]=0, r[i] = old r[i-1] for i ∈ [1, n].
pub(crate) fn shift_left_1(b: &mut B, r: &[QubitId]) {
    let n1 = r.len(); // n+1
                      // Swap r[n] ↔ r[0] first: r[0] gets the known-0 top bit.
    b.swap(r[n1 - 1], r[0]);
    // Then propagate: swap r[n] ↔ r[n-1], r[n-1] ↔ r[n-2], ..., r[2] ↔ r[1].
    for i in (2..n1).rev() {
        b.swap(r[i], r[i - 1]);
    }
}

/// Inverse of `shift_left_1`: shifts an (n+1)-bit register right by 1.
/// ASSUMES r[0]=0 before the shift (i.e., was even).
#[allow(dead_code)]
pub(crate) fn shift_right_1(b: &mut B, r: &[QubitId]) {
    let n1 = r.len();
    for i in 2..n1 {
        b.swap(r[i], r[i - 1]);
    }
    b.swap(r[n1 - 1], r[0]);
}

pub(crate) fn shift_tmp_up_for_sparse_const(
    b: &mut B,
    tmp: &[QubitId],
    p: U256,
    mut delta: usize,
    undo: &mut Vec<SparseConstShiftUndo>,
) {
    while delta >= 22 {
        let (spill, flag_inv, ovf) = mod_shift_left_by_k(b, tmp, p, 22);
        undo.push(SparseConstShiftUndo::Chunk(22, spill, flag_inv, ovf));
        delta -= 22;
    }
    if delta >= 12 {
        let (spill, flag_inv, ovf) = mod_shift_left_by_k(b, tmp, p, delta);
        undo.push(SparseConstShiftUndo::Chunk(delta, spill, flag_inv, ovf));
    } else if delta > 0 {
        for _ in 0..delta {
            mod_double_inplace_fast(b, tmp, p);
        }
        undo.push(SparseConstShiftUndo::Doubles(delta));
    }
}
