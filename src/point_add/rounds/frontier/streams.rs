//! `frontier::streams` — verbatim split of the original `frontier` module.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

pub(crate) fn round632_low_interior_forward_init(
    b: &mut B,
    frontier: &[QubitId],
    active: QubitId,
    eq_scratch: &[QubitId],
) {
    const PACKED_WIDTH: usize = N + 1;
    b.x(active);
    round631_emit_eq_const_toggle(b, frontier, 0, active, eq_scratch);
    round631_emit_eq_const_toggle(b, frontier, 1, active, eq_scratch);
    let _ = PACKED_WIDTH;
}

pub(crate) fn round632_low_interior_forward_next(
    b: &mut B,
    frontier: &[QubitId],
    active: QubitId,
    eq_scratch: &[QubitId],
    bit: usize,
) {
    round631_emit_eq_const_toggle(b, frontier, bit + 2, active, eq_scratch);
}

pub(crate) fn round632_low_interior_backward_init(
    b: &mut B,
    frontier: &[QubitId],
    active: QubitId,
    eq_scratch: &[QubitId],
    width: usize,
) {
    round631_emit_eq_const_toggle(b, frontier, width, active, eq_scratch);
}

pub(crate) fn round632_low_interior_backward_next(
    b: &mut B,
    frontier: &[QubitId],
    active: QubitId,
    eq_scratch: &[QubitId],
    bit: usize,
) {
    if bit > 0 {
        round631_emit_eq_const_toggle(b, frontier, bit + 1, active, eq_scratch);
    }
}

pub(crate) fn round632_low_interior_backward_clean(
    b: &mut B,
    frontier: &[QubitId],
    active: QubitId,
    eq_scratch: &[QubitId],
) {
    round631_emit_eq_const_toggle(b, frontier, 1, active, eq_scratch);
    round631_emit_eq_const_toggle(b, frontier, 0, active, eq_scratch);
    b.x(active);
}

pub(crate) fn round632_high_interior_forward_init(
    b: &mut B,
    frontier: &[QubitId],
    active: QubitId,
    eq_scratch: &[QubitId],
    width: usize,
) {
    b.x(active);
    round631_emit_eq_const_toggle(b, frontier, width - 1, active, eq_scratch);
    round631_emit_eq_const_toggle(b, frontier, width, active, eq_scratch);
}

pub(crate) fn round632_high_interior_forward_next(
    b: &mut B,
    frontier: &[QubitId],
    active: QubitId,
    eq_scratch: &[QubitId],
    bit: usize,
) {
    if bit > 1 {
        round631_emit_eq_const_toggle(b, frontier, bit - 1, active, eq_scratch);
    }
}

pub(crate) fn round632_high_interior_forward_clean(
    b: &mut B,
    frontier: &[QubitId],
    active: QubitId,
    eq_scratch: &[QubitId],
) {
    round631_emit_eq_const_toggle(b, frontier, 0, active, eq_scratch);
}

pub(crate) fn round632_high_interior_backward_init(
    b: &mut B,
    frontier: &[QubitId],
    active: QubitId,
    eq_scratch: &[QubitId],
) {
    round631_emit_eq_const_toggle(b, frontier, 0, active, eq_scratch);
}

pub(crate) fn round632_high_interior_backward_next(
    b: &mut B,
    frontier: &[QubitId],
    active: QubitId,
    eq_scratch: &[QubitId],
    bit: usize,
) {
    if bit + 1 < N + 1 {
        round631_emit_eq_const_toggle(b, frontier, bit, active, eq_scratch);
    }
}

pub(crate) fn round632_high_interior_backward_clean(
    b: &mut B,
    frontier: &[QubitId],
    active: QubitId,
    eq_scratch: &[QubitId],
    width: usize,
) {
    b.x(active);
    round631_emit_eq_const_toggle(b, frontier, width - 1, active, eq_scratch);
    round631_emit_eq_const_toggle(b, frontier, width, active, eq_scratch);
}

pub(crate) fn round651_low_prefix_forward_init(
    b: &mut B,
    frontier: &[QubitId],
    active: QubitId,
    eq_scratch: &[QubitId],
) {
    b.x(active);
    round631_emit_eq_const_toggle(b, frontier, 0, active, eq_scratch);
}

pub(crate) fn round651_low_prefix_forward_next(
    b: &mut B,
    frontier: &[QubitId],
    active: QubitId,
    eq_scratch: &[QubitId],
    bit: usize,
) {
    round631_emit_eq_const_toggle(b, frontier, bit + 1, active, eq_scratch);
}

pub(crate) fn round651_low_prefix_backward_init(
    b: &mut B,
    frontier: &[QubitId],
    active: QubitId,
    eq_scratch: &[QubitId],
    width: usize,
) {
    round631_emit_eq_const_toggle(b, frontier, width, active, eq_scratch);
}

pub(crate) fn round651_low_prefix_backward_next(
    b: &mut B,
    frontier: &[QubitId],
    active: QubitId,
    eq_scratch: &[QubitId],
    bit: usize,
) {
    if bit > 0 {
        round631_emit_eq_const_toggle(b, frontier, bit, active, eq_scratch);
    }
}

pub(crate) fn round651_low_prefix_backward_clean(
    b: &mut B,
    frontier: &[QubitId],
    active: QubitId,
    eq_scratch: &[QubitId],
) {
    b.x(active);
    round631_emit_eq_const_toggle(b, frontier, 0, active, eq_scratch);
}
