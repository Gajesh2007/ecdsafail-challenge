//! `frontier::latch` — verbatim split of the original `frontier` module.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

pub(crate) fn round658_latch_borrow_majority(
    b: &mut B,
    body_ctrl: QubitId,
    carry: QubitId,
    dst_bit: QubitId,
    src_bit: QubitId,
    borrow_flag: QubitId,
    scratch: QubitId,
) {
    mcx3_polar(
        b,
        body_ctrl,
        true,
        src_bit,
        true,
        carry,
        true,
        borrow_flag,
        scratch,
    );
    mcx3_polar(
        b,
        body_ctrl,
        true,
        dst_bit,
        false,
        carry,
        true,
        borrow_flag,
        scratch,
    );
    mcx3_polar(
        b,
        body_ctrl,
        true,
        dst_bit,
        false,
        src_bit,
        true,
        borrow_flag,
        scratch,
    );
}

pub(crate) fn round665_latch_add_carry_majority(
    b: &mut B,
    body_ctrl: QubitId,
    carry: QubitId,
    dst_bit: QubitId,
    src_bit: QubitId,
    carry_flag: QubitId,
    scratch: QubitId,
) {
    mcx3_polar(
        b, body_ctrl, true, carry, true, dst_bit, true, carry_flag, scratch,
    );
    mcx3_polar(
        b, body_ctrl, true, carry, true, src_bit, true, carry_flag, scratch,
    );
    mcx3_polar(
        b, body_ctrl, true, dst_bit, true, src_bit, true, carry_flag, scratch,
    );
}
