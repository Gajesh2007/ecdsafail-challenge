//! `frontier::misc2` — verbatim split of the original `frontier` module.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

pub(crate) fn round667_emit_active_oriented_delta_le10(
    b: &mut B,
    start_a: &[QubitId],
    start_b: &[QubitId],
    step_ctrl: QubitId,
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

    b.set_phase("round667_nonborrow_oriented_delta_le10");
    for delta in 1..=MAX_DELTA {
        for lower in 0..=FRONTIER_VALUES - 1 - delta {
            round653_compute_pair_ctrl(
                b,
                start_a,
                start_b,
                lower + delta,
                lower,
                step_ctrl,
                eq_a,
                eq_b,
                pair_ctrl,
                body_scratch,
                eq_scratch,
            );
            for (bit, delta_bit) in delta_bits.iter().enumerate() {
                if ((delta >> bit) & 1) == 1 {
                    mcx2_polar(b, pair_ctrl, true, borrow_ctrl, false, *delta_bit);
                }
            }
            round653_uncompute_pair_ctrl(
                b,
                start_a,
                start_b,
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

    b.set_phase("round667_borrow_oriented_delta_le10");
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

pub(crate) fn round667_emit_branch_delta_growth_sub(
    b: &mut B,
    start: &[QubitId],
    delta_bits: &[QubitId],
    growth_ctrl: QubitId,
    branch_ctrl: QubitId,
    branch_polarity: bool,
    tmp_ctrl: QubitId,
) {
    for (bit, delta_bit) in delta_bits.iter().enumerate() {
        mcx2_polar(b, *delta_bit, true, branch_ctrl, branch_polarity, tmp_ctrl);
        csub_nbit_const_direct_fast(b, start, U256::from(1u64 << bit), tmp_ctrl);
        mcx2_polar(b, *delta_bit, true, branch_ctrl, branch_polarity, tmp_ctrl);
    }

    mcx2_polar(b, growth_ctrl, true, branch_ctrl, branch_polarity, tmp_ctrl);
    csub_nbit_const_direct_fast(b, start, U256::from(1u64), tmp_ctrl);
    mcx2_polar(b, growth_ctrl, true, branch_ctrl, branch_polarity, tmp_ctrl);
}

pub(crate) fn round667_emit_oriented_delta_growth_frontier_update(
    b: &mut B,
    start_a: &[QubitId],
    start_b: &[QubitId],
    delta_bits: &[QubitId],
    growth_ctrl: QubitId,
    borrow_ctrl: QubitId,
    tmp_ctrl: QubitId,
) {
    debug_assert_eq!(start_a.len(), 9);
    debug_assert_eq!(start_b.len(), 9);
    debug_assert_eq!(delta_bits.len(), 4);

    b.set_phase("round667_nonborrow_update_start_a");
    round667_emit_branch_delta_growth_sub(
        b,
        start_a,
        delta_bits,
        growth_ctrl,
        borrow_ctrl,
        false,
        tmp_ctrl,
    );

    b.set_phase("round667_borrow_update_start_b");
    round667_emit_branch_delta_growth_sub(
        b,
        start_b,
        delta_bits,
        growth_ctrl,
        borrow_ctrl,
        true,
        tmp_ctrl,
    );

    b.set_phase("round667_borrow_swap_refreshed_frontiers");
    for i in 0..start_a.len() {
        cswap(b, borrow_ctrl, start_a[i], start_b[i]);
    }
}

pub(crate) fn round671_emit_multi_match_toggle(
    b: &mut B,
    stream: &[QubitId],
    active: QubitId,
    cursor_eq: QubitId,
    offset: usize,
    code: &[bool],
    target: QubitId,
    scratch: &[QubitId],
) {
    let controls = 2 + code.len();
    debug_assert!(scratch.len() >= controls.saturating_sub(2));
    for (idx, bit) in code.iter().enumerate() {
        if !*bit {
            b.x(stream[offset + idx]);
        }
    }
    if controls == 2 {
        b.ccx(active, cursor_eq, target);
    } else {
        b.ccx(active, cursor_eq, scratch[0]);
        for idx in 0..code.len() - 1 {
            let lhs = if idx == 0 { scratch[0] } else { scratch[idx] };
            b.ccx(lhs, stream[offset + idx], scratch[idx + 1]);
        }
        b.ccx(
            scratch[code.len() - 1],
            stream[offset + code.len() - 1],
            target,
        );
        for idx in (0..code.len() - 1).rev() {
            let lhs = if idx == 0 { scratch[0] } else { scratch[idx] };
            b.ccx(lhs, stream[offset + idx], scratch[idx + 1]);
        }
        b.ccx(active, cursor_eq, scratch[0]);
    }
    for (idx, bit) in code.iter().enumerate().rev() {
        if !*bit {
            b.x(stream[offset + idx]);
        }
    }
}

pub(crate) fn round671_emit_decode_symbol_toggle(
    b: &mut B,
    stream: &[QubitId],
    cursor: &[QubitId],
    active: QubitId,
    symbol_bits: &[QubitId],
    from_suffix: bool,
    cursor_eq: QubitId,
    match_flag: QubitId,
    cursor_eq_scratch: &[QubitId],
    match_scratch: &[QubitId],
) {
    const K_SYMBOLS: usize = 12;
    let stream_len = stream.len();
    for symbol in 0..K_SYMBOLS {
        let code = round671_k_code(symbol);
        let len = code.len();
        if from_suffix {
            for end in len..=stream_len {
                round631_emit_eq_const_toggle(b, cursor, end, cursor_eq, cursor_eq_scratch);
                round671_emit_multi_match_toggle(
                    b,
                    stream,
                    active,
                    cursor_eq,
                    end - len,
                    code,
                    match_flag,
                    match_scratch,
                );
                for (bit, q) in symbol_bits.iter().enumerate() {
                    if ((symbol >> bit) & 1) == 1 {
                        b.cx(match_flag, *q);
                    }
                }
                round671_emit_multi_match_toggle(
                    b,
                    stream,
                    active,
                    cursor_eq,
                    end - len,
                    code,
                    match_flag,
                    match_scratch,
                );
                round631_emit_eq_const_toggle(b, cursor, end, cursor_eq, cursor_eq_scratch);
            }
        } else {
            for start in 0..=stream_len - len {
                round631_emit_eq_const_toggle(b, cursor, start, cursor_eq, cursor_eq_scratch);
                round671_emit_multi_match_toggle(
                    b,
                    stream,
                    active,
                    cursor_eq,
                    start,
                    code,
                    match_flag,
                    match_scratch,
                );
                for (bit, q) in symbol_bits.iter().enumerate() {
                    if ((symbol >> bit) & 1) == 1 {
                        b.cx(match_flag, *q);
                    }
                }
                round671_emit_multi_match_toggle(
                    b,
                    stream,
                    active,
                    cursor_eq,
                    start,
                    code,
                    match_flag,
                    match_scratch,
                );
                round631_emit_eq_const_toggle(b, cursor, start, cursor_eq, cursor_eq_scratch);
            }
        }
    }
}

pub(crate) fn round671_emit_cursor_sub_symbol_len(
    b: &mut B,
    cursor: &[QubitId],
    active: QubitId,
    symbol_bits: &[QubitId],
    symbol_eq: QubitId,
    len_ctrl: QubitId,
    symbol_eq_scratch: &[QubitId],
) {
    for symbol in 0..12 {
        let len = round671_k_code(symbol).len();
        round631_emit_eq_const_toggle(b, symbol_bits, symbol, symbol_eq, symbol_eq_scratch);
        mcx2_polar(b, active, true, symbol_eq, true, len_ctrl);
        csub_nbit_const_direct_fast(b, cursor, U256::from(len as u64), len_ctrl);
        mcx2_polar(b, active, true, symbol_eq, true, len_ctrl);
        round631_emit_eq_const_toggle(b, symbol_bits, symbol, symbol_eq, symbol_eq_scratch);
    }
}

pub(crate) fn round632_emit_high_add_body(
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

    b.set_phase("round632_high_add_forward_interior");
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

    b.set_phase("round632_high_add_boundary_sum");
    for bit in (0..PACKED_WIDTH).rev() {
        round631_emit_eq_const_toggle(b, frontier, bit, boundary, eq_scratch);
        let carry = if bit + 1 == PACKED_WIDTH {
            c_in
        } else {
            src[bit + 1]
        };
        round632_compute_body_ctrl(b, step_ctrl, boundary, body_ctrl);
        b.ccx(body_ctrl, carry, dst[bit]);
        b.ccx(body_ctrl, src[bit], dst[bit]);
        round632_uncompute_body_ctrl(b, step_ctrl, boundary, body_ctrl);
        round631_emit_eq_const_toggle(b, frontier, bit, boundary, eq_scratch);
    }

    b.set_phase("round632_high_add_backward_interior");
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

pub(crate) fn round634_emit_min_frontier_compute(
    b: &mut B,
    lhs: &[QubitId],
    rhs: &[QubitId],
    min_out: &[QubitId],
    rhs_lt_lhs: QubitId,
) {
    debug_assert_eq!(lhs.len(), rhs.len());
    debug_assert_eq!(lhs.len(), min_out.len());

    b.set_phase("round634_copy_lhs_frontier_to_min");
    for i in 0..lhs.len() {
        b.cx(lhs[i], min_out[i]);
    }

    b.set_phase("round634_compute_rhs_lt_lhs");
    cmp_lt_into(b, rhs, lhs, rhs_lt_lhs);

    b.set_phase("round634_mux_rhs_into_min");
    for i in 0..lhs.len() {
        b.ccx(rhs_lt_lhs, lhs[i], min_out[i]);
        b.ccx(rhs_lt_lhs, rhs[i], min_out[i]);
    }
}

pub(crate) fn round647_emit_decrement_start_if_high_nonzero(
    b: &mut B,
    start: &[QubitId],
    flag: QubitId,
    eq_scratch: &[QubitId],
) {
    const PACKED_WIDTH: usize = N + 1;
    debug_assert_eq!(start.len(), 9);
    debug_assert!(eq_scratch.len() >= 7);

    // Compute flag = (start != 257), decrement under it, then uncompute from
    // the output.  The predicate is unchanged by this saturating-at-257 map:
    // zero-high states stay 257, nonzero-high states land in 0..255.
    round631_emit_eq_const_toggle(b, start, PACKED_WIDTH, flag, eq_scratch);
    b.x(flag);
    csub_nbit_const_direct_fast(b, start, U256::from(1u64), flag);
    b.x(flag);
    round631_emit_eq_const_toggle(b, start, PACKED_WIDTH, flag, eq_scratch);
}

pub(crate) fn round651_compute_common_frontier_gt(
    b: &mut B,
    lhs: &[QubitId],
    rhs: &[QubitId],
    frontier: &[QubitId],
    step_ctrl: QubitId,
    flag: QubitId,
    active: QubitId,
    body_ctrl: QubitId,
    c_in: QubitId,
    body_scratch: QubitId,
    eq_scratch: &[QubitId],
) {
    const PACKED_WIDTH: usize = N + 1;
    debug_assert_eq!(lhs.len(), PACKED_WIDTH);
    debug_assert_eq!(rhs.len(), PACKED_WIDTH);

    b.set_phase("round651_common_gt_negate_rhs_prefix");
    round651_low_prefix_forward_init(b, frontier, active, eq_scratch);
    for bit in 0..PACKED_WIDTH {
        round632_compute_body_ctrl(b, step_ctrl, active, body_ctrl);
        b.cx(body_ctrl, rhs[bit]);
        round632_uncompute_body_ctrl(b, step_ctrl, active, body_ctrl);
        round651_low_prefix_forward_next(b, frontier, active, eq_scratch, bit);
    }

    b.set_phase("round651_common_gt_forward_maj");
    round651_low_prefix_forward_init(b, frontier, active, eq_scratch);
    for bit in 0..PACKED_WIDTH {
        let carry = if bit == 0 { c_in } else { rhs[bit - 1] };
        round632_compute_body_ctrl(b, step_ctrl, active, body_ctrl);
        ctrl_maj(b, body_ctrl, carry, lhs[bit], rhs[bit], body_scratch);
        round632_uncompute_body_ctrl(b, step_ctrl, active, body_ctrl);
        round651_low_prefix_forward_next(b, frontier, active, eq_scratch, bit);
    }

    b.set_phase("round651_common_gt_boundary_latch");
    for bit in 0..PACKED_WIDTH {
        round631_emit_eq_const_toggle(b, frontier, bit + 1, active, eq_scratch);
        round632_compute_body_ctrl(b, step_ctrl, active, body_ctrl);
        b.ccx(body_ctrl, rhs[bit], flag);
        round632_uncompute_body_ctrl(b, step_ctrl, active, body_ctrl);
        round631_emit_eq_const_toggle(b, frontier, bit + 1, active, eq_scratch);
    }

    b.set_phase("round651_common_gt_backward_maj");
    round651_low_prefix_backward_init(b, frontier, active, eq_scratch, PACKED_WIDTH);
    for bit in (0..PACKED_WIDTH).rev() {
        let carry = if bit == 0 { c_in } else { rhs[bit - 1] };
        round632_compute_body_ctrl(b, step_ctrl, active, body_ctrl);
        ctrl_inv_maj(b, body_ctrl, carry, lhs[bit], rhs[bit], body_scratch);
        round632_uncompute_body_ctrl(b, step_ctrl, active, body_ctrl);
        round651_low_prefix_backward_next(b, frontier, active, eq_scratch, bit);
    }
    round651_low_prefix_backward_clean(b, frontier, active, eq_scratch);

    b.set_phase("round651_common_gt_unnegate_rhs_prefix");
    round651_low_prefix_forward_init(b, frontier, active, eq_scratch);
    for bit in 0..PACKED_WIDTH {
        round632_compute_body_ctrl(b, step_ctrl, active, body_ctrl);
        b.cx(body_ctrl, rhs[bit]);
        round632_uncompute_body_ctrl(b, step_ctrl, active, body_ctrl);
        round651_low_prefix_forward_next(b, frontier, active, eq_scratch, bit);
    }
}

pub(crate) fn round653_ctrl_or2_into(
    b: &mut B,
    ctrl: QubitId,
    x: QubitId,
    y: QubitId,
    out: QubitId,
    scratch: QubitId,
) {
    b.ccx(ctrl, x, out);
    b.ccx(ctrl, y, out);
    mcx3_polar(b, ctrl, true, x, true, y, true, out, scratch);
}

pub(crate) fn round653_ctrl_range_or_into(
    b: &mut B,
    ctrl: QubitId,
    lane: &[QubitId],
    start: usize,
    width: usize,
    flag: QubitId,
    or_chain: &[QubitId],
    scratch: QubitId,
) {
    debug_assert!(width >= 1);
    debug_assert!(start + width <= lane.len());
    debug_assert!(or_chain.len() >= width.saturating_sub(1));

    if width == 1 {
        b.ccx(ctrl, lane[start], flag);
        return;
    }

    round653_ctrl_or2_into(b, ctrl, lane[start], lane[start + 1], or_chain[0], scratch);
    for offset in 2..width {
        round653_ctrl_or2_into(
            b,
            ctrl,
            or_chain[offset - 2],
            lane[start + offset],
            or_chain[offset - 1],
            scratch,
        );
    }
    b.cx(or_chain[width - 2], flag);
    for offset in (2..width).rev() {
        round653_ctrl_or2_into(
            b,
            ctrl,
            or_chain[offset - 2],
            lane[start + offset],
            or_chain[offset - 1],
            scratch,
        );
    }
    round653_ctrl_or2_into(b, ctrl, lane[start], lane[start + 1], or_chain[0], scratch);
}

pub(crate) fn round653_compute_pair_ctrl(
    b: &mut B,
    start_a: &[QubitId],
    start_b: &[QubitId],
    value_a: usize,
    value_b: usize,
    step_ctrl: QubitId,
    eq_a: QubitId,
    eq_b: QubitId,
    pair_ctrl: QubitId,
    body_scratch: QubitId,
    eq_scratch: &[QubitId],
) {
    round631_emit_eq_const_toggle(b, start_a, value_a, eq_a, eq_scratch);
    round631_emit_eq_const_toggle(b, start_b, value_b, eq_b, eq_scratch);
    mcx3_polar(
        b,
        step_ctrl,
        true,
        eq_a,
        true,
        eq_b,
        true,
        pair_ctrl,
        body_scratch,
    );
}

pub(crate) fn round653_emit_bounded_range_pairs(
    b: &mut B,
    lane: &[QubitId],
    start_a: &[QubitId],
    start_b: &[QubitId],
    step_ctrl: QubitId,
    range_flag: QubitId,
    a_is_upper: bool,
    eq_a: QubitId,
    eq_b: QubitId,
    pair_ctrl: QubitId,
    body_scratch: QubitId,
    eq_scratch: &[QubitId],
    or_chain: &[QubitId],
) {
    const PACKED_WIDTH: usize = N + 1;
    const MAX_DELTA: usize = 12;

    for delta in 1..=MAX_DELTA {
        for lower in 0..=PACKED_WIDTH - delta {
            let (value_a, value_b) = if a_is_upper {
                (lower + delta, lower)
            } else {
                (lower, lower + delta)
            };
            round653_compute_pair_ctrl(
                b,
                start_a,
                start_b,
                value_a,
                value_b,
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
                lane,
                lower,
                delta,
                range_flag,
                or_chain,
                body_scratch,
            );
            round653_uncompute_pair_ctrl(
                b,
                start_a,
                start_b,
                value_a,
                value_b,
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

pub(crate) fn round654_emit_bounded_final_pairs(
    b: &mut B,
    lane: &[QubitId],
    start_a: &[QubitId],
    start_b: &[QubitId],
    step_ctrl: QubitId,
    common_gt: QubitId,
    gt_flag: QubitId,
    a_is_upper: bool,
    eq_a: QubitId,
    eq_b: QubitId,
    pair_ctrl: QubitId,
    range_tmp: QubitId,
    body_scratch: QubitId,
    eq_scratch: &[QubitId],
    or_chain: &[QubitId],
) {
    const PACKED_WIDTH: usize = N + 1;
    const MAX_DELTA: usize = 12;

    for delta in 1..=MAX_DELTA {
        for lower in 0..=PACKED_WIDTH - delta {
            let (value_a, value_b) = if a_is_upper {
                (lower + delta, lower)
            } else {
                (lower, lower + delta)
            };
            round653_compute_pair_ctrl(
                b,
                start_a,
                start_b,
                value_a,
                value_b,
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
                lane,
                lower,
                delta,
                range_tmp,
                or_chain,
                body_scratch,
            );
            if a_is_upper {
                b.x(common_gt);
                b.ccx(range_tmp, common_gt, gt_flag);
                b.x(common_gt);
            } else {
                b.ccx(range_tmp, common_gt, gt_flag);
            }
            round653_ctrl_range_or_into(
                b,
                pair_ctrl,
                lane,
                lower,
                delta,
                range_tmp,
                or_chain,
                body_scratch,
            );
            round653_uncompute_pair_ctrl(
                b,
                start_a,
                start_b,
                value_a,
                value_b,
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

pub(crate) fn round654_emit_overcutoff_final_pairs(
    b: &mut B,
    start_a: &[QubitId],
    start_b: &[QubitId],
    step_ctrl: QubitId,
    common_gt: QubitId,
    gt_flag: QubitId,
    a_is_upper: bool,
    eq_a: QubitId,
    eq_b: QubitId,
    pair_ctrl: QubitId,
    body_scratch: QubitId,
    eq_scratch: &[QubitId],
) {
    const PACKED_WIDTH: usize = N + 1;
    const MAX_DELTA: usize = 12;

    for delta in MAX_DELTA + 1..=PACKED_WIDTH {
        for lower in 0..=PACKED_WIDTH - delta {
            let (value_a, value_b) = if a_is_upper {
                (lower + delta, lower)
            } else {
                (lower, lower + delta)
            };
            round653_compute_pair_ctrl(
                b,
                start_a,
                start_b,
                value_a,
                value_b,
                step_ctrl,
                eq_a,
                eq_b,
                pair_ctrl,
                body_scratch,
                eq_scratch,
            );
            if a_is_upper {
                mcx2_polar(b, pair_ctrl, true, common_gt, false, gt_flag);
            } else {
                b.ccx(pair_ctrl, common_gt, gt_flag);
            }
            round653_uncompute_pair_ctrl(
                b,
                start_a,
                start_b,
                value_a,
                value_b,
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
