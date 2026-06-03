//! `bench::misc` — verbatim split of the original `bench` module.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

pub(crate) fn init_small_const_reg(b: &mut B, reg: &[QubitId], value: u64) {
    for (i, &q) in reg.iter().enumerate() {
        if i < u64::BITS as usize && ((value >> i) & 1) != 0 {
            b.x(q);
        }
    }
}

pub(crate) fn build_dialog_gcd_raw_tobitvector_fit_bench_builder() -> B {
    let mut b = B::new();
    let tx = b.alloc_qubits(N);
    b.declare_qubit_register(&tx);
    let ty = b.alloc_qubits(N);
    b.declare_qubit_register(&ty);
    let ox = b.alloc_bits(N);
    b.declare_bit_register(&ox);
    let oy = b.alloc_bits(N);
    b.declare_bit_register(&oy);
    let dialog_log = b.alloc_qubits(DIALOG_GCD_RAW_LOG_BITS);

    let _terminal_u = emit_dialog_gcd_raw_tobitvector(&mut b, &tx, &dialog_log, SECP256K1_P);
    let _ = (ty, ox, oy);
    b
}

pub fn build_dialog_gcd_raw_tobitvector_fit_bench_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_dialog_gcd_raw_tobitvector_fit_bench_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub fn build_dialog_gcd_raw_tobitvector_fit_bench() -> Vec<Op> {
    build_dialog_gcd_raw_tobitvector_fit_bench_builder().ops
}

pub(crate) fn build_dialog_gcd_raw_apply_fit_bench_builder() -> B {
    let mut b = B::new();
    let tx = b.alloc_qubits(N);
    b.declare_qubit_register(&tx);
    let ty = b.alloc_qubits(N);
    b.declare_qubit_register(&ty);
    let ox = b.alloc_bits(N);
    b.declare_bit_register(&ox);
    let oy = b.alloc_bits(N);
    b.declare_bit_register(&oy);
    let dialog_log = b.alloc_qubits(DIALOG_GCD_RAW_LOG_BITS);

    emit_dialog_gcd_raw_apply_bitvector(&mut b, &dialog_log, &tx, &ty, SECP256K1_P);

    b.set_phase("dialog_gcd_raw_apply_free_log");
    b.free_vec(&dialog_log);
    b
}

pub fn build_dialog_gcd_raw_apply_fit_bench_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_dialog_gcd_raw_apply_fit_bench_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub fn build_dialog_gcd_raw_apply_fit_bench() -> Vec<Op> {
    build_dialog_gcd_raw_apply_fit_bench_builder().ops
}

pub(crate) fn build_dialog_gcd_raw_ipmul_fit_bench_builder() -> B {
    let mut b = B::new();
    let tx = b.alloc_qubits(N);
    b.declare_qubit_register(&tx);
    let ty = b.alloc_qubits(N);
    b.declare_qubit_register(&ty);
    let ox = b.alloc_bits(N);
    b.declare_bit_register(&ox);
    let oy = b.alloc_bits(N);
    b.declare_bit_register(&oy);

    emit_dialog_gcd_raw_ipmul(&mut b, &tx, &ty, SECP256K1_P);
    let _ = (ox, oy);
    b
}

pub fn build_dialog_gcd_raw_ipmul_fit_bench_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_dialog_gcd_raw_ipmul_fit_bench_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub fn build_dialog_gcd_raw_ipmul_fit_bench() -> Vec<Op> {
    build_dialog_gcd_raw_ipmul_fit_bench_builder().ops
}

pub(crate) fn build_dialog_gcd_compressed_block_primitive_fit_bench_builder() -> B {
    let mut b = B::new();
    let tx = b.alloc_qubits(N);
    b.declare_qubit_register(&tx);
    let ty = b.alloc_qubits(N);
    b.declare_qubit_register(&ty);
    let ox = b.alloc_bits(N);
    b.declare_bit_register(&ox);
    let oy = b.alloc_bits(N);
    b.declare_bit_register(&oy);

    let block = b.alloc_qubits(6);
    let pair = b.alloc_qubits(2);

    b.set_phase("dialog_gcd_round763_compressor");
    emit_dialog_gcd_round763_compressor(&mut b, &block);
    b.set_phase("dialog_gcd_round763_compressor_inverse");
    emit_dialog_gcd_round763_compressor_inverse(&mut b, &block);

    b.set_phase("dialog_gcd_round763_compressed_block_swapper");
    emit_dialog_gcd_round763_compressed_block_swapper(&mut b, &pair, &block[..5], block[5], 1);
    let _ = (tx, ty, ox, oy);
    b
}

pub fn build_dialog_gcd_compressed_block_primitive_fit_bench_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_dialog_gcd_compressed_block_primitive_fit_bench_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub fn build_dialog_gcd_compressed_block_primitive_fit_bench() -> Vec<Op> {
    build_dialog_gcd_compressed_block_primitive_fit_bench_builder().ops
}

pub(crate) fn build_dialog_gcd_high_tail_alias_fit_bench_builder() -> B {
    let layout = dialog_gcd_high_tail_alias_layout();
    let mut b = if std::env::var("POINT_ADD_COUNT_ONLY").ok().as_deref() == Some("1") {
        B::new_count_only()
    } else {
        B::new()
    };
    let tx = b.alloc_qubits(N);
    b.declare_qubit_register(&tx);
    let ty = b.alloc_qubits(N);
    b.declare_qubit_register(&ty);
    let ox = b.alloc_bits(N);
    b.declare_bit_register(&ox);
    let oy = b.alloc_bits(N);
    b.declare_bit_register(&oy);

    b.set_phase("dialog_gcd_high_tail_alias_alloc_envelope");
    let u_ext = b.alloc_qubits(DIALOG_GCD_HIGH_TAIL_ALIAS_EXTENDED_BITS);
    let v_tail = b.alloc_qubits(DIALOG_GCD_HIGH_TAIL_ALIAS_EXTENSION_BITS);
    let borrowed_carries = b.alloc_qubits(N - 1);
    let absorber_scratch = b.alloc_qubits(4);
    assert_eq!(
        2 * N
            + DIALOG_GCD_HIGH_TAIL_ALIAS_EXTENDED_BITS
            + DIALOG_GCD_HIGH_TAIL_ALIAS_EXTENSION_BITS
            + (N - 1)
            + absorber_scratch.len(),
        DIALOG_GCD_HIGH_TAIL_ALIAS_PROJECTED_Q
    );

    b.set_phase("dialog_gcd_high_tail_alias_touch_u_ext");
    dialog_gcd_high_tail_touch_qubits_for_count(&mut b, &u_ext);
    b.set_phase("dialog_gcd_high_tail_alias_touch_v_tail");
    dialog_gcd_high_tail_touch_qubits_for_count(&mut b, &v_tail);
    b.set_phase("dialog_gcd_high_tail_alias_touch_borrowed_carries");
    dialog_gcd_high_tail_touch_qubits_for_count(&mut b, &borrowed_carries);
    b.set_phase("dialog_gcd_high_tail_alias_touch_absorber_scratch");
    dialog_gcd_high_tail_touch_qubits_for_count(&mut b, &absorber_scratch);

    b.set_phase("dialog_gcd_high_tail_alias_transcript_cells");
    for block in &layout.blocks {
        assert!(block.group < DIALOG_GCD_HIGH_TAIL_ALIAS_BLOCKS);
        assert!(block.first_step <= block.last_step);
        for &cell in &block.cells {
            let q = dialog_gcd_high_tail_alias_cell_qubit(cell, &u_ext, &tx, &v_tail);
            b.x(q);
            b.x(q);
        }
    }

    b.set_phase("dialog_gcd_high_tail_alias_apply_borrow_true_u");
    dialog_gcd_high_tail_touch_qubits_for_count(&mut b, &u_ext[..N]);

    if dialog_gcd_raw_ipmul_clear_p_residual_enabled() {
        b.set_phase("dialog_gcd_high_tail_alias_clear_p_residual_source_lane");
        for i in 0..N {
            if bit(SECP256K1_P, i) {
                b.x(ty[i]);
            }
        }
    }

    b.set_phase("dialog_gcd_high_tail_alias_free_envelope");
    b.free_vec(&absorber_scratch);
    b.free_vec(&borrowed_carries);
    b.free_vec(&v_tail);
    b.free_vec(&u_ext);
    let _ = (ox, oy);
    b
}

pub(crate) fn build_dialog_gcd_high_tail_transcript_overhead_bench_builder() -> B {
    let layout = dialog_gcd_high_tail_alias_layout();
    let mut b = if std::env::var("POINT_ADD_COUNT_ONLY").ok().as_deref() == Some("1") {
        B::new_count_only()
    } else {
        B::new()
    };
    let tx = b.alloc_qubits(N);
    b.declare_qubit_register(&tx);
    let ty = b.alloc_qubits(N);
    b.declare_qubit_register(&ty);
    let ox = b.alloc_bits(N);
    b.declare_bit_register(&ox);
    let oy = b.alloc_bits(N);
    b.declare_bit_register(&oy);

    b.set_phase("dialog_gcd_high_tail_transcript_alloc_envelope");
    let u_ext = b.alloc_qubits(DIALOG_GCD_HIGH_TAIL_ALIAS_EXTENDED_BITS);
    let v_tail = b.alloc_qubits(DIALOG_GCD_HIGH_TAIL_ALIAS_EXTENSION_BITS);
    let borrowed_carries = b.alloc_qubits(N - 1);
    let absorber_scratch = b.alloc_qubits(4);
    let pair = &absorber_scratch[..2];
    let scratch = absorber_scratch[2];

    emit_dialog_gcd_high_tail_transcript_overhead(
        &mut b, &layout, &u_ext, &tx, &v_tail, pair, scratch,
    );

    b.set_phase("dialog_gcd_high_tail_transcript_touch_borrowed_lanes");
    dialog_gcd_high_tail_touch_qubits_for_count(&mut b, &u_ext[..N]);
    dialog_gcd_high_tail_touch_qubits_for_count(&mut b, &borrowed_carries);

    if dialog_gcd_raw_ipmul_clear_p_residual_enabled() {
        b.set_phase("dialog_gcd_high_tail_transcript_clear_p_residual_source_lane");
        for i in 0..N {
            if bit(SECP256K1_P, i) {
                b.x(ty[i]);
            }
        }
    }

    b.set_phase("dialog_gcd_high_tail_transcript_free_envelope");
    b.free_vec(&absorber_scratch);
    b.free_vec(&borrowed_carries);
    b.free_vec(&v_tail);
    b.free_vec(&u_ext);
    let _ = (ox, oy);
    b
}

pub fn build_dialog_gcd_high_tail_alias_fit_bench_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_dialog_gcd_high_tail_alias_fit_bench_builder();
    let rows = if b.count_only {
        b.counted_phase_rows.clone()
    } else {
        phase_resources(&b.ops, &b.phase_transitions)
    };
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub fn build_dialog_gcd_high_tail_alias_fit_bench() -> Vec<Op> {
    build_dialog_gcd_high_tail_alias_fit_bench_builder().ops
}

pub fn build_dialog_gcd_high_tail_transcript_overhead_bench_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_dialog_gcd_high_tail_transcript_overhead_bench_builder();
    let rows = if b.count_only {
        b.counted_phase_rows.clone()
    } else {
        phase_resources(&b.ops, &b.phase_transitions)
    };
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub fn build_dialog_gcd_high_tail_transcript_overhead_bench() -> Vec<Op> {
    build_dialog_gcd_high_tail_transcript_overhead_bench_builder().ops
}

pub(crate) fn build_round125_jsf_operator_bench_builder() -> B {
    let mut b = B::new();
    let tx = b.alloc_qubits(N);
    b.declare_qubit_register(&tx);
    let ty = b.alloc_qubits(N);
    b.declare_qubit_register(&ty);
    let ox = b.alloc_bits(N);
    b.declare_bit_register(&ox);
    let oy = b.alloc_bits(N);
    b.declare_bit_register(&oy);

    b.set_phase("round125_jsf_operator_abi");
    round125_jsf::emit_round125_jsf_operator_roundtrip(&mut b, &tx, &ty);
    let _ = (ox, oy);
    b
}

pub fn build_round125_jsf_operator_bench_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round125_jsf_operator_bench_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub fn build_round125_jsf_operator_bench() -> Vec<Op> {
    build_round125_jsf_operator_bench_builder().ops
}

pub(crate) fn build_round158_numeric_endpoint_step_bench_builder() -> B {
    let mut b = B::new();
    let u = b.alloc_qubits(N);
    b.declare_qubit_register(&u);
    let v = b.alloc_qubits(N);
    b.declare_qubit_register(&v);
    let coeff_b = b.alloc_qubits(N + 1);
    b.declare_qubit_register(&coeff_b);
    let coeff_d = b.alloc_qubits(N + 1);
    b.declare_qubit_register(&coeff_d);
    let q = b.alloc_qubits(N);
    b.declare_qubit_register(&q);

    b.set_phase("round158_numeric_endpoint_step");
    round158_halfgcd_splice_live::emit_round158_numeric_endpoint_step(
        &mut b, &u, &v, &coeff_b, &coeff_d, &q,
    );

    b
}

pub fn build_round158_numeric_endpoint_step_bench_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round158_numeric_endpoint_step_bench_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub fn build_round158_numeric_endpoint_step_bench() -> Vec<Op> {
    build_round158_numeric_endpoint_step_bench_builder().ops
}

pub(crate) fn build_round197_numeric_endpoint_step_copy_clean_bench_builder() -> B {
    let mut b = B::new();
    let u = b.alloc_qubits(N);
    b.declare_qubit_register(&u);
    let v = b.alloc_qubits(N);
    b.declare_qubit_register(&v);
    let coeff_b = b.alloc_qubits(N + 1);
    b.declare_qubit_register(&coeff_b);
    let coeff_d = b.alloc_qubits(N + 1);
    b.declare_qubit_register(&coeff_d);
    let q = b.alloc_qubits(N);
    b.declare_qubit_register(&q);
    let u_out = b.alloc_qubits(N);
    b.declare_qubit_register(&u_out);
    let v_out = b.alloc_qubits(N);
    b.declare_qubit_register(&v_out);
    let coeff_b_out = b.alloc_qubits(N + 1);
    b.declare_qubit_register(&coeff_b_out);
    let coeff_d_out = b.alloc_qubits(N + 1);
    b.declare_qubit_register(&coeff_d_out);

    b.set_phase("round197_numeric_endpoint_step_copy_clean");
    round158_halfgcd_splice_live::emit_round197_numeric_endpoint_step_copy_clean(
        &mut b,
        &u,
        &v,
        &coeff_b,
        &coeff_d,
        &q,
        &u_out,
        &v_out,
        &coeff_b_out,
        &coeff_d_out,
    );

    b
}

pub fn build_round197_numeric_endpoint_step_copy_clean_bench_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round197_numeric_endpoint_step_copy_clean_bench_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub fn build_round197_numeric_endpoint_step_copy_clean_bench() -> Vec<Op> {
    build_round197_numeric_endpoint_step_copy_clean_bench_builder().ops
}

pub(crate) fn build_round197_numeric_endpoint_step_clean_q_from_coeff_bench_builder(
    initial_endpoint_step: bool,
    q_bits: usize,
) -> B {
    assert!((1..=N).contains(&q_bits));
    let mut b = B::new();
    let u = b.alloc_qubits(N);
    b.declare_qubit_register(&u);
    let v = b.alloc_qubits(N);
    b.declare_qubit_register(&v);
    let coeff_b = b.alloc_qubits(N + 1);
    b.declare_qubit_register(&coeff_b);
    let coeff_d = b.alloc_qubits(N + 1);
    b.declare_qubit_register(&coeff_d);
    let q = b.alloc_qubits(q_bits);
    b.declare_qubit_register(&q);

    b.set_phase(if initial_endpoint_step {
        "round197_numeric_endpoint_step_clean_q_from_coeff_initial"
    } else {
        "round197_numeric_endpoint_step_clean_q_from_coeff_body"
    });
    round158_halfgcd_splice_live::emit_round197_numeric_endpoint_step_clean_q_from_coeff(
        &mut b,
        &u,
        &v,
        &coeff_b,
        &coeff_d,
        &q,
        initial_endpoint_step,
    );

    b
}

pub fn build_round197_numeric_endpoint_step_clean_q_from_coeff_bench_phase_resources(
    initial_endpoint_step: bool,
    q_bits: usize,
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round197_numeric_endpoint_step_clean_q_from_coeff_bench_builder(
        initial_endpoint_step,
        q_bits,
    );
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub fn build_round197_numeric_endpoint_step_clean_q_from_coeff_bench(
    initial_endpoint_step: bool,
    q_bits: usize,
) -> Vec<Op> {
    build_round197_numeric_endpoint_step_clean_q_from_coeff_bench_builder(
        initial_endpoint_step,
        q_bits,
    )
    .ops
}

pub(crate) fn build_round146_halfgcd_decoder_reuse_bench_builder() -> B {
    let mut b = B::new();
    let tx = b.alloc_qubits(N);
    b.declare_qubit_register(&tx);
    let ty = b.alloc_qubits(N);
    b.declare_qubit_register(&ty);
    let ox = b.alloc_bits(N);
    b.declare_bit_register(&ox);
    let oy = b.alloc_bits(N);
    b.declare_bit_register(&oy);

    // Round145 observed max standalone decoder profile:
    // width=141, q_bits=13, forward T=10,816, standalone Q=439 in Python
    // KMX (438 in this Rust emitter because the carry wrapper is one qubit
    // tighter).  The PA ledger charges four semantic decoder uses; this bench
    // emits four roundtrips through the same quotient lane to make scratch
    // reuse visible in the Rust/KMX path.
    const WIDTH: usize = 141;
    const Q_BITS: usize = 13;
    let quotient = b.alloc_qubits(Q_BITS);
    for pass in 0..4 {
        b.set_phase(match pass {
            0 => "round146_halfgcd_decoder_reuse_pass0",
            1 => "round146_halfgcd_decoder_reuse_pass1",
            2 => "round146_halfgcd_decoder_reuse_pass2",
            _ => "round146_halfgcd_decoder_reuse_pass3",
        });
        emit_round146_decoder_roundtrip(&mut b, &tx[..WIDTH], &ty[..WIDTH], &quotient[..]);
    }
    b.free_vec(&quotient);

    b
}

pub fn build_round146_halfgcd_decoder_reuse_bench_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round146_halfgcd_decoder_reuse_bench_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub fn build_round146_halfgcd_decoder_reuse_bench() -> Vec<Op> {
    build_round146_halfgcd_decoder_reuse_bench_builder().ops
}

pub(crate) fn build_round146_halfgcd_decoder_sequence_bench_builder() -> B {
    let mut b = B::new();
    let tx = b.alloc_qubits(N);
    b.declare_qubit_register(&tx);
    let ty = b.alloc_qubits(N);
    b.declare_qubit_register(&ty);
    let ox = b.alloc_bits(N);
    b.declare_bit_register(&ox);
    let oy = b.alloc_bits(N);
    b.declare_bit_register(&oy);

    let profile = halfgcd_coeff_decoder::halfgcd_coeff_decoder_prefix_profile_round145(
        round146_semantic_max_divisor(),
    );
    let quotient = b.alloc_qubits(profile.max_q_bits);
    for pass in 0..4 {
        b.set_phase(match pass {
            0 => "round146_halfgcd_decoder_sequence_pass0",
            1 => "round146_halfgcd_decoder_sequence_pass1",
            2 => "round146_halfgcd_decoder_sequence_pass2",
            _ => "round146_halfgcd_decoder_sequence_pass3",
        });
        for step in &profile.steps {
            emit_round146_decoder_roundtrip(
                &mut b,
                &tx[..step.width],
                &ty[..step.width],
                &quotient[..step.q_bits],
            );
        }
    }
    b.free_vec(&quotient);

    b
}

pub fn build_round146_halfgcd_decoder_sequence_bench_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round146_halfgcd_decoder_sequence_bench_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub fn build_round146_halfgcd_decoder_sequence_bench() -> Vec<Op> {
    build_round146_halfgcd_decoder_sequence_bench_builder().ops
}
