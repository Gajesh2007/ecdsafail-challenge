
#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

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
