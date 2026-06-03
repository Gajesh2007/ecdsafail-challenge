//! `modular::add` — verbatim split of the original `modular` module.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

/// `acc := (acc + a) mod p`. Both `acc` and `a` are n-bit quantum registers
/// with value in [0, p). Solinas reduction using c = 2^n - p: sum ∈ [0, 2p),
/// then add c, branch on top bit to either clear it (reduction) or undo
/// the add (no reduction). Saves one full (n+1)-wide Cuccaro compared to
/// the sub-p/add-p/csub-p pattern.
pub(crate) fn mod_add_qq(b: &mut B, acc: &[QubitId], a: &[QubitId], p: U256) {
    let n = acc.len();
    assert_eq!(n, a.len());
    debug_assert_eq!(n, 256);

    let (acc_ext, acc_ovf) = ext_reg(b, acc);
    let (a_ext, a_ovf) = ext_reg(b, a);

    // Step 1: (n+1)-bit add. acc_ext ∈ [0, 2p).
    add_nbit_qq(b, &a_ext, &acc_ext);

    // Step 2: add c. If sum was >= p, the top bit of (sum + c) becomes 1.
    let c = U256::MAX.wrapping_sub(p).wrapping_add(U256::from(1));
    add_nbit_const(b, &acc_ext, c);

    // Step 3: flag := acc_ovf (= top bit of sum + c).
    let flag = b.alloc_qubit();
    b.cx(acc_ovf, flag);

    // Step 4: if flag=0 (no reduction needed), undo the add of c.
    b.x(flag);
    csub_nbit_const(b, &acc_ext, c, flag);
    b.x(flag);

    // Step 5: if flag=1, clear the top bit (drops 2^n → yields sum - p).
    b.cx(flag, acc_ovf);

    // Step 6: uncompute flag. Same identity as the old version:
    //   flag == (acc_final < a_orig)
    // because in the flag=1 case acc_final = acc_orig + a - p < a (since acc_orig < p),
    // and in the flag=0 case acc_final = acc_orig + a ≥ a.
    cmp_lt_into(b, &acc_ext[..n], &a_ext[..n], flag);
    b.free(flag);

    unext_reg(b, a_ovf);
    unext_reg(b, acc_ovf);
    let _ = (acc_ext, a_ext);
}

/// Low-peak `acc := (acc + a) mod p`. Identical structure to `mod_add_qq` but
/// the two Solinas-constant corrections (`+c`, conditional `-c`) are vented onto
/// the operand `a_ext` as dirty scratch (2 clean qubits) instead of a fresh
/// n-qubit loaded-constant register. The main add and the flag-uncompute compare
/// stay ancilla-free (Cuccaro / cmp_lt_into), so the only transient is +2 clean.
/// Used inside the round84 Solinas reduction where the materialized `load_const`
/// coexisting with tmp_ext + z1_reg was the peak binder. `c = 2^256 - p` fits in
/// 64 bits, so `c_low` carries the whole constant.
pub(crate) fn mod_add_qq_vent(b: &mut B, acc: &[QubitId], a: &[QubitId], p: U256) {
    let n = acc.len();
    assert_eq!(n, a.len());
    debug_assert_eq!(n, 256);

    let (acc_ext, acc_ovf) = ext_reg(b, acc);
    let (a_ext, a_ovf) = ext_reg(b, a);

    add_nbit_qq(b, &a_ext, &acc_ext);

    let c = U256::MAX.wrapping_sub(p).wrapping_add(U256::from(1));
    let c_low = c.as_limbs()[0];
    let n1 = acc_ext.len();
    {
        let q_clean2: [QubitId; 2] = [b.alloc_qubit(), b.alloc_qubit()];
        venting::iadd_dirty_2clean_classical(
            b,
            &acc_ext,
            &a_ext[..n1 - 2],
            &q_clean2,
            c_low,
            false,
        );
        b.free(q_clean2[0]);
        b.free(q_clean2[1]);
    }

    let flag = b.alloc_qubit();
    b.cx(acc_ovf, flag);

    b.x(flag);
    {
        let q_clean2: [QubitId; 2] = [b.alloc_qubit(), b.alloc_qubit()];
        venting::cisub_dirty_2clean_classical(
            b,
            &acc_ext,
            &a_ext[..n1 - 2],
            &q_clean2,
            c_low,
            flag,
        );
        b.free(q_clean2[0]);
        b.free(q_clean2[1]);
    }
    b.x(flag);

    b.cx(flag, acc_ovf);

    cmp_lt_into(b, &acc_ext[..n], &a_ext[..n], flag);
    b.free(flag);

    unext_reg(b, a_ovf);
    unext_reg(b, acc_ovf);
    let _ = (acc_ext, a_ext);
}

pub(crate) fn mod_add_qc(b: &mut B, acc: &[QubitId], c: U256, p: U256) {
    // acc := (acc + c) mod p. c is a compile-time constant.
    let n = acc.len();
    let a = load_const(b, n, c);
    mod_add_qq_fast(b, acc, &a, p);
    unload_const(b, &a, c);
}

pub(crate) fn mod_add_qb(b: &mut B, acc: &[QubitId], bits: &[BitId], p: U256) {
    // acc := (acc + bits) mod p. `bits` is a classical bit register.
    let a = load_bits(b, bits);
    mod_add_qq_fast(b, acc, &a, p);
    unload_bits(b, &a, bits);
}

pub(crate) fn mod_add_qb_phase_clean(b: &mut B, acc: &[QubitId], bits: &[BitId], p: U256) {
    let a = load_bits(b, bits);
    mod_add_qq(b, acc, &a, p);
    unload_bits(b, &a, bits);
}

pub(crate) fn spill_qoffset_addsub_lowq(
    b: &mut B,
    spill: &[QubitId],
    target: &[QubitId],
    is_sub: bool,
    dirty_src: Option<&[QubitId]>,
) {
    let width = target.len();
    if let Some(dirty) = dirty_src {
        if width > 4 && dirty.len() >= width - 2 {
            let zero = b.alloc_qubit();
            let clean: [QubitId; 2] = [b.alloc_qubit(), b.alloc_qubit()];
            let mut offset = Vec::with_capacity(width);
            for i in 0..width {
                offset.push(if i < spill.len() { spill[i] } else { zero });
            }
            if is_sub {
                venting::isub_dirty_2clean_qoffset(b, target, dirty, &clean, &offset);
            } else {
                venting::iadd_dirty_2clean_qoffset(b, target, dirty, &clean, &offset, false);
            }
            b.free(clean[1]);
            b.free(clean[0]);
            b.free(zero);
            return;
        }
    }

    let padded = b.alloc_qubits(width);
    for i in 0..spill.len().min(width) {
        b.cx(spill[i], padded[i]);
    }
    let c_in = b.alloc_qubit();
    if is_sub {
        cuccaro_sub(b, &padded, target, c_in);
    } else {
        cuccaro_add(b, &padded, target, c_in);
    }
    b.free(c_in);
    for i in 0..spill.len().min(width) {
        b.cx(spill[i], padded[i]);
    }
    b.free_vec(&padded);
}

/// Like `mod_add_qq` but uses `cmp_lt_into_fast` for the flag uncompute.
/// NOT safe inside emit_inverse blocks.
pub(crate) fn mod_add_qq_fast(b: &mut B, acc: &[QubitId], a: &[QubitId], p: U256) {
    let n = acc.len();
    assert_eq!(n, a.len());
    debug_assert_eq!(n, 256);

    let (acc_ext, acc_ovf) = ext_reg(b, acc);
    let (a_ext, a_ovf) = ext_reg(b, a);

    // Use fast (measurement-based) Cuccaro everywhere.
    add_nbit_qq_fast(b, &a_ext, &acc_ext);
    let c = U256::MAX.wrapping_sub(p).wrapping_add(U256::from(1));
    // add_nbit_const with fast Cuccaro OR venting (using `a` as dirty).
    let use_vent = kal_vent_modadd_enabled();
    if use_vent {
        let n1 = acc_ext.len();
        // Use `a_ext` as dirty qubits (it was just used as add operand,
        // its value is preserved through the venting sub-protocol).
        let c_low = c.as_limbs()[0];
        let q_clean2: [QubitId; 2] = [b.alloc_qubit(), b.alloc_qubit()];
        venting::iadd_dirty_2clean_classical(
            b,
            &acc_ext,
            &a_ext[..n1 - 2],
            &q_clean2,
            c_low,
            false,
        );
        b.free(q_clean2[0]);
        b.free(q_clean2[1]);
    } else if secp_direct_const_arith_enabled() {
        add_nbit_const_direct_uncontrolled_fast(b, &acc_ext, c);
    } else {
        let n1 = acc_ext.len();
        let ca = load_const(b, n1, c);
        add_nbit_qq_fast(b, &ca, &acc_ext);
        unload_const(b, &ca, c);
    }
    let flag = b.alloc_qubit();
    b.cx(acc_ovf, flag);
    b.x(flag);
    // csub_nbit_const with fast Cuccaro OR venting.
    if use_vent {
        let c_low = c.as_limbs()[0];
        let n1 = acc_ext.len();
        let q_clean2: [QubitId; 2] = [b.alloc_qubit(), b.alloc_qubit()];
        venting::cisub_dirty_2clean_classical(
            b,
            &acc_ext,
            &a_ext[..n1 - 2],
            &q_clean2,
            c_low,
            flag,
        );
        b.free(q_clean2[0]);
        b.free(q_clean2[1]);
    } else if secp_direct_const_arith_enabled() {
        csub_nbit_const_direct_fast(b, &acc_ext, c, flag);
    } else {
        let n1 = acc_ext.len();
        let ca = b.alloc_qubits(n1);
        for i in 0..n1 {
            if bit(c, i) {
                b.cx(flag, ca[i]);
            }
        }
        sub_nbit_qq_fast(b, &ca, &acc_ext);
        for i in 0..n1 {
            if bit(c, i) {
                b.cx(flag, ca[i]);
            }
        }
        b.free_vec(&ca);
    }
    b.x(flag);
    b.cx(flag, acc_ovf);
    cmp_lt_into_fast(b, &acc_ext[..n], &a_ext[..n], flag);
    b.free(flag);

    unext_reg(b, a_ovf);
    unext_reg(b, acc_ovf);
    let _ = (acc_ext, a_ext);
}

/// Specialization of mod_add_qq_fast when acc = 0 on entry. Replaces the
/// initial Cuccaro add with CX-copy (0 CCX instead of n-1 CCX).
/// Saves 255 CCX per call.
pub(crate) fn mod_add_qq_fast_from_zero(b: &mut B, acc: &[QubitId], a: &[QubitId], p: U256) {
    let n = acc.len();
    assert_eq!(n, a.len());
    debug_assert_eq!(n, 256);

    let (acc_ext, acc_ovf) = ext_reg(b, acc);
    let (a_ext, a_ovf) = ext_reg(b, a);

    // acc is 0 on entry. CX-copy a into acc (0 CCX). Top bits both 0.
    for i in 0..n {
        b.cx(a[i], acc[i]);
    }
    // acc_ovf and a_ovf are both 0 (both freshly allocated as 0 by ext_reg).

    let c = U256::MAX.wrapping_sub(p).wrapping_add(U256::from(1));
    let use_vent = kal_vent_modadd_enabled();
    if use_vent {
        let n1 = acc_ext.len();
        let c_low = c.as_limbs()[0];
        let q_clean2: [QubitId; 2] = [b.alloc_qubit(), b.alloc_qubit()];
        venting::iadd_dirty_2clean_classical(
            b,
            &acc_ext,
            &a_ext[..n1 - 2],
            &q_clean2,
            c_low,
            false,
        );
        b.free(q_clean2[0]);
        b.free(q_clean2[1]);
    } else {
        let n1 = acc_ext.len();
        let ca = load_const(b, n1, c);
        add_nbit_qq_fast(b, &ca, &acc_ext);
        unload_const(b, &ca, c);
    }
    let flag = b.alloc_qubit();
    b.cx(acc_ovf, flag);
    b.x(flag);
    if use_vent {
        let c_low = c.as_limbs()[0];
        let n1 = acc_ext.len();
        let q_clean2: [QubitId; 2] = [b.alloc_qubit(), b.alloc_qubit()];
        venting::cisub_dirty_2clean_classical(
            b,
            &acc_ext,
            &a_ext[..n1 - 2],
            &q_clean2,
            c_low,
            flag,
        );
        b.free(q_clean2[0]);
        b.free(q_clean2[1]);
    } else {
        let n1 = acc_ext.len();
        let ca = b.alloc_qubits(n1);
        for i in 0..n1 {
            if bit(c, i) {
                b.cx(flag, ca[i]);
            }
        }
        sub_nbit_qq_fast(b, &ca, &acc_ext);
        for i in 0..n1 {
            if bit(c, i) {
                b.cx(flag, ca[i]);
            }
        }
        b.free_vec(&ca);
    }
    b.x(flag);
    b.cx(flag, acc_ovf);
    cmp_lt_into_fast(b, &acc_ext[..n], &a_ext[..n], flag);
    b.free(flag);

    unext_reg(b, a_ovf);
    unext_reg(b, acc_ovf);
    let _ = (acc_ext, a_ext);
}
