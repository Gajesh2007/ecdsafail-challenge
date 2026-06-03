//! `bench::scaled` — verbatim split of the original `bench` module.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

pub(crate) fn scaled_by_controlled_microstep(
    b: &mut B,
    r: &[QubitId],
    s: &[QubitId],
    odd: QubitId,
    a: QubitId,
    p: U256,
) {
    // Direct scaled Bernstein-Yang tagged-DIV microstep:
    //   C: (r,s) -> (r, s/2)
    //   B: (r,s) -> (r, (s+r)/2)
    //   A: (r,s) -> (s, (s-r)/2)
    // A is emitted as swap, neg(second row), selected add, halve.
    for i in 0..r.len() {
        cswap(b, a, r[i], s[i]);
    }
    by_cmod_neg_inplace_fast(b, s, a, p);
    cmod_add_qq(b, s, r, odd, p);
    mod_halve_inplace_fast(b, s, p);
}

pub(crate) fn scaled_by_controlled_microstep_inverse_negr_for_bench(
    b: &mut B,
    u_neg_r: &[QubitId],
    s: &[QubitId],
    odd: QubitId,
    a: QubitId,
    p: U256,
) {
    // Inverse scaled BY step in the sign-flipped frame u=-r:
    //   C: (u,s) -> (u, 2s)
    //   B: (u,s) -> (u, 2s+u)
    //   A: (u,s) -> (u+2s, -u)
    // This product-clean path avoids centered parity history entirely.  Use the
    // canonical controlled negation so a logically-zero final u can be freed.
    mod_double_inplace_fast(b, s, p);
    cmod_add_qq(b, s, u_neg_r, odd, p);
    for i in 0..u_neg_r.len() {
        cswap(b, a, u_neg_r[i], s[i]);
    }
    by_cmod_neg_inplace_canonical_for_bench(b, s, a, p);
}

pub(crate) fn write_pair2_product_and_clean_lam_with_scaled_by_bench(
    b: &mut B,
    lam: &[QubitId],
    denom: &[QubitId],
    product: &[QubitId],
    p: U256,
) {
    // Last-shot BY architecture: use scaled BY inverse/product-clean directly
    // for pair2.  Given q=lam and denominator x, the inverse scaled replay maps
    // (sign(f)*q, 0) -> (0, q*x).  In the u=-r frame the input is
    // u = -sign(f)*q, so f>0 selects -q and f<0 leaves q.  This deletes pair2's
    // old q*x multiplication and avoids centered parity history; it still uses
    // the direct 576-step denominator generator and is therefore a correctness
    // probe, not yet SOTA-shaped.
    const STEPS: usize = 576;
    const DBITS: usize = 12;
    b.set_phase("pair2_by_scaled_product_alloc");
    let f = b.alloc_qubits(STEPS);
    let g = b.alloc_qubits(STEPS);
    let delta = b.alloc_qubits(DBITS);
    let odd = b.alloc_qubits(STEPS);
    let a_ctrl = b.alloc_qubits(STEPS);

    for i in 0..N {
        if bit(p, i) {
            b.x(f[i]);
        }
        b.cx(denom[i], g[i]);
    }
    b.x(delta[0]);

    b.set_phase("pair2_by_scaled_product_generate");
    by_generate_signed_controls_for_bench(b, &f, &g, &delta, &odd, &a_ctrl, None);

    b.set_phase("pair2_by_scaled_product_frame");
    let f_pos = b.alloc_qubit();
    b.x(f_pos);
    b.cx(f[STEPS - 1], f_pos);
    by_cmod_neg_inplace_canonical_for_bench(b, lam, f_pos, p);

    b.set_phase("pair2_by_scaled_product_inverse");
    for i in (0..STEPS).rev() {
        scaled_by_controlled_microstep_inverse_negr_for_bench(
            b, lam, product, odd[i], a_ctrl[i], p,
        );
    }

    b.set_phase("pair2_by_scaled_product_clear_frame");
    b.cx(f[STEPS - 1], f_pos);
    b.x(f_pos);
    b.free(f_pos);

    b.set_phase("pair2_by_scaled_product_reverse_den");
    by_reverse_signed_controls_for_bench(b, &f, &g, &delta, &odd, &a_ctrl, None);

    b.set_phase("pair2_by_scaled_product_clear");
    for i in 0..N {
        b.cx(denom[i], g[i]);
        if bit(p, i) {
            b.x(f[i]);
        }
    }
    b.x(delta[0]);
    b.free_vec(&a_ctrl);
    b.free_vec(&odd);
    b.free_vec(&delta);
    b.free_vec(&g);
    b.free_vec(&f);
}
