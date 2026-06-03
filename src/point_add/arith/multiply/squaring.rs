//! `multiply::squaring` — verbatim split of the original `multiply` module.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

pub(crate) fn square_tx_and_combined_ty_l2minus3qx(
    b: &mut B,
    tx: &[QubitId],
    ty: &[QubitId],
    lam: &[QubitId],
    ox: &[BitId],
    p: U256,
) {
    let n = tx.len();
    debug_assert_eq!(n, 256);
    debug_assert_eq!(ty.len(), n);
    debug_assert_eq!(lam.len(), n);

    b.set_phase("affine_combined_square");
    let tmp_ext = b.alloc_qubits(2 * n);
    schoolbook_square_symmetric_lowq(b, lam, &tmp_ext);

    b.set_phase("affine_combined_breg_red");
    let breg = b.alloc_qubits(n);
    mod_add_solinas_ext_product(b, &breg, &tmp_ext, p);
    mod_sub_double_qb(b, &breg, ox, p);
    mod_sub_qb(b, &breg, ox, p);

    b.set_phase("affine_combined_y_mul");
    if env_flag_enabled("POINT_ADD_AFFINE_COMBINED_Y_KARATSUBA_LOWQ", false) {
        mod_mul_add_into_acc_karatsuba_lowq(b, ty, lam, &breg, p);
    } else {
        mod_mul_add_into_acc_selected(b, ty, lam, &breg, p, "POINT_ADD_AFFINE_COMBINED_Y_MUL");
    }

    b.set_phase("affine_combined_breg_unred");
    mod_add_qb(b, &breg, ox, p);
    mod_add_double_qb(b, &breg, ox, p);
    mod_sub_solinas_ext_product(b, &breg, &tmp_ext, p);
    b.free_vec(&breg);

    b.set_phase("affine_combined_tx_update");
    mod_sub_solinas_ext_product(b, tx, &tmp_ext, p);
    mod_add_double_qb(b, tx, ox, p);
    mod_add_qb(b, tx, ox, p);
    mod_neg_inplace_fast(b, tx, p);

    schoolbook_square_symmetric_lowq_inverse(b, lam, &tmp_ext);
    b.free_vec(&tmp_ext);
}

pub(crate) fn squaring_sub_from_acc_walk_controls_lowq(b: &mut B, acc: &[QubitId], x: &[QubitId], p: U256) {
    let n = acc.len();
    debug_assert_eq!(n, 256);
    debug_assert_eq!(x.len(), n);

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
}
