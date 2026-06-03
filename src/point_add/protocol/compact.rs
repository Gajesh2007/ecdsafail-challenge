//! `protocol::compact` — verbatim split of the original `protocol` module.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

pub(crate) fn build_compact_point_add(
    b: &mut B,
    tx: &[QubitId],
    ty: &[QubitId],
    ox: &[BitId],
    oy: &[BitId],
    p: U256,
) {
    if std::env::var("COMPACT_POINT_ADD_CLEAN_EARLY_INV")
        .ok()
        .as_deref()
        == Some("1")
    {
        build_compact_point_add_clean_early_inv(b, tx, ty, ox, oy, p);
        return;
    }

    // At entry: tx = dx, ty = dy (after step 1-2 subtraction)
    //
    // Compact architecture using Fermat inversion:
    // 1. inv_dx = dx^{p-2} (Fermat) → fresh register
    // 2. lam = dy * inv_dx → fresh register
    // 3. ty -= lam * tx → ty = 0
    // 4. tx = dx - lam² → affine corrections → tx = Rx - Qx
    // 5. ty = lam * tx → Ry calculation
    // 6. Cleanup via second Fermat inversion

    let n = tx.len();

    // inv_dx = dx^{-1} mod p (Fermat)
    let inv_dx = b.alloc_qubits(n);
    b.set_phase("fermat_inv_dx");
    fermat_inv::fermat_inv(b, tx, &inv_dx, p);

    // lam = dy * inv_dx = λ (Horner write-into-zero)
    let lam = b.alloc_qubits(n);
    b.set_phase("compact_lam_mul");
    fermat_inv::horner_mul_add(b, &lam, ty, &inv_dx, p);

    // ty -= lam * tx → ty = dy - λ*dx = 0
    b.set_phase("compact_ty_zero");
    fermat_inv::horner_mul_sub(b, ty, &lam, tx, p);

    // tx = dx - λ²
    b.set_phase("compact_lam_sq");
    fermat_inv::mod_mul_sub_inplace(b, tx, &lam, &lam, p);

    // Affine corrections: tx = -(tx + 3*Qx) = Rx - Qx
    mod_add_qb(b, tx, ox, p); // tx = dx - λ² + Qx
    mod_add_double_qb(b, tx, ox, p); // tx = dx - λ² + 3Qx
    mod_neg_inplace_fast(b, tx, p); // tx = λ² - dx - 3Qx = Rx - Qx

    // ty = lam * tx = λ(Qx - Rx) = Ry + Qy
    b.set_phase("compact_ty_mul");
    fermat_inv::horner_mul_add(b, ty, &lam, tx, p);
    // ty -= Qy → ty = Ry
    mod_sub_qb(b, ty, oy, p);

    // Cleanup: uncompute lam using second Fermat inversion
    // inv_rxqx = (Rx - Qx)^{-1}
    // lam = λ. λ = (Qy + Ry) / (Qx - Rx) = -(Qy + Ry) / (Rx - Qx)
    // So lam = -(Qy + Ry) * inv(Rx-Qx)
    // Currently ty = Ry, tx = Rx - Qx
    // Qy + Ry: we can compute ty + Qy = Ry + Qy
    //
    // Actually: we need to zero lam. Currently:
    //   lam = λ, tx = Rx - Qx, ty = Ry
    //   inv_rxqx = (Rx-Qx)^{-1}
    //   λ * (Rx-Qx) = -(Ry + Qy) [from the EC addition formula]
    //   Wait: λ = (Qy + Ry) / (Qx - Rx) = -(Qy + Ry) / (Rx - Qx)
    //   So: lam * tx = -((Qy + Ry) / (Rx-Qx)) * (Rx-Qx) = -(Qy + Ry)
    //   So: lam = -(Qy + Ry) * (Rx-Qx)^{-1}
    //   lam * (Rx-Qx) + (Qy + Ry) = 0
    //   lam * tx + (ty + Qy) = 0  ... since tx=Rx-Qx, ty=Ry
    //
    // To zero lam: we need lam + (ty + Qy) * inv_rxqx = 0
    // i.e., lam += (ty + Qy) * inv_rxqx
    //
    // Compute ty + Qy first:
    mod_add_qb(b, ty, oy, p); // ty = Ry + Qy

    // inv_rxqx = (Rx-Qx)^{-1} = tx^{-1}
    let inv_rxqx = b.alloc_qubits(n);
    b.set_phase("fermat_inv_rxqx");
    fermat_inv::fermat_inv(b, tx, &inv_rxqx, p);

    // lam += (Ry + Qy) * (Rx-Qx)^{-1} → lam = 0
    b.set_phase("compact_lam_cleanup");
    fermat_inv::horner_mul_add(b, &lam, ty, &inv_rxqx, p);

    // ty = Ry + Qy. Subtract Qy to get Ry.
    mod_sub_qb(b, ty, oy, p); // ty = Ry

    // tx = Rx - Qx. Add Qx to get Rx.
    mod_add_qb(b, tx, ox, p); // tx = Rx

    // Free lam (now zero)
    b.free_vec(&lam);

    // Uncompute inv_dx and inv_rxqx
    // inv_dx = dx^{-1}. We no longer have dx (tx = Rx now).
    // We need emit_inverse to reverse the Fermat inv.
    // This is the same in-place cleanup obstruction as the dx^3 one-inversion
    // path: after overwriting tx/ty, recovering dx or Rx-Qx for inverse cleanup
    // requires the inverse affine add denominator, i.e. a second inversion.
    if std::env::var("COMPACT_POINT_ADD_ALLOW_DIRTY_RESET")
        .ok()
        .as_deref()
        != Some("1")
    {
        panic!(
            "COMPACT_POINT_ADD_BLOCKED: inv_dx and inv_rxqx are nonzero here; \
             set COMPACT_POINT_ADD_ALLOW_DIRTY_RESET=1 only for explicitly \
             dirty resource probes. {ONE_INV_DX3_AFFINE_PA_BLOCKER}"
        );
    }
    b.free_vec(&inv_dx);
    b.free_vec(&inv_rxqx);
}

pub(crate) fn build_compact_point_add_clean_early_inv(
    b: &mut B,
    tx: &[QubitId],
    ty: &[QubitId],
    ox: &[BitId],
    oy: &[BitId],
    p: U256,
) {
    assert_eq!(
        p, SECP256K1_P,
        "compact clean early-inverse route is secp256k1-only"
    );
    assert_eq!(tx.len(), N);
    assert_eq!(ty.len(), N);
    assert_eq!(ox.len(), N);
    assert_eq!(oy.len(), N);

    let n = tx.len();

    // Entry: tx=dx=P.x-Q.x, ty=dy=P.y-Q.y.
    let inv_dx = b.alloc_qubits(n);
    b.set_phase("compact_clean_fermat_inv_dx");
    fermat_inv::fermat_inv_clean_lowq(b, tx, &inv_dx, p);

    let lam = b.alloc_qubits(n);
    b.set_phase("compact_clean_lam_mul");
    fermat_inv::horner_mul_add_clean_lowq(b, &lam, ty, &inv_dx, p);

    b.set_phase("compact_clean_dy_zero");
    fermat_inv::horner_mul_sub_clean_lowq(b, ty, &lam, tx, p);

    b.set_phase("compact_clean_fermat_inv_dx_uncompute");
    emit_inverse(b, |b| {
        fermat_inv::fermat_inv_clean_lowq_with_tmp(b, tx, &inv_dx, ty, p)
    });
    b.free_vec(&inv_dx);

    b.set_phase("compact_clean_rx_minus_qx");
    fermat_inv::mod_mul_sub_inplace_clean_lowq(b, tx, &lam, &lam, p);
    mod_add_qb_phase_clean(b, tx, ox, p);
    mod_add_double_qb_phase_clean(b, tx, ox, p);
    mod_neg_inplace(b, tx, p);

    let inv_rxqx = b.alloc_qubits(n);
    b.set_phase("compact_clean_fermat_inv_rxqx");
    fermat_inv::fermat_inv_clean_lowq_with_tmp(b, tx, &inv_rxqx, ty, p);

    // tx=Rx-Qx. Since Ry+Qy = -lambda*(Rx-Qx), subtract the product.
    b.set_phase("compact_clean_ry_plus_qy");
    fermat_inv::horner_mul_sub_clean_lowq(b, ty, &lam, tx, p);

    b.set_phase("compact_clean_lam_cleanup");
    fermat_inv::horner_mul_add_clean_lowq(b, &lam, ty, &inv_rxqx, p);

    b.set_phase("compact_clean_fermat_inv_rxqx_uncompute");
    emit_inverse(b, |b| {
        fermat_inv::fermat_inv_clean_lowq_with_tmp(b, tx, &inv_rxqx, &lam, p)
    });
    b.free_vec(&inv_rxqx);

    mod_sub_qb_phase_clean(b, ty, oy, p);
    mod_add_qb_phase_clean(b, tx, ox, p);
    b.free_vec(&lam);
}
