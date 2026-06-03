//! `bench::scaffold` — verbatim split of the original `bench` module.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

pub(crate) fn emit_scaled_by_pattern_replay_benchmark_scaffold(b: &mut B, p: U256) {
    // Benchmark-path integration smoke test for the scaled-BY thesis.  This is
    // deliberately a clean no-op (all controls/data start at zero), appended
    // after the exact point-add output is already computed.  It lets the main
    // harness, alternate-seed check, qubit analyzer, and free-clean checks see a
    // real 560-step scaled-BY replay with the intended raw-pattern qubit shape:
    // 560 persistent odd-pattern bits plus one 16-bit A-control scratch window.
    // It is not the SOTA replacement path; it is a correctness/width/cost hook
    // that proves the replay body can live inside the benchmark circuit.
    b.set_phase("by_pattern_replay_bench_alloc");
    let odd_pattern = b.alloc_qubits(560);
    let a_window = b.alloc_qubits(16);
    let r = b.alloc_qubits(N);
    let s = b.alloc_qubits(N);
    b.set_phase("by_pattern_replay_bench_560");
    for i in 0..560 {
        scaled_by_controlled_microstep(b, &r, &s, odd_pattern[i], a_window[i & 15], p);
    }
    b.set_phase("by_pattern_replay_bench_free");
    b.free_vec(&s);
    b.free_vec(&r);
    b.free_vec(&a_window);
    b.free_vec(&odd_pattern);
}

pub(crate) fn emit_centered_signed_by_replay_body_benchmark_scaffold(b: &mut B, p: U256) {
    // Harness integration smoke test for the centered signed redundant replay.
    // Reuses one zero odd/A/parity control so the clean no-op fits next to the
    // live point-add outputs; this exercises the 873.6k-CCX body without adding
    // the still-unsolved persistent parity/history bank to the default circuit.
    const WIDE: usize = N + 4;
    b.set_phase("by_centered_replay_body_bench_alloc");
    let odd = b.alloc_qubit();
    let a = b.alloc_qubit();
    let parity = b.alloc_qubit();
    let r = b.alloc_qubits(WIDE);
    let s = b.alloc_qubits(WIDE);
    b.set_phase("by_centered_replay_body_bench_560");
    for _ in 0..560 {
        centered_signed_by_microstep_for_bench(b, &r, &s, odd, a, parity, p);
    }
    b.set_phase("by_centered_replay_body_bench_free");
    b.free_vec(&s);
    b.free_vec(&r);
    b.free(parity);
    b.free(a);
    b.free(odd);
}

pub(crate) fn emit_centered_signed_by_clean_roundtrip_benchmark_scaffold(b: &mut B, p: U256) {
    // Production-harness smoke test for the all-exact clean centered replay
    // fallback.  It appends a net no-op after point-add: 560 forward steps
    // using a fixed real BY control trace from the by.rs clean-560 sampler,
    // parity recomputation from restored rows.  This intentionally carries the
    // full raw odd/A/parity history, matching the 3.2M-CCX clean fallback shape
    // from by.rs; it is a smoke hook, not a SOTA path.
    const WIDE: usize = N + 4;
    const ODD_WORDS: [u64; 9] = [
        0x9f0102a4a879b9a7,
        0x39950f607ecb1db3,
        0xefaf7e99e64fb43a,
        0x6f3857abf7ed1f44,
        0x5b90e29f6d3d3b0c,
        0xb9f3f86e0ff7143e,
        0xb54e3a746addb473,
        0xd88e00e18c323864,
        0x00000000066e560a,
    ];
    const A_WORDS: [u64; 9] = [
        0x9501008408488925,
        0x0881002054411510,
        0x2525548924450402,
        0x2508548955211544,
        0x4910209521111104,
        0x8911080205550412,
        0x9542124422548410,
        0x4802002104120824,
        0x0000000002220202,
    ];
    const START_S_WORDS: [u64; 5] = [
        0x543668999ebc619a,
        0xe53862dc6983ea27,
        0x70aaecb9190602dd,
        0x0d5ac6c9f6d54fca,
        0x0000000000000000,
    ];
    b.set_phase("by_centered_clean_roundtrip_bench_alloc");
    let odd = b.alloc_qubits(560);
    let a_ctrl = b.alloc_qubits(560);
    let parity = b.alloc_qubits(560);
    let r = b.alloc_qubits(WIDE);
    let s = b.alloc_qubits(WIDE);
    for i in 0..560 {
        if ((ODD_WORDS[i / 64] >> (i % 64)) & 1) != 0 {
            b.x(odd[i]);
        }
        if ((A_WORDS[i / 64] >> (i % 64)) & 1) != 0 {
            b.x(a_ctrl[i]);
        }
    }
    // Centered tagged input for the fixed sampler pair; r=0.
    for i in 0..WIDE {
        if ((START_S_WORDS[i / 64] >> (i % 64)) & 1) != 0 {
            b.x(s[i]);
        }
    }
    b.set_phase("by_centered_clean_roundtrip_bench_forward");
    for i in 0..560 {
        centered_signed_by_microstep_all_exact_for_bench(
            b, &r, &s, odd[i], a_ctrl[i], parity[i], p,
        );
    }
    b.set_phase("by_centered_clean_roundtrip_bench_inverse");
    for i in (0..560).rev() {
        centered_signed_by_microstep_inverse_all_exact_for_bench(
            b, &r, &s, odd[i], a_ctrl[i], parity[i], p,
        );
        centered_signed_by_clear_parity_after_inverse_for_bench(b, &r, &s, odd[i], parity[i]);
    }
    b.set_phase("by_centered_clean_roundtrip_bench_free");
    for i in 0..WIDE {
        if ((START_S_WORDS[i / 64] >> (i % 64)) & 1) != 0 {
            b.x(s[i]);
        }
    }
    for i in 0..560 {
        if ((A_WORDS[i / 64] >> (i % 64)) & 1) != 0 {
            b.x(a_ctrl[i]);
        }
        if ((ODD_WORDS[i / 64] >> (i % 64)) & 1) != 0 {
            b.x(odd[i]);
        }
    }
    // Leave the zeroed scratch allocated in this smoke hook. If any of it is
    // nonzero the ancilla-garbage checker catches it directly; avoiding R here
    // keeps the hook from hiding restoration bugs behind reset phase noise.
    let _ = (odd, a_ctrl, parity, r, s);
}

pub(crate) fn emit_centered_signed_by_fast_clean_roundtrip_benchmark_scaffold(b: &mut B, p: U256) {
    // Same fixed-trace clean roundtrip as BY_CENTERED_CLEAN_ROUNDTRIP_BENCH,
    // but using the fast MBU centered signed replay body.  This is the quickest
    // harness check after the unhalve sign-history fix: if this passes, the
    // sub-million centered replay body is compatible with real parity cleanup.
    const WIDE: usize = N + 4;
    const ODD_WORDS: [u64; 9] = [
        0x9f0102a4a879b9a7,
        0x39950f607ecb1db3,
        0xefaf7e99e64fb43a,
        0x6f3857abf7ed1f44,
        0x5b90e29f6d3d3b0c,
        0xb9f3f86e0ff7143e,
        0xb54e3a746addb473,
        0xd88e00e18c323864,
        0x00000000066e560a,
    ];
    const A_WORDS: [u64; 9] = [
        0x9501008408488925,
        0x0881002054411510,
        0x2525548924450402,
        0x2508548955211544,
        0x4910209521111104,
        0x8911080205550412,
        0x9542124422548410,
        0x4802002104120824,
        0x0000000002220202,
    ];
    const START_S_WORDS: [u64; 5] = [
        0x543668999ebc619a,
        0xe53862dc6983ea27,
        0x70aaecb9190602dd,
        0x0d5ac6c9f6d54fca,
        0x0000000000000000,
    ];
    b.set_phase("by_centered_fast_clean_roundtrip_bench_alloc");
    let odd = b.alloc_qubits(560);
    let a_ctrl = b.alloc_qubits(560);
    let parity = b.alloc_qubits(560);
    let r = b.alloc_qubits(WIDE);
    let s = b.alloc_qubits(WIDE);
    for i in 0..560 {
        if ((ODD_WORDS[i / 64] >> (i % 64)) & 1) != 0 {
            b.x(odd[i]);
        }
        if ((A_WORDS[i / 64] >> (i % 64)) & 1) != 0 {
            b.x(a_ctrl[i]);
        }
    }
    for i in 0..WIDE {
        if ((START_S_WORDS[i / 64] >> (i % 64)) & 1) != 0 {
            b.x(s[i]);
        }
    }
    b.set_phase("by_centered_fast_clean_roundtrip_bench_forward");
    for i in 0..560 {
        centered_signed_by_microstep_for_bench(b, &r, &s, odd[i], a_ctrl[i], parity[i], p);
    }
    b.set_phase("by_centered_fast_clean_roundtrip_bench_inverse");
    for i in (0..560).rev() {
        centered_signed_by_microstep_inverse_for_bench(b, &r, &s, odd[i], a_ctrl[i], parity[i], p);
        centered_signed_by_clear_parity_after_inverse_for_bench(b, &r, &s, odd[i], parity[i]);
    }
    b.set_phase("by_centered_fast_clean_roundtrip_bench_free");
    for i in 0..WIDE {
        if ((START_S_WORDS[i / 64] >> (i % 64)) & 1) != 0 {
            b.x(s[i]);
        }
    }
    for i in 0..560 {
        if ((A_WORDS[i / 64] >> (i % 64)) & 1) != 0 {
            b.x(a_ctrl[i]);
        }
        if ((ODD_WORDS[i / 64] >> (i % 64)) & 1) != 0 {
            b.x(odd[i]);
        }
    }
    let _ = (odd, a_ctrl, parity, r, s);
}

pub(crate) fn emit_single_inv_strategy_c_shape_benchmark_scaffold(b: &mut B, p: U256) {
    // Hardest-piece-first probe for the one-division family. This is not a
    // point-add replacement; it is a clean shape benchmark for a Strategy-C-like
    // scaffold: one inversion on dx^3, plus the surrounding square/multiply
    // chain that a real one-DIV path would need to carry.
    const ITERS: usize = 404;
    let lowq_unv_square = std::env::var("SINGLE_INV_C_LOWQ_UNV_SQUARE")
        .ok()
        .as_deref()
        == Some("1");
    let lowq_undx2 = std::env::var("SINGLE_INV_C_LOWQ_UNDX2").ok().as_deref() == Some("1");
    let skip_ry = std::env::var("SINGLE_INV_C_SKIP_RY").ok().as_deref() == Some("1");
    b.set_phase("single_inv_c_shape_alloc");
    let dx = b.alloc_qubits(N);
    let dy = b.alloc_qubits(N);
    let dx2 = b.alloc_qubits(N);
    let w = b.alloc_qubits(N);
    init_small_const_reg(b, &dx, 3);
    init_small_const_reg(b, &dy, 5);

    b.set_phase("single_inv_c_shape_dx2");
    squaring_add_to_acc_schoolbook(b, &dx2, &dx, p);
    b.set_phase("single_inv_c_shape_w");
    mod_mul_write_into_zero_acc_schoolbook(b, &w, &dx2, &dx, p);

    b.set_phase("single_inv_c_shape_inv");
    with_kal_inv_raw(b, &w, p, ITERS, |b, inv_raw| {
        let v = b.alloc_qubits(N);
        let dx_winv = b.alloc_qubits(N);
        let rx = b.alloc_qubits(N);

        b.set_phase("single_inv_c_shape_v_seed_square");
        squaring_add_to_acc_schoolbook(b, &v, &dy, p);

        b.set_phase("single_inv_c_shape_v_add_mul");
        mod_mul_add_into_acc_schoolbook(b, &v, &dx2, &dy, p);

        b.set_phase("single_inv_c_shape_dx_winv");
        mod_mul_write_into_zero_acc_schoolbook(b, &dx_winv, &dx, inv_raw, p);

        b.set_phase("single_inv_c_shape_rx");
        mod_mul_write_into_zero_acc_schoolbook(b, &rx, &v, &dx_winv, p);

        b.set_phase("single_inv_c_shape_unrx");
        mod_mul_sub_qq(b, &rx, &v, &dx_winv, p);
        b.set_phase("single_inv_c_shape_undx_winv");
        mod_mul_sub_qq(b, &dx_winv, &dx, inv_raw, p);

        if !skip_ry {
            let core = b.alloc_qubits(N);
            let ry = b.alloc_qubits(N);
            b.set_phase("single_inv_c_shape_core");
            mod_mul_write_into_zero_acc_schoolbook(b, &core, &dx2, &dy, p);
            b.set_phase("single_inv_c_shape_ry");
            mod_mul_write_into_zero_acc_schoolbook(b, &ry, &core, inv_raw, p);
            b.set_phase("single_inv_c_shape_unry");
            mod_mul_sub_qq(b, &ry, &core, inv_raw, p);
            b.set_phase("single_inv_c_shape_uncore");
            mod_mul_sub_qq(b, &core, &dx2, &dy, p);
            b.free_vec(&ry);
            b.free_vec(&core);
        }

        b.set_phase("single_inv_c_shape_unv_mul");
        mod_mul_sub_qq(b, &v, &dx2, &dy, p);
        b.set_phase("single_inv_c_shape_unv_square");
        if lowq_unv_square {
            squaring_sub_from_acc_schoolbook_lowq_shift22(b, &v, &dy, p);
        } else {
            squaring_sub_from_acc_schoolbook(b, &v, &dy, p);
        }

        b.free_vec(&v);
    });

    if std::env::var("SINGLE_INV_C_FREE_DY_AFTER_BODY")
        .ok()
        .as_deref()
        == Some("1")
    {
        init_small_const_reg(b, &dy, 5);
        b.free_vec(&dy);
    }

    b.set_phase("single_inv_c_shape_unw");
    mod_mul_sub_qq(b, &w, &dx2, &dx, p);
    b.set_phase("single_inv_c_shape_undx2");
    if lowq_undx2 {
        squaring_sub_from_acc_schoolbook_lowq_shift22(b, &dx2, &dx, p);
    } else {
        squaring_sub_from_acc_schoolbook(b, &dx2, &dx, p);
    }

    init_small_const_reg(b, &dy, 5);
    init_small_const_reg(b, &dx, 3);
    b.set_phase("single_inv_c_shape_free");
    b.free_vec(&w);
    b.free_vec(&dx2);
    b.free_vec(&dy);
    b.free_vec(&dx);
}

pub(crate) fn emit_centered_restoring_qbit_benchmark_scaffold(b: &mut B) {
    const WIDTH: usize = 256;
    b.set_phase("centered_restoring_qbit_alloc");
    let u = b.alloc_qubits(WIDTH);
    let v = b.alloc_qubits(WIDTH);
    let q = b.alloc_qubit();
    init_small_const_reg(b, &u, 9);
    init_small_const_reg(b, &v, 5);
    b.set_phase("centered_restoring_qbit_trial");
    centered_restoring_trial_subtract_clean(b, &u, &v, q);
    b.set_phase("centered_restoring_qbit_free");
    // This scaffold uses fixed constants with a known successful trial, so
    // return the observed quotient bit to |0> before freeing it.
    b.x(q);
    b.free(q);
    init_small_const_reg(b, &v, 5);
    init_small_const_reg(b, &u, 9);
    b.free_vec(&v);
    b.free_vec(&u);
}

pub(crate) fn emit_centered_by_denominator_derived_controls_benchmark_scaffold(
    b: &mut B,
    tx: &[QubitId],
    p: U256,
) {
    // First functional integration step beyond fixed traces: derive the BY odd/A
    // controls reversibly from a live quantum denominator copy (here the current
    // output x register), run a clean fast centered replay roundtrip on scratch,
    // then reverse the denominator generator to clean the controls.  The replay
    // scratch is zero so this is still a no-op, but the control bank is now
    // genuinely denominator-derived rather than hard-coded.
    const STEPS: usize = 560;
    const DBITS: usize = 12;
    const WIDE: usize = N + 4;
    b.set_phase("by_centered_denom_controls_bench_alloc");
    let f = b.alloc_qubits(STEPS);
    let g = b.alloc_qubits(STEPS);
    let delta = b.alloc_qubits(DBITS);
    let odd = b.alloc_qubits(STEPS);
    let a_ctrl = b.alloc_qubits(STEPS);
    let parity = b.alloc_qubits(STEPS);
    let r = b.alloc_qubits(WIDE);
    let s = b.alloc_qubits(WIDE);

    for i in 0..N {
        if bit(p, i) {
            b.x(f[i]);
        }
        b.cx(tx[i], g[i]);
    }
    b.x(delta[0]);

    b.set_phase("by_centered_denom_controls_bench_generate");
    for i in 0..STEPS {
        let rem = STEPS - i;
        by_2adic_branch_step_for_bench(b, &f[..rem], &g[..rem], &delta, odd[i], a_ctrl[i]);
    }

    b.set_phase("by_centered_denom_controls_bench_replay");
    for i in 0..STEPS {
        centered_signed_by_microstep_for_bench(b, &r, &s, odd[i], a_ctrl[i], parity[i], p);
    }
    for i in (0..STEPS).rev() {
        centered_signed_by_microstep_inverse_for_bench(b, &r, &s, odd[i], a_ctrl[i], parity[i], p);
        centered_signed_by_clear_parity_after_inverse_for_bench(b, &r, &s, odd[i], parity[i]);
    }

    b.set_phase("by_centered_denom_controls_bench_reverse");
    for i in (0..STEPS).rev() {
        let rem = STEPS - i;
        by_2adic_branch_step_reverse_for_bench(b, &f[..rem], &g[..rem], &delta, odd[i], a_ctrl[i]);
    }

    b.set_phase("by_centered_denom_controls_bench_clear");
    b.x(delta[0]);
    for i in 0..N {
        b.cx(tx[i], g[i]);
        if bit(p, i) {
            b.x(f[i]);
        }
    }
    let _ = (f, g, delta, odd, a_ctrl, parity, r, s);
}

pub(crate) fn emit_centered_by_denom_controls_live_numerator_benchmark_scaffold(
    b: &mut B,
    tx: &[QubitId],
    ty: &[QubitId],
    p: U256,
) {
    // Same denominator-derived control component, but now the centered replay
    // scratch is a nonzero live numerator-derived value: a centered copy of the
    // current y register.  The fast centered replay is still run as a
    // forward+inverse no-op, but it now exercises arbitrary quantum numerator
    // data rather than the zero scratch used by the first denominator hook.
    const STEPS: usize = 560;
    const DBITS: usize = 12;
    const WIDE: usize = N + 4;
    b.set_phase("by_centered_live_num_bench_alloc_num");
    let r = b.alloc_qubits(WIDE);
    let s = b.alloc_qubits(WIDE);
    let center_flag = by_load_centered_copy_for_bench(b, ty, &s, p);

    b.set_phase("by_centered_live_num_bench_alloc_den");
    let f = b.alloc_qubits(STEPS);
    let g = b.alloc_qubits(STEPS);
    let delta = b.alloc_qubits(DBITS);
    let odd = b.alloc_qubits(STEPS);
    let a_ctrl = b.alloc_qubits(STEPS);
    let parity = b.alloc_qubits(STEPS);
    for i in 0..N {
        if bit(p, i) {
            b.x(f[i]);
        }
        b.cx(tx[i], g[i]);
    }
    b.x(delta[0]);

    b.set_phase("by_centered_live_num_bench_generate");
    for i in 0..STEPS {
        let rem = STEPS - i;
        by_2adic_branch_step_for_bench(b, &f[..rem], &g[..rem], &delta, odd[i], a_ctrl[i]);
    }

    b.set_phase("by_centered_live_num_bench_replay");
    for i in 0..STEPS {
        centered_signed_by_microstep_for_bench(b, &r, &s, odd[i], a_ctrl[i], parity[i], p);
    }
    for i in (0..STEPS).rev() {
        centered_signed_by_microstep_inverse_for_bench(b, &r, &s, odd[i], a_ctrl[i], parity[i], p);
        centered_signed_by_clear_parity_after_inverse_for_bench(b, &r, &s, odd[i], parity[i]);
    }

    b.set_phase("by_centered_live_num_bench_reverse_den");
    for i in (0..STEPS).rev() {
        let rem = STEPS - i;
        by_2adic_branch_step_reverse_for_bench(b, &f[..rem], &g[..rem], &delta, odd[i], a_ctrl[i]);
    }

    b.set_phase("by_centered_live_num_bench_clear");
    b.x(delta[0]);
    for i in 0..N {
        b.cx(tx[i], g[i]);
        if bit(p, i) {
            b.x(f[i]);
        }
    }
    by_unload_centered_copy_for_bench(b, ty, &s, p, center_flag);
    let _ = (f, g, delta, odd, a_ctrl, parity, r, s);
}
