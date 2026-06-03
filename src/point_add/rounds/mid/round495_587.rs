//! `mid::round495_587` — verbatim split of the original `mid` module.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

pub(crate) fn round495_d1_source_live_product_tail_pa_enabled() -> bool {
    std::env::var("ROUND495_D1_SOURCE_LIVE_PRODUCT_TAIL_PA")
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn round495_d1_source_live_cubic_tail_pa_enabled() -> bool {
    std::env::var("ROUND495_D1_SOURCE_LIVE_CUBIC_TAIL_PA")
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn round495_d1_source_live_pair2_iters() -> usize {
    let (env_name, value) = if let Ok(s) = std::env::var("ROUND495_D1_SOURCE_LIVE_PAIR2_ITERS") {
        ("ROUND495_D1_SOURCE_LIVE_PAIR2_ITERS", s)
    } else if let Ok(s) = std::env::var("KAL_PAIR2_ITERS") {
        ("KAL_PAIR2_ITERS", s)
    } else {
        return ROUND8_QTAIL_PAIR2_MIN_SAFE_ITERS;
    };
    let iters = value
        .parse::<usize>()
        .unwrap_or_else(|_| panic!("{env_name} must be a usize, got {value:?}"));
    checked_kaliski_iters(
        "round495 D1 source-live pair2",
        env_name,
        iters,
        ROUND8_QTAIL_PAIR2_MIN_SAFE_ITERS,
    )
}

pub(crate) fn round495_emit_d1_source_live_lam(
    b: &mut B,
    tx: &[QubitId],
    ty: &[QubitId],
    p: U256,
) -> Vec<QubitId> {
    let pair1_iters = round181_d1_pair1_iters();
    let lam = b.alloc_qubits(N);
    b.set_phase("round495_d1_source_live_pair1_compute_lambda");
    d1_direct_quotient_compute_into_zero(b, tx, ty, &lam, p, pair1_iters);
    b.set_phase("round495_d1_source_live_negate_lambda_for_tail");
    mod_neg_inplace_fast(b, &lam, p);
    lam
}

pub(crate) fn emit_round495_d1_source_live_product_tail_pa(
    b: &mut B,
    tx: &[QubitId],
    ty: &[QubitId],
    ox: &[BitId],
    oy: &[BitId],
    p: U256,
) {
    let lam = round495_emit_d1_source_live_lam(b, tx, ty, p);
    let pair2_iters = round495_d1_source_live_pair2_iters();
    emit_source_live_clean_product_tail(b, tx, ty, &lam, ox, oy, p, pair2_iters);
}

pub(crate) fn emit_round495_d1_source_live_cubic_tail_pa(
    b: &mut B,
    tx: &[QubitId],
    ty: &[QubitId],
    ox: &[BitId],
    oy: &[BitId],
    p: U256,
) {
    let lam = round495_emit_d1_source_live_lam(b, tx, ty, p);
    let pair2_iters = round495_d1_source_live_pair2_iters();
    let clean_lam = if std::env::var("PA_SOURCE_LIVE_CUBIC_HMR_PHASE_REPAIR")
        .ok()
        .as_deref()
        == Some("1")
    {
        SourceLiveCubicLamClean::HmrPhaseRepair {
            inverse_iters: pair2_iters,
        }
    } else if std::env::var("PA_SOURCE_LIVE_CUBIC_LAMBDA_CLEAN")
        .ok()
        .as_deref()
        == Some("1")
    {
        SourceLiveCubicLamClean::Inverse {
            inverse_iters: pair2_iters,
        }
    } else if std::env::var("PA_SOURCE_LIVE_CUBIC_PRODUCT_CLEAN")
        .ok()
        .as_deref()
        == Some("1")
    {
        SourceLiveCubicLamClean::Product {
            inverse_iters: pair2_iters,
        }
    } else {
        SourceLiveCubicLamClean::Dirty
    };
    let dirty = matches!(clean_lam, SourceLiveCubicLamClean::Dirty);
    emit_source_live_cubic_xtail_ytail(b, tx, ty, &lam, ox, oy, p, clean_lam);
    if dirty {
        b.set_phase("source_live_cubic_dirty_free_lam");
        b.free_vec(&lam);
    }
}

pub(crate) fn round499_zero_condition_hmr_erase_enabled() -> bool {
    if std::env::var("ROUND499_ZERO_COND_HMR_ERASE")
        .ok()
        .as_deref()
        != Some("1")
    {
        return false;
    }
    assert!(
        std::env::var("ROUND499_ALLOW_GOOGLE_FAILING_PROBE")
            .ok()
            .as_deref()
            == Some("1"),
        "ROUND499_ZERO_COND_HMR_ERASE is invalid under the Zenodo/Google simulator: \
         condition-false HMR does not reset the qubit.  Use \
         ROUND499_ALLOW_GOOGLE_FAILING_PROBE=1 only to reproduce the rejected probe."
    );
    true
}

pub(crate) fn round556_shifted_source_row_width_from_env() -> usize {
    std::env::var(ROUND556_SHIFTED_SOURCE_ROW_WIDTH_ENV)
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|&width| width >= 2)
        .unwrap_or(258)
}

pub(crate) fn round556_shifted_source_row_qbits_from_env(width: usize) -> usize {
    std::env::var(ROUND556_SHIFTED_SOURCE_ROW_QBITS_ENV)
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|&q_bits| (1..=width).contains(&q_bits))
        .unwrap_or(26.min(width))
}

pub(crate) fn round556_cmp_lt_shifted_source_into_fast(
    b: &mut B,
    left: &[QubitId],
    source: &[QubitId],
    shift: usize,
    flag: QubitId,
) {
    assert_eq!(left.len(), source.len());
    assert!(shift < left.len());

    // Compare left < (source << shift) without allocating the shifted word.
    // The low shifted positions are compile-time zero, so they cannot create a
    // borrow/carry from the initial zero carry-in.  The comparison is therefore
    // exactly left[shift..] < source[..width-shift].
    let active = left.len() - shift;
    cmp_lt_into_fast(b, &left[shift..], &source[..active], flag);
}

pub(crate) fn round556_cmp_lt_shifted_source_into_borrowed_cin(
    b: &mut B,
    left: &[QubitId],
    source: &[QubitId],
    shift: usize,
    flag: QubitId,
    cmp_cin: QubitId,
) {
    assert_eq!(left.len(), source.len());
    assert!(shift < left.len());

    let active = left.len() - shift;
    cmp_lt_into_borrowed_cin(b, &left[shift..], &source[..active], flag, cmp_cin);
}

pub(crate) fn round556_cmp_lt_shifted_source_into_fast_borrowed_carries(
    b: &mut B,
    left: &[QubitId],
    source: &[QubitId],
    shift: usize,
    flag: QubitId,
    cmp_cin: QubitId,
    cmp_carries: &[QubitId],
) {
    assert_eq!(left.len(), source.len());
    assert!(shift < left.len());

    let active = left.len() - shift;
    cmp_lt_into_fast_borrowed_carries(
        b,
        &left[shift..],
        &source[..active],
        flag,
        cmp_cin,
        &cmp_carries[..active],
    );
}

pub(crate) fn round556_load_gated_shifted_source(
    b: &mut B,
    control: QubitId,
    source: &[QubitId],
    shift: usize,
    gated: &[QubitId],
) {
    assert_eq!(source.len(), gated.len());
    assert!(shift < source.len());
    for bit in shift..source.len() {
        b.ccx(control, source[bit - shift], gated[bit]);
    }
}

pub(crate) fn round556_clear_gated_shifted_source_hmr(
    b: &mut B,
    control: QubitId,
    source: &[QubitId],
    shift: usize,
    gated: &[QubitId],
) {
    assert_eq!(source.len(), gated.len());
    assert!(shift < source.len());
    for bit in (shift..source.len()).rev() {
        let m = b.alloc_bit();
        b.hmr(gated[bit], m);
        b.cz_if(control, source[bit - shift], m);
        b.bit_store0(m);
    }
}

pub(crate) fn round556_load_gated_word(b: &mut B, control: QubitId, source: &[QubitId], gated: &[QubitId]) {
    assert_eq!(source.len(), gated.len());
    for i in 0..source.len() {
        b.ccx(control, source[i], gated[i]);
    }
}

pub(crate) fn round556_clear_gated_word_hmr(
    b: &mut B,
    control: QubitId,
    source: &[QubitId],
    gated: &[QubitId],
) {
    assert_eq!(source.len(), gated.len());
    for i in (0..source.len()).rev() {
        let m = b.alloc_bit();
        b.hmr(gated[i], m);
        b.cz_if(control, source[i], m);
        b.bit_store0(m);
    }
}

pub(crate) fn round556_fused_sign_controlled_addsub_digit(
    b: &mut B,
    acc: &[QubitId],
    addend: &[QubitId],
    sign: QubitId,
) {
    assert_eq!(acc.len(), addend.len());
    assert!(acc.len() >= 2);

    // sign=1 means add; sign=0 means subtract.  Toggle the addend and carry-in
    // by !sign, then run the same HMR-clean Cuccaro adder for both cases.
    let nonnegative = b.alloc_qubit();
    b.x(nonnegative);
    b.cx(sign, nonnegative);
    for &wire in addend {
        b.cx(nonnegative, wire);
    }
    cuccaro_add_fast(b, addend, acc, nonnegative);
    for &wire in addend.iter().rev() {
        b.cx(nonnegative, wire);
    }
    b.cx(sign, nonnegative);
    b.x(nonnegative);
    b.free(nonnegative);
}

pub(crate) fn round556_fused_sign_controlled_addsub_digit_borrowed(
    b: &mut B,
    acc: &[QubitId],
    addend: &[QubitId],
    sign: QubitId,
    nonnegative: QubitId,
    carries: &[QubitId],
) {
    assert_eq!(acc.len(), addend.len());
    assert!(acc.len() >= 2);
    assert!(carries.len() >= acc.len() - 1);

    // Same add/sub convention as `round556_fused_sign_controlled_addsub_digit`,
    // but the caller supplies the one-bit sign scratch and the carry lane.
    b.x(nonnegative);
    b.cx(sign, nonnegative);
    for &wire in addend {
        b.cx(nonnegative, wire);
    }
    cuccaro_add_fast_borrowed_carries(b, addend, acc, nonnegative, &carries[..acc.len() - 1]);
    for &wire in addend.iter().rev() {
        b.cx(nonnegative, wire);
    }
    b.cx(sign, nonnegative);
    b.x(nonnegative);
}

pub(crate) fn round556_emit_forward_remainder_digit(
    b: &mut B,
    rem: &[QubitId],
    rem_divisor: &[QubitId],
    gated: &[QubitId],
    lt_tmp: QubitId,
    qbit: QubitId,
    shift: usize,
) {
    round556_cmp_lt_shifted_source_into_fast(b, rem, rem_divisor, shift, lt_tmp);
    b.x(lt_tmp);
    b.cx(lt_tmp, qbit);
    b.x(lt_tmp);
    b.x(qbit);
    b.cx(qbit, lt_tmp);
    b.x(qbit);
    round556_load_gated_shifted_source(b, qbit, rem_divisor, shift, gated);
    round556_fused_sign_controlled_addsub_digit(b, rem, gated, lt_tmp);
    round556_clear_gated_shifted_source_hmr(b, qbit, rem_divisor, shift, gated);
}

pub(crate) fn round556_emit_forward_remainder_digit_borrowed(
    b: &mut B,
    rem: &[QubitId],
    rem_divisor: &[QubitId],
    gated: &[QubitId],
    lt_tmp: QubitId,
    nonnegative: QubitId,
    cmp_carries: &[QubitId],
    carries: &[QubitId],
    qbit: QubitId,
    shift: usize,
) {
    round556_cmp_lt_shifted_source_into_fast_borrowed_carries(
        b,
        rem,
        rem_divisor,
        shift,
        lt_tmp,
        nonnegative,
        cmp_carries,
    );
    b.x(lt_tmp);
    b.cx(lt_tmp, qbit);
    b.x(lt_tmp);
    b.x(qbit);
    b.cx(qbit, lt_tmp);
    b.x(qbit);
    round556_load_gated_shifted_source(b, qbit, rem_divisor, shift, gated);
    round556_fused_sign_controlled_addsub_digit_borrowed(
        b,
        rem,
        gated,
        lt_tmp,
        nonnegative,
        carries,
    );
    round556_clear_gated_shifted_source_hmr(b, qbit, rem_divisor, shift, gated);
}

pub(crate) fn round556_emit_coeff_update_erase_digit(
    b: &mut B,
    coeff: &[QubitId],
    coeff_divisor: &[QubitId],
    gated: &[QubitId],
    sign_one: QubitId,
    qbit: QubitId,
    shift: usize,
) {
    round556_load_gated_shifted_source(b, qbit, coeff_divisor, shift, gated);
    round556_fused_sign_controlled_addsub_digit(b, coeff, gated, sign_one);
    round556_clear_gated_shifted_source_hmr(b, qbit, coeff_divisor, shift, gated);
    b.x(qbit);
    round556_cmp_lt_shifted_source_into_fast(b, coeff, coeff_divisor, shift, qbit);
}

pub(crate) fn round556_emit_coeff_update_erase_digit_borrowed(
    b: &mut B,
    coeff: &[QubitId],
    coeff_divisor: &[QubitId],
    gated: &[QubitId],
    sign_one: QubitId,
    nonnegative: QubitId,
    cmp_carries: &[QubitId],
    carries: &[QubitId],
    qbit: QubitId,
    shift: usize,
) {
    round556_load_gated_shifted_source(b, qbit, coeff_divisor, shift, gated);
    round556_fused_sign_controlled_addsub_digit_borrowed(
        b,
        coeff,
        gated,
        sign_one,
        nonnegative,
        carries,
    );
    round556_clear_gated_shifted_source_hmr(b, qbit, coeff_divisor, shift, gated);
    b.x(qbit);
    round556_cmp_lt_shifted_source_into_fast_borrowed_carries(
        b,
        coeff,
        coeff_divisor,
        shift,
        qbit,
        nonnegative,
        cmp_carries,
    );
}

pub(crate) fn round556_emit_word_controlled_add(
    b: &mut B,
    coeff: &[QubitId],
    coeff_divisor: &[QubitId],
    gated: &[QubitId],
    sign_one: QubitId,
    control: QubitId,
) {
    round556_load_gated_word(b, control, coeff_divisor, gated);
    round556_fused_sign_controlled_addsub_digit(b, coeff, gated, sign_one);
    round556_clear_gated_word_hmr(b, control, coeff_divisor, gated);
}

pub(crate) fn round556_emit_sigma_subtract(
    b: &mut B,
    coeff: &[QubitId],
    coeff_divisor: &[QubitId],
    gated: &[QubitId],
    sigma: QubitId,
    sign_zero: QubitId,
) {
    round556_load_gated_word(b, sigma, coeff_divisor, gated);
    round556_fused_sign_controlled_addsub_digit(b, coeff, gated, sign_zero);
    round556_clear_gated_word_hmr(b, sigma, coeff_divisor, gated);
}

pub(crate) fn emit_round556_shifted_source_row_component(
    b: &mut B,
    rem: &[QubitId],
    rem_divisor: &[QubitId],
    coeff: &[QubitId],
    coeff_divisor: &[QubitId],
    sigma: QubitId,
    q_increment: QubitId,
    lt_tmp: QubitId,
    sign_one: QubitId,
    qbits: &[QubitId],
    gated: &[QubitId],
) {
    let width = rem.len();
    assert_eq!(rem_divisor.len(), width);
    assert_eq!(coeff.len(), width);
    assert_eq!(coeff_divisor.len(), width);
    assert_eq!(gated.len(), width);
    assert!(!qbits.is_empty());
    assert!(qbits.len() <= width);

    b.set_phase("round556_shifted_source_remainder_digits");
    for q_index in (0..qbits.len()).rev() {
        round556_emit_forward_remainder_digit(
            b,
            rem,
            rem_divisor,
            gated,
            lt_tmp,
            qbits[q_index],
            q_index,
        );
    }

    b.set_phase("round556_shifted_source_coeff_digits");
    b.x(sign_one);
    for q_index in 0..qbits.len() {
        round556_emit_coeff_update_erase_digit(
            b,
            coeff,
            coeff_divisor,
            gated,
            sign_one,
            qbits[q_index],
            q_index,
        );
    }
    round556_emit_word_controlled_add(b, coeff, coeff_divisor, gated, sign_one, q_increment);
    b.x(sign_one);

    b.set_phase("round556_shifted_source_sigma_subtract");
    round556_emit_sigma_subtract(b, coeff, coeff_divisor, gated, sigma, lt_tmp);
}

pub(crate) fn build_round556_shifted_source_row_component_builder(width: usize, q_bits: usize) -> B {
    assert!(width >= 2);
    assert!((1..=width).contains(&q_bits));

    let mut b = B::new();
    let rem = b.alloc_qubits(width);
    b.declare_qubit_register(&rem);
    let rem_divisor = b.alloc_qubits(width);
    b.declare_qubit_register(&rem_divisor);
    let coeff = b.alloc_qubits(width);
    b.declare_qubit_register(&coeff);
    let coeff_divisor = b.alloc_qubits(width);
    b.declare_qubit_register(&coeff_divisor);
    let sigma = b.alloc_qubit();
    let q_increment = b.alloc_qubit();
    let lt_tmp = b.alloc_qubit();
    let sign_one = b.alloc_qubit();
    let qbits = b.alloc_qubits(q_bits);
    let mut meta = vec![sigma, q_increment, lt_tmp, sign_one];
    meta.extend(qbits.iter().copied());
    b.declare_qubit_register(&meta);
    let gated = b.alloc_qubits(width);

    emit_round556_shifted_source_row_component(
        &mut b,
        &rem,
        &rem_divisor,
        &coeff,
        &coeff_divisor,
        sigma,
        q_increment,
        lt_tmp,
        sign_one,
        &qbits,
        &gated,
    );

    b.set_phase("round556_shifted_source_free_gated");
    b.free_vec(&gated);
    b
}

pub fn build_round556_shifted_source_row_component(width: usize, q_bits: usize) -> Vec<Op> {
    build_round556_shifted_source_row_component_builder(width, q_bits).ops
}

pub fn build_round556_shifted_source_row_component_phase_resources(
    width: usize,
    q_bits: usize,
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round556_shifted_source_row_component_builder(width, q_bits);
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub(crate) fn round564_compute_polarized_d(
    b: &mut B,
    x: &[QubitId],
    ox: &[BitId],
    oy: &[BitId],
    eq: QubitId,
    p: U256,
) -> Vec<QubitId> {
    let d = load_bits(b, ox);
    mod_sub_qq_fast(b, &d, x, p);

    let two_oy = load_bits(b, oy);
    mod_double_inplace_fast(b, &two_oy, p);
    cmod_add_qq(b, &d, &two_oy, eq, p);
    mod_halve_inplace_fast(b, &two_oy, p);
    unload_bits(b, &two_oy, oy);
    d
}

pub(crate) fn round564_uncompute_polarized_d(
    b: &mut B,
    d: &[QubitId],
    x: &[QubitId],
    ox: &[BitId],
    oy: &[BitId],
    eq: QubitId,
    p: U256,
) {
    let two_oy = load_bits(b, oy);
    mod_double_inplace_fast(b, &two_oy, p);
    cmod_sub_qq(b, d, &two_oy, eq, p);
    mod_halve_inplace_fast(b, &two_oy, p);
    unload_bits(b, &two_oy, oy);

    mod_add_qq_fast(b, d, x, p);
    unload_bits(b, d, ox);
}

pub(crate) fn round564_square_add_selected(b: &mut B, acc: &[QubitId], x: &[QubitId], p: U256) {
    if std::env::var("ROUND564_YTAIL_PHASE_CLEAN_SQUARES")
        .ok()
        .as_deref()
        == Some("1")
    {
        squaring_add_to_acc_schoolbook_phase_clean(b, acc, x, p);
    } else {
        squaring_add_to_acc_schoolbook(b, acc, x, p);
    }
}

pub(crate) fn round564_square_sub_selected(b: &mut B, acc: &[QubitId], x: &[QubitId], p: U256) {
    if std::env::var("ROUND564_YTAIL_PHASE_CLEAN_SQUARES")
        .ok()
        .as_deref()
        == Some("1")
    {
        squaring_sub_from_acc_schoolbook_phase_clean(b, acc, x, p);
    } else {
        squaring_sub_from_acc_schoolbook_lowq_shift22(b, acc, x, p);
    }
}

pub(crate) fn round564_emit_polarized_third_square_ytail_component(
    b: &mut B,
    x: &[QubitId],
    u: &[QubitId],
    ox: &[BitId],
    oy: &[BitId],
    y_acc: &[QubitId],
    p: U256,
) {
    debug_assert_eq!(x.len(), N);
    debug_assert_eq!(u.len(), N);
    debug_assert_eq!(ox.len(), N);
    debug_assert_eq!(oy.len(), N);
    debug_assert_eq!(y_acc.len(), N);

    b.set_phase("round564_ytail_eq_diff");
    let eq_diff = load_bits(b, ox);
    mod_sub_qq_fast(b, &eq_diff, x, p);
    let eq = b.alloc_qubit();
    toggle_eq_zero_flag_fast(b, &eq_diff, eq);

    b.set_phase("round564_ytail_compute_d");
    let d = round564_compute_polarized_d(b, x, ox, oy, eq, p);

    b.set_phase("round564_ytail_compute_u_plus_d");
    let u_plus_d = b.alloc_qubits(N);
    for i in 0..N {
        b.cx(u[i], u_plus_d[i]);
    }
    mod_add_qq_fast(b, &u_plus_d, &d, p);

    b.set_phase("round564_ytail_add_u_plus_d_square");
    round564_square_add_selected(b, y_acc, &u_plus_d, p);
    b.set_phase("round564_ytail_sub_u_square");
    round564_square_sub_selected(b, y_acc, u, p);
    b.set_phase("round564_ytail_sub_d_square");
    round564_square_sub_selected(b, y_acc, &d, p);

    b.set_phase("round564_ytail_halve_polarized_product");
    mod_halve_inplace_fast(b, y_acc, p);
    b.set_phase("round564_ytail_sub_offset_y");
    mod_sub_qb(b, y_acc, oy, p);

    b.set_phase("round564_ytail_compute_derivative_square");
    let ox_q = load_bits(b, ox);
    let derivative = b.alloc_qubits(N);
    round564_square_add_selected(b, &derivative, &ox_q, p);
    b.set_phase("round564_ytail_sub_eq_derivative_x3");
    for _ in 0..3 {
        cmod_sub_qq(b, y_acc, &derivative, eq, p);
    }
    b.set_phase("round564_ytail_uncompute_derivative_square");
    round564_square_sub_selected(b, &derivative, &ox_q, p);
    b.free_vec(&derivative);
    unload_bits(b, &ox_q, ox);

    b.set_phase("round564_ytail_uncompute_u_plus_d");
    mod_sub_qq_fast(b, &u_plus_d, &d, p);
    for i in (0..N).rev() {
        b.cx(u[i], u_plus_d[i]);
    }
    b.free_vec(&u_plus_d);

    b.set_phase("round564_ytail_uncompute_d");
    round564_uncompute_polarized_d(b, &d, x, ox, oy, eq, p);

    b.set_phase("round564_ytail_uncompute_eq");
    toggle_eq_zero_flag_fast(b, &eq_diff, eq);
    b.free(eq);
    mod_add_qq_fast(b, &eq_diff, x, p);
    unload_bits(b, &eq_diff, ox);
}

pub fn build_round564_polarized_third_square_ytail_component() -> Vec<Op> {
    build_round564_polarized_third_square_ytail_component_builder().ops
}

pub(crate) fn build_round564_polarized_third_square_ytail_component_builder() -> B {
    let mut b = B::new();
    let x = b.alloc_qubits(N);
    b.declare_qubit_register(&x);
    let u = b.alloc_qubits(N);
    b.declare_qubit_register(&u);
    let ox = b.alloc_bits(N);
    b.declare_bit_register(&ox);
    let oy = b.alloc_bits(N);
    b.declare_bit_register(&oy);
    let y_acc = b.alloc_qubits(N);
    b.declare_qubit_register(&y_acc);

    round564_emit_polarized_third_square_ytail_component(
        &mut b,
        &x,
        &u,
        &ox,
        &oy,
        &y_acc,
        SECP256K1_P,
    );
    b
}

pub fn build_round564_polarized_third_square_ytail_component_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round564_polarized_third_square_ytail_component_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}

pub(crate) fn round587_wide_raw_horner_consumer(
    b: &mut B,
    numerator: &[QubitId],
    quotient: &[QubitId],
    raw_r: &[QubitId],
    p: U256,
    inverse_iters: usize,
) {
    debug_assert_eq!(numerator.len(), N);
    debug_assert_eq!(quotient.len(), N);
    debug_assert_eq!(raw_r.len(), 399);

    b.set_phase("round587_wide_raw_horner_consume_399");
    for i in (0..raw_r.len()).rev() {
        if i + 1 < raw_r.len() {
            mod_double_inplace_fast(b, quotient, p);
        }
        cmod_add_qq(b, quotient, numerator, raw_r[i], p);
    }

    b.set_phase("round587_wide_raw_horner_unscale");
    mod_neg_inplace_fast(b, quotient, p);
    for _ in 0..inverse_iters {
        mod_halve_inplace_fast(b, quotient, p);
    }
}

pub(crate) fn build_round587_packed_owner_wide_raw_splice_builder() -> B {
    let mut b = B::new();

    let numerator = b.alloc_qubits(N);
    b.declare_qubit_register(&numerator);
    let quotient = b.alloc_qubits(N);
    b.declare_qubit_register(&quotient);

    // Proposed fixed-depth packed Kaliski owner skeleton:
    //   packed_us = u low bits || reversed s high bits, 257 bits
    //   packed_vr = terminal v plus raw r lane, 399 bits
    //   m_hist    = one fixed-depth branch bit per live production iteration
    let packed_us = b.alloc_qubits(257);
    b.declare_qubit_register(&packed_us);
    let packed_vr_raw = b.alloc_qubits(399);
    b.declare_qubit_register(&packed_vr_raw);
    let m_hist = b.alloc_qubits(399);
    b.declare_qubit_register(&m_hist);

    round587_wide_raw_horner_consumer(
        &mut b,
        &numerator,
        &quotient,
        &packed_vr_raw,
        SECP256K1_P,
        400,
    );

    let _ = (packed_us, m_hist);
    b
}

pub fn build_round587_packed_owner_wide_raw_splice_component() -> Vec<Op> {
    build_round587_packed_owner_wide_raw_splice_builder().ops
}

pub fn build_round587_packed_owner_wide_raw_splice_phase_resources(
) -> (Vec<Op>, Vec<PhaseResource>, u32, &'static str) {
    let b = build_round587_packed_owner_wide_raw_splice_builder();
    let rows = phase_resources(&b.ops, &b.phase_transitions);
    (b.ops, rows, b.peak_qubits, b.peak_phase)
}
