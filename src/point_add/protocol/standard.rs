//! `protocol::standard` — verbatim split of the original `protocol` module.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

pub(crate) fn build_standard_point_add(
    b: &mut B,
    tx: &[QubitId],
    ty: &[QubitId],
    ox: &[BitId],
    oy: &[BitId],
    p: U256,
) {
    if halfgcd_live_pa::round162_halfgcd_live_pa_enabled() {
        halfgcd_live_pa::emit_round162_halfgcd_live_pa_or_fail(b, tx, ty, ox, oy, p);
    }

    if round158_halfgcd_splice_live::round158_live_prefix_pa_route_enabled() {
        round158_halfgcd_splice_live::abort_round158_live_prefix_pa_route(tx, ty, ox, oy, p);
    }

    let pair2_branch_inv = std::env::var("KAL_PAIR2_BRANCH_INV_ROLL").ok().as_deref() == Some("1");
    let prescale_pair1 = std::env::var("KAL_PRESCALE_PAIR1_SAFE").ok().as_deref() == Some("1");
    let prescale_pair1_mixed =
        std::env::var("KAL_PRESCALE_PAIR1_MIXED").ok().as_deref() == Some("1");
    let prescale_pair1_chunked =
        std::env::var("KAL_PRESCALE_PAIR1_CHUNKED").ok().as_deref() == Some("1");
    let prescale_pair1_folded =
        std::env::var("KAL_PRESCALE_PAIR1_FOLDED").ok().as_deref() == Some("1");
    let prescale_pair1_folded_chunked = std::env::var("KAL_PRESCALE_PAIR1_FOLDED_CHUNKED")
        .ok()
        .as_deref()
        == Some("1");
    let prescale_pair2 = std::env::var("KAL_PRESCALE_PAIR2_SAFE").ok().as_deref() == Some("1");
    let prescale_pair2_mixed =
        std::env::var("KAL_PRESCALE_PAIR2_MIXED").ok().as_deref() == Some("1");
    let prescale_pair2_chunked =
        std::env::var("KAL_PRESCALE_PAIR2_CHUNKED").ok().as_deref() == Some("1");
    let prescale_pair2_folded =
        std::env::var("KAL_PRESCALE_PAIR2_FOLDED").ok().as_deref() == Some("1");
    let prescale_pair2_folded_chunked = std::env::var("KAL_PRESCALE_PAIR2_FOLDED_CHUNKED")
        .ok()
        .as_deref()
        == Some("1");
    let by_pair1_centered = std::env::var("BY_CENTERED_PAIR1_REPLACE").ok().as_deref() == Some("1");
    let by_pair2_centered = std::env::var("BY_CENTERED_PAIR2_REPLACE").ok().as_deref() == Some("1");
    let by_pair2_scaled_product = std::env::var("BY_SCALED_PAIR2_PRODUCT_REPLACE")
        .ok()
        .as_deref()
        == Some("1");
    let round200_full_gcd_pair1 = std::env::var("ROUND200_FULL_GCD_PAIR1_REPLACE")
        .ok()
        .as_deref()
        == Some("1");
    let source_live_cubic_ytail =
        std::env::var("PA_SOURCE_LIVE_CUBIC_YTAIL").ok().as_deref() == Some("1");
    let source_live_cubic_lambda_clean = std::env::var("PA_SOURCE_LIVE_CUBIC_LAMBDA_CLEAN")
        .ok()
        .as_deref()
        == Some("1");
    let source_live_cubic_product_clean = std::env::var("PA_SOURCE_LIVE_CUBIC_PRODUCT_CLEAN")
        .ok()
        .as_deref()
        == Some("1");
    let source_live_cubic_hmr_phase_repair = std::env::var("PA_SOURCE_LIVE_CUBIC_HMR_PHASE_REPAIR")
        .ok()
        .as_deref()
        == Some("1");
    let source_live_clean_product_tail = std::env::var("PA_SOURCE_LIVE_CLEAN_PRODUCT_TAIL")
        .ok()
        .as_deref()
        == Some("1");
    let source_live_product_hmr_overwrite = std::env::var("PA_SOURCE_LIVE_PRODUCT_HMR_OVERWRITE")
        .ok()
        .as_deref()
        == Some("1");
    let source_live_product_hmr_quotient_phase_repair =
        std::env::var("PA_SOURCE_LIVE_PRODUCT_HMR_QUOTIENT_PHASE_REPAIR")
            .ok()
            .as_deref()
            == Some("1");
    let source_live_product_hmr_direct_quotient_phase_repair =
        std::env::var("PA_SOURCE_LIVE_PRODUCT_HMR_DIRECT_QUOTIENT_PHASE_REPAIR")
            .ok()
            .as_deref()
            == Some("1");
    let source_live_product_hmr_single_inverse_phase_repair =
        std::env::var("PA_SOURCE_LIVE_PRODUCT_HMR_SINGLE_INVERSE_PHASE_REPAIR")
            .ok()
            .as_deref()
            == Some("1");
    let source_live_product_centered_quotient_clean =
        std::env::var("PA_SOURCE_LIVE_PRODUCT_CENTERED_QUOTIENT_CLEAN")
            .ok()
            .as_deref()
            == Some("1");
    let source_live_product_hmr_allow_dirty =
        std::env::var("PA_SOURCE_LIVE_PRODUCT_HMR_ALLOW_PHASE_DIRTY")
            .ok()
            .as_deref()
            == Some("1");
    let source_live_tail = source_live_cubic_ytail
        || source_live_cubic_lambda_clean
        || source_live_cubic_product_clean
        || source_live_cubic_hmr_phase_repair
        || source_live_clean_product_tail;
    let source_live_cubic_allow_dirty = std::env::var("PA_SOURCE_LIVE_CUBIC_ALLOW_PHASE_DIRTY")
        .ok()
        .as_deref()
        == Some("1");
    let source_live_cubic_borrow_pair1 = std::env::var("PA_SOURCE_LIVE_CUBIC_BORROW_PAIR1")
        .ok()
        .as_deref()
        == Some("1");
    let coeff_channel_div = std::env::var("KAL_TAGGED_DIV_COEFF_CHANNEL")
        .ok()
        .as_deref()
        == Some("1");
    let branch_hist_div = std::env::var("KAL_TAGGED_DIV_BRANCH_HIST").ok().as_deref() == Some("1");
    let branch_stream_div = std::env::var("KAL_TAGGED_DIV_BRANCH_STREAM")
        .ok()
        .as_deref()
        == Some("1");
    let branch_term_div = std::env::var("KAL_TAGGED_DIV_BRANCH_TERM").ok().as_deref() == Some("1");
    let branch_term_roll_div = std::env::var("KAL_TAGGED_DIV_BRANCH_TERM_ROLL")
        .ok()
        .as_deref()
        == Some("1");
    let tagged_div_validate = coeff_channel_div
        || branch_hist_div
        || branch_stream_div
        || branch_term_div
        || branch_term_roll_div
        || std::env::var("KAL_TAGGED_DIV_VALIDATE").ok().as_deref() == Some("1");
    let pair1_iters = std::env::var("KAL_PAIR1_ITERS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .map(|iters| {
            checked_kaliski_iters(
                "round8 pair1",
                "KAL_PAIR1_ITERS",
                iters,
                ROUND24_PAIR1_MIN_SAFE_ITERS,
            )
        })
        .unwrap_or(ROUND24_PAIR1_MIN_SAFE_ITERS);
    // The tagged validation paths change the op stream / Fiat-Shamir seed;
    // keep pair2 at the prior robust 404 setting to avoid conflating the
    // algebra probe with an iteration-threshold phase cliff.  Env overrides are
    // for approximate-correctness threshold research only; default remains the
    // exact checked setting.  For the normal exact path, full-harness probes
    // after the R_SMALL_THRESHOLD=260 update found pair2=400 clean; pair2=399
    // remains outside the verified safety margin.  A Google-sample KMX row with
    // pair1=400,pair2=400 passes 9024, but the source alt-seed diagnostic still
    // catches a phase batch, so the robust source default stays at pair1=404.
    let pair2_default = if tagged_div_validate || pair2_branch_inv {
        404
    } else {
        400
    };
    let pair2_iters = std::env::var("KAL_PAIR2_ITERS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .map(|iters| {
            checked_kaliski_iters(
                "round8 pair2",
                "KAL_PAIR2_ITERS",
                iters,
                ROUND8_QTAIL_PAIR2_MIN_SAFE_ITERS,
            )
        })
        .unwrap_or(pair2_default);
    let lend_ty_pair1_backward =
        std::env::var("KAL_LEND_TY_PAIR1_BACKWARD").ok().as_deref() == Some("1");
    let free_lam_before_pair2_backward = std::env::var("KAL_FREE_LAM_BEFORE_PAIR2_BACKWARD")
        .ok()
        .as_deref()
        == Some("1");
    let defer_pair2_product = (std::env::var("KAL_DEFER_PAIR2_PRODUCT").ok().as_deref()
        == Some("1")
        || std::env::var("KAL_DEFER_PAIR2_PRODUCT_LOWQ")
            .ok()
            .as_deref()
            == Some("1"))
        && !by_pair2_centered
        && !pair2_branch_inv
        && !prescale_pair2
        && !prescale_pair2_mixed
        && !prescale_pair2_chunked
        && !prescale_pair2_folded
        && !prescale_pair2_folded_chunked
        && !by_pair2_scaled_product;
    let affine_combined_y = env_flag_enabled("POINT_ADD_AFFINE_COMBINED_Y", true)
        && !round200_full_gcd_pair1
        && !source_live_tail
        && !source_live_cubic_borrow_pair1
        && !by_pair1_centered
        && !by_pair2_centered
        && !by_pair2_scaled_product
        && !coeff_channel_div
        && !branch_hist_div
        && !branch_stream_div
        && !branch_term_div
        && !branch_term_roll_div
        && !tagged_div_validate
        && !pair2_branch_inv
        && !prescale_pair1
        && !prescale_pair1_mixed
        && !prescale_pair1_chunked
        && !prescale_pair1_folded
        && !prescale_pair1_folded_chunked
        && !prescale_pair2
        && !prescale_pair2_mixed
        && !prescale_pair2_chunked
        && !prescale_pair2_folded
        && !prescale_pair2_folded_chunked
        && !lend_ty_pair1_backward
        && !defer_pair2_product;
    assert!(
        [
            source_live_cubic_ytail,
            source_live_cubic_lambda_clean,
            source_live_cubic_product_clean,
            source_live_cubic_hmr_phase_repair,
            source_live_clean_product_tail
        ]
        .iter()
        .filter(|&&enabled| enabled)
        .count()
            <= 1,
        "select only one source-live y-tail"
    );
    if source_live_cubic_ytail {
        assert!(
            source_live_cubic_allow_dirty,
            "PA_SOURCE_LIVE_CUBIC_YTAIL leaves the separate lam word nonzero; \
             set PA_SOURCE_LIVE_CUBIC_ALLOW_PHASE_DIRTY=1 only for red-team \
             resource probes, not candidate KMX emission"
        );
        assert!(
            !by_pair1_centered
                && !coeff_channel_div
                && !branch_hist_div
                && !branch_stream_div
                && !branch_term_div
                && !branch_term_roll_div
                && !tagged_div_validate
                && !prescale_pair1
                && !prescale_pair1_mixed
                && !prescale_pair1_chunked
                && !prescale_pair1_folded
                && !prescale_pair1_folded_chunked
                && !lend_ty_pair1_backward
                && !pair2_branch_inv
                && !prescale_pair2
                && !prescale_pair2_mixed
                && !prescale_pair2_chunked
                && !prescale_pair2_folded
                && !prescale_pair2_folded_chunked
                && !by_pair2_centered
                && !by_pair2_scaled_product
                && !defer_pair2_product
                && !free_lam_before_pair2_backward,
            "PA_SOURCE_LIVE_CUBIC_YTAIL requires a raw pair1 Kaliski path and bypasses pair2"
        );
    }
    if source_live_clean_product_tail {
        assert!(
            !(source_live_product_hmr_overwrite && source_live_product_centered_quotient_clean),
            "select only one source-live product-tail lambda cleanup"
        );
        assert!(
            [
                source_live_product_hmr_quotient_phase_repair,
                source_live_product_hmr_direct_quotient_phase_repair,
                source_live_product_hmr_single_inverse_phase_repair,
                source_live_product_hmr_allow_dirty,
            ]
            .iter()
            .filter(|&&enabled| enabled)
            .count()
                <= 1,
            "select only one source-live HMR phase policy"
        );
        assert!(
            !source_live_product_hmr_overwrite
                || source_live_product_hmr_quotient_phase_repair
                || source_live_product_hmr_direct_quotient_phase_repair
                || source_live_product_hmr_single_inverse_phase_repair
                || source_live_product_hmr_allow_dirty,
            "PA_SOURCE_LIVE_PRODUCT_HMR_OVERWRITE measures the old lam word and is phase-dirty; \
             set PA_SOURCE_LIVE_PRODUCT_HMR_QUOTIENT_PHASE_REPAIR=1 for the counted in-place repair, \
             set PA_SOURCE_LIVE_PRODUCT_HMR_DIRECT_QUOTIENT_PHASE_REPAIR=1 for the output-side quotient phase repair, or \
             set PA_SOURCE_LIVE_PRODUCT_HMR_SINGLE_INVERSE_PHASE_REPAIR=1 for the live-inverse output-side repair, or \
             set PA_SOURCE_LIVE_PRODUCT_HMR_ALLOW_PHASE_DIRTY=1 only for red-team resource probes, \
             not candidate KMX emission"
        );
        assert!(
            !round200_full_gcd_pair1
                && !by_pair1_centered
                && !coeff_channel_div
                && !branch_hist_div
                && !branch_stream_div
                && !branch_term_div
                && !branch_term_roll_div
                && !tagged_div_validate
                && !prescale_pair1
                && !prescale_pair1_mixed
                && !prescale_pair1_chunked
                && !prescale_pair1_folded
                && !prescale_pair1_folded_chunked
                && !lend_ty_pair1_backward
                && !pair2_branch_inv
                && !prescale_pair2
                && !prescale_pair2_mixed
                && !prescale_pair2_chunked
                && !prescale_pair2_folded
                && !prescale_pair2_folded_chunked
                && !by_pair2_centered
                && !by_pair2_scaled_product
                && !defer_pair2_product
                && !free_lam_before_pair2_backward,
            "PA_SOURCE_LIVE_CLEAN_PRODUCT_TAIL requires a raw pair1 Kaliski path and bypasses pair2"
        );
    }
    if source_live_cubic_lambda_clean {
        assert!(
            !round200_full_gcd_pair1
                && !by_pair1_centered
                && !coeff_channel_div
                && !branch_hist_div
                && !branch_stream_div
                && !branch_term_div
                && !branch_term_roll_div
                && !tagged_div_validate
                && !prescale_pair1
                && !prescale_pair1_mixed
                && !prescale_pair1_chunked
                && !prescale_pair1_folded
                && !prescale_pair1_folded_chunked
                && !lend_ty_pair1_backward
                && !pair2_branch_inv
                && !prescale_pair2
                && !prescale_pair2_mixed
                && !prescale_pair2_chunked
                && !prescale_pair2_folded
                && !prescale_pair2_folded_chunked
                && !by_pair2_centered
                && !by_pair2_scaled_product
                && !defer_pair2_product
                && !free_lam_before_pair2_backward,
            "PA_SOURCE_LIVE_CUBIC_LAMBDA_CLEAN requires a raw pair1 Kaliski path and bypasses pair2 product emission"
        );
    }
    if source_live_cubic_product_clean {
        assert!(
            !round200_full_gcd_pair1
                && !by_pair1_centered
                && !coeff_channel_div
                && !branch_hist_div
                && !branch_stream_div
                && !branch_term_div
                && !branch_term_roll_div
                && !tagged_div_validate
                && !prescale_pair1
                && !prescale_pair1_mixed
                && !prescale_pair1_chunked
                && !prescale_pair1_folded
                && !prescale_pair1_folded_chunked
                && !lend_ty_pair1_backward
                && !pair2_branch_inv
                && !prescale_pair2
                && !prescale_pair2_mixed
                && !prescale_pair2_chunked
                && !prescale_pair2_folded
                && !prescale_pair2_folded_chunked
                && !by_pair2_centered
                && !by_pair2_scaled_product
                && !defer_pair2_product
                && !free_lam_before_pair2_backward,
            "PA_SOURCE_LIVE_CUBIC_PRODUCT_CLEAN requires a raw pair1 Kaliski path and bypasses pair2 product emission"
        );
    }
    if source_live_cubic_hmr_phase_repair {
        assert!(
            !round200_full_gcd_pair1
                && !by_pair1_centered
                && !coeff_channel_div
                && !branch_hist_div
                && !branch_stream_div
                && !branch_term_div
                && !branch_term_roll_div
                && !tagged_div_validate
                && !prescale_pair1
                && !prescale_pair1_mixed
                && !prescale_pair1_chunked
                && !prescale_pair1_folded
                && !prescale_pair1_folded_chunked
                && !lend_ty_pair1_backward
                && !pair2_branch_inv
                && !prescale_pair2
                && !prescale_pair2_mixed
                && !prescale_pair2_chunked
                && !prescale_pair2_folded
                && !prescale_pair2_folded_chunked
                && !by_pair2_centered
                && !by_pair2_scaled_product
                && !defer_pair2_product
                && !free_lam_before_pair2_backward,
            "PA_SOURCE_LIVE_CUBIC_HMR_PHASE_REPAIR requires a raw pair1 Kaliski path and bypasses pair2"
        );
    }
    if tagged_div_validate && !by_pair1_centered {
        // Structural validation path for the 600-scratch DIV idea: seed the
        // numerator as dy+dx, so the Kaliski coefficient output is tagged by
        // a known k*dx term. This is default-off because it adds gates; it is
        // an algebra/circuit integration probe, not a benchmark optimization.
        b.set_phase("tagged_div_seed");
        mod_add_qq_fast(b, &ty, &tx, p);
    }

    let lam_cell: std::cell::RefCell<Option<Vec<QubitId>>> = std::cell::RefCell::new(None);
    let ty_lent_pair1_backward = std::cell::Cell::new(false);
    if round200_full_gcd_pair1 {
        let lam_inner = emit_round200_semantic_full_gcd_pair1_checkpoint_in_place_with_options(
            b,
            &tx,
            &ty,
            p,
            source_live_tail,
        );
        *lam_cell.borrow_mut() = Some(lam_inner);
    } else if by_pair1_centered {
        let lam_inner = compute_pair1_lam_with_centered_by_bench(b, &tx, &ty, p);
        b.set_phase("pair1_by_centered_zero_ty_mul2");
        mod_mul_add_into_acc_selected(b, &ty, &lam_inner, &tx, p, "PAIR1_ZERO_TY_MUL");
        *lam_cell.borrow_mut() = Some(lam_inner);
    } else if branch_term_roll_div {
        // Compressed branch stream with a rolling active flag. This keeps the
        // 9-bit terminal index qubit saving, but avoids branch_term's expensive
        // per-iteration `term_idx > i` comparator and double cmod-add replay.
        let lam_inner = b.alloc_qubits(N);
        let lam_coeff = lam_inner.clone();
        let ty_coeff: Vec<QubitId> = ty.to_vec();
        b.set_phase("pair1_kaliski_branch_term_roll");
        with_kal_branch_term_roll_tagged_div(
            b,
            &tx,
            p,
            pair1_iters,
            (&lam_coeff, &ty_coeff),
            |b| {
                b.set_phase("pair1_branch_term_roll_halve");
                for _ in 0..pair1_iters {
                    mod_halve_inplace_fast(b, &lam_inner, p);
                }
                b.set_phase("pair1_branch_term_roll_untag_lam");
                mod_add_qc(b, &lam_inner, U256::from(1u64), p);
                *lam_cell.borrow_mut() = Some(lam_inner);
            },
        );
    } else if branch_term_div {
        // Compressed branch stream: store m_hist+a_hist plus a 9-bit terminal
        // index instead of a full add_hist. Coefficient replay reconstructs
        // active VG adds using term_idx > i.
        let lam_inner = b.alloc_qubits(N);
        let lam_coeff = lam_inner.clone();
        let ty_coeff: Vec<QubitId> = ty.to_vec();
        b.set_phase("pair1_kaliski_branch_term");
        with_kal_branch_term_tagged_div(b, &tx, p, pair1_iters, (&lam_coeff, &ty_coeff), |b| {
            b.set_phase("pair1_branch_term_halve");
            for _ in 0..pair1_iters {
                mod_halve_inplace_fast(b, &lam_inner, p);
            }
            b.set_phase("pair1_branch_term_untag_lam");
            mod_add_qc(b, &lam_inner, U256::from(1u64), p);
            *lam_cell.borrow_mut() = Some(lam_inner);
        });
    } else if branch_stream_div {
        // Branch-generation stream: record just branch histories, free the
        // denominator state, then replay those histories into the tagged
        // coefficient channel. This tests the qubit shape that a future
        // self-cleaning DIV would need.
        let lam_inner = b.alloc_qubits(N);
        let lam_coeff = lam_inner.clone();
        let ty_coeff: Vec<QubitId> = ty.to_vec();
        b.set_phase("pair1_kaliski_branch_stream");
        with_kal_branch_stream_tagged_div(b, &tx, p, pair1_iters, (&lam_coeff, &ty_coeff), |b| {
            b.set_phase("pair1_branch_stream_halve");
            for _ in 0..pair1_iters {
                mod_halve_inplace_fast(b, &lam_inner, p);
            }
            b.set_phase("pair1_branch_stream_untag_lam");
            mod_add_qc(b, &lam_inner, U256::from(1u64), p);
            *lam_cell.borrow_mut() = Some(lam_inner);
        });
    } else if branch_hist_div {
        // More aggressive structural probe: do not run the ordinary inverse
        // coefficient `(r,s)` at all. Store `a_hist` next to `m_hist`; together
        // they recover the branch pair while the external `(lam,ty)` channel
        // receives the tagged quotient.
        let lam_inner = b.alloc_qubits(N);
        let lam_coeff = lam_inner.clone();
        let ty_coeff: Vec<QubitId> = ty.to_vec();
        b.set_phase("pair1_kaliski_branch_hist_coeff");
        with_kal_branch_tagged_div_coeff(b, &tx, p, pair1_iters, (&lam_coeff, &ty_coeff), |b| {
            b.set_phase("pair1_branch_hist_halve");
            for _ in 0..pair1_iters {
                mod_halve_inplace_fast(b, &lam_inner, p);
            }
            b.set_phase("pair1_branch_hist_untag_lam");
            mod_add_qc(b, &lam_inner, U256::from(1u64), p);
            *lam_cell.borrow_mut() = Some(lam_inner);
        });
    } else if coeff_channel_div {
        // Experimental structural path: compute the tagged quotient by carrying
        // an external coefficient pair `(lam_inner, ty)` through the Kaliski
        // forward pass. This removes pair1's two schoolbook multiplications;
        // the ordinary inverse state is still present solely to provide clean
        // branch controls and to be Bennett-uncomputed afterwards.
        let lam_inner = b.alloc_qubits(N);
        let lam_coeff = lam_inner.clone();
        let ty_coeff: Vec<QubitId> = ty.to_vec();
        b.set_phase("pair1_kaliski_forward_coeff_channel");
        with_kal_inv_raw_coeff(
            b,
            &tx,
            p,
            pair1_iters,
            Some((&lam_coeff, &ty_coeff)),
            |b, _inv_raw| {
                b.set_phase("pair1_coeff_channel_halve");
                for _ in 0..pair1_iters {
                    mod_halve_inplace_fast(b, &lam_inner, p);
                }
                // lam_inner = -(lambda+1) after consuming tagged ty=(dy+dx).
                // Add 1 to recover the normal lam_inner=-lambda expected by
                // the remaining point-add scaffold.
                b.set_phase("pair1_coeff_channel_untag_lam");
                mod_add_qc(b, &lam_inner, U256::from(1u64), p);
                b.set_phase("pair1_kaliski_backward");
                *lam_cell.borrow_mut() = Some(lam_inner);
            },
        );
    } else if prescale_pair1
        || prescale_pair1_mixed
        || prescale_pair1_chunked
        || prescale_pair1_folded
        || prescale_pair1_folded_chunked
    {
        // Scale absorption probe: Kaliski raw output is `-v^-1 * 2^iters`.
        // Feed `v = 2^iters * dx` so the exposed raw inverse is exactly
        // `-dx^-1`; this deletes the pair1 correction-halving loop.
        if prescale_pair1_folded || prescale_pair1_folded_chunked {
            if prescale_pair1_folded_chunked {
                b.set_phase("pair1_kaliski_forward_prescaled_folded_chunked");
                with_kal_inv_raw_prescaled_chunked(b, &tx, p, pair1_iters, |b, inv_raw| {
                    let lam_inner = b.alloc_qubits(N);
                    b.set_phase("pair1_prescale_mul1");
                    mod_mul_write_into_zero_acc_schoolbook(b, &lam_inner, &ty, inv_raw, p);
                    b.set_phase("pair1_prescale_mul2");
                    mod_mul_add_into_acc_selected(b, &ty, &lam_inner, &tx, p, "PAIR1_ZERO_TY_MUL");
                    b.set_phase("pair1_kaliski_backward_prescaled_folded_chunked");
                    *lam_cell.borrow_mut() = Some(lam_inner);
                });
            } else {
                b.set_phase("pair1_kaliski_forward_prescaled_folded");
                with_kal_inv_raw_prescaled_mixed(b, &tx, p, pair1_iters, |b, inv_raw| {
                    let lam_inner = b.alloc_qubits(N);
                    b.set_phase("pair1_prescale_mul1");
                    mod_mul_write_into_zero_acc_schoolbook(b, &lam_inner, &ty, inv_raw, p);
                    b.set_phase("pair1_prescale_mul2");
                    mod_mul_add_into_acc_selected(b, &ty, &lam_inner, &tx, p, "PAIR1_ZERO_TY_MUL");
                    b.set_phase("pair1_kaliski_backward_prescaled_folded");
                    *lam_cell.borrow_mut() = Some(lam_inner);
                });
            }
        } else {
            // SAFE path uses exact Cuccaro arithmetic because the generic fast
            // prescaler was classically correct but alt-seed phase-unsafe. The
            // MIXED path keeps fast shifts but exact q-q add/sub. CHUNKED keeps
            // the exact q-q add/sub contract but replaces long scale walks with
            // Solinas k-bit shifts between sparse set-bit positions.  The
            // full pair1+pair2 folded-chunked harness is phase-clean and saves
            // Toffoli, but even after source borrowing it peaks at 2897q, so
            // keep it opt-in until the shifted prescaler is fused or made
            // lower-peak without reusing phase-tainted scratch as Kaliski state.
            let scaled_tx = b.alloc_qubits(N);
            let scale = pow_mod_2_k(p, pair1_iters);
            b.set_phase("pair1_prescale_den_safe");
            if prescale_pair1_chunked {
                mul_by_const_acc_chunked_shifts_inplace_src(b, &tx, scale, &scaled_tx, p, false);
            } else if prescale_pair1_mixed {
                mul_by_const_acc_exact_adds_fast_shifts(b, &tx, scale, &scaled_tx, p, false);
            } else {
                mul_by_const_acc_phase_clean(b, &tx, scale, &scaled_tx, p, false);
            }
            b.set_phase("pair1_kaliski_forward_prescaled_safe");
            with_kal_inv_raw(b, &scaled_tx, p, pair1_iters, |b, inv_raw| {
                let lam_inner = b.alloc_qubits(N);
                b.set_phase("pair1_prescale_mul1");
                mod_mul_write_into_zero_acc_schoolbook(b, &lam_inner, &ty, inv_raw, p);
                b.set_phase("pair1_prescale_mul2");
                mod_mul_add_into_acc_selected(b, &ty, &lam_inner, &tx, p, "PAIR1_ZERO_TY_MUL");
                b.set_phase("pair1_kaliski_backward_prescaled_safe");
                *lam_cell.borrow_mut() = Some(lam_inner);
            });
            b.set_phase("pair1_unprescale_den_safe");
            if prescale_pair1_chunked {
                mul_by_const_acc_chunked_shifts_inplace_src(b, &tx, scale, &scaled_tx, p, true);
            } else if prescale_pair1_mixed {
                mul_by_const_acc_exact_adds_fast_shifts(b, &tx, scale, &scaled_tx, p, true);
            } else {
                mul_by_const_acc_phase_clean(b, &tx, scale, &scaled_tx, p, true);
            }
            b.free_vec(&scaled_tx);
        }
    } else if source_live_tail && source_live_cubic_borrow_pair1 {
        b.set_phase("pair1_source_live_cubic_borrow_v_kaliski_forward");
        with_kal_inv_raw_borrowing_v(b, &tx, p, pair1_iters, |b, inv_raw| {
            let lam_inner = b.alloc_qubits(N);
            b.set_phase("pair1_source_live_cubic_borrow_v_mul1");
            match std::env::var("SOURCE_LIVE_PAIR1_LAM_MUL").ok().as_deref() {
                Some("schoolbook") => {
                    mod_mul_add_into_acc_schoolbook(b, &lam_inner, &ty, inv_raw, p)
                }
                Some("schoolbook_peak_lowq") => {
                    mod_mul_add_into_acc_schoolbook_peak_lowq(
                        b,
                        &lam_inner,
                        &ty,
                        inv_raw,
                        p,
                    )
                }
                Some("karatsuba1") | Some("1") => {
                    mod_mul_add_into_acc_karatsuba(b, &lam_inner, &ty, inv_raw, p)
                }
                Some("karatsuba2") | Some("2") => {
                    mod_mul_add_into_acc_karatsuba2(b, &lam_inner, &ty, inv_raw, p)
                }
                Some(other) => panic!(
                    "unsupported SOURCE_LIVE_PAIR1_LAM_MUL={other}; expected schoolbook, schoolbook_peak_lowq, karatsuba1, karatsuba2"
                ),
                None => {
                    mod_mul_add_into_acc_schoolbook_phase_clean(
                        b,
                        &lam_inner,
                        &ty,
                        inv_raw,
                        p,
                    )
                }
            }
            b.set_phase("pair1_source_live_cubic_borrow_v_halve");
            for _ in 0..pair1_iters {
                mod_halve_inplace_fast(b, &lam_inner, p);
            }
            *lam_cell.borrow_mut() = Some(lam_inner);
            b.set_phase("pair1_source_live_cubic_borrow_v_kaliski_backward");
        });
    } else {
        b.set_phase("pair1_kaliski_forward");
        with_kal_inv_raw(b, &tx, p, pair1_iters, |b, inv_raw| {
            let lam_inner = b.alloc_qubits(N);
            b.set_phase("pair1_mul1");
            if source_live_tail {
                match std::env::var("SOURCE_LIVE_PAIR1_LAM_MUL").ok().as_deref() {
                    Some("schoolbook") => {
                        mod_mul_add_into_acc_schoolbook(b, &lam_inner, &ty, inv_raw, p)
                    }
                    Some("schoolbook_peak_lowq") => {
                        mod_mul_add_into_acc_schoolbook_peak_lowq(
                            b,
                            &lam_inner,
                            &ty,
                            inv_raw,
                            p,
                        )
                    }
                    Some("karatsuba1") | Some("1") => {
                        mod_mul_add_into_acc_karatsuba(b, &lam_inner, &ty, inv_raw, p)
                    }
                    Some("karatsuba2") | Some("2") => {
                        mod_mul_add_into_acc_karatsuba2(b, &lam_inner, &ty, inv_raw, p)
                    }
                    Some(other) => panic!(
                        "unsupported SOURCE_LIVE_PAIR1_LAM_MUL={other}; expected schoolbook, schoolbook_peak_lowq, karatsuba1, karatsuba2"
                    ),
                    None => {
                        mod_mul_add_into_acc_schoolbook_phase_clean(
                            b,
                            &lam_inner,
                            &ty,
                            inv_raw,
                            p,
                        )
                    }
                }
            } else {
                mod_mul_write_into_zero_acc_selected(
                    b,
                    &lam_inner,
                    &ty,
                    inv_raw,
                    p,
                    "PAIR1_MUL1_WRITE",
                );
            }
            b.set_phase("pair1_halve");
            for _ in 0..pair1_iters {
                mod_halve_inplace_fast(b, &lam_inner, p);
            }
            if source_live_tail {
                b.set_phase("pair1_preserve_dy_for_source_live_tail");
            } else if affine_combined_y {
                b.set_phase("pair1_mul2_deferred_affine_combined_y");
            } else {
                b.set_phase("pair1_mul2");
                mod_mul_add_into_acc_selected(b, &ty, &lam_inner, &tx, p, "PAIR1_ZERO_TY_MUL");
            }
            if tagged_div_validate {
                // lam_inner = -(lambda+1) after consuming tagged ty=(dy+dx).
                // Add 1 to recover the normal lam_inner=-lambda expected by the
                // remaining point-add scaffold.
                b.set_phase("tagged_div_untag_lam");
                mod_add_qc(b, &lam_inner, U256::from(1u64), p);
            }
            if lend_ty_pair1_backward {
                b.set_phase("pair1_lend_zero_ty_before_kaliski_backward");
                b.free_vec(&ty);
                ty_lent_pair1_backward.set(true);
            }
            b.set_phase("pair1_kaliski_backward");
            *lam_cell.borrow_mut() = Some(lam_inner);
        });
    }
    if ty_lent_pair1_backward.get() {
        b.set_phase("pair1_reacquire_ty_after_kaliski_backward");
        b.reacquire_vec(&ty);
    }
    let lam: Vec<QubitId> = lam_cell.into_inner().expect("lam set");

    if source_live_clean_product_tail {
        emit_source_live_clean_product_tail(b, &tx, &ty, &lam, &ox, &oy, p, pair2_iters);
        return;
    }

    if source_live_cubic_lambda_clean {
        emit_source_live_cubic_xtail_ytail(
            b,
            &tx,
            &ty,
            &lam,
            &ox,
            &oy,
            p,
            SourceLiveCubicLamClean::Inverse {
                inverse_iters: pair2_iters,
            },
        );
        return;
    }

    if source_live_cubic_product_clean {
        emit_source_live_cubic_xtail_ytail(
            b,
            &tx,
            &ty,
            &lam,
            &ox,
            &oy,
            p,
            SourceLiveCubicLamClean::Product {
                inverse_iters: pair2_iters,
            },
        );
        return;
    }

    if source_live_cubic_hmr_phase_repair {
        emit_source_live_cubic_xtail_ytail(
            b,
            &tx,
            &ty,
            &lam,
            &ox,
            &oy,
            p,
            SourceLiveCubicLamClean::HmrPhaseRepair {
                inverse_iters: pair2_iters,
            },
        );
        return;
    }

    if source_live_cubic_ytail {
        emit_source_live_cubic_xtail_ytail(
            b,
            &tx,
            &ty,
            &lam,
            &ox,
            &oy,
            p,
            SourceLiveCubicLamClean::Dirty,
        );
        // Dirty experimental path only: lam is still `-lambda` here, so this
        // reset is the measured source of the inverted phase failures.
        b.set_phase("source_live_cubic_dirty_free_lam");
        b.free_vec(&lam);
        return;
    }

    if affine_combined_y {
        square_tx_and_combined_ty_l2minus3qx(b, &tx, &ty, &lam, &ox, p);
    } else {
        mod_mul_sub_qq(b, &tx, &lam, &lam, p);
        mod_add_double_qb(b, &tx, &ox, p);
        mod_add_qb(b, &tx, &ox, p);
        mod_neg_inplace_fast(b, &tx, p);
    }
    let lam_freed_before_pair2_backward = std::cell::Cell::new(false);
    if by_pair2_scaled_product {
        b.set_phase("pair2_by_scaled_product");
        write_pair2_product_and_clean_lam_with_scaled_by_bench(b, &lam, &tx, &ty, p);
        b.set_phase("pair2_by_scaled_product_cleanup");
        mod_sub_qb(b, &ty, &oy, p);
    } else {
        let ty_lent_pair2_forward = std::cell::Cell::new(false);
        if affine_combined_y {
            b.set_phase("mul3_deferred_affine_combined_y");
        } else if defer_pair2_product {
            b.set_phase("pair2_lend_zero_ty_before_kaliski_forward");
            b.free_vec(&ty);
            ty_lent_pair2_forward.set(true);
        } else {
            b.set_phase("mul3_between_pair");
            mod_mul_write_into_zero_acc_karatsuba2(b, &ty, &lam, &tx, p);
        }
        if by_pair2_centered {
            b.set_phase("pair2_by_centered_compute_correction");
            add_neg_quotient_into_acc_with_centered_by_bench(b, &lam, &tx, &ty, p);
            b.set_phase("pair2_by_centered_cleanup");
            mod_sub_qb(b, &ty, &oy, p);
        } else {
            b.set_phase("pair2_kaliski_forward");
            if pair2_branch_inv {
                // Compact exact inversion scaffold for pair2: branch histories +
                // coefficient replay compute inv_raw, then replay is reversed after
                // lam cleanup. This targets qubit shape rather than Toffoli.
                with_kal_branch_inv_raw_roll(b, &tx, p, pair2_iters, |b, inv_raw| {
                    b.set_phase("pair2_branch_inv_double");
                    for _ in 0..pair2_iters {
                        mod_double_inplace_fast(b, &lam, p);
                    }
                    b.set_phase("pair2_branch_inv_mul");
                    mod_mul_add_into_acc_selected(b, &lam, inv_raw, &ty, p, "PAIR2_CLEAN_LAM_MUL");
                    b.set_phase("pair2_branch_inv_cleanup");
                    mod_sub_qb(b, &ty, &oy, p);
                });
            } else if prescale_pair2
                || prescale_pair2_mixed
                || prescale_pair2_chunked
                || prescale_pair2_folded
                || prescale_pair2_folded_chunked
            {
                // Pair2 scale absorption: feed `2^iters * (Rx-Qx)` so the raw inverse
                // is exact and the lam-doubling correction loop disappears.
                if prescale_pair2_folded || prescale_pair2_folded_chunked {
                    if prescale_pair2_folded_chunked {
                        with_kal_inv_raw_prescaled_chunked(b, &tx, p, pair2_iters, |b, inv_raw| {
                            b.set_phase("pair2_prescale_mul");
                            mod_mul_add_into_acc_selected(
                                b,
                                &lam,
                                inv_raw,
                                &ty,
                                p,
                                "PAIR2_CLEAN_LAM_MUL",
                            );
                            b.set_phase("pair2_prescale_cleanup");
                            mod_sub_qb(b, &ty, &oy, p);
                            b.set_phase("pair2_kaliski_backward_prescaled_folded_chunked");
                        });
                    } else {
                        with_kal_inv_raw_prescaled_mixed(b, &tx, p, pair2_iters, |b, inv_raw| {
                            b.set_phase("pair2_prescale_mul");
                            mod_mul_add_into_acc_selected(
                                b,
                                &lam,
                                inv_raw,
                                &ty,
                                p,
                                "PAIR2_CLEAN_LAM_MUL",
                            );
                            b.set_phase("pair2_prescale_cleanup");
                            mod_sub_qb(b, &ty, &oy, p);
                            b.set_phase("pair2_kaliski_backward_prescaled_folded");
                        });
                    }
                } else {
                    let scaled_tx = b.alloc_qubits(N);
                    let scale = pow_mod_2_k(p, pair2_iters);
                    b.set_phase("pair2_prescale_den_safe");
                    if prescale_pair2_chunked {
                        mul_by_const_acc_chunked_shifts_inplace_src(
                            b, &tx, scale, &scaled_tx, p, false,
                        );
                    } else if prescale_pair2_mixed {
                        mul_by_const_acc_exact_adds_fast_shifts(
                            b, &tx, scale, &scaled_tx, p, false,
                        );
                    } else {
                        mul_by_const_acc_phase_clean(b, &tx, scale, &scaled_tx, p, false);
                    }
                    with_kal_inv_raw(b, &scaled_tx, p, pair2_iters, |b, inv_raw| {
                        b.set_phase("pair2_prescale_mul");
                        mod_mul_add_into_acc_selected(
                            b,
                            &lam,
                            inv_raw,
                            &ty,
                            p,
                            "PAIR2_CLEAN_LAM_MUL",
                        );
                        b.set_phase("pair2_prescale_cleanup");
                        mod_sub_qb(b, &ty, &oy, p);
                        b.set_phase("pair2_kaliski_backward_prescaled_safe");
                    });
                    b.set_phase("pair2_unprescale_den_safe");
                    if prescale_pair2_chunked {
                        mul_by_const_acc_chunked_shifts_inplace_src(
                            b, &tx, scale, &scaled_tx, p, true,
                        );
                    } else if prescale_pair2_mixed {
                        mul_by_const_acc_exact_adds_fast_shifts(b, &tx, scale, &scaled_tx, p, true);
                    } else {
                        mul_by_const_acc_phase_clean(b, &tx, scale, &scaled_tx, p, true);
                    }
                    b.free_vec(&scaled_tx);
                }
            } else {
                let borrow_pair2_tx_inverse =
                    std::env::var("KAL_PAIR2_BORROW_TX_INVERSE").ok().as_deref() == Some("1")
                        && !defer_pair2_product;
                if borrow_pair2_tx_inverse {
                    b.set_phase("pair2_kaliski_forward_borrow_tx");
                    with_kal_inv_raw_borrowing_v(b, &tx, p, pair2_iters, |b, inv_raw| {
                        b.set_phase("pair2_double");
                        for _ in 0..pair2_iters {
                            mod_double_inplace_fast(b, &lam, p);
                        }
                        b.set_phase("pair2_mul");
                        mod_mul_add_into_acc_selected(
                            b,
                            &lam,
                            inv_raw,
                            &ty,
                            p,
                            "PAIR2_CLEAN_LAM_MUL",
                        );
                        b.set_phase("pair2_cleanup");
                        mod_sub_qb(b, &ty, &oy, p);
                        if free_lam_before_pair2_backward {
                            b.set_phase("pair2_free_lam_before_kaliski_backward");
                            b.free_vec(&lam);
                            lam_freed_before_pair2_backward.set(true);
                        }
                        b.set_phase("pair2_kaliski_backward_borrow_tx");
                    });
                } else {
                    with_kal_inv_raw(b, &tx, p, pair2_iters, |b, inv_raw| {
                        if defer_pair2_product && ty_lent_pair2_forward.get() {
                            b.set_phase("pair2_reacquire_ty_and_mul3_after_kaliski_forward");
                            b.reacquire_vec(&ty);
                            if std::env::var("KAL_DEFER_PAIR2_PRODUCT_LOWQ")
                                .ok()
                                .as_deref()
                                == Some("1")
                            {
                                mod_mul_add_qq(b, &ty, &lam, &tx, p);
                            } else {
                                mod_mul_write_into_zero_acc_karatsuba2(b, &ty, &lam, &tx, p);
                            }
                        }
                        b.set_phase("pair2_double");
                        for _ in 0..pair2_iters {
                            mod_double_inplace_fast(b, &lam, p);
                        }
                        b.set_phase("pair2_mul");
                        mod_mul_add_into_acc_selected(
                            b,
                            &lam,
                            inv_raw,
                            &ty,
                            p,
                            "PAIR2_CLEAN_LAM_MUL",
                        );
                        b.set_phase("pair2_cleanup");
                        mod_sub_qb(b, &ty, &oy, p);
                        if free_lam_before_pair2_backward {
                            b.set_phase("pair2_free_lam_before_kaliski_backward");
                            b.free_vec(&lam);
                            lam_freed_before_pair2_backward.set(true);
                        }
                        b.set_phase("pair2_kaliski_backward");
                    });
                }
            }
        }
    }
    mod_add_qb(b, &tx, &ox, p);
    if !lam_freed_before_pair2_backward.get() {
        b.free_vec(&lam);
    }
}
