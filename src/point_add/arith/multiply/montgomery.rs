
#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

/// Montgomery multiply using sparse REDC reduction.
/// Computes: acc := (acc * x) * R^{-1} mod p
pub(crate) fn mont_mul(b: &mut B, acc: &[QubitId], x: &[QubitId], _p: U256) {
    let n = acc.len();
    debug_assert_eq!(n, 256);
    let tmp = b.alloc_qubits(2 * n);

    // Phase 1: raw product t = acc * x
    schoolbook_mul_into_addsub(b, acc, x, &tmp);

    // Phase 2: compute m = t_low * c^{-1} mod 2^32
    // c^{-1} = 0x9D84D9F1, sparse with 19 set bits
    let m = b.alloc_qubits(32);

    // Copy t_low to m, then add shifted copies for each set bit
    // This is the sparse multiplication: m = sum of (t_low << pos)
    for i in 0..32 {
        b.cx(tmp[i], m[i]);
    }
    // Add shifted copies for each set bit position
    for pos in &MONT_CINV_POS[1..] {
        // Skip 0, already copied
        let shift = *pos;
        for i in 0..(32 - shift) {
            b.cx(tmp[i], m[i + shift]);
        }
    }

    // Phase 3: result = t_high + m (the cheap reduction!)
    for i in 0..n {
        b.cx(tmp[n + i], acc[i]);
    }
    for i in 0..32 {
        b.cx(m[i], acc[i]);
    }

    // Cleanup: uncompute in reverse order
    for pos in MONT_CINV_POS[1..].iter().rev() {
        let shift = *pos;
        for i in (0..(32 - shift)).rev() {
            b.cx(tmp[i], m[i + shift]);
        }
    }
    for i in 0..32 {
        b.cx(tmp[i], m[i]);
    }
    schoolbook_mul_into_addsub_inverse(b, acc, x, &tmp);
    b.free_vec(&m);
    b.free_vec(&tmp);
}

