//! `multiply::cores` — verbatim split of the original `multiply` module.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

pub(crate) fn mod_mul_add_qq(b: &mut B, acc: &[QubitId], x: &[QubitId], y: &[QubitId], p: U256) {
    // acc += x * y mod p. Walk the multiplicand in place to avoid the
    // doubled tmp register and its qubit cost. For squaring, snapshot the
    // original control bits once before the in-place doubling walk.
    let n = acc.len();
    let is_squaring = x[0] == y[0];
    if is_squaring {
        let ctrl_copy = b.alloc_qubits(n);
        for i in 0..n {
            b.cx(x[i], ctrl_copy[i]);
        }
        for i in 0..n {
            cmod_add_qq(b, acc, x, ctrl_copy[i], p);
            if i < n - 1 {
                mod_double_inplace_fast(b, x, p);
            }
        }
        for _ in 0..(n - 1) {
            mod_halve_inplace_fast(b, x, p);
        }
        for i in 0..n {
            b.cx(x[i], ctrl_copy[i]);
        }
        b.free_vec(&ctrl_copy);
    } else {
        for i in 0..n {
            cmod_add_qq(b, acc, x, y[i], p);
            if i < n - 1 {
                mod_double_inplace_fast(b, x, p);
            }
        }
        for _ in 0..(n - 1) {
            mod_halve_inplace_fast(b, x, p);
        }
    }
}

/// Horner-method multiplication: acc += x * y mod p.
/// REQUIRES acc = 0 on entry. Doubles the accumulator (MSB-first),
/// avoiding the tmp register and 255 halvings entirely.
pub(crate) fn mod_mul_horner_add_qq(b: &mut B, acc: &[QubitId], x: &[QubitId], y: &[QubitId], p: U256) {
    let n = acc.len();
    for i in (0..n).rev() {
        if i < n - 1 {
            mod_double_inplace_fast(b, acc, p);
        }
        cmod_add_qq(b, acc, x, y[i], p);
    }
}

/// Exact inverse of `mod_mul_horner_add_qq` on the accumulator:
/// if `acc` currently holds `x * y mod p`, this maps it back to 0 while
/// leaving `x` and `y` unchanged.
pub(crate) fn mod_mul_horner_unadd_qq(b: &mut B, acc: &[QubitId], x: &[QubitId], y: &[QubitId], p: U256) {
    let n = acc.len();
    let is_squaring = x[0] == y[0];
    if is_squaring {
        for i in 0..n {
            cmod_sub_qq(b, acc, x, y[i], p);
            if i < n - 1 {
                mod_halve_inplace_fast(b, acc, p);
            }
        }
    } else {
        mod_neg_inplace_fast(b, x, p);
        for i in 0..n {
            cmod_add_qq(b, acc, x, y[i], p);
            if i < n - 1 {
                mod_halve_inplace_fast(b, acc, p);
            }
        }
        mod_neg_inplace_fast(b, x, p);
    }
}


// ─────────────────────────────────────────────────────────────────────────────────────
// Litinski add-subtract (arXiv:2410.00899) primitives
// ─────────────────────────────────────────────────────────────────────────────────────

/// Controlled add-subtract on (n+1)-bit `acc` with n-bit `x` (padded with 0 at top).
///   ctrl=1 : acc += x  (mod 2^(n+1))
///   ctrl=0 : acc -= x  (mod 2^(n+1))
/// Implementation: conditionally two's-complement (~x + 1) via flip-x plus c_in,
/// then run a single unconditional Gidney/Cuccaro add. Cost = n-1 Toffoli (same as
/// uncontrolled (n+1)-bit add without carry-out).
pub(crate) fn controlled_add_subtract_fast(b: &mut B, x: &[QubitId], acc: &[QubitId], ctrl: QubitId) {
    let n = x.len();
    debug_assert_eq!(acc.len(), n + 1);

    // x_ext: n+1 bits with top pad bit = 0. Only the low n bits of x_ext are flipped
    // when ctrl=0 (two's-complement subtract via ~a + 1). The pad bit stays 0.
    let pad = b.alloc_qubit();
    let mut x_ext = x.to_vec();
    x_ext.push(pad);

    let c_in = b.alloc_qubit();

    // If ctrl=0, we want x_ext[0..n] = ~x and c_in = 1. Encode via x(ctrl) + cx.
    b.x(ctrl);
    for i in 0..n {
        b.cx(ctrl, x_ext[i]);
    }
    b.cx(ctrl, c_in);

    cuccaro_add_fast(b, &x_ext, acc, c_in);

    b.cx(ctrl, c_in);
    for i in 0..n {
        b.cx(ctrl, x_ext[i]);
    }
    b.x(ctrl);

    b.free(c_in);
    b.free(pad);
}

/// Low-peak variant of `controlled_add_subtract_fast` using non-fast
/// Cuccaro (no carry ancillae). Saves ~n qubits of transient peak at the
/// cost of ~n extra Toffolis per call. Useful when called inside the
/// Kaliski-body mul sites where peak is tight.
pub(crate) fn controlled_add_subtract_lowq(b: &mut B, x: &[QubitId], acc: &[QubitId], ctrl: QubitId) {
    let n = x.len();
    debug_assert_eq!(acc.len(), n + 1);

    let pad = b.alloc_qubit();
    let mut x_ext = x.to_vec();
    x_ext.push(pad);

    let c_in = b.alloc_qubit();

    b.x(ctrl);
    for i in 0..n {
        b.cx(ctrl, x_ext[i]);
    }
    b.cx(ctrl, c_in);

    cuccaro_add(b, &x_ext, acc, c_in);

    b.cx(ctrl, c_in);
    for i in 0..n {
        b.cx(ctrl, x_ext[i]);
    }
    b.x(ctrl);

    b.free(c_in);
    b.free(pad);
}

/// Inverse of `controlled_add_subtract_lowq`.
pub(crate) fn controlled_add_subtract_lowq_inverse(b: &mut B, x: &[QubitId], acc: &[QubitId], ctrl: QubitId) {
    let n = x.len();
    debug_assert_eq!(acc.len(), n + 1);

    let pad = b.alloc_qubit();
    let mut x_ext = x.to_vec();
    x_ext.push(pad);

    let c_in = b.alloc_qubit();

    b.x(ctrl);
    for i in 0..n {
        b.cx(ctrl, x_ext[i]);
    }
    b.cx(ctrl, c_in);

    cuccaro_sub(b, &x_ext, acc, c_in);

    b.cx(ctrl, c_in);
    for i in 0..n {
        b.cx(ctrl, x_ext[i]);
    }
    b.x(ctrl);

    b.free(c_in);
    b.free(pad);
}

/// Inverse of controlled_add_subtract_fast: swap add↔sub.
///   ctrl=1 : acc -= x
///   ctrl=0 : acc += x
pub(crate) fn controlled_add_subtract_fast_inverse(b: &mut B, x: &[QubitId], acc: &[QubitId], ctrl: QubitId) {
    let n = x.len();
    debug_assert_eq!(acc.len(), n + 1);

    let pad = b.alloc_qubit();
    let mut x_ext = x.to_vec();
    x_ext.push(pad);

    let c_in = b.alloc_qubit();

    b.x(ctrl);
    for i in 0..n {
        b.cx(ctrl, x_ext[i]);
    }
    b.cx(ctrl, c_in);

    cuccaro_sub_fast(b, &x_ext, acc, c_in);

    b.cx(ctrl, c_in);
    for i in 0..n {
        b.cx(ctrl, x_ext[i]);
    }
    b.x(ctrl);

    b.free(c_in);
    b.free(pad);
}

pub(crate) fn mod_mul_sub_qq(b: &mut B, acc: &[QubitId], x: &[QubitId], y: &[QubitId], p: U256) {
    // acc -= x * y mod p. Negate x, run schoolbook ADD (cheaper than sub),
    // then restore x. For x≠y we can walk the negated multiplicand in place
    // and halve it back afterwards, avoiding the doubled tmp register. For
    // squaring we snapshot the original control bits once into `ctrl_copy`,
    // then reuse the same in-place walk on the negated x.
    let n = acc.len();
    let is_squaring = x[0] == y[0]; // same register → squaring
    if is_squaring {
        // Use the schoolbook squarer for the squaring case (~170k savings).
        if std::env::var("SECP_D1_WALK_SQUARE").ok().as_deref() == Some("1") {
            squaring_sub_from_acc_walk_controls_lowq(b, acc, x, p);
        } else if std::env::var("SECP_D1_LOWQ_SQUARE").ok().as_deref() == Some("1") {
            squaring_sub_from_acc_schoolbook_lowq_shift22(b, acc, x, p);
        } else {
            squaring_sub_from_acc_schoolbook(b, acc, x, p);
        }
        return;
    }
    if false {
        // Hold the original x bits fixed for control while x itself walks
        // through (-x)*2^i mod p.
        let ctrl_copy = b.alloc_qubits(n);
        for i in 0..n {
            b.cx(x[i], ctrl_copy[i]);
        }
        mod_neg_inplace_fast(b, x, p);
        for i in 0..n {
            cmod_add_qq(b, acc, x, ctrl_copy[i], p);
            if i < n - 1 {
                mod_double_inplace_fast(b, x, p);
            }
        }
        for _ in 0..(n - 1) {
            mod_halve_inplace_fast(b, x, p);
        }
        mod_neg_inplace_fast(b, x, p);
        for i in 0..n {
            b.cx(x[i], ctrl_copy[i]);
        }
        b.free_vec(&ctrl_copy);
    } else {
        // Keep x negated during the loop and walk it in place.
        mod_neg_inplace_fast(b, x, p);
        for i in 0..n {
            cmod_add_qq(b, acc, x, y[i], p);
            if i < n - 1 {
                mod_double_inplace_fast(b, x, p);
            }
        }
        for _ in 0..(n - 1) {
            mod_halve_inplace_fast(b, x, p);
        }
        mod_neg_inplace_fast(b, x, p);
    }
}

pub(crate) fn mod_mul_add_qb(b: &mut B, acc: &[QubitId], x: &[QubitId], y: &[BitId], p: U256) {
    let n = acc.len();
    let tmp = b.alloc_qubits(n);
    for i in 0..n {
        b.cx(x[i], tmp[i]);
    }
    for i in 0..n {
        // Mask the whole conditional-add body by y[i]: on shots where
        // y[i]=0 nothing needs to happen AND nothing should be counted.
        b.push_condition(y[i]);
        cmod_add_qq_bit(b, acc, &tmp, y[i], p);
        b.pop_condition();
        if i < n - 1 {
            mod_double_inplace_fast(b, &tmp, p);
        }
    }
    for _ in 0..(n - 1) {
        mod_halve_inplace_fast(b, &tmp, p);
    }
    for i in 0..n {
        b.cx(x[i], tmp[i]);
    }
    b.free_vec(&tmp);
}

pub(crate) fn mod_mul_sub_qb(b: &mut B, acc: &[QubitId], x: &[QubitId], y: &[BitId], p: U256) {
    let n = acc.len();
    let tmp = b.alloc_qubits(n);
    for i in 0..n {
        b.cx(x[i], tmp[i]);
    }
    for i in 0..n {
        b.push_condition(y[i]);
        cmod_sub_qq_bit(b, acc, &tmp, y[i], p);
        b.pop_condition();
        if i < n - 1 {
            mod_double_inplace_fast(b, &tmp, p);
        }
    }
    for _ in 0..(n - 1) {
        mod_halve_inplace_fast(b, &tmp, p);
    }
    for i in 0..n {
        b.cx(x[i], tmp[i]);
    }
    b.free_vec(&tmp);
}

/// In-place classical-constant multiplication: v := v * c mod p.
///
/// Uses the standard compute-in-fresh-then-uncompute pattern:
///   1. tmp = 0
///   2. tmp += v * c                         (shift-and-add, classical c)
///   3. v -= tmp * c^{-1} = v - v*c*c^{-1} = 0  (classical c^{-1})
///   4. swap v, tmp
///   5. free tmp
pub(crate) fn in_place_mul_const(b: &mut B, v: &[QubitId], c: U256, p: U256) {
    let n = v.len();
    let tmp = b.alloc_qubits(n);
    mul_by_const_acc(b, v, c, &tmp, p, false); // tmp += v * c
    let c_inv = classical_modinv(c, p);
    mul_by_const_acc(b, &tmp, c_inv, v, p, true); // v -= tmp * c_inv
    for i in 0..n {
        b.swap(v[i], tmp[i]);
    }
    b.free_vec(&tmp);
}

/// `acc ±= x * c mod p`. `c` is a classical constant. Does NOT fold acc.
/// Maintains a doubling copy of x in a temp register; adds it to acc at
/// positions where c has a bit set.
pub(crate) fn mul_by_const_acc(b: &mut B, x: &[QubitId], c: U256, acc: &[QubitId], p: U256, subtract: bool) {
    mul_by_const_acc_impl(b, x, c, acc, p, subtract, true, true);
}

/// Phase-clean variant of [`mul_by_const_acc`].  It uses exact Cuccaro based
/// add/double/halve blocks rather than the measurement-based fast variants.
/// This is too costly for production, but useful as an algebra-validating
/// fallback when the fast constant multiplier introduces alt-seed phase.
pub(crate) fn mul_by_const_acc_phase_clean(
    b: &mut B,
    x: &[QubitId],
    c: U256,
    acc: &[QubitId],
    p: U256,
    subtract: bool,
) {
    mul_by_const_acc_impl(b, x, c, acc, p, subtract, false, false);
}

/// Mixed variant for diagnosing the prescaler phase: exact q-q add/sub at the
/// sparse constant bits, but fast modular double/halve to walk between bit
/// positions.  If this is phase-clean, the culprit is the fast q-q add/sub, not
/// the scale-walk itself.
pub(crate) fn mul_by_const_acc_exact_adds_fast_shifts(
    b: &mut B,
    x: &[QubitId],
    c: U256,
    acc: &[QubitId],
    p: U256,
    subtract: bool,
) {
    mul_by_const_acc_impl(b, x, c, acc, p, subtract, false, true);
}

pub(crate) fn undo_sparse_const_shifts(b: &mut B, tmp: &[QubitId], p: U256, undo: Vec<SparseConstShiftUndo>) {
    for item in undo.into_iter().rev() {
        match item {
            SparseConstShiftUndo::Doubles(k) => {
                for _ in 0..k {
                    mod_halve_inplace_fast(b, tmp, p);
                }
            }
            SparseConstShiftUndo::Chunk(k, spill, flag_inv, ovf) => {
                mod_shift_right_by_k(b, tmp, p, k, spill, flag_inv, ovf);
            }
        }
    }
}

/// `acc ±= x * c mod p` using exact q-q add/sub at sparse constant bits, but
/// jumping between distant bit positions with the Solinas k-bit shifter instead
/// of one modular double per zero bit.  This borrows `x` itself as the moving
/// 2^i*x lane and restores it before returning, removing the field-sized tmp
/// register from prescaled Kaliski initialization.
pub(crate) fn mul_by_const_acc_chunked_shifts_inplace_src(
    b: &mut B,
    x: &[QubitId],
    c: U256,
    acc: &[QubitId],
    p: U256,
    subtract: bool,
) {
    if c == U256::ZERO {
        return;
    }

    let mut positions = Vec::new();
    for i in 0..256 {
        if bit(c, i) {
            positions.push(i);
        }
    }

    let mut undo = Vec::new();
    let mut cur = 0usize;
    for pos in positions {
        shift_tmp_up_for_sparse_const(b, x, p, pos - cur, &mut undo);
        cur = pos;
        if subtract {
            mod_sub_qq(b, acc, x, p);
        } else {
            mod_add_qq(b, acc, x, p);
        }
    }

    undo_sparse_const_shifts(b, x, p, undo);
}

pub(crate) fn mul_by_const_acc_impl(
    b: &mut B,
    x: &[QubitId],
    c: U256,
    acc: &[QubitId],
    p: U256,
    subtract: bool,
    fast_adds: bool,
    fast_shifts: bool,
) {
    let n = x.len();
    if c == U256::ZERO {
        return;
    }

    // tmp := x  (via CX copy)
    let tmp = b.alloc_qubits(n);
    for i in 0..n {
        b.cx(x[i], tmp[i]);
    }

    // Iterate bits of c from LSB to MSB. At step i, tmp holds x * 2^i mod p.
    // Add tmp to acc if bit i of c is set. Then double tmp for the next step.
    //
    // We iterate up through the highest set bit of c, plus any trailing zero
    // bits (we must double enough times to make uncomputation clean).
    let mut top = 0usize;
    for i in 0..256 {
        if bit(c, i) {
            top = i;
        }
    }

    for i in 0..=top {
        if bit(c, i) {
            if fast_adds {
                if subtract {
                    mod_sub_qq_fast(b, acc, &tmp, p);
                } else {
                    mod_add_qq_fast(b, acc, &tmp, p);
                }
            } else if subtract {
                mod_sub_qq(b, acc, &tmp, p);
            } else {
                mod_add_qq(b, acc, &tmp, p);
            }
        }
        if i < top {
            if fast_shifts {
                mod_double_inplace_fast(b, &tmp, p);
            } else {
                mod_double_inplace(b, &tmp, p);
            }
        }
    }

    // At this point tmp = x * 2^top mod p. Halve it back `top` times to
    // recover x, then uncompute via cx.
    for _ in 0..top {
        if fast_shifts {
            mod_halve_inplace_fast(b, &tmp, p);
        } else {
            mod_halve_inplace(b, &tmp, p);
        }
    }
    for i in 0..n {
        b.cx(x[i], tmp[i]);
    }
    b.free_vec(&tmp);
}

pub(crate) fn mod_mul_add_into_acc_selected(
    b: &mut B,
    acc: &[QubitId],
    x: &[QubitId],
    y: &[QubitId],
    p: U256,
    env_name: &str,
) {
    let local = std::env::var(env_name).ok();
    let global = std::env::var("PA_MUL_ADD_INTO_ACC").ok();
    let default = match env_name {
        "PAIR1_ZERO_TY_MUL" => "karatsuba_lowq",
        "PAIR2_CLEAN_LAM_MUL" => "karatsuba1",
        _ => "schoolbook",
    };
    match local.as_deref().or(global.as_deref()).unwrap_or(default) {
        "walk" => mod_mul_add_qq(b, acc, x, y, p),
        "karatsuba1" | "1" => mod_mul_add_into_acc_karatsuba(b, acc, x, y, p),
        "karatsuba2" | "2" => mod_mul_add_into_acc_karatsuba2(b, acc, x, y, p),
        "karatsuba_lowq" | "lowq" => {
            mod_mul_add_into_acc_karatsuba_lowq(b, acc, x, y, p)
        }
        "schoolbook_peak_lowq" => mod_mul_add_into_acc_schoolbook_peak_lowq(b, acc, x, y, p),
        "schoolbook" => mod_mul_add_into_acc_schoolbook(b, acc, x, y, p),
        other => panic!(
            "unsupported {env_name}={other}; expected schoolbook, schoolbook_peak_lowq, walk, karatsuba1, karatsuba2, or karatsuba_lowq"
        ),
    }
}

pub(crate) fn mod_mul_write_into_zero_acc_selected(
    b: &mut B,
    acc: &[QubitId],
    x: &[QubitId],
    y: &[QubitId],
    p: U256,
    env_name: &str,
) {
    let local = std::env::var(env_name).ok();
    let global = std::env::var("PA_MUL_WRITE_INTO_ZERO_ACC").ok();
    let default = match env_name {
        "PAIR1_MUL1_WRITE" => "karatsuba1",
        _ => "schoolbook",
    };
    match local.as_deref().or(global.as_deref()).unwrap_or(default) {
        "walk" => mod_mul_add_qq(b, acc, x, y, p),
        "karatsuba1" | "1" => mod_mul_write_into_zero_acc_karatsuba(b, acc, x, y, p),
        "karatsuba_lowq" | "lowq" => mod_mul_write_into_zero_acc_karatsuba_lowq(b, acc, x, y, p),
        "karatsuba2" | "2" => mod_mul_write_into_zero_acc_karatsuba2(b, acc, x, y, p),
        "schoolbook_peak_lowq" => mod_mul_write_into_zero_acc_schoolbook_peak_lowq(b, acc, x, y, p),
        "schoolbook_lowq" => mod_mul_write_into_zero_acc_schoolbook_lowq(b, acc, x, y, p),
        "schoolbook" => mod_mul_write_into_zero_acc_schoolbook(b, acc, x, y, p),
        other => panic!(
            "unsupported {env_name}={other}; expected schoolbook, schoolbook_lowq, schoolbook_peak_lowq, walk, karatsuba1, karatsuba_lowq, or karatsuba2"
        ),
    }
}
