//! `bench::centered2` — verbatim split of the original `bench` module.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

pub(crate) fn build_direct_centered_branch_replay_finalizer_fit_bench_builder() -> B {
    let mut b = B::new();
    let tx = b.alloc_qubits(N);
    b.declare_qubit_register(&tx);
    let ty = b.alloc_qubits(N);
    b.declare_qubit_register(&ty);
    let ox = b.alloc_bits(N);
    b.declare_bit_register(&ox);
    let oy = b.alloc_bits(N);
    b.declare_bit_register(&oy);

    b.set_phase("direct_centered_branch_replay_finalizer_alloc_envelope");
    let digit_lane = b.alloc_qubits(DIRECT_CENTERED_LOW_BRANCH_DIGIT_LANE_BITS);
    let meta = b.alloc_qubits(DIRECT_CENTERED_LOW_BRANCH_META_BITS);
    let prefix = b.alloc_qubits(DIRECT_CENTERED_LOW_BRANCH_PREFIX_BITS);
    let branch = b.alloc_qubits(DIRECT_CENTERED_EXPLICIT_BRANCH_HISTORY_BITS);
    let touch = b.alloc_qubits(DIRECT_CENTERED_BRANCH_SIDECAR_TOUCH_BITS);

    b.set_phase("direct_centered_branch_replay_seed_history");
    for i in 0..branch.len() {
        b.ccx(tx[(7 * i + 2) % N], ty[(11 * i + 5) % N], branch[i]);
    }

    b.set_phase("direct_centered_branch_replay_all_digits");
    for &branch_bit in &branch {
        emit_direct_centered_branch_digit_update_clean(
            &mut b,
            &digit_lane,
            &ty,
            branch_bit,
            touch[2],
            &prefix[..N],
            touch[1],
        );
    }

    b.set_phase("direct_centered_branch_replay_clear_nonfinal_history");
    for i in (1..branch.len()).rev() {
        b.ccx(tx[(7 * i + 2) % N], ty[(11 * i + 5) % N], branch[i]);
    }

    let carries: Vec<QubitId> = prefix[N..]
        .iter()
        .chain(meta.iter())
        .chain(touch[3..].iter())
        .chain(branch[1..].iter())
        .copied()
        .take(N - 1)
        .collect();
    assert_eq!(carries.len(), N - 1);

    b.set_phase("direct_centered_branch_replay_fast_finalizer");
    emit_direct_centered_branch_retained_finalizer_fast(
        &mut b,
        &digit_lane,
        &ty,
        branch[0],
        &prefix[..N],
        touch[0],
        &carries,
    );

    b.set_phase("direct_centered_branch_replay_clear_final_history");
    b.ccx(tx[2], ty[5], branch[0]);

    b.set_phase("direct_centered_branch_replay_finalizer_free_envelope");
    b.free_vec(&touch);
    b.free_vec(&branch);
    b.free_vec(&prefix);
    b.free_vec(&meta);
    b.free_vec(&digit_lane);
    let _ = (ox, oy);
    b
}

pub fn build_direct_centered_branch_replay_finalizer_fit_bench_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_direct_centered_branch_replay_finalizer_fit_bench_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub fn build_direct_centered_branch_replay_finalizer_fit_bench() -> Vec<Op> {
    build_direct_centered_branch_replay_finalizer_fit_bench_builder().ops
}

pub(crate) fn build_direct_centered_branch_predicate_step_fit_bench_builder() -> B {
    let mut b = B::new();
    let tx = b.alloc_qubits(N);
    b.declare_qubit_register(&tx);
    let ty = b.alloc_qubits(N);
    b.declare_qubit_register(&ty);
    let ox = b.alloc_bits(N);
    b.declare_bit_register(&ox);
    let oy = b.alloc_bits(N);
    b.declare_bit_register(&oy);

    b.set_phase("direct_centered_branch_predicate_step_alloc_envelope");
    let digit_lane = b.alloc_qubits(DIRECT_CENTERED_LOW_BRANCH_DIGIT_LANE_BITS);
    let meta = b.alloc_qubits(DIRECT_CENTERED_LOW_BRANCH_META_BITS);
    let prefix = b.alloc_qubits(DIRECT_CENTERED_LOW_BRANCH_PREFIX_BITS);
    let branch = b.alloc_qubits(DIRECT_CENTERED_EXPLICIT_BRANCH_HISTORY_BITS);
    let touch = b.alloc_qubits(DIRECT_CENTERED_BRANCH_SIDECAR_TOUCH_BITS);

    emit_direct_centered_low_path_branch_toggle(
        &mut b,
        &tx,
        &ty,
        branch[0],
        &prefix[..(N + 1)],
        touch[0],
        touch[1],
    );

    b.set_phase("direct_centered_branch_predicate_step_replay_digit");
    emit_direct_centered_branch_digit_update_clean(
        &mut b,
        &digit_lane,
        &ty,
        branch[0],
        touch[2],
        &prefix[..N],
        touch[1],
    );

    emit_direct_centered_low_path_branch_toggle(
        &mut b,
        &tx,
        &ty,
        branch[0],
        &prefix[..(N + 1)],
        touch[0],
        touch[1],
    );

    b.set_phase("direct_centered_branch_predicate_step_free_envelope");
    b.free_vec(&touch);
    b.free_vec(&branch);
    b.free_vec(&prefix);
    b.free_vec(&meta);
    b.free_vec(&digit_lane);
    let _ = (ox, oy);
    b
}

pub fn build_direct_centered_branch_predicate_step_fit_bench_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_direct_centered_branch_predicate_step_fit_bench_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub fn build_direct_centered_branch_predicate_step_fit_bench() -> Vec<Op> {
    build_direct_centered_branch_predicate_step_fit_bench_builder().ops
}

pub(crate) fn build_direct_centered_qlow_lowpath_branch_row_fit_bench_builder(q_bits: usize) -> B {
    assert!((1..=N).contains(&q_bits));
    let mut b = B::new();
    let tx = b.alloc_qubits(N);
    b.declare_qubit_register(&tx);
    let ty = b.alloc_qubits(N);
    b.declare_qubit_register(&ty);
    let ox = b.alloc_bits(N);
    b.declare_bit_register(&ox);
    let oy = b.alloc_bits(N);
    b.declare_bit_register(&oy);

    b.set_phase("direct_centered_qlow_lowpath_branch_alloc_envelope");
    let digit_lane = b.alloc_qubits(DIRECT_CENTERED_LOW_BRANCH_DIGIT_LANE_BITS);
    let meta = b.alloc_qubits(DIRECT_CENTERED_LOW_BRANCH_META_BITS);
    let prefix = b.alloc_qubits(DIRECT_CENTERED_LOW_BRANCH_PREFIX_BITS);
    let branch = b.alloc_qubits(DIRECT_CENTERED_EXPLICIT_BRANCH_HISTORY_BITS);
    let touch = b.alloc_qubits(DIRECT_CENTERED_BRANCH_SIDECAR_TOUCH_BITS);

    b.set_phase("direct_centered_qlow_lowpath_branch_touch_envelope");
    direct_centered_touch_qubits_for_count(&mut b, &digit_lane);
    direct_centered_touch_qubits_for_count(&mut b, &meta);
    direct_centered_touch_qubits_for_count(&mut b, &prefix);
    direct_centered_touch_qubits_for_count(&mut b, &branch);
    direct_centered_touch_qubits_for_count(&mut b, &touch);

    emit_direct_centered_qlow_lowpath_branch_row_step(
        &mut b,
        &tx,
        &ty,
        &digit_lane[..q_bits],
        branch[0],
        &prefix[..N],
        &prefix[..(N + 1)],
        touch[0],
        touch[1],
        touch[2],
        touch[3],
    );

    let _ = (ox, oy);
    b
}

pub fn build_direct_centered_qlow_lowpath_branch_row_fit_bench_phase_resources(
    q_bits: usize,
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_direct_centered_qlow_lowpath_branch_row_fit_bench_builder(q_bits);
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub fn build_direct_centered_qlow_lowpath_branch_row_fit_bench(q_bits: usize) -> Vec<Op> {
    build_direct_centered_qlow_lowpath_branch_row_fit_bench_builder(q_bits).ops
}

pub(crate) fn build_direct_centered_qlow_lowpath_branch_digit_row_fit_bench_builder(q_bits: usize) -> B {
    assert!((1..=DIRECT_CENTERED_LOW_BRANCH_META_BITS).contains(&q_bits));
    let mut b = B::new();
    let tx = b.alloc_qubits(N);
    b.declare_qubit_register(&tx);
    let ty = b.alloc_qubits(N);
    b.declare_qubit_register(&ty);
    let ox = b.alloc_bits(N);
    b.declare_bit_register(&ox);
    let oy = b.alloc_bits(N);
    b.declare_bit_register(&oy);

    b.set_phase("direct_centered_qlow_lowpath_branch_digit_alloc_envelope");
    let digit_lane = b.alloc_qubits(DIRECT_CENTERED_LOW_BRANCH_DIGIT_LANE_BITS);
    let meta = b.alloc_qubits(DIRECT_CENTERED_LOW_BRANCH_META_BITS);
    let prefix = b.alloc_qubits(DIRECT_CENTERED_LOW_BRANCH_PREFIX_BITS);
    let branch = b.alloc_qubits(DIRECT_CENTERED_EXPLICIT_BRANCH_HISTORY_BITS);
    let touch = b.alloc_qubits(DIRECT_CENTERED_BRANCH_SIDECAR_TOUCH_BITS);

    b.set_phase("direct_centered_qlow_lowpath_branch_digit_touch_envelope");
    direct_centered_touch_qubits_for_count(&mut b, &digit_lane);
    direct_centered_touch_qubits_for_count(&mut b, &meta);
    direct_centered_touch_qubits_for_count(&mut b, &prefix);
    direct_centered_touch_qubits_for_count(&mut b, &branch);
    direct_centered_touch_qubits_for_count(&mut b, &touch);

    emit_direct_centered_qlow_lowpath_branch_row_step(
        &mut b,
        &tx,
        &ty,
        &meta[..q_bits],
        branch[0],
        &prefix[..N],
        &prefix[..(N + 1)],
        touch[0],
        touch[1],
        touch[2],
        touch[3],
    );

    b.set_phase("direct_centered_qlow_lowpath_branch_digit_row_update_clean");
    emit_direct_centered_branch_digit_update_clean(
        &mut b,
        &digit_lane,
        &ty,
        branch[0],
        touch[4],
        &prefix[..N],
        touch[5],
    );

    let _ = (ox, oy);
    b
}

pub fn build_direct_centered_qlow_lowpath_branch_digit_row_fit_bench_phase_resources(
    q_bits: usize,
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_direct_centered_qlow_lowpath_branch_digit_row_fit_bench_builder(q_bits);
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub fn build_direct_centered_qlow_lowpath_branch_digit_row_fit_bench(q_bits: usize) -> Vec<Op> {
    build_direct_centered_qlow_lowpath_branch_digit_row_fit_bench_builder(q_bits).ops
}

pub(crate) fn build_direct_centered_shifted_source_qbit_row_fit_bench_builder(q_bits: usize) -> B {
    assert!((1..=DIRECT_CENTERED_LOW_BRANCH_META_BITS).contains(&q_bits));
    let mut b = B::new();
    let tx = b.alloc_qubits(N);
    b.declare_qubit_register(&tx);
    let ty = b.alloc_qubits(N);
    b.declare_qubit_register(&ty);
    let ox = b.alloc_bits(N);
    b.declare_bit_register(&ox);
    let oy = b.alloc_bits(N);
    b.declare_bit_register(&oy);

    b.set_phase("direct_centered_shifted_source_qbit_alloc_envelope");
    let digit_lane = b.alloc_qubits(DIRECT_CENTERED_LOW_BRANCH_DIGIT_LANE_BITS);
    let meta = b.alloc_qubits(DIRECT_CENTERED_LOW_BRANCH_META_BITS);
    let prefix = b.alloc_qubits(DIRECT_CENTERED_LOW_BRANCH_PREFIX_BITS);
    let branch = b.alloc_qubits(DIRECT_CENTERED_EXPLICIT_BRANCH_HISTORY_BITS);
    let touch = b.alloc_qubits(DIRECT_CENTERED_BRANCH_SIDECAR_TOUCH_BITS);

    b.set_phase("direct_centered_shifted_source_qbit_touch_envelope");
    direct_centered_touch_qubits_for_count(&mut b, &digit_lane);
    direct_centered_touch_qubits_for_count(&mut b, &meta);
    direct_centered_touch_qubits_for_count(&mut b, &prefix);
    direct_centered_touch_qubits_for_count(&mut b, &branch);
    direct_centered_touch_qubits_for_count(&mut b, &touch);

    let carries: Vec<QubitId> = prefix[N..]
        .iter()
        .chain(branch.iter())
        .chain(touch[3..].iter())
        .copied()
        .take(N - 1)
        .collect();
    assert_eq!(carries.len(), N - 1);

    emit_direct_centered_shifted_source_qbit_row(
        &mut b,
        &tx,
        &ty,
        &digit_lane,
        &ty,
        &meta[..q_bits],
        &prefix[..N],
        touch[0],
        touch[1],
        touch[2],
        &carries,
    );

    let _ = (ox, oy);
    b
}

pub fn build_direct_centered_shifted_source_qbit_row_fit_bench_phase_resources(
    q_bits: usize,
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_direct_centered_shifted_source_qbit_row_fit_bench_builder(q_bits);
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub fn build_direct_centered_shifted_source_qbit_row_fit_bench(q_bits: usize) -> Vec<Op> {
    build_direct_centered_shifted_source_qbit_row_fit_bench_builder(q_bits).ops
}

pub(crate) fn build_direct_centered_row_transition_fit_bench_builder() -> B {
    let mut b = B::new();
    let tx = b.alloc_qubits(N);
    b.declare_qubit_register(&tx);
    let ty = b.alloc_qubits(N);
    b.declare_qubit_register(&ty);
    let ox = b.alloc_bits(N);
    b.declare_bit_register(&ox);
    let oy = b.alloc_bits(N);
    b.declare_bit_register(&oy);

    b.set_phase("direct_centered_row_transition_alloc_envelope");
    let digit_lane = b.alloc_qubits(DIRECT_CENTERED_LOW_BRANCH_DIGIT_LANE_BITS);
    let meta = b.alloc_qubits(DIRECT_CENTERED_LOW_BRANCH_META_BITS);
    let prefix = b.alloc_qubits(DIRECT_CENTERED_LOW_BRANCH_PREFIX_BITS);
    let branch = b.alloc_qubits(DIRECT_CENTERED_EXPLICIT_BRANCH_HISTORY_BITS);
    let touch = b.alloc_qubits(DIRECT_CENTERED_BRANCH_SIDECAR_TOUCH_BITS);

    b.set_phase("direct_centered_row_transition_touch_envelope");
    direct_centered_touch_qubits_for_count(&mut b, &digit_lane);
    direct_centered_touch_qubits_for_count(&mut b, &meta);
    direct_centered_touch_qubits_for_count(&mut b, &prefix);
    direct_centered_touch_qubits_for_count(&mut b, &branch);
    direct_centered_touch_qubits_for_count(&mut b, &touch);

    let carries: Vec<QubitId> = prefix[N..]
        .iter()
        .chain(meta.iter())
        .chain(touch[1..].iter())
        .chain(branch[1..].iter())
        .copied()
        .take(N - 1)
        .collect();
    assert_eq!(carries.len(), N - 1);

    emit_direct_centered_remainder_abs_swap_transition(
        &mut b,
        &tx,
        &ty,
        branch[0],
        &digit_lane,
        &carries,
    );

    let _ = (ox, oy);
    b
}

pub fn build_direct_centered_row_transition_fit_bench_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_direct_centered_row_transition_fit_bench_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub fn build_direct_centered_row_transition_fit_bench() -> Vec<Op> {
    build_direct_centered_row_transition_fit_bench_builder().ops
}

pub(crate) fn build_direct_centered_predicate_replay_finalizer_fit_bench_builder() -> B {
    let mut b = B::new();
    let tx = b.alloc_qubits(N);
    b.declare_qubit_register(&tx);
    let ty = b.alloc_qubits(N);
    b.declare_qubit_register(&ty);
    let ox = b.alloc_bits(N);
    b.declare_bit_register(&ox);
    let oy = b.alloc_bits(N);
    b.declare_bit_register(&oy);

    b.set_phase("direct_centered_predicate_replay_finalizer_alloc_envelope");
    let digit_lane = b.alloc_qubits(DIRECT_CENTERED_LOW_BRANCH_DIGIT_LANE_BITS);
    let meta = b.alloc_qubits(DIRECT_CENTERED_LOW_BRANCH_META_BITS);
    let prefix = b.alloc_qubits(DIRECT_CENTERED_LOW_BRANCH_PREFIX_BITS);
    let branch = b.alloc_qubits(DIRECT_CENTERED_EXPLICIT_BRANCH_HISTORY_BITS);
    let touch = b.alloc_qubits(DIRECT_CENTERED_BRANCH_SIDECAR_TOUCH_BITS);

    b.set_phase("direct_centered_predicate_replay_all_rows");
    for &branch_bit in &branch {
        emit_direct_centered_low_path_branch_toggle(
            &mut b,
            &tx,
            &ty,
            branch_bit,
            &prefix[..(N + 1)],
            touch[0],
            touch[1],
        );
        emit_direct_centered_branch_digit_update_clean(
            &mut b,
            &digit_lane,
            &ty,
            branch_bit,
            touch[2],
            &prefix[..N],
            touch[1],
        );
    }

    b.set_phase("direct_centered_predicate_replay_clear_nonfinal_history");
    for i in (1..branch.len()).rev() {
        emit_direct_centered_low_path_branch_toggle(
            &mut b,
            &tx,
            &ty,
            branch[i],
            &prefix[..(N + 1)],
            touch[0],
            touch[1],
        );
    }

    let carries: Vec<QubitId> = prefix[N..]
        .iter()
        .chain(meta.iter())
        .chain(touch[3..].iter())
        .chain(branch[1..].iter())
        .copied()
        .take(N - 1)
        .collect();
    assert_eq!(carries.len(), N - 1);

    b.set_phase("direct_centered_predicate_replay_fast_finalizer");
    emit_direct_centered_branch_retained_finalizer_fast(
        &mut b,
        &digit_lane,
        &ty,
        branch[0],
        &prefix[..N],
        touch[0],
        &carries,
    );

    b.set_phase("direct_centered_predicate_replay_clear_final_history");
    emit_direct_centered_low_path_branch_toggle(
        &mut b,
        &tx,
        &ty,
        branch[0],
        &prefix[..(N + 1)],
        touch[0],
        touch[1],
    );

    b.set_phase("direct_centered_predicate_replay_finalizer_free_envelope");
    b.free_vec(&touch);
    b.free_vec(&branch);
    b.free_vec(&prefix);
    b.free_vec(&meta);
    b.free_vec(&digit_lane);
    let _ = (ox, oy);
    b
}

pub fn build_direct_centered_predicate_replay_finalizer_fit_bench_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_direct_centered_predicate_replay_finalizer_fit_bench_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub fn build_direct_centered_predicate_replay_finalizer_fit_bench() -> Vec<Op> {
    build_direct_centered_predicate_replay_finalizer_fit_bench_builder().ops
}

pub(crate) fn build_direct_centered_inline_predicate_finalizer_delta_fit_bench_builder() -> B {
    let mut b = B::new();
    let tx = b.alloc_qubits(N);
    b.declare_qubit_register(&tx);
    let ty = b.alloc_qubits(N);
    b.declare_qubit_register(&ty);
    let ox = b.alloc_bits(N);
    b.declare_bit_register(&ox);
    let oy = b.alloc_bits(N);
    b.declare_bit_register(&oy);

    b.set_phase("direct_centered_inline_predicate_delta_alloc_dual_history_envelope");
    let digit_lane = b.alloc_qubits(DIRECT_CENTERED_LOW_BRANCH_DIGIT_LANE_BITS);
    let meta = b.alloc_qubits(DIRECT_CENTERED_LOW_BRANCH_META_BITS);
    let prefix = b.alloc_qubits(DIRECT_CENTERED_LOW_BRANCH_PREFIX_BITS);
    let branch = b.alloc_qubits(DIRECT_CENTERED_EXPLICIT_BRANCH_HISTORY_BITS);
    let touch = b.alloc_qubits(DIRECT_CENTERED_BRANCH_SIDECAR_TOUCH_BITS);
    let qlow_history = b.alloc_qubits(DIRECT_CENTERED_EXPLICIT_BRANCH_HISTORY_BITS);

    let final_branch_index = branch.len() - 1;
    for &branch_bit in &branch[..final_branch_index] {
        emit_direct_centered_low_path_branch_toggle(
            &mut b,
            &tx,
            &ty,
            branch_bit,
            &prefix[..(N + 1)],
            touch[0],
            touch[1],
        );
        emit_direct_centered_low_path_branch_toggle(
            &mut b,
            &tx,
            &ty,
            branch_bit,
            &prefix[..(N + 1)],
            touch[0],
            touch[1],
        );
    }

    emit_direct_centered_low_path_branch_toggle(
        &mut b,
        &tx,
        &ty,
        branch[final_branch_index],
        &prefix[..(N + 1)],
        touch[0],
        touch[1],
    );

    let carries: Vec<QubitId> = prefix[N..]
        .iter()
        .chain(meta.iter())
        .chain(touch[3..].iter())
        .chain(branch[..final_branch_index].iter())
        .chain(qlow_history.iter())
        .copied()
        .take(N - 1)
        .collect();
    assert_eq!(carries.len(), N - 1);

    b.set_phase("direct_centered_inline_predicate_delta_fast_finalizer");
    emit_direct_centered_branch_retained_finalizer_fast(
        &mut b,
        &digit_lane,
        &ty,
        branch[final_branch_index],
        &prefix[..N],
        touch[2],
        &carries,
    );

    emit_direct_centered_low_path_branch_toggle(
        &mut b,
        &tx,
        &ty,
        branch[final_branch_index],
        &prefix[..(N + 1)],
        touch[0],
        touch[1],
    );

    b.set_phase("direct_centered_inline_predicate_delta_free_dual_history_envelope");
    b.free_vec(&qlow_history);
    b.free_vec(&touch);
    b.free_vec(&branch);
    b.free_vec(&prefix);
    b.free_vec(&meta);
    b.free_vec(&digit_lane);
    let _ = (ox, oy);
    b
}

pub fn build_direct_centered_inline_predicate_finalizer_delta_fit_bench_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_direct_centered_inline_predicate_finalizer_delta_fit_bench_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub fn build_direct_centered_inline_predicate_finalizer_delta_fit_bench() -> Vec<Op> {
    build_direct_centered_inline_predicate_finalizer_delta_fit_bench_builder().ops
}

pub(crate) fn build_direct_centered_sidecar_finalizer_fit_bench_builder() -> B {
    let mut b = B::new();
    let tx = b.alloc_qubits(N);
    b.declare_qubit_register(&tx);
    let ty = b.alloc_qubits(N);
    b.declare_qubit_register(&ty);
    let ox = b.alloc_bits(N);
    b.declare_bit_register(&ox);
    let oy = b.alloc_bits(N);
    b.declare_bit_register(&oy);

    b.set_phase("direct_centered_sidecar_finalizer_alloc_envelope");
    let digit_lane = b.alloc_qubits(DIRECT_CENTERED_LOW_BRANCH_DIGIT_LANE_BITS);
    let meta = b.alloc_qubits(DIRECT_CENTERED_LOW_BRANCH_META_BITS);
    let prefix = b.alloc_qubits(DIRECT_CENTERED_LOW_BRANCH_PREFIX_BITS);
    let branch = b.alloc_qubits(DIRECT_CENTERED_EXPLICIT_BRANCH_HISTORY_BITS);
    let touch = b.alloc_qubits(DIRECT_CENTERED_BRANCH_SIDECAR_TOUCH_BITS);

    b.set_phase("direct_centered_sidecar_finalizer_reuse_prefix_scratch");
    emit_direct_centered_branch_retained_finalizer(
        &mut b,
        &digit_lane,
        &ty,
        branch[0],
        &prefix[..N],
        touch[0],
    );

    b.set_phase("direct_centered_sidecar_finalizer_free_envelope");
    b.free_vec(&touch);
    b.free_vec(&branch);
    b.free_vec(&prefix);
    b.free_vec(&meta);
    b.free_vec(&digit_lane);
    let _ = (tx, ox, oy);
    b
}

pub(crate) fn build_direct_centered_sidecar_fast_finalizer_fit_bench_builder() -> B {
    let mut b = B::new();
    let tx = b.alloc_qubits(N);
    b.declare_qubit_register(&tx);
    let ty = b.alloc_qubits(N);
    b.declare_qubit_register(&ty);
    let ox = b.alloc_bits(N);
    b.declare_bit_register(&ox);
    let oy = b.alloc_bits(N);
    b.declare_bit_register(&oy);

    b.set_phase("direct_centered_sidecar_fast_finalizer_alloc_envelope");
    let digit_lane = b.alloc_qubits(DIRECT_CENTERED_LOW_BRANCH_DIGIT_LANE_BITS);
    let meta = b.alloc_qubits(DIRECT_CENTERED_LOW_BRANCH_META_BITS);
    let prefix = b.alloc_qubits(DIRECT_CENTERED_LOW_BRANCH_PREFIX_BITS);
    let branch = b.alloc_qubits(DIRECT_CENTERED_EXPLICIT_BRANCH_HISTORY_BITS);
    let touch = b.alloc_qubits(DIRECT_CENTERED_BRANCH_SIDECAR_TOUCH_BITS);

    let carries: Vec<QubitId> = prefix[N..]
        .iter()
        .chain(meta.iter())
        .chain(touch[1..].iter())
        .chain(branch[1..].iter())
        .copied()
        .take(N - 1)
        .collect();
    assert_eq!(carries.len(), N - 1);

    b.set_phase("direct_centered_sidecar_fast_finalizer_reuse_cleared_sidecar_scratch");
    emit_direct_centered_branch_retained_finalizer_fast(
        &mut b,
        &digit_lane,
        &ty,
        branch[0],
        &prefix[..N],
        touch[0],
        &carries,
    );

    b.set_phase("direct_centered_sidecar_fast_finalizer_free_envelope");
    b.free_vec(&touch);
    b.free_vec(&branch);
    b.free_vec(&prefix);
    b.free_vec(&meta);
    b.free_vec(&digit_lane);
    let _ = (tx, ox, oy);
    b
}

pub fn build_direct_centered_sidecar_finalizer_fit_bench_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_direct_centered_sidecar_finalizer_fit_bench_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub fn build_direct_centered_sidecar_finalizer_fit_bench() -> Vec<Op> {
    build_direct_centered_sidecar_finalizer_fit_bench_builder().ops
}

pub fn build_direct_centered_sidecar_fast_finalizer_fit_bench_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_direct_centered_sidecar_fast_finalizer_fit_bench_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub fn build_direct_centered_sidecar_fast_finalizer_fit_bench() -> Vec<Op> {
    build_direct_centered_sidecar_fast_finalizer_fit_bench_builder().ops
}
