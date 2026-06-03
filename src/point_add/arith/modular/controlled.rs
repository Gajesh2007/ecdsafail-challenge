//! `modular::controlled` — verbatim split of the original `modular` module.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

pub(crate) fn cmod_add_qq(b: &mut B, acc: &[QubitId], a: &[QubitId], ctrl: QubitId, p: U256) {
    let n = acc.len();
    let f = b.alloc_qubits(n);
    for i in 0..n {
        b.ccx(ctrl, a[i], f[i]);
    }
    mod_add_qq_fast(b, acc, &f, p);
    // Gidney measurement-based AND uncomputation: f[i] = ctrl AND a[i],
    // which is unchanged by mod_add_qq (Cuccaro restores the addend).
    // HMR + classically-conditioned CZ costs 0 Toffoli vs 256 CCX.
    for i in 0..n {
        let m = b.alloc_bit();
        b.hmr(f[i], m);
        b.cz_if(ctrl, a[i], m);
    }
    b.free_vec(&f);
}

pub(crate) fn cmod_sub_qq(b: &mut B, acc: &[QubitId], a: &[QubitId], ctrl: QubitId, p: U256) {
    let n = acc.len();
    let f = b.alloc_qubits(n);
    for i in 0..n {
        b.ccx(ctrl, a[i], f[i]);
    }
    mod_sub_qq_fast(b, acc, &f, p);
    for i in 0..n {
        let m = b.alloc_bit();
        b.hmr(f[i], m);
        b.cz_if(ctrl, a[i], m);
    }
    b.free_vec(&f);
}

pub(crate) fn cmod_add_qq_lowq(b: &mut B, acc: &[QubitId], a: &[QubitId], ctrl: QubitId, p: U256) {
    let n = acc.len();
    let f = b.alloc_qubits(n);
    for i in 0..n {
        b.ccx(ctrl, a[i], f[i]);
    }
    mod_add_qq(b, acc, &f, p);
    for i in 0..n {
        let m = b.alloc_bit();
        b.hmr(f[i], m);
        b.cz_if(ctrl, a[i], m);
    }
    b.free_vec(&f);
}

pub(crate) fn cmod_sub_qq_lowq(b: &mut B, acc: &[QubitId], a: &[QubitId], ctrl: QubitId, p: U256) {
    let n = acc.len();
    let f = b.alloc_qubits(n);
    for i in 0..n {
        b.ccx(ctrl, a[i], f[i]);
    }
    mod_sub_qq(b, acc, &f, p);
    for i in 0..n {
        let m = b.alloc_bit();
        b.hmr(f[i], m);
        b.cz_if(ctrl, a[i], m);
    }
    b.free_vec(&f);
}

pub(crate) fn cmod_add_qq_bit(b: &mut B, acc: &[QubitId], a: &[QubitId], ctrl: BitId, p: U256) {
    let n = acc.len();
    let f = b.alloc_qubits(n);
    for i in 0..n {
        b.cx_if(a[i], f[i], ctrl);
    }
    mod_add_qq_fast(b, acc, &f, p);
    for i in 0..n {
        b.cx_if(a[i], f[i], ctrl);
    }
    b.free_vec(&f);
}

pub(crate) fn cmod_sub_qq_bit(b: &mut B, acc: &[QubitId], a: &[QubitId], ctrl: BitId, p: U256) {
    let n = acc.len();
    let f = b.alloc_qubits(n);
    for i in 0..n {
        b.cx_if(a[i], f[i], ctrl);
    }
    mod_sub_qq_fast(b, acc, &f, p);
    for i in 0..n {
        b.cx_if(a[i], f[i], ctrl);
    }
    b.free_vec(&f);
}

pub(crate) fn cmod_add_qc_fast(b: &mut B, acc: &[QubitId], c: U256, ctrl: QubitId, p: U256) {
    let n = acc.len();
    let f = b.alloc_qubits(n);
    for i in 0..n {
        if bit(c, i) {
            b.cx(ctrl, f[i]);
        }
    }
    mod_add_qq_fast(b, acc, &f, p);
    for i in 0..n {
        if bit(c, i) {
            b.cx(ctrl, f[i]);
        }
    }
    b.free_vec(&f);
}

pub(crate) fn cmod_sub_qc_fast(b: &mut B, acc: &[QubitId], c: U256, ctrl: QubitId, p: U256) {
    let n = acc.len();
    let f = b.alloc_qubits(n);
    for i in 0..n {
        if bit(c, i) {
            b.cx(ctrl, f[i]);
        }
    }
    mod_sub_qq_fast(b, acc, &f, p);
    for i in 0..n {
        if bit(c, i) {
            b.cx(ctrl, f[i]);
        }
    }
    b.free_vec(&f);
}

pub(crate) fn cmod_double_inplace(b: &mut B, v: &[QubitId], p: U256, ctrl: QubitId) {
    let n = v.len();
    let ovf = b.alloc_qubit();
    let mut v_ext: Vec<QubitId> = v.to_vec();
    v_ext.push(ovf);

    // Conditional left-shift: if ctrl=1, v[n-1] → ovf; v[i] → v[i+1].
    cswap(b, ctrl, v[n - 1], ovf);
    for i in (0..n - 1).rev() {
        cswap(b, ctrl, v[i], v[i + 1]);
    }

    csub_nbit_const(b, &v_ext, p, ctrl);
    cadd_nbit_const(b, &v_ext, p, ovf);
    // ovf ends at 0 by the same argument as mod_double_inplace.
    b.free(ovf);
}

/// `cmod_halve_inplace` = exact inverse of `cmod_double_inplace`.
pub(crate) fn cmod_halve_inplace(b: &mut B, v: &[QubitId], p: U256, ctrl: QubitId) {
    let n = v.len();
    let ovf = b.alloc_qubit();
    let mut v_ext: Vec<QubitId> = v.to_vec();
    v_ext.push(ovf);

    // Inverse of: cadd(v_ext, p, ovf).
    csub_nbit_const(b, &v_ext, p, ovf);
    // Inverse of: csub(v_ext, p, ctrl).
    cadd_nbit_const(b, &v_ext, p, ctrl);
    // Inverse of cswap cascade (self-inverse; reversed order).
    for i in 0..n - 1 {
        cswap(b, ctrl, v[i], v[i + 1]);
    }
    cswap(b, ctrl, v[n - 1], ovf);

    b.free(ovf);
}

pub(crate) fn cmod_sub_qq_lowq_borrowed_subtrahend(
    b: &mut B,
    acc: &[QubitId],
    a: &[QubitId],
    ctrl: QubitId,
    p: U256,
    f: &[QubitId],
) {
    assert_eq!(acc.len(), N);
    assert_eq!(a.len(), N);
    assert_eq!(f.len(), N);

    for i in 0..N {
        b.ccx(ctrl, a[i], f[i]);
    }
    mod_sub_qq(b, acc, f, p);
    for i in (0..N).rev() {
        b.ccx(ctrl, a[i], f[i]);
    }
}
