//! `frontier::builders2` — verbatim split of the original `frontier` module.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

pub(crate) fn build_round666_retained_delta_borrow_branch_builder() -> B {
    const FRONTIER_BITS: usize = 9;
    const PACKED_WIDTH: usize = N + 1;
    const MAX_DELTA: usize = 19;
    const DELTA_BITS: usize = 4;

    let mut b = B::new();
    let lane_a = b.alloc_qubits(PACKED_WIDTH);
    b.declare_qubit_register(&lane_a);
    let lane_b = b.alloc_qubits(PACKED_WIDTH);
    b.declare_qubit_register(&lane_b);
    let start_a = b.alloc_qubits(FRONTIER_BITS);
    b.declare_qubit_register(&start_a);
    let start_b = b.alloc_qubits(FRONTIER_BITS);
    b.declare_qubit_register(&start_b);
    let step_ctrl = b.alloc_qubit();
    let a_flag = b.alloc_qubit();
    let raw_borrow = b.alloc_qubit();
    let growth_flag = b.alloc_qubit();
    let delta_bits = b.alloc_qubits(DELTA_BITS);
    b.declare_qubit_register(&[step_ctrl, a_flag, raw_borrow, growth_flag]);
    b.declare_qubit_register(&delta_bits);

    let eq_a = b.alloc_qubit();
    let eq_b = b.alloc_qubit();
    let pair_ctrl = b.alloc_qubit();
    let body_scratch = b.alloc_qubit();
    let eq_scratch = b.alloc_qubits(FRONTIER_BITS - 2);
    let or_chain = b.alloc_qubits(MAX_DELTA - 1);

    b.set_phase("round666_seed_a_from_source_window");
    round659_emit_source_over_frontier_pairs(
        &mut b,
        &lane_a,
        &start_a,
        &start_b,
        step_ctrl,
        a_flag,
        eq_a,
        eq_b,
        pair_ctrl,
        body_scratch,
        &eq_scratch,
        &or_chain,
    );

    b.set_phase("round666_free_source_window_scratch");
    b.free_vec(&or_chain);
    b.free_vec(&eq_scratch);
    b.free(body_scratch);
    b.free(pair_ctrl);
    b.free(eq_b);
    b.free(eq_a);

    round634_emit_step3_cswap(&mut b, a_flag, &lane_a, &lane_b, &start_a, &start_b);

    let active = b.alloc_qubit();
    let boundary = b.alloc_qubit();
    let body_ctrl = b.alloc_qubit();
    let c_in = b.alloc_qubit();
    let body_scratch = b.alloc_qubit();
    let eq_scratch = b.alloc_qubits(FRONTIER_BITS - 2);
    let min_start = b.alloc_qubits(FRONTIER_BITS);
    let rhs_lt_lhs = b.alloc_qubit();

    round658_emit_low_sub_body_with_borrow(
        &mut b,
        &lane_a,
        &lane_b,
        &start_b,
        step_ctrl,
        raw_borrow,
        active,
        boundary,
        body_ctrl,
        c_in,
        body_scratch,
        &eq_scratch,
    );
    b.cx(raw_borrow, a_flag);

    let eq_a2 = b.alloc_qubit();
    let eq_b2 = b.alloc_qubit();
    let pair_ctrl2 = b.alloc_qubit();
    round663_emit_borrow_start_delta_le10(
        &mut b,
        &start_a,
        &start_b,
        raw_borrow,
        &delta_bits,
        eq_a2,
        eq_b2,
        pair_ctrl2,
        body_scratch,
        &eq_scratch,
    );
    b.free(pair_ctrl2);
    b.free(eq_b2);
    b.free(eq_a2);

    round662_emit_low_add_body(
        &mut b,
        &lane_a,
        &lane_b,
        &start_b,
        raw_borrow,
        active,
        boundary,
        body_ctrl,
        c_in,
        body_scratch,
        &eq_scratch,
    );

    round634_emit_min_frontier_compute(&mut b, &start_a, &start_b, &min_start, rhs_lt_lhs);
    round665_emit_high_add_body_with_growth_tap(
        &mut b,
        &lane_a,
        &lane_b,
        &min_start,
        raw_borrow,
        growth_flag,
        active,
        boundary,
        body_ctrl,
        c_in,
        body_scratch,
        &eq_scratch,
    );
    round634_emit_min_frontier_uncompute(&mut b, &start_a, &start_b, &min_start, rhs_lt_lhs);

    round632_emit_low_sub_body(
        &mut b,
        &lane_b,
        &lane_a,
        &start_b,
        raw_borrow,
        active,
        boundary,
        body_ctrl,
        c_in,
        body_scratch,
        &eq_scratch,
    );

    round666_emit_lane_cswap(&mut b, raw_borrow, &lane_a, &lane_b);
    round664_emit_delta_growth_frontier_update(
        &mut b,
        &start_a,
        &start_b,
        &delta_bits,
        growth_flag,
        raw_borrow,
    );

    b.set_phase("round666_free_retained_delta_branch_scratch");
    b.free(rhs_lt_lhs);
    b.free_vec(&min_start);
    b.free_vec(&eq_scratch);
    b.free(body_scratch);
    b.free(c_in);
    b.free(body_ctrl);
    b.free(boundary);
    b.free(active);
    b
}

pub fn build_round666_retained_delta_borrow_branch_component() -> Vec<Op> {
    build_round666_retained_delta_borrow_branch_builder().ops
}

pub fn build_round666_retained_delta_borrow_branch_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round666_retained_delta_borrow_branch_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub(crate) fn build_round667_unified_active_high_refresh_builder_with_tail(include_shift_step9: bool) -> B {
    const FRONTIER_BITS: usize = 9;
    const PACKED_WIDTH: usize = N + 1;
    const MAX_DELTA: usize = 19;
    const DELTA_BITS: usize = 4;

    let mut b = B::new();
    let lane_a = b.alloc_qubits(PACKED_WIDTH);
    b.declare_qubit_register(&lane_a);
    let lane_b = b.alloc_qubits(PACKED_WIDTH);
    b.declare_qubit_register(&lane_b);
    let start_a = b.alloc_qubits(FRONTIER_BITS);
    b.declare_qubit_register(&start_a);
    let start_b = b.alloc_qubits(FRONTIER_BITS);
    b.declare_qubit_register(&start_b);
    let step_ctrl = b.alloc_qubit();
    let a_flag = b.alloc_qubit();
    let raw_borrow = b.alloc_qubit();
    let growth_flag = b.alloc_qubit();
    let delta_bits = b.alloc_qubits(DELTA_BITS);
    b.declare_qubit_register(&[step_ctrl, a_flag, raw_borrow, growth_flag]);
    b.declare_qubit_register(&delta_bits);

    let eq_a = b.alloc_qubit();
    let eq_b = b.alloc_qubit();
    let pair_ctrl = b.alloc_qubit();
    let body_scratch = b.alloc_qubit();
    let eq_scratch = b.alloc_qubits(FRONTIER_BITS - 2);
    let or_chain = b.alloc_qubits(MAX_DELTA - 1);

    b.set_phase("round667_seed_a_from_source_window");
    round659_emit_source_over_frontier_pairs(
        &mut b,
        &lane_a,
        &start_a,
        &start_b,
        step_ctrl,
        a_flag,
        eq_a,
        eq_b,
        pair_ctrl,
        body_scratch,
        &eq_scratch,
        &or_chain,
    );

    b.set_phase("round667_free_source_window_scratch");
    b.free_vec(&or_chain);
    b.free_vec(&eq_scratch);
    b.free(body_scratch);
    b.free(pair_ctrl);
    b.free(eq_b);
    b.free(eq_a);

    round634_emit_step3_cswap(&mut b, a_flag, &lane_a, &lane_b, &start_a, &start_b);

    let active = b.alloc_qubit();
    let boundary = b.alloc_qubit();
    let body_ctrl = b.alloc_qubit();
    let c_in = b.alloc_qubit();
    let body_scratch = b.alloc_qubit();
    let eq_scratch = b.alloc_qubits(FRONTIER_BITS - 2);
    let min_start = b.alloc_qubits(FRONTIER_BITS);
    let rhs_lt_lhs = b.alloc_qubit();

    round658_emit_low_sub_body_with_borrow(
        &mut b,
        &lane_a,
        &lane_b,
        &start_b,
        step_ctrl,
        raw_borrow,
        active,
        boundary,
        body_ctrl,
        c_in,
        body_scratch,
        &eq_scratch,
    );
    b.cx(raw_borrow, a_flag);

    let eq_a2 = b.alloc_qubit();
    let eq_b2 = b.alloc_qubit();
    let pair_ctrl2 = b.alloc_qubit();
    round667_emit_active_oriented_delta_le10(
        &mut b,
        &start_a,
        &start_b,
        step_ctrl,
        raw_borrow,
        &delta_bits,
        eq_a2,
        eq_b2,
        pair_ctrl2,
        body_scratch,
        &eq_scratch,
    );
    b.free(pair_ctrl2);
    b.free(eq_b2);
    b.free(eq_a2);

    round634_emit_min_frontier_compute(&mut b, &start_a, &start_b, &min_start, rhs_lt_lhs);
    round667_emit_high_add_body_with_growth_tap_borrow_polar(
        &mut b,
        &lane_b,
        &lane_a,
        &min_start,
        step_ctrl,
        raw_borrow,
        false,
        growth_flag,
        active,
        boundary,
        body_ctrl,
        c_in,
        body_scratch,
        &eq_scratch,
    );
    round634_emit_min_frontier_uncompute(&mut b, &start_a, &start_b, &min_start, rhs_lt_lhs);

    b.set_phase("round667_apply_nonborrow_frontier_refresh");
    round667_emit_branch_delta_growth_sub(
        &mut b,
        &start_a,
        &delta_bits,
        growth_flag,
        raw_borrow,
        false,
        body_ctrl,
    );

    round662_emit_low_add_body(
        &mut b,
        &lane_a,
        &lane_b,
        &start_b,
        raw_borrow,
        active,
        boundary,
        body_ctrl,
        c_in,
        body_scratch,
        &eq_scratch,
    );

    round634_emit_min_frontier_compute(&mut b, &start_a, &start_b, &min_start, rhs_lt_lhs);
    round665_emit_high_add_body_with_growth_tap(
        &mut b,
        &lane_a,
        &lane_b,
        &min_start,
        raw_borrow,
        growth_flag,
        active,
        boundary,
        body_ctrl,
        c_in,
        body_scratch,
        &eq_scratch,
    );
    round634_emit_min_frontier_uncompute(&mut b, &start_a, &start_b, &min_start, rhs_lt_lhs);

    round632_emit_low_sub_body(
        &mut b,
        &lane_b,
        &lane_a,
        &start_b,
        raw_borrow,
        active,
        boundary,
        body_ctrl,
        c_in,
        body_scratch,
        &eq_scratch,
    );

    round666_emit_lane_cswap(&mut b, raw_borrow, &lane_a, &lane_b);

    b.set_phase("round667_apply_borrow_frontier_refresh");
    round667_emit_branch_delta_growth_sub(
        &mut b,
        &start_b,
        &delta_bits,
        growth_flag,
        raw_borrow,
        true,
        body_ctrl,
    );
    b.set_phase("round667_borrow_swap_refreshed_frontiers");
    for i in 0..start_a.len() {
        cswap(&mut b, raw_borrow, start_a[i], start_b[i]);
    }

    if include_shift_step9 {
        let clean2 = [active, boundary];
        round648_emit_packed_shift_double_frontier_dirty(
            &mut b,
            &lane_b,
            &start_b,
            &lane_a,
            &clean2,
            body_ctrl,
            &eq_scratch,
        );
        round648_emit_step9_cswap(&mut b, a_flag, &lane_a, &lane_b, &start_a, &start_b);
    }

    b.set_phase("round667_free_unified_active_refresh_scratch");
    b.free(rhs_lt_lhs);
    b.free_vec(&min_start);
    b.free_vec(&eq_scratch);
    b.free(body_scratch);
    b.free(c_in);
    b.free(body_ctrl);
    b.free(boundary);
    b.free(active);
    b
}

pub(crate) fn build_round667_unified_active_high_refresh_builder() -> B {
    build_round667_unified_active_high_refresh_builder_with_tail(false)
}

pub fn build_round667_unified_active_high_refresh_component() -> Vec<Op> {
    build_round667_unified_active_high_refresh_builder().ops
}

pub fn build_round667_unified_active_high_refresh_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round667_unified_active_high_refresh_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub(crate) fn build_round671_bifix_k_stream_pop_builder() -> B {
    const STREAM_BITS: usize = 326;
    const CURSOR_BITS: usize = 9;
    const SYMBOL_BITS: usize = 4;
    const MAX_CODE_BITS: usize = 12;

    let mut b = B::new();
    let stream = b.alloc_qubits(STREAM_BITS);
    b.declare_qubit_register(&stream);
    let cursor = b.alloc_qubits(CURSOR_BITS);
    b.declare_qubit_register(&cursor);
    let active = b.alloc_qubit();
    b.declare_qubit_register(&[active]);

    let symbol_bits = b.alloc_qubits(SYMBOL_BITS);
    b.declare_qubit_register(&symbol_bits);
    let cursor_eq = b.alloc_qubit();
    let match_flag = b.alloc_qubit();
    let symbol_eq = b.alloc_qubit();
    let len_ctrl = b.alloc_qubit();
    let cursor_eq_scratch = b.alloc_qubits(CURSOR_BITS - 2);
    let match_scratch = b.alloc_qubits(MAX_CODE_BITS);
    let symbol_eq_scratch = b.alloc_qubits(SYMBOL_BITS - 2);

    b.set_phase("round671_suffix_decode_k_symbol");
    round671_emit_decode_symbol_toggle(
        &mut b,
        &stream,
        &cursor,
        active,
        &symbol_bits,
        true,
        cursor_eq,
        match_flag,
        &cursor_eq_scratch,
        &match_scratch,
    );

    b.set_phase("round671_move_cursor_to_code_start");
    round671_emit_cursor_sub_symbol_len(
        &mut b,
        &cursor,
        active,
        &symbol_bits,
        symbol_eq,
        len_ctrl,
        &symbol_eq_scratch,
    );

    b.set_phase("round671_prefix_clear_k_symbol");
    round671_emit_decode_symbol_toggle(
        &mut b,
        &stream,
        &cursor,
        active,
        &symbol_bits,
        false,
        cursor_eq,
        match_flag,
        &cursor_eq_scratch,
        &match_scratch,
    );

    b.set_phase("round671_free_decoder_scratch");
    b.free_vec(&symbol_eq_scratch);
    b.free_vec(&match_scratch);
    b.free_vec(&cursor_eq_scratch);
    b.free(len_ctrl);
    b.free(symbol_eq);
    b.free(match_flag);
    b.free(cursor_eq);
    b.free_vec(&symbol_bits);
    b
}

pub fn build_round671_bifix_k_stream_pop_component() -> Vec<Op> {
    build_round671_bifix_k_stream_pop_builder().ops
}

pub fn build_round671_bifix_k_stream_pop_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round671_bifix_k_stream_pop_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub(crate) fn build_round635_reversible_frontier_refresh_skeleton_builder() -> B {
    const FRONTIER_BITS: usize = 9;
    const PACKED_WIDTH: usize = N + 1;

    let mut b = B::new();
    let lane_a = b.alloc_qubits(PACKED_WIDTH);
    b.declare_qubit_register(&lane_a);
    let lane_b = b.alloc_qubits(PACKED_WIDTH);
    b.declare_qubit_register(&lane_b);
    let start_a = b.alloc_qubits(FRONTIER_BITS);
    b.declare_qubit_register(&start_a);
    let start_b = b.alloc_qubits(FRONTIER_BITS);
    b.declare_qubit_register(&start_b);
    let refresh_start = b.alloc_qubits(FRONTIER_BITS);
    b.declare_qubit_register(&refresh_start);
    let swap_ctrl = b.alloc_qubit();
    let add_ctrl = b.alloc_qubit();
    b.declare_qubit_register(&[swap_ctrl, add_ctrl]);

    let active = b.alloc_qubit();
    let boundary = b.alloc_qubit();
    let body_ctrl = b.alloc_qubit();
    let c_in = b.alloc_qubit();
    let body_scratch = b.alloc_qubit();
    let eq_scratch = b.alloc_qubits(FRONTIER_BITS - 2);
    let min_start = b.alloc_qubits(FRONTIER_BITS);
    let rhs_lt_lhs = b.alloc_qubit();

    round634_emit_step3_cswap(&mut b, swap_ctrl, &lane_a, &lane_b, &start_a, &start_b);

    round632_emit_low_sub_body(
        &mut b,
        &lane_a,
        &lane_b,
        &start_b,
        add_ctrl,
        active,
        boundary,
        body_ctrl,
        c_in,
        body_scratch,
        &eq_scratch,
    );

    round634_emit_min_frontier_compute(&mut b, &start_a, &start_b, &min_start, rhs_lt_lhs);
    round632_emit_high_add_body(
        &mut b,
        &lane_b,
        &lane_a,
        &min_start,
        add_ctrl,
        active,
        boundary,
        body_ctrl,
        c_in,
        body_scratch,
        &eq_scratch,
    );

    b.set_phase("round635_inverse_high_add_for_old_frontier_refresh");
    emit_inverse(&mut b, |b| {
        round632_emit_high_add_body(
            b,
            &lane_b,
            &lane_a,
            &min_start,
            add_ctrl,
            active,
            boundary,
            body_ctrl,
            c_in,
            body_scratch,
            &eq_scratch,
        );
    });

    round632_emit_high_add_body(
        &mut b,
        &lane_b,
        &lane_a,
        &min_start,
        add_ctrl,
        active,
        boundary,
        body_ctrl,
        c_in,
        body_scratch,
        &eq_scratch,
    );
    round634_emit_min_frontier_uncompute(&mut b, &start_a, &start_b, &min_start, rhs_lt_lhs);

    b.set_phase("round635_free_refresh_skeleton_scratch");
    b.free(rhs_lt_lhs);
    b.free_vec(&min_start);
    b.free_vec(&eq_scratch);
    b.free(body_scratch);
    b.free(c_in);
    b.free(body_ctrl);
    b.free(boundary);
    b.free(active);
    b.free_vec(&refresh_start);
    b
}

pub fn build_round635_reversible_frontier_refresh_skeleton_component() -> Vec<Op> {
    build_round635_reversible_frontier_refresh_skeleton_builder().ops
}

pub fn build_round635_reversible_frontier_refresh_skeleton_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round635_reversible_frontier_refresh_skeleton_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub(crate) fn build_round651_common_frontier_compare_builder() -> B {
    const FRONTIER_BITS: usize = 9;
    const PACKED_WIDTH: usize = N + 1;

    let mut b = B::new();
    let lhs = b.alloc_qubits(PACKED_WIDTH);
    b.declare_qubit_register(&lhs);
    let rhs = b.alloc_qubits(PACKED_WIDTH);
    b.declare_qubit_register(&rhs);
    let frontier = b.alloc_qubits(FRONTIER_BITS);
    b.declare_qubit_register(&frontier);
    let step_ctrl = b.alloc_qubit();
    let flag = b.alloc_qubit();
    b.declare_qubit_register(&[step_ctrl, flag]);

    let active = b.alloc_qubit();
    let body_ctrl = b.alloc_qubit();
    let c_in = b.alloc_qubit();
    let body_scratch = b.alloc_qubit();
    let eq_scratch = b.alloc_qubits(FRONTIER_BITS - 2);

    round651_compute_common_frontier_gt(
        &mut b,
        &lhs,
        &rhs,
        &frontier,
        step_ctrl,
        flag,
        active,
        body_ctrl,
        c_in,
        body_scratch,
        &eq_scratch,
    );

    b.set_phase("round651_free_common_frontier_compare_scratch");
    b.free_vec(&eq_scratch);
    b.free(body_scratch);
    b.free(c_in);
    b.free(body_ctrl);
    b.free(active);
    b
}

pub fn build_round651_common_frontier_compare_component() -> Vec<Op> {
    build_round651_common_frontier_compare_builder().ops
}

pub fn build_round651_common_frontier_compare_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round651_common_frontier_compare_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub(crate) fn build_round653_bounded_start_delta_range_or_builder() -> B {
    const FRONTIER_BITS: usize = 9;
    const PACKED_WIDTH: usize = N + 1;
    const MAX_DELTA: usize = 12;

    let mut b = B::new();
    let lane_a = b.alloc_qubits(PACKED_WIDTH);
    b.declare_qubit_register(&lane_a);
    let lane_b = b.alloc_qubits(PACKED_WIDTH);
    b.declare_qubit_register(&lane_b);
    let start_a = b.alloc_qubits(FRONTIER_BITS);
    b.declare_qubit_register(&start_a);
    let start_b = b.alloc_qubits(FRONTIER_BITS);
    b.declare_qubit_register(&start_b);
    let step_ctrl = b.alloc_qubit();
    let range_flag = b.alloc_qubit();
    b.declare_qubit_register(&[step_ctrl, range_flag]);

    let eq_a = b.alloc_qubit();
    let eq_b = b.alloc_qubit();
    let pair_ctrl = b.alloc_qubit();
    let body_scratch = b.alloc_qubit();
    let eq_scratch = b.alloc_qubits(FRONTIER_BITS - 2);
    let or_chain = b.alloc_qubits(MAX_DELTA - 1);

    b.set_phase("round653_a_upper_bounded_range_or");
    round653_emit_bounded_range_pairs(
        &mut b,
        &lane_a,
        &start_a,
        &start_b,
        step_ctrl,
        range_flag,
        true,
        eq_a,
        eq_b,
        pair_ctrl,
        body_scratch,
        &eq_scratch,
        &or_chain,
    );

    b.set_phase("round653_b_upper_bounded_range_or");
    round653_emit_bounded_range_pairs(
        &mut b,
        &lane_b,
        &start_a,
        &start_b,
        step_ctrl,
        range_flag,
        false,
        eq_a,
        eq_b,
        pair_ctrl,
        body_scratch,
        &eq_scratch,
        &or_chain,
    );

    b.set_phase("round653_free_bounded_range_or_scratch");
    b.free_vec(&or_chain);
    b.free_vec(&eq_scratch);
    b.free(body_scratch);
    b.free(pair_ctrl);
    b.free(eq_b);
    b.free(eq_a);
    b
}

pub fn build_round653_bounded_start_delta_range_or_component() -> Vec<Op> {
    build_round653_bounded_start_delta_range_or_builder().ops
}

pub fn build_round653_bounded_start_delta_range_or_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round653_bounded_start_delta_range_or_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}
