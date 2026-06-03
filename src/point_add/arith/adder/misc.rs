//! `adder::misc` — verbatim split of the original `adder` module.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;


pub(crate) fn centered_restoring_trial_subtract_clean(
    b: &mut B,
    u: &[QubitId],
    v: &[QubitId],
    q_success: QubitId,
) {
    // Trial subtract for a centered-Euclid quotient bit. Compute the borrow,
    // copy out the success bit, then undo with the arithmetic inverse instead
    // of replaying the Cuccaro subtract wrapper through emit_inverse.
    assert_eq!(u.len(), v.len());
    let top_u = b.alloc_qubit();
    let top_v = b.alloc_qubit();
    let mut u_ext = u.to_vec();
    u_ext.push(top_u);
    let mut v_ext = v.to_vec();
    v_ext.push(top_v);
    sub_nbit_qq(b, &v_ext, &u_ext);
    b.cx(top_u, q_success);
    b.x(q_success);
    add_nbit_qq(b, &v_ext, &u_ext);
    b.free(top_v);
    b.free(top_u);
}
