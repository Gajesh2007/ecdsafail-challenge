//! `frontier::misc1` — verbatim split of the original `frontier` module.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

pub(crate) fn round631_emit_eq_const_toggle(
    b: &mut B,
    frontier: &[QubitId],
    value: usize,
    target: QubitId,
    scratch: &[QubitId],
) {
    let n = frontier.len();
    assert!(n >= 2);
    assert!(value < (1usize << n));
    assert!(scratch.len() >= n - 2);

    for i in 0..n {
        if ((value >> i) & 1) == 0 {
            b.x(frontier[i]);
        }
    }

    if n == 2 {
        b.ccx(frontier[0], frontier[1], target);
    } else {
        b.ccx(frontier[0], frontier[1], scratch[0]);
        for i in 2..n - 1 {
            b.ccx(scratch[i - 2], frontier[i], scratch[i - 1]);
        }
        b.ccx(scratch[n - 3], frontier[n - 1], target);
        for i in (2..n - 1).rev() {
            b.ccx(scratch[i - 2], frontier[i], scratch[i - 1]);
        }
        b.ccx(frontier[0], frontier[1], scratch[0]);
    }

    for i in (0..n).rev() {
        if ((value >> i) & 1) == 0 {
            b.x(frontier[i]);
        }
    }
}

pub(crate) fn round640_emit_eq_const_except_toggle(
    b: &mut B,
    bits: &[QubitId],
    skip: usize,
    value: usize,
    target: QubitId,
    scratch: &[QubitId],
) {
    assert!(skip < bits.len());
    let mut selected = Vec::with_capacity(bits.len() - 1);
    let mut compressed = 0usize;
    for (idx, bit) in bits.iter().enumerate() {
        if idx == skip {
            continue;
        }
        if ((value >> idx) & 1) != 0 {
            compressed |= 1usize << selected.len();
        }
        selected.push(*bit);
    }
    round631_emit_eq_const_toggle(b, &selected, compressed, target, scratch);
}

pub(crate) fn round631_emit_low_active_stream(
    b: &mut B,
    frontier: &[QubitId],
    active: QubitId,
    scratch: &[QubitId],
    width: usize,
) {
    b.x(active);
    round631_emit_eq_const_toggle(b, frontier, 0, active, scratch);
    for value in 1..=width {
        round631_emit_eq_const_toggle(b, frontier, value, active, scratch);
    }
}

pub(crate) fn round631_emit_high_active_stream(
    b: &mut B,
    frontier: &[QubitId],
    active: QubitId,
    scratch: &[QubitId],
    width: usize,
) {
    b.x(active);
    round631_emit_eq_const_toggle(b, frontier, width, active, scratch);
    for value in (0..width).rev() {
        round631_emit_eq_const_toggle(b, frontier, value, active, scratch);
    }
}

pub(crate) fn round631_emit_boundary_stream(
    b: &mut B,
    frontier: &[QubitId],
    boundary: QubitId,
    scratch: &[QubitId],
    values: impl Iterator<Item = usize>,
) {
    for value in values {
        round631_emit_eq_const_toggle(b, frontier, value, boundary, scratch);
        round631_emit_eq_const_toggle(b, frontier, value, boundary, scratch);
    }
}

pub(crate) fn round632_compute_body_ctrl(b: &mut B, step_ctrl: QubitId, local: QubitId, body_ctrl: QubitId) {
    b.ccx(step_ctrl, local, body_ctrl);
}

pub(crate) fn round632_emit_low_sub_body(
    b: &mut B,
    src: &[QubitId],
    dst: &[QubitId],
    frontier: &[QubitId],
    step_ctrl: QubitId,
    active: QubitId,
    boundary: QubitId,
    body_ctrl: QubitId,
    c_in: QubitId,
    body_scratch: QubitId,
    eq_scratch: &[QubitId],
) {
    const PACKED_WIDTH: usize = N + 1;
    debug_assert_eq!(src.len(), PACKED_WIDTH);
    debug_assert_eq!(dst.len(), PACKED_WIDTH);

    b.set_phase("round632_low_sub_forward_interior");
    round632_low_interior_forward_init(b, frontier, active, eq_scratch);
    for bit in 0..PACKED_WIDTH - 1 {
        let carry = if bit == 0 { c_in } else { src[bit - 1] };
        round632_compute_body_ctrl(b, step_ctrl, active, body_ctrl);
        ctrl_inv_uma(b, body_ctrl, carry, dst[bit], src[bit], body_scratch);
        round632_uncompute_body_ctrl(b, step_ctrl, active, body_ctrl);
        round632_low_interior_forward_next(b, frontier, active, eq_scratch, bit);
    }

    b.set_phase("round632_low_sub_boundary_sum");
    for bit in 0..PACKED_WIDTH {
        round631_emit_eq_const_toggle(b, frontier, bit + 1, boundary, eq_scratch);
        let carry = if bit == 0 { c_in } else { src[bit - 1] };
        round632_compute_body_ctrl(b, step_ctrl, boundary, body_ctrl);
        b.ccx(body_ctrl, src[bit], dst[bit]);
        b.ccx(body_ctrl, carry, dst[bit]);
        round632_uncompute_body_ctrl(b, step_ctrl, boundary, body_ctrl);
        round631_emit_eq_const_toggle(b, frontier, bit + 1, boundary, eq_scratch);
    }

    b.set_phase("round632_low_sub_backward_interior");
    round632_low_interior_backward_init(b, frontier, active, eq_scratch, PACKED_WIDTH);
    for bit in (0..PACKED_WIDTH - 1).rev() {
        let carry = if bit == 0 { c_in } else { src[bit - 1] };
        round632_compute_body_ctrl(b, step_ctrl, active, body_ctrl);
        ctrl_inv_maj(b, body_ctrl, carry, dst[bit], src[bit], body_scratch);
        round632_uncompute_body_ctrl(b, step_ctrl, active, body_ctrl);
        round632_low_interior_backward_next(b, frontier, active, eq_scratch, bit);
    }
    round632_low_interior_backward_clean(b, frontier, active, eq_scratch);
}

pub(crate) fn round658_emit_low_sub_body_with_borrow(
    b: &mut B,
    src: &[QubitId],
    dst: &[QubitId],
    frontier: &[QubitId],
    step_ctrl: QubitId,
    borrow_flag: QubitId,
    active: QubitId,
    boundary: QubitId,
    body_ctrl: QubitId,
    c_in: QubitId,
    body_scratch: QubitId,
    eq_scratch: &[QubitId],
) {
    const PACKED_WIDTH: usize = N + 1;
    debug_assert_eq!(src.len(), PACKED_WIDTH);
    debug_assert_eq!(dst.len(), PACKED_WIDTH);

    b.set_phase("round658_low_sub_borrow_forward_interior");
    round632_low_interior_forward_init(b, frontier, active, eq_scratch);
    for bit in 0..PACKED_WIDTH - 1 {
        let carry = if bit == 0 { c_in } else { src[bit - 1] };
        round632_compute_body_ctrl(b, step_ctrl, active, body_ctrl);
        ctrl_inv_uma(b, body_ctrl, carry, dst[bit], src[bit], body_scratch);
        round632_uncompute_body_ctrl(b, step_ctrl, active, body_ctrl);
        round632_low_interior_forward_next(b, frontier, active, eq_scratch, bit);
    }

    b.set_phase("round658_low_sub_boundary_borrow_and_sum");
    for bit in 0..PACKED_WIDTH {
        round631_emit_eq_const_toggle(b, frontier, bit + 1, boundary, eq_scratch);
        let carry = if bit == 0 { c_in } else { src[bit - 1] };
        round632_compute_body_ctrl(b, step_ctrl, boundary, body_ctrl);
        round658_latch_borrow_majority(
            b,
            body_ctrl,
            carry,
            dst[bit],
            src[bit],
            borrow_flag,
            body_scratch,
        );
        b.ccx(body_ctrl, src[bit], dst[bit]);
        b.ccx(body_ctrl, carry, dst[bit]);
        round632_uncompute_body_ctrl(b, step_ctrl, boundary, body_ctrl);
        round631_emit_eq_const_toggle(b, frontier, bit + 1, boundary, eq_scratch);
    }

    b.set_phase("round658_low_sub_borrow_backward_interior");
    round632_low_interior_backward_init(b, frontier, active, eq_scratch, PACKED_WIDTH);
    for bit in (0..PACKED_WIDTH - 1).rev() {
        let carry = if bit == 0 { c_in } else { src[bit - 1] };
        round632_compute_body_ctrl(b, step_ctrl, active, body_ctrl);
        ctrl_inv_maj(b, body_ctrl, carry, dst[bit], src[bit], body_scratch);
        round632_uncompute_body_ctrl(b, step_ctrl, active, body_ctrl);
        round632_low_interior_backward_next(b, frontier, active, eq_scratch, bit);
    }
    round632_low_interior_backward_clean(b, frontier, active, eq_scratch);
}

pub(crate) fn round659_emit_source_over_frontier_pairs(
    b: &mut B,
    source_lane: &[QubitId],
    source_start: &[QubitId],
    dst_start: &[QubitId],
    step_ctrl: QubitId,
    overflow_flag: QubitId,
    eq_a: QubitId,
    eq_b: QubitId,
    pair_ctrl: QubitId,
    body_scratch: QubitId,
    eq_scratch: &[QubitId],
    or_chain: &[QubitId],
) {
    const PACKED_WIDTH: usize = N + 1;
    const MAX_DELTA: usize = 19;
    debug_assert_eq!(source_lane.len(), PACKED_WIDTH);
    debug_assert!(or_chain.len() >= MAX_DELTA - 1);

    for delta in 1..=MAX_DELTA {
        for lower in 0..=PACKED_WIDTH - delta {
            round653_compute_pair_ctrl(
                b,
                source_start,
                dst_start,
                lower + delta,
                lower,
                step_ctrl,
                eq_a,
                eq_b,
                pair_ctrl,
                body_scratch,
                eq_scratch,
            );
            round653_ctrl_range_or_into(
                b,
                pair_ctrl,
                source_lane,
                lower,
                delta,
                overflow_flag,
                or_chain,
                body_scratch,
            );
            round653_uncompute_pair_ctrl(
                b,
                source_start,
                dst_start,
                lower + delta,
                lower,
                step_ctrl,
                eq_a,
                eq_b,
                pair_ctrl,
                body_scratch,
                eq_scratch,
            );
        }
    }
}

pub(crate) fn round660_emit_low_add_carry_retap(
    b: &mut B,
    src: &[QubitId],
    dst_after_sub: &[QubitId],
    frontier: &[QubitId],
    step_ctrl: QubitId,
    carry_flag: QubitId,
    active: QubitId,
    boundary: QubitId,
    body_ctrl: QubitId,
    c_in: QubitId,
    body_scratch: QubitId,
    eq_scratch: &[QubitId],
) {
    const PACKED_WIDTH: usize = N + 1;
    debug_assert_eq!(src.len(), PACKED_WIDTH);
    debug_assert_eq!(dst_after_sub.len(), PACKED_WIDTH);

    b.set_phase("round660_low_add_carry_retap_forward_interior");
    round632_low_interior_forward_init(b, frontier, active, eq_scratch);
    for bit in 0..PACKED_WIDTH - 1 {
        let carry = if bit == 0 { c_in } else { src[bit - 1] };
        round632_compute_body_ctrl(b, step_ctrl, active, body_ctrl);
        ctrl_maj(
            b,
            body_ctrl,
            carry,
            dst_after_sub[bit],
            src[bit],
            body_scratch,
        );
        round632_uncompute_body_ctrl(b, step_ctrl, active, body_ctrl);
        round632_low_interior_forward_next(b, frontier, active, eq_scratch, bit);
    }

    b.set_phase("round660_low_add_carry_retap_boundary");
    for bit in 0..PACKED_WIDTH {
        round631_emit_eq_const_toggle(b, frontier, bit + 1, boundary, eq_scratch);
        let carry = if bit == 0 { c_in } else { src[bit - 1] };
        round632_compute_body_ctrl(b, step_ctrl, boundary, body_ctrl);
        ctrl_maj(
            b,
            body_ctrl,
            carry,
            dst_after_sub[bit],
            src[bit],
            body_scratch,
        );
        b.ccx(body_ctrl, src[bit], carry_flag);
        ctrl_inv_maj(
            b,
            body_ctrl,
            carry,
            dst_after_sub[bit],
            src[bit],
            body_scratch,
        );
        round632_uncompute_body_ctrl(b, step_ctrl, boundary, body_ctrl);
        round631_emit_eq_const_toggle(b, frontier, bit + 1, boundary, eq_scratch);
    }

    b.set_phase("round660_low_add_carry_retap_backward_interior");
    round632_low_interior_backward_init(b, frontier, active, eq_scratch, PACKED_WIDTH);
    for bit in (0..PACKED_WIDTH - 1).rev() {
        let carry = if bit == 0 { c_in } else { src[bit - 1] };
        round632_compute_body_ctrl(b, step_ctrl, active, body_ctrl);
        ctrl_inv_maj(
            b,
            body_ctrl,
            carry,
            dst_after_sub[bit],
            src[bit],
            body_scratch,
        );
        round632_uncompute_body_ctrl(b, step_ctrl, active, body_ctrl);
        round632_low_interior_backward_next(b, frontier, active, eq_scratch, bit);
    }
    round632_low_interior_backward_clean(b, frontier, active, eq_scratch);
}

pub(crate) fn round662_emit_low_add_body(
    b: &mut B,
    src: &[QubitId],
    dst: &[QubitId],
    frontier: &[QubitId],
    step_ctrl: QubitId,
    active: QubitId,
    boundary: QubitId,
    body_ctrl: QubitId,
    c_in: QubitId,
    body_scratch: QubitId,
    eq_scratch: &[QubitId],
) {
    const PACKED_WIDTH: usize = N + 1;
    debug_assert_eq!(src.len(), PACKED_WIDTH);
    debug_assert_eq!(dst.len(), PACKED_WIDTH);

    b.set_phase("round662_low_add_forward_interior");
    round632_low_interior_forward_init(b, frontier, active, eq_scratch);
    for bit in 0..PACKED_WIDTH - 1 {
        let carry = if bit == 0 { c_in } else { src[bit - 1] };
        round632_compute_body_ctrl(b, step_ctrl, active, body_ctrl);
        ctrl_maj(b, body_ctrl, carry, dst[bit], src[bit], body_scratch);
        round632_uncompute_body_ctrl(b, step_ctrl, active, body_ctrl);
        round632_low_interior_forward_next(b, frontier, active, eq_scratch, bit);
    }

    b.set_phase("round662_low_add_boundary_sum");
    for bit in 0..PACKED_WIDTH {
        round631_emit_eq_const_toggle(b, frontier, bit + 1, boundary, eq_scratch);
        let carry = if bit == 0 { c_in } else { src[bit - 1] };
        round632_compute_body_ctrl(b, step_ctrl, boundary, body_ctrl);
        b.ccx(body_ctrl, carry, dst[bit]);
        b.ccx(body_ctrl, src[bit], dst[bit]);
        round632_uncompute_body_ctrl(b, step_ctrl, boundary, body_ctrl);
        round631_emit_eq_const_toggle(b, frontier, bit + 1, boundary, eq_scratch);
    }

    b.set_phase("round662_low_add_backward_interior");
    round632_low_interior_backward_init(b, frontier, active, eq_scratch, PACKED_WIDTH);
    for bit in (0..PACKED_WIDTH - 1).rev() {
        let carry = if bit == 0 { c_in } else { src[bit - 1] };
        round632_compute_body_ctrl(b, step_ctrl, active, body_ctrl);
        ctrl_uma(b, body_ctrl, carry, dst[bit], src[bit], body_scratch);
        round632_uncompute_body_ctrl(b, step_ctrl, active, body_ctrl);
        round632_low_interior_backward_next(b, frontier, active, eq_scratch, bit);
    }
    round632_low_interior_backward_clean(b, frontier, active, eq_scratch);
}

pub(crate) fn round663_emit_borrow_start_delta_le10(
    b: &mut B,
    start_a: &[QubitId],
    start_b: &[QubitId],
    borrow_ctrl: QubitId,
    delta_bits: &[QubitId],
    eq_a: QubitId,
    eq_b: QubitId,
    pair_ctrl: QubitId,
    body_scratch: QubitId,
    eq_scratch: &[QubitId],
) {
    const FRONTIER_VALUES: usize = N + 2;
    const MAX_DELTA: usize = 10;
    debug_assert_eq!(start_a.len(), 9);
    debug_assert_eq!(start_b.len(), 9);
    debug_assert_eq!(delta_bits.len(), 4);

    b.set_phase("round663_borrow_start_delta_le10");
    for delta in 1..=MAX_DELTA {
        for lower in 0..=FRONTIER_VALUES - 1 - delta {
            round653_compute_pair_ctrl(
                b,
                start_a,
                start_b,
                lower,
                lower + delta,
                borrow_ctrl,
                eq_a,
                eq_b,
                pair_ctrl,
                body_scratch,
                eq_scratch,
            );
            for (bit, delta_bit) in delta_bits.iter().enumerate() {
                if ((delta >> bit) & 1) == 1 {
                    b.cx(pair_ctrl, *delta_bit);
                }
            }
            round653_uncompute_pair_ctrl(
                b,
                start_a,
                start_b,
                lower,
                lower + delta,
                borrow_ctrl,
                eq_a,
                eq_b,
                pair_ctrl,
                body_scratch,
                eq_scratch,
            );
        }
    }
}

pub(crate) fn round664_emit_delta_growth_frontier_update(
    b: &mut B,
    start_a: &[QubitId],
    start_b: &[QubitId],
    delta_bits: &[QubitId],
    growth_ctrl: QubitId,
    swap_ctrl: QubitId,
) {
    debug_assert_eq!(start_a.len(), 9);
    debug_assert_eq!(start_b.len(), 9);
    debug_assert_eq!(delta_bits.len(), 4);

    b.set_phase("round664_apply_delta_to_start_b");
    for (bit, delta_bit) in delta_bits.iter().enumerate() {
        csub_nbit_const_direct_fast(b, start_b, U256::from(1u64 << bit), *delta_bit);
    }

    b.set_phase("round664_apply_growth_decrement");
    csub_nbit_const_direct_fast(b, start_b, U256::from(1u64), growth_ctrl);

    b.set_phase("round664_swap_refreshed_frontiers");
    for i in 0..start_a.len() {
        cswap(b, swap_ctrl, start_a[i], start_b[i]);
    }
}

pub(crate) fn round665_emit_high_add_body_with_growth_tap(
    b: &mut B,
    src: &[QubitId],
    dst: &[QubitId],
    frontier: &[QubitId],
    step_ctrl: QubitId,
    growth_flag: QubitId,
    active: QubitId,
    boundary: QubitId,
    body_ctrl: QubitId,
    c_in: QubitId,
    body_scratch: QubitId,
    eq_scratch: &[QubitId],
) {
    const PACKED_WIDTH: usize = N + 1;
    debug_assert_eq!(src.len(), PACKED_WIDTH);
    debug_assert_eq!(dst.len(), PACKED_WIDTH);

    b.set_phase("round665_high_add_growth_forward_interior");
    round632_high_interior_forward_init(b, frontier, active, eq_scratch, PACKED_WIDTH);
    for bit in (1..PACKED_WIDTH).rev() {
        let carry = if bit + 1 == PACKED_WIDTH {
            c_in
        } else {
            src[bit + 1]
        };
        round632_compute_body_ctrl(b, step_ctrl, active, body_ctrl);
        ctrl_maj(b, body_ctrl, carry, dst[bit], src[bit], body_scratch);
        round632_uncompute_body_ctrl(b, step_ctrl, active, body_ctrl);
        round632_high_interior_forward_next(b, frontier, active, eq_scratch, bit);
    }
    round632_high_interior_forward_clean(b, frontier, active, eq_scratch);

    b.set_phase("round665_high_add_growth_boundary_sum");
    for bit in (0..PACKED_WIDTH).rev() {
        round631_emit_eq_const_toggle(b, frontier, bit, boundary, eq_scratch);
        let carry = if bit + 1 == PACKED_WIDTH {
            c_in
        } else {
            src[bit + 1]
        };
        round632_compute_body_ctrl(b, step_ctrl, boundary, body_ctrl);
        round665_latch_add_carry_majority(
            b,
            body_ctrl,
            carry,
            dst[bit],
            src[bit],
            growth_flag,
            body_scratch,
        );
        b.ccx(body_ctrl, carry, dst[bit]);
        b.ccx(body_ctrl, src[bit], dst[bit]);
        round632_uncompute_body_ctrl(b, step_ctrl, boundary, body_ctrl);
        round631_emit_eq_const_toggle(b, frontier, bit, boundary, eq_scratch);
    }

    b.set_phase("round665_high_add_growth_backward_interior");
    round632_high_interior_backward_init(b, frontier, active, eq_scratch);
    for bit in 1..PACKED_WIDTH {
        let carry = if bit + 1 == PACKED_WIDTH {
            c_in
        } else {
            src[bit + 1]
        };
        round632_compute_body_ctrl(b, step_ctrl, active, body_ctrl);
        ctrl_uma(b, body_ctrl, carry, dst[bit], src[bit], body_scratch);
        round632_uncompute_body_ctrl(b, step_ctrl, active, body_ctrl);
        round632_high_interior_backward_next(b, frontier, active, eq_scratch, bit);
    }
    round632_high_interior_backward_clean(b, frontier, active, eq_scratch, PACKED_WIDTH);
}

pub(crate) fn round667_compute_body_ctrl_borrow_polar(
    b: &mut B,
    step_ctrl: QubitId,
    local: QubitId,
    borrow_ctrl: QubitId,
    borrow_polarity: bool,
    body_ctrl: QubitId,
    scratch: QubitId,
) {
    mcx3_polar(
        b,
        step_ctrl,
        true,
        local,
        true,
        borrow_ctrl,
        borrow_polarity,
        body_ctrl,
        scratch,
    );
}

pub(crate) fn round667_emit_high_add_body_with_growth_tap_borrow_polar(
    b: &mut B,
    src: &[QubitId],
    dst: &[QubitId],
    frontier: &[QubitId],
    step_ctrl: QubitId,
    borrow_ctrl: QubitId,
    borrow_polarity: bool,
    growth_flag: QubitId,
    active: QubitId,
    boundary: QubitId,
    body_ctrl: QubitId,
    c_in: QubitId,
    body_scratch: QubitId,
    eq_scratch: &[QubitId],
) {
    const PACKED_WIDTH: usize = N + 1;
    debug_assert_eq!(src.len(), PACKED_WIDTH);
    debug_assert_eq!(dst.len(), PACKED_WIDTH);

    b.set_phase("round667_nonborrow_high_add_growth_forward_interior");
    round632_high_interior_forward_init(b, frontier, active, eq_scratch, PACKED_WIDTH);
    for bit in (1..PACKED_WIDTH).rev() {
        let carry = if bit + 1 == PACKED_WIDTH {
            c_in
        } else {
            src[bit + 1]
        };
        round667_compute_body_ctrl_borrow_polar(
            b,
            step_ctrl,
            active,
            borrow_ctrl,
            borrow_polarity,
            body_ctrl,
            body_scratch,
        );
        ctrl_maj(b, body_ctrl, carry, dst[bit], src[bit], body_scratch);
        round667_compute_body_ctrl_borrow_polar(
            b,
            step_ctrl,
            active,
            borrow_ctrl,
            borrow_polarity,
            body_ctrl,
            body_scratch,
        );
        round632_high_interior_forward_next(b, frontier, active, eq_scratch, bit);
    }
    round632_high_interior_forward_clean(b, frontier, active, eq_scratch);

    b.set_phase("round667_nonborrow_high_add_growth_boundary_sum");
    for bit in (0..PACKED_WIDTH).rev() {
        round631_emit_eq_const_toggle(b, frontier, bit, boundary, eq_scratch);
        let carry = if bit + 1 == PACKED_WIDTH {
            c_in
        } else {
            src[bit + 1]
        };
        round667_compute_body_ctrl_borrow_polar(
            b,
            step_ctrl,
            boundary,
            borrow_ctrl,
            borrow_polarity,
            body_ctrl,
            body_scratch,
        );
        round665_latch_add_carry_majority(
            b,
            body_ctrl,
            carry,
            dst[bit],
            src[bit],
            growth_flag,
            body_scratch,
        );
        b.ccx(body_ctrl, carry, dst[bit]);
        b.ccx(body_ctrl, src[bit], dst[bit]);
        round667_compute_body_ctrl_borrow_polar(
            b,
            step_ctrl,
            boundary,
            borrow_ctrl,
            borrow_polarity,
            body_ctrl,
            body_scratch,
        );
        round631_emit_eq_const_toggle(b, frontier, bit, boundary, eq_scratch);
    }

    b.set_phase("round667_nonborrow_high_add_growth_backward_interior");
    round632_high_interior_backward_init(b, frontier, active, eq_scratch);
    for bit in 1..PACKED_WIDTH {
        let carry = if bit + 1 == PACKED_WIDTH {
            c_in
        } else {
            src[bit + 1]
        };
        round667_compute_body_ctrl_borrow_polar(
            b,
            step_ctrl,
            active,
            borrow_ctrl,
            borrow_polarity,
            body_ctrl,
            body_scratch,
        );
        ctrl_uma(b, body_ctrl, carry, dst[bit], src[bit], body_scratch);
        round667_compute_body_ctrl_borrow_polar(
            b,
            step_ctrl,
            active,
            borrow_ctrl,
            borrow_polarity,
            body_ctrl,
            body_scratch,
        );
        round632_high_interior_backward_next(b, frontier, active, eq_scratch, bit);
    }
    round632_high_interior_backward_clean(b, frontier, active, eq_scratch, PACKED_WIDTH);
}
