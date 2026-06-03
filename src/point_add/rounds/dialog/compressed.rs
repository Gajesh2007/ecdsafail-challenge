#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;


// ─── merged from compressed1.rs ───

pub(crate) fn dialog_gcd_compressed_sidecar_log_enabled() -> bool {
    std::env::var(DIALOG_GCD_COMPRESSED_SIDECAR_LOG_ENV)
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn dialog_gcd_compressed_block_lifecycle_enabled() -> bool {
    std::env::var(DIALOG_GCD_COMPRESSED_BLOCK_LIFECYCLE_ENV)
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn dialog_gcd_compressed_sidecar_blocks() -> usize {
    (dialog_gcd_active_iterations() + DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE - 1)
        / DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE
}

pub(crate) fn dialog_gcd_compressed_sidecar_bits() -> usize {
    dialog_gcd_compressed_sidecar_blocks() * DIALOG_GCD_HIGH_TAIL_ALIAS_BLOCK_BITS
}

pub(crate) fn dialog_gcd_compressed_sidecar_block(compressed_log: &[QubitId], step: usize) -> &[QubitId] {
    let block = step / DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE;
    let start = block * DIALOG_GCD_HIGH_TAIL_ALIAS_BLOCK_BITS;
    &compressed_log[start..start + DIALOG_GCD_HIGH_TAIL_ALIAS_BLOCK_BITS]
}

pub(crate) fn dialog_gcd_host_reverse_raw_block_enabled() -> bool {
    std::env::var("DIALOG_GCD_HOST_REVERSE_RAW_BLOCK")
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn dialog_gcd_reverse_raw_block_host<'a>(
    u: &'a [QubitId],
    compressed_log: &'a [QubitId],
    block: usize,
) -> Option<&'a [QubitId]> {
    if !dialog_gcd_host_reverse_raw_block_enabled() {
        return None;
    }
    let (start, _) = dialog_gcd_compressed_sidecar_block_step_range(block);
    let active_width = dialog_gcd_tobitvector_active_width(start);
    let want = 2 * active_width - 1;
    if u.len().saturating_sub(active_width) >= want + 2 * DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE {
        return Some(&u[u.len() - 2 * DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE..]);
    }
    let future_start = (block + 1) * DIALOG_GCD_HIGH_TAIL_ALIAS_BLOCK_BITS;
    let future = compressed_log.get(future_start..)?;
    if future.len() >= want + 2 * DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE {
        Some(&future[future.len() - 2 * DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE..])
    } else {
        None
    }
}

pub(crate) fn dialog_gcd_forward_raw_block_host<'a>(
    u: &'a [QubitId],
    compressed_log: &'a [QubitId],
    block: usize,
) -> Option<&'a [QubitId]> {
    if !dialog_gcd_host_reverse_raw_block_enabled() {
        return None;
    }
    let (start, _) = dialog_gcd_compressed_sidecar_block_step_range(block);
    let active_width = dialog_gcd_tobitvector_active_width(start);
    let want = 2 * active_width - 1;
    let future_start = (block + 1) * DIALOG_GCD_HIGH_TAIL_ALIAS_BLOCK_BITS;
    if let Some(future) = compressed_log.get(future_start..) {
        if future.len() >= want + 2 * DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE {
            return Some(&future[future.len() - 2 * DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE..]);
        }
    }
    if u.len().saturating_sub(active_width) >= want + 2 * DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE {
        Some(&u[u.len() - 2 * DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE..])
    } else {
        None
    }
}

pub(crate) fn dialog_gcd_compressed_sidecar_future_carry_slice(
    compressed_log: &[QubitId],
    step: usize,
    active_width: usize,
) -> Option<&[QubitId]> {
    if !dialog_gcd_raw_tobitvector_borrow_future_log_carries_enabled() {
        return None;
    }
    let carry_need = active_width.saturating_sub(1);
    // When hosting the gated register too, request up to carry(n-1)+gated(n)=2n-1
    // clean slots; the consumer splits the returned slice. Graceful: never return
    // fewer than carry_need (so carry borrowing is preserved), never more than
    // what the future region holds.
    let want = if dialog_gcd_host_gated_enabled() {
        2 * active_width - 1
    } else {
        carry_need
    };
    let next_block = step / DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE + 1;
    let start = next_block * DIALOG_GCD_HIGH_TAIL_ALIAS_BLOCK_BITS;
    compressed_log
        .get(start..)
        .filter(|future| future.len() >= carry_need)
        .map(|future| &future[..future.len().min(want)])
}

pub(crate) fn dialog_gcd_compressed_sidecar_block_step_range(block: usize) -> (usize, usize) {
    let start = block * DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE;
    let end = (start + DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE).min(dialog_gcd_active_iterations());
    (start, end)
}

pub(crate) fn dialog_gcd_copy_compressed_block_to_raw(
    b: &mut B,
    compressed_block: &[QubitId],
    raw_block: &[QubitId],
) {
    assert_eq!(
        compressed_block.len(),
        DIALOG_GCD_HIGH_TAIL_ALIAS_BLOCK_BITS
    );
    assert_eq!(raw_block.len(), 2 * DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE);
    for i in 0..DIALOG_GCD_HIGH_TAIL_ALIAS_BLOCK_BITS {
        b.cx(compressed_block[i], raw_block[i]);
    }
    emit_dialog_gcd_round763_compressor_inverse(b, raw_block);
}

pub(crate) fn emit_dialog_gcd_compressed_sidecar_tobitvector_steps_block_lifecycle(
    b: &mut B,
    u: &[QubitId],
    v: &[QubitId],
    compressed_log: &[QubitId],
    raw_block: &[QubitId],
) {
    assert_eq!(u.len(), N);
    assert_eq!(v.len(), N);
    assert!(
        raw_block.is_empty() || raw_block.len() == 2 * DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE
    );
    assert!(compressed_log.len() >= dialog_gcd_compressed_sidecar_bits());

    for block in 0..dialog_gcd_compressed_sidecar_blocks() {
        let (start, end) = dialog_gcd_compressed_sidecar_block_step_range(block);
        let hosted_raw_block = dialog_gcd_forward_raw_block_host(u, compressed_log, block);
        let owned_raw_block = if dialog_gcd_host_reverse_raw_block_enabled() && hosted_raw_block.is_none() {
            b.alloc_qubits(2 * DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE)
        } else {
            Vec::new()
        };
        let raw_block = hosted_raw_block.unwrap_or_else(|| {
            if owned_raw_block.is_empty() {
                raw_block
            } else {
                &owned_raw_block
            }
        });
        for step in start..end {
            let slot = step - start;
            let b0 = raw_block[2 * slot];
            let b0_and_b1 = raw_block[2 * slot + 1];
            let active_width = dialog_gcd_tobitvector_active_width(step);
            let u_active = &u[..active_width];
            let v_active = &v[..active_width];
            let compare_bits = dialog_gcd_compare_bits_for_step(step, active_width);

            let borrowed_carries = dialog_gcd_pick_borrow_slice(
                dialog_gcd_compressed_sidecar_future_carry_slice(
                    compressed_log,
                    step,
                    active_width,
                ),
                u,
                active_width,
            );

            b.set_phase("dialog_gcd_compressed_block_tobitvector_branch_bits");
            b.cx(v[0], b0);
            if dialog_gcd_fused_branch_bits_enabled() {
                // Fused path derives b0_and_b1 from the in-flight comparator carry
                // and never materializes a separate `cmp` ancilla. Allocating it
                // here would add a dead live-qubit at the branch_bits peak instant
                // (peak is measured by simultaneously-live count, not qubit-id reuse),
                // so it is allocated only on the non-fused branch below.
                if dialog_gcd_branch_bits_host_comparator_enabled() {
                    // Host the comparator's c_in+carries transient on the idle
                    // future-log slice (the same slice the subtract borrows below;
                    // it is unwritten at the comparator instant) so branch_bits no
                    // longer allocates its own peak qubit. Value-exact; the slice is
                    // returned clean by the measured uncompute sweep.
                    dialog_gcd_ccx_cmp_gt_truncated_into_width_hosted(
                        b,
                        u_active,
                        v_active,
                        b0,
                        b0_and_b1,
                        compare_bits,
                        borrowed_carries,
                    );
                } else {
                    dialog_gcd_ccx_cmp_gt_truncated_into_width(
                        b,
                        u_active,
                        v_active,
                        b0,
                        b0_and_b1,
                        compare_bits,
                    );
                }
            } else {
                let cmp = b.alloc_qubit();
                dialog_gcd_cmp_gt_truncated_into_width(b, u_active, v_active, cmp, compare_bits);
                b.ccx(b0, cmp, b0_and_b1);
                dialog_gcd_cmp_gt_truncated_into_width(b, u_active, v_active, cmp, compare_bits);
                b.free(cmp);
            }

            b.set_phase("dialog_gcd_compressed_block_tobitvector_cswap");
            for (i, (&ui, &vi)) in u_active.iter().zip(v_active.iter()).enumerate() {
                if i == 0 && dialog_gcd_odd_u_lowbit_fastpath_enabled() {
                    continue;
                }
                cswap(b, b0_and_b1, ui, vi);
            }

            b.set_phase("dialog_gcd_compressed_block_tobitvector_subtract");
            dialog_gcd_controlled_sub_selected(b, u_active, v_active, b0, borrowed_carries);

            b.set_phase("dialog_gcd_compressed_block_tobitvector_shift");
            dialog_gcd_shift_right_assuming_even(b, v_active);
        }

        b.set_phase("dialog_gcd_compressed_block_tobitvector_compress_block");
        emit_dialog_gcd_round763_compressor(b, raw_block);
        let compressed_block = dialog_gcd_compressed_sidecar_block(compressed_log, start);
        for i in 0..DIALOG_GCD_HIGH_TAIL_ALIAS_BLOCK_BITS {
            b.swap(raw_block[i], compressed_block[i]);
        }
        if !owned_raw_block.is_empty() {
            b.free_vec(&owned_raw_block);
        }
    }
}

pub(crate) fn emit_dialog_gcd_compressed_sidecar_tobitvector_steps_reverse_block_lifecycle(
    b: &mut B,
    u: &[QubitId],
    v: &[QubitId],
    compressed_log: &[QubitId],
    raw_block: &[QubitId],
) {
    assert_eq!(u.len(), N);
    assert_eq!(v.len(), N);
    assert!(
        raw_block.is_empty() || raw_block.len() == 2 * DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE
    );
    assert!(compressed_log.len() >= dialog_gcd_compressed_sidecar_bits());

    for block in (0..dialog_gcd_compressed_sidecar_blocks()).rev() {
        let (start, end) = dialog_gcd_compressed_sidecar_block_step_range(block);
        let compressed_block = dialog_gcd_compressed_sidecar_block(compressed_log, start);
        let hosted_raw_block = dialog_gcd_reverse_raw_block_host(u, compressed_log, block);
        let owned_raw_block = if dialog_gcd_host_reverse_raw_block_enabled() && hosted_raw_block.is_none() {
            b.alloc_qubits(2 * DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE)
        } else {
            Vec::new()
        };
        let raw_block = hosted_raw_block.unwrap_or_else(|| {
            if owned_raw_block.is_empty() {
                raw_block
            } else {
                &owned_raw_block
            }
        });

        b.set_phase("dialog_gcd_compressed_block_tobitvector_reverse_decompress_block");
        for i in 0..DIALOG_GCD_HIGH_TAIL_ALIAS_BLOCK_BITS {
            b.swap(compressed_block[i], raw_block[i]);
        }
        emit_dialog_gcd_round763_compressor_inverse(b, raw_block);

        for step in (start..end).rev() {
            let slot = step - start;
            let b0 = raw_block[2 * slot];
            let b0_and_b1 = raw_block[2 * slot + 1];
            let active_width = dialog_gcd_tobitvector_active_width(step);
            let u_active = &u[..active_width];
            let v_active = &v[..active_width];
            let compare_bits = dialog_gcd_compare_bits_for_step(step, active_width);

            b.set_phase("dialog_gcd_compressed_block_tobitvector_reverse_unshift");
            dialog_gcd_unshift_right_assuming_even(b, v_active);

            b.set_phase("dialog_gcd_compressed_block_tobitvector_reverse_add");
            let borrowed_carries = dialog_gcd_pick_borrow_slice(
                dialog_gcd_compressed_sidecar_future_carry_slice(
                    compressed_log,
                    step,
                    active_width,
                ),
                u,
                active_width,
            );
            dialog_gcd_controlled_add_selected(b, u_active, v_active, b0, borrowed_carries);

            b.set_phase("dialog_gcd_compressed_block_tobitvector_reverse_cswap");
            for (i, (&ui, &vi)) in u_active.iter().zip(v_active.iter()).enumerate() {
                if i == 0 && dialog_gcd_odd_u_lowbit_fastpath_enabled() {
                    continue;
                }
                cswap(b, b0_and_b1, ui, vi);
            }

            b.set_phase("dialog_gcd_compressed_block_tobitvector_reverse_branch_bits");
            if dialog_gcd_fused_branch_bits_enabled() {
                // Fused path: no separate `cmp` ancilla (derives b0_and_b1 from the
                // comparator carry). Allocating it would add a dead live-qubit at the
                // reverse_branch_bits peak instant, so allocate only on the non-fused
                // branch below. See forward lifecycle for the rationale.
                if dialog_gcd_branch_bits_host_comparator_enabled() {
                    // Mirror of the forward path: host the comparator transient on
                    // the idle future-log slice (same slice the add borrowed above).
                    dialog_gcd_ccx_cmp_gt_truncated_into_width_hosted(
                        b,
                        u_active,
                        v_active,
                        b0,
                        b0_and_b1,
                        compare_bits,
                        borrowed_carries,
                    );
                } else {
                    dialog_gcd_ccx_cmp_gt_truncated_into_width(
                        b,
                        u_active,
                        v_active,
                        b0,
                        b0_and_b1,
                        compare_bits,
                    );
                }
            } else {
                let cmp = b.alloc_qubit();
                dialog_gcd_cmp_gt_truncated_into_width(b, u_active, v_active, cmp, compare_bits);
                b.ccx(b0, cmp, b0_and_b1);
                dialog_gcd_cmp_gt_truncated_into_width(b, u_active, v_active, cmp, compare_bits);
                b.free(cmp);
            }
            b.cx(v[0], b0);
        }
        if !owned_raw_block.is_empty() {
            b.free_vec(&owned_raw_block);
        }
    }
}

pub(crate) fn emit_dialog_gcd_compressed_sidecar_apply_bitvector_block_lifecycle(
    b: &mut B,
    compressed_log: &[QubitId],
    x: &[QubitId],
    y: &[QubitId],
    p: U256,
    raw_block: &[QubitId],
) {
    assert_eq!(x.len(), N);
    assert_eq!(y.len(), N);
    assert_eq!(raw_block.len(), 2 * DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE);

    for block in (0..dialog_gcd_compressed_sidecar_blocks()).rev() {
        let (start, end) = dialog_gcd_compressed_sidecar_block_step_range(block);
        let compressed_block = dialog_gcd_compressed_sidecar_block(compressed_log, start);

        b.set_phase("dialog_gcd_compressed_block_apply_decompress_block");
        dialog_gcd_copy_compressed_block_to_raw(b, compressed_block, raw_block);

        for step in (start..end).rev() {
            let slot = step - start;
            let b0 = raw_block[2 * slot];
            let b0_and_b1 = raw_block[2 * slot + 1];

            b.set_phase("dialog_gcd_compressed_block_apply_double_y");
            mod_double_inplace_fast(b, y, p);

            b.set_phase("dialog_gcd_compressed_block_apply_cadd");
            if dialog_gcd_raw_apply_materialized_special_add_enabled() {
                dialog_gcd_cmod_add_materialized_pseudomersenne(b, y, x, b0, p);
            } else if dialog_gcd_raw_apply_direct_special_add_enabled() {
                dialog_gcd_cmod_add_pseudomersenne_lowq(b, y, x, b0, p);
            } else {
                cmod_add_qq_lowq(b, y, x, b0, p);
            }

            b.set_phase("dialog_gcd_compressed_block_apply_cswap");
            for (&xi, &yi) in x.iter().zip(y.iter()) {
                cswap(b, b0_and_b1, xi, yi);
            }
        }

        b.set_phase("dialog_gcd_compressed_block_apply_clear_block_copy");
        dialog_gcd_clear_raw_block_copy(b, compressed_block, raw_block);
    }
}

pub(crate) fn emit_dialog_gcd_compressed_sidecar_apply_bitvector_reverse_exact_block_lifecycle(
    b: &mut B,
    compressed_log: &[QubitId],
    x: &[QubitId],
    y: &[QubitId],
    p: U256,
    raw_block: &[QubitId],
) {
    assert_eq!(x.len(), N);
    assert_eq!(y.len(), N);
    assert_eq!(raw_block.len(), 2 * DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE);

    for block in 0..dialog_gcd_compressed_sidecar_blocks() {
        let (start, end) = dialog_gcd_compressed_sidecar_block_step_range(block);
        let compressed_block = dialog_gcd_compressed_sidecar_block(compressed_log, start);

        b.set_phase("dialog_gcd_compressed_block_apply_reverse_decompress_block");
        dialog_gcd_copy_compressed_block_to_raw(b, compressed_block, raw_block);

        for step in start..end {
            let slot = step - start;
            let b0 = raw_block[2 * slot];
            let b0_and_b1 = raw_block[2 * slot + 1];

            b.set_phase("dialog_gcd_compressed_block_apply_reverse_cswap");
            for (&xi, &yi) in x.iter().zip(y.iter()) {
                cswap(b, b0_and_b1, xi, yi);
            }

            b.set_phase("dialog_gcd_compressed_block_apply_reverse_csub");
            if dialog_gcd_raw_apply_reverse_materialized_special_sub_enabled() {
                dialog_gcd_cmod_sub_materialized_pseudomersenne(b, y, x, b0, p);
            } else if dialog_gcd_raw_apply_reverse_fast_sub_enabled() {
                cmod_sub_qq(b, y, x, b0, p);
            } else {
                cmod_sub_qq_lowq(b, y, x, b0, p);
            }

            b.set_phase("dialog_gcd_compressed_block_apply_reverse_halve_y");
            mod_halve_inplace_fast(b, y, p);
        }

        b.set_phase("dialog_gcd_compressed_block_apply_reverse_clear_block_copy");
        dialog_gcd_clear_raw_block_copy(b, compressed_block, raw_block);
    }
}

pub(crate) fn emit_dialog_gcd_compressed_sidecar_tobitvector_steps(
    b: &mut B,
    u: &[QubitId],
    v: &[QubitId],
    compressed_log: &[QubitId],
    pair: &[QubitId],
    scratch: QubitId,
) {
    assert_eq!(u.len(), N);
    assert_eq!(v.len(), N);
    assert_eq!(pair.len(), 2);
    assert!(compressed_log.len() >= dialog_gcd_compressed_sidecar_bits());

    for step in 0..dialog_gcd_active_iterations() {
        let b0 = pair[0];
        let b0_and_b1 = pair[1];
        let cmp = b.alloc_qubit();
        let active_width = dialog_gcd_tobitvector_active_width(step);
        let u_active = &u[..active_width];
        let v_active = &v[..active_width];
        let compare_bits = dialog_gcd_compare_bits_for_step(step, active_width);

        b.set_phase("dialog_gcd_compressed_sidecar_tobitvector_branch_bits");
        b.cx(v[0], b0);
        if dialog_gcd_fused_branch_bits_enabled() {
            dialog_gcd_ccx_cmp_gt_truncated_into_width(
                b,
                u_active,
                v_active,
                b0,
                b0_and_b1,
                compare_bits,
            );
        } else {
            dialog_gcd_cmp_gt_truncated_into_width(b, u_active, v_active, cmp, compare_bits);
            b.ccx(b0, cmp, b0_and_b1);
            dialog_gcd_cmp_gt_truncated_into_width(b, u_active, v_active, cmp, compare_bits);
        }
        b.free(cmp);

        b.set_phase("dialog_gcd_compressed_sidecar_tobitvector_cswap");
        for (i, (&ui, &vi)) in u_active.iter().zip(v_active.iter()).enumerate() {
            if i == 0 && dialog_gcd_odd_u_lowbit_fastpath_enabled() {
                continue;
            }
            cswap(b, b0_and_b1, ui, vi);
        }

        b.set_phase("dialog_gcd_compressed_sidecar_tobitvector_subtract");
        let borrowed_carries =
            dialog_gcd_compressed_sidecar_future_carry_slice(compressed_log, step, active_width);
        dialog_gcd_controlled_sub_selected(b, u_active, v_active, b0, borrowed_carries);

        b.set_phase("dialog_gcd_compressed_sidecar_tobitvector_shift");
        dialog_gcd_shift_right_assuming_even(b, v_active);

        b.set_phase("dialog_gcd_compressed_sidecar_tobitvector_absorb_pair");
        let block = dialog_gcd_compressed_sidecar_block(compressed_log, step);
        emit_dialog_gcd_round763_compressed_block_swapper(
            b,
            pair,
            block,
            scratch,
            step % DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE,
        );
    }
}

pub(crate) fn emit_dialog_gcd_compressed_sidecar_tobitvector_steps_reverse(
    b: &mut B,
    u: &[QubitId],
    v: &[QubitId],
    compressed_log: &[QubitId],
    pair: &[QubitId],
    scratch: QubitId,
) {
    assert_eq!(u.len(), N);
    assert_eq!(v.len(), N);
    assert_eq!(pair.len(), 2);
    assert!(compressed_log.len() >= dialog_gcd_compressed_sidecar_bits());

    for step in (0..dialog_gcd_active_iterations()).rev() {
        b.set_phase("dialog_gcd_compressed_sidecar_tobitvector_reverse_load_pair");
        let block = dialog_gcd_compressed_sidecar_block(compressed_log, step);
        emit_dialog_gcd_round763_compressed_block_swapper(
            b,
            pair,
            block,
            scratch,
            step % DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE,
        );

        let b0 = pair[0];
        let b0_and_b1 = pair[1];
        let cmp = b.alloc_qubit();
        let active_width = dialog_gcd_tobitvector_active_width(step);
        let u_active = &u[..active_width];
        let v_active = &v[..active_width];
        let compare_bits = dialog_gcd_compare_bits_for_step(step, active_width);

        b.set_phase("dialog_gcd_compressed_sidecar_tobitvector_reverse_unshift");
        dialog_gcd_unshift_right_assuming_even(b, v_active);

        b.set_phase("dialog_gcd_compressed_sidecar_tobitvector_reverse_add");
        let borrowed_carries =
            dialog_gcd_compressed_sidecar_future_carry_slice(compressed_log, step, active_width);
        dialog_gcd_controlled_add_selected(b, u_active, v_active, b0, borrowed_carries);

        b.set_phase("dialog_gcd_compressed_sidecar_tobitvector_reverse_cswap");
        for (i, (&ui, &vi)) in u_active.iter().zip(v_active.iter()).enumerate() {
            if i == 0 && dialog_gcd_odd_u_lowbit_fastpath_enabled() {
                continue;
            }
            cswap(b, b0_and_b1, ui, vi);
        }

        b.set_phase("dialog_gcd_compressed_sidecar_tobitvector_reverse_branch_bits");
        if dialog_gcd_fused_branch_bits_enabled() {
            dialog_gcd_ccx_cmp_gt_truncated_into_width(
                b,
                u_active,
                v_active,
                b0,
                b0_and_b1,
                compare_bits,
            );
        } else {
            dialog_gcd_cmp_gt_truncated_into_width(b, u_active, v_active, cmp, compare_bits);
            b.ccx(b0, cmp, b0_and_b1);
            dialog_gcd_cmp_gt_truncated_into_width(b, u_active, v_active, cmp, compare_bits);
        }
        b.free(cmp);
        b.cx(v[0], b0);
    }
}

pub(crate) fn emit_dialog_gcd_compressed_sidecar_apply_bitvector(
    b: &mut B,
    compressed_log: &[QubitId],
    x: &[QubitId],
    y: &[QubitId],
    p: U256,
    pair: &[QubitId],
    scratch: QubitId,
) {
    assert_eq!(x.len(), N);
    assert_eq!(y.len(), N);
    assert_eq!(pair.len(), 2);

    for step in (0..dialog_gcd_active_iterations()).rev() {
        b.set_phase("dialog_gcd_compressed_sidecar_apply_load_pair");
        let block = dialog_gcd_compressed_sidecar_block(compressed_log, step);
        emit_dialog_gcd_round763_compressed_block_swapper(
            b,
            pair,
            block,
            scratch,
            step % DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE,
        );

        let b0 = pair[0];
        let b0_and_b1 = pair[1];

        b.set_phase("dialog_gcd_compressed_sidecar_apply_double_y");
        mod_double_inplace_fast(b, y, p);

        b.set_phase("dialog_gcd_compressed_sidecar_apply_cadd");
        if dialog_gcd_raw_apply_materialized_special_add_enabled() {
            dialog_gcd_cmod_add_materialized_pseudomersenne(b, y, x, b0, p);
        } else if dialog_gcd_raw_apply_direct_special_add_enabled() {
            dialog_gcd_cmod_add_pseudomersenne_lowq(b, y, x, b0, p);
        } else {
            cmod_add_qq_lowq(b, y, x, b0, p);
        }

        b.set_phase("dialog_gcd_compressed_sidecar_apply_cswap");
        for (&xi, &yi) in x.iter().zip(y.iter()) {
            cswap(b, b0_and_b1, xi, yi);
        }

        b.set_phase("dialog_gcd_compressed_sidecar_apply_unload_pair");
        let block = dialog_gcd_compressed_sidecar_block(compressed_log, step);
        emit_dialog_gcd_round763_compressed_block_swapper(
            b,
            pair,
            block,
            scratch,
            step % DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE,
        );
    }
}

pub(crate) fn emit_dialog_gcd_compressed_sidecar_apply_bitvector_reverse_exact(
    b: &mut B,
    compressed_log: &[QubitId],
    x: &[QubitId],
    y: &[QubitId],
    p: U256,
    pair: &[QubitId],
    scratch: QubitId,
) {
    assert_eq!(x.len(), N);
    assert_eq!(y.len(), N);
    assert_eq!(pair.len(), 2);

    for step in 0..dialog_gcd_active_iterations() {
        b.set_phase("dialog_gcd_compressed_sidecar_apply_reverse_load_pair");
        let block = dialog_gcd_compressed_sidecar_block(compressed_log, step);
        emit_dialog_gcd_round763_compressed_block_swapper(
            b,
            pair,
            block,
            scratch,
            step % DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE,
        );

        let b0 = pair[0];
        let b0_and_b1 = pair[1];

        b.set_phase("dialog_gcd_compressed_sidecar_apply_reverse_cswap");
        for (&xi, &yi) in x.iter().zip(y.iter()) {
            cswap(b, b0_and_b1, xi, yi);
        }

        b.set_phase("dialog_gcd_compressed_sidecar_apply_reverse_csub");
        if dialog_gcd_raw_apply_reverse_materialized_special_sub_enabled() {
            dialog_gcd_cmod_sub_materialized_pseudomersenne(b, y, x, b0, p);
        } else if dialog_gcd_raw_apply_reverse_fast_sub_enabled() {
            cmod_sub_qq(b, y, x, b0, p);
        } else {
            cmod_sub_qq_lowq(b, y, x, b0, p);
        }

        b.set_phase("dialog_gcd_compressed_sidecar_apply_reverse_halve_y");
        mod_halve_inplace_fast(b, y, p);

        b.set_phase("dialog_gcd_compressed_sidecar_apply_reverse_unload_pair");
        let block = dialog_gcd_compressed_sidecar_block(compressed_log, step);
        emit_dialog_gcd_round763_compressed_block_swapper(
            b,
            pair,
            block,
            scratch,
            step % DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE,
        );
    }
}

pub(crate) fn emit_dialog_gcd_compressed_sidecar_ipmul_block_lifecycle(
    b: &mut B,
    factor: &[QubitId],
    target: &[QubitId],
    p: U256,
) {
    assert_eq!(factor.len(), N);
    assert_eq!(target.len(), N);

    let compressed_log = b.alloc_qubits(dialog_gcd_compressed_sidecar_bits());
    let raw_block = if dialog_gcd_host_reverse_raw_block_enabled() {
        Vec::new()
    } else {
        b.alloc_qubits(2 * DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE)
    };
    let u = b.alloc_qubits(N);
    b.set_phase("dialog_gcd_compressed_block_ipmul_load_p");
    for i in 0..N {
        if bit(p, i) {
            b.x(u[i]);
        }
    }

    b.set_phase("dialog_gcd_compressed_block_ipmul_tobitvector");
    emit_dialog_gcd_compressed_sidecar_tobitvector_steps_block_lifecycle(
        b,
        &u,
        factor,
        &compressed_log,
        &raw_block,
    );

    if dialog_gcd_raw_ipmul_terminal_reuse_enabled() {
        b.set_phase("dialog_gcd_compressed_block_ipmul_release_terminal_u");
        b.x(u[0]);
        b.free_vec(&u);

        b.set_phase("dialog_gcd_compressed_block_ipmul_apply_bitvector_reuse_factor_zero");
        let apply_raw_block = if dialog_gcd_host_reverse_raw_block_enabled() {
            b.alloc_qubits(2 * DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE)
        } else {
            Vec::new()
        };
        emit_dialog_gcd_compressed_sidecar_apply_bitvector_block_lifecycle(
            b,
            &compressed_log,
            target,
            factor,
            p,
            if apply_raw_block.is_empty() { &raw_block } else { &apply_raw_block },
        );
        if !apply_raw_block.is_empty() {
            b.free_vec(&apply_raw_block);
        }

        if dialog_gcd_raw_ipmul_clear_p_residual_enabled() {
            b.set_phase("dialog_gcd_compressed_block_ipmul_clear_p_residual_source_lane");
            for i in 0..N {
                if bit(p, i) {
                    b.x(target[i]);
                }
            }
        }

        b.set_phase("dialog_gcd_compressed_block_ipmul_swap_product_into_target");
        for i in 0..N {
            b.swap(target[i], factor[i]);
        }

        b.set_phase("dialog_gcd_compressed_block_ipmul_reacquire_terminal_u");
        b.reacquire_vec(&u);
        b.set_phase("dialog_gcd_compressed_block_ipmul_seed_terminal_u");
        b.x(u[0]);

        b.set_phase("dialog_gcd_compressed_block_ipmul_uncompute_tobitvector");
        emit_dialog_gcd_compressed_sidecar_tobitvector_steps_reverse_block_lifecycle(
            b,
            &u,
            factor,
            &compressed_log,
            &raw_block,
        );

        b.set_phase("dialog_gcd_compressed_block_ipmul_unload_p");
        for i in 0..N {
            if bit(p, i) {
                b.x(u[i]);
            }
        }
        b.free_vec(&u);
        if !raw_block.is_empty() {
            b.free_vec(&raw_block);
        }
        b.free_vec(&compressed_log);
        return;
    }

    let tmp = b.alloc_qubits(N);
    b.set_phase("dialog_gcd_compressed_block_ipmul_apply_bitvector");
    emit_dialog_gcd_compressed_sidecar_apply_bitvector_block_lifecycle(
        b,
        &compressed_log,
        target,
        &tmp,
        p,
        &raw_block,
    );

    b.set_phase("dialog_gcd_compressed_block_ipmul_swap_product_into_target");
    for i in 0..N {
        b.swap(target[i], tmp[i]);
    }

    b.set_phase("dialog_gcd_compressed_block_ipmul_free_zero_tmp");
    b.free_vec(&tmp);

    b.set_phase("dialog_gcd_compressed_block_ipmul_uncompute_tobitvector");
    emit_dialog_gcd_compressed_sidecar_tobitvector_steps_reverse_block_lifecycle(
        b,
        &u,
        factor,
        &compressed_log,
        &raw_block,
    );

    b.set_phase("dialog_gcd_compressed_block_ipmul_unload_p");
    for i in 0..N {
        if bit(p, i) {
            b.x(u[i]);
        }
    }
    b.free_vec(&u);
    b.free_vec(&raw_block);
    b.free_vec(&compressed_log);
}

// ─── merged from compressed2.rs ───

pub(crate) fn emit_dialog_gcd_compressed_sidecar_ipmul(
    b: &mut B,
    factor: &[QubitId],
    target: &[QubitId],
    p: U256,
) {
    assert_eq!(factor.len(), N);
    assert_eq!(target.len(), N);

    if dialog_gcd_compressed_block_lifecycle_enabled() {
        emit_dialog_gcd_compressed_sidecar_ipmul_block_lifecycle(b, factor, target, p);
        return;
    }

    let compressed_log = b.alloc_qubits(dialog_gcd_compressed_sidecar_bits());
    let pair = b.alloc_qubits(2);
    let compressor_scratch = b.alloc_qubit();
    let u = b.alloc_qubits(N);
    b.set_phase("dialog_gcd_compressed_sidecar_ipmul_load_p");
    for i in 0..N {
        if bit(p, i) {
            b.x(u[i]);
        }
    }

    b.set_phase("dialog_gcd_compressed_sidecar_ipmul_tobitvector");
    emit_dialog_gcd_compressed_sidecar_tobitvector_steps(
        b,
        &u,
        factor,
        &compressed_log,
        &pair,
        compressor_scratch,
    );

    if dialog_gcd_raw_ipmul_terminal_reuse_enabled() {
        b.set_phase("dialog_gcd_compressed_sidecar_ipmul_release_terminal_u");
        b.x(u[0]);
        b.free_vec(&u);

        b.set_phase("dialog_gcd_compressed_sidecar_ipmul_apply_bitvector_reuse_factor_zero");
        emit_dialog_gcd_compressed_sidecar_apply_bitvector(
            b,
            &compressed_log,
            target,
            factor,
            p,
            &pair,
            compressor_scratch,
        );

        if dialog_gcd_raw_ipmul_clear_p_residual_enabled() {
            b.set_phase("dialog_gcd_compressed_sidecar_ipmul_clear_p_residual_source_lane");
            for i in 0..N {
                if bit(p, i) {
                    b.x(target[i]);
                }
            }
        }

        b.set_phase("dialog_gcd_compressed_sidecar_ipmul_swap_product_into_target");
        for i in 0..N {
            b.swap(target[i], factor[i]);
        }

        b.set_phase("dialog_gcd_compressed_sidecar_ipmul_reacquire_terminal_u");
        b.reacquire_vec(&u);
        b.set_phase("dialog_gcd_compressed_sidecar_ipmul_seed_terminal_u");
        b.x(u[0]);

        b.set_phase("dialog_gcd_compressed_sidecar_ipmul_uncompute_tobitvector");
        emit_dialog_gcd_compressed_sidecar_tobitvector_steps_reverse(
            b,
            &u,
            factor,
            &compressed_log,
            &pair,
            compressor_scratch,
        );

        b.set_phase("dialog_gcd_compressed_sidecar_ipmul_unload_p");
        for i in 0..N {
            if bit(p, i) {
                b.x(u[i]);
            }
        }
        b.free_vec(&u);
        b.free(compressor_scratch);
        b.free_vec(&pair);
        b.free_vec(&compressed_log);
        return;
    }

    let tmp = b.alloc_qubits(N);
    b.set_phase("dialog_gcd_compressed_sidecar_ipmul_apply_bitvector");
    emit_dialog_gcd_compressed_sidecar_apply_bitvector(
        b,
        &compressed_log,
        target,
        &tmp,
        p,
        &pair,
        compressor_scratch,
    );

    b.set_phase("dialog_gcd_compressed_sidecar_ipmul_swap_product_into_target");
    for i in 0..N {
        b.swap(target[i], tmp[i]);
    }

    b.set_phase("dialog_gcd_compressed_sidecar_ipmul_free_zero_tmp");
    b.free_vec(&tmp);

    b.set_phase("dialog_gcd_compressed_sidecar_ipmul_uncompute_tobitvector");
    emit_dialog_gcd_compressed_sidecar_tobitvector_steps_reverse(
        b,
        &u,
        factor,
        &compressed_log,
        &pair,
        compressor_scratch,
    );

    b.set_phase("dialog_gcd_compressed_sidecar_ipmul_unload_p");
    for i in 0..N {
        if bit(p, i) {
            b.x(u[i]);
        }
    }
    b.free_vec(&u);
    b.free(compressor_scratch);
    b.free_vec(&pair);
    b.free_vec(&compressed_log);
}

pub(crate) fn emit_dialog_gcd_compressed_sidecar_quotient_block_lifecycle(
    b: &mut B,
    factor: &[QubitId],
    target: &[QubitId],
    p: U256,
) {
    assert_eq!(factor.len(), N);
    assert_eq!(target.len(), N);

    let compressed_log = b.alloc_qubits(dialog_gcd_compressed_sidecar_bits());
    let raw_block = if dialog_gcd_host_reverse_raw_block_enabled() {
        Vec::new()
    } else {
        b.alloc_qubits(2 * DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE)
    };
    let u = b.alloc_qubits(N);
    b.set_phase("dialog_gcd_compressed_block_quotient_load_p");
    for i in 0..N {
        if bit(p, i) {
            b.x(u[i]);
        }
    }

    b.set_phase("dialog_gcd_compressed_block_quotient_tobitvector");
    emit_dialog_gcd_compressed_sidecar_tobitvector_steps_block_lifecycle(
        b,
        &u,
        factor,
        &compressed_log,
        &raw_block,
    );

    if dialog_gcd_raw_quotient_terminal_reuse_enabled() {
        b.set_phase("dialog_gcd_compressed_block_quotient_release_terminal_u");
        b.x(u[0]);
        b.free_vec(&u);

        b.set_phase("dialog_gcd_compressed_block_quotient_apply_reverse_reuse_factor_zero");
        let apply_raw_block = if dialog_gcd_host_reverse_raw_block_enabled() {
            b.alloc_qubits(2 * DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE)
        } else {
            Vec::new()
        };
        emit_dialog_gcd_compressed_sidecar_apply_bitvector_reverse_exact_block_lifecycle(
            b,
            &compressed_log,
            factor,
            target,
            p,
            if apply_raw_block.is_empty() { &raw_block } else { &apply_raw_block },
        );
        if !apply_raw_block.is_empty() {
            b.free_vec(&apply_raw_block);
        }

        b.set_phase("dialog_gcd_compressed_block_quotient_swap_quotient_into_target");
        for i in 0..N {
            b.swap(target[i], factor[i]);
        }

        b.set_phase("dialog_gcd_compressed_block_quotient_reacquire_terminal_u");
        b.reacquire_vec(&u);
        b.set_phase("dialog_gcd_compressed_block_quotient_seed_terminal_u");
        b.x(u[0]);

        b.set_phase("dialog_gcd_compressed_block_quotient_uncompute_tobitvector");
        emit_dialog_gcd_compressed_sidecar_tobitvector_steps_reverse_block_lifecycle(
            b,
            &u,
            factor,
            &compressed_log,
            &raw_block,
        );

        b.set_phase("dialog_gcd_compressed_block_quotient_unload_p");
        for i in 0..N {
            if bit(p, i) {
                b.x(u[i]);
            }
        }
        b.free_vec(&u);
        if !raw_block.is_empty() {
            b.free_vec(&raw_block);
        }
        b.free_vec(&compressed_log);
        return;
    }

    b.set_phase("dialog_gcd_compressed_block_quotient_apply_reverse");
    emit_dialog_gcd_compressed_sidecar_apply_bitvector_reverse_exact_block_lifecycle(
        b,
        &compressed_log,
        factor,
        target,
        p,
        &raw_block,
    );

    b.set_phase("dialog_gcd_compressed_block_quotient_uncompute_tobitvector");
    emit_dialog_gcd_compressed_sidecar_tobitvector_steps_reverse_block_lifecycle(
        b,
        &u,
        factor,
        &compressed_log,
        &raw_block,
    );

    b.set_phase("dialog_gcd_compressed_block_quotient_unload_p");
    for i in 0..N {
        if bit(p, i) {
            b.x(u[i]);
        }
    }
    b.free_vec(&u);
    b.free_vec(&raw_block);
    b.free_vec(&compressed_log);
}

pub(crate) fn emit_dialog_gcd_compressed_sidecar_quotient(
    b: &mut B,
    factor: &[QubitId],
    target: &[QubitId],
    p: U256,
) {
    assert_eq!(factor.len(), N);
    assert_eq!(target.len(), N);

    if dialog_gcd_compressed_block_lifecycle_enabled() {
        emit_dialog_gcd_compressed_sidecar_quotient_block_lifecycle(b, factor, target, p);
        return;
    }

    let compressed_log = b.alloc_qubits(dialog_gcd_compressed_sidecar_bits());
    let pair = b.alloc_qubits(2);
    let compressor_scratch = b.alloc_qubit();
    let u = b.alloc_qubits(N);
    b.set_phase("dialog_gcd_compressed_sidecar_quotient_load_p");
    for i in 0..N {
        if bit(p, i) {
            b.x(u[i]);
        }
    }

    b.set_phase("dialog_gcd_compressed_sidecar_quotient_tobitvector");
    emit_dialog_gcd_compressed_sidecar_tobitvector_steps(
        b,
        &u,
        factor,
        &compressed_log,
        &pair,
        compressor_scratch,
    );

    if dialog_gcd_raw_quotient_terminal_reuse_enabled() {
        b.set_phase("dialog_gcd_compressed_sidecar_quotient_release_terminal_u");
        b.x(u[0]);
        b.free_vec(&u);

        b.set_phase("dialog_gcd_compressed_sidecar_quotient_apply_reverse_reuse_factor_zero");
        emit_dialog_gcd_compressed_sidecar_apply_bitvector_reverse_exact(
            b,
            &compressed_log,
            factor,
            target,
            p,
            &pair,
            compressor_scratch,
        );

        b.set_phase("dialog_gcd_compressed_sidecar_quotient_swap_quotient_into_target");
        for i in 0..N {
            b.swap(target[i], factor[i]);
        }

        b.set_phase("dialog_gcd_compressed_sidecar_quotient_reacquire_terminal_u");
        b.reacquire_vec(&u);
        b.set_phase("dialog_gcd_compressed_sidecar_quotient_seed_terminal_u");
        b.x(u[0]);

        b.set_phase("dialog_gcd_compressed_sidecar_quotient_uncompute_tobitvector");
        emit_dialog_gcd_compressed_sidecar_tobitvector_steps_reverse(
            b,
            &u,
            factor,
            &compressed_log,
            &pair,
            compressor_scratch,
        );

        b.set_phase("dialog_gcd_compressed_sidecar_quotient_unload_p");
        for i in 0..N {
            if bit(p, i) {
                b.x(u[i]);
            }
        }
        b.free_vec(&u);
        b.free(compressor_scratch);
        b.free_vec(&pair);
        b.free_vec(&compressed_log);
        return;
    }

    b.set_phase("dialog_gcd_compressed_sidecar_quotient_apply_reverse");
    emit_dialog_gcd_compressed_sidecar_apply_bitvector_reverse_exact(
        b,
        &compressed_log,
        factor,
        target,
        p,
        &pair,
        compressor_scratch,
    );

    b.set_phase("dialog_gcd_compressed_sidecar_quotient_uncompute_tobitvector");
    emit_dialog_gcd_compressed_sidecar_tobitvector_steps_reverse(
        b,
        &u,
        factor,
        &compressed_log,
        &pair,
        compressor_scratch,
    );

    b.set_phase("dialog_gcd_compressed_sidecar_quotient_unload_p");
    for i in 0..N {
        if bit(p, i) {
            b.x(u[i]);
        }
    }
    b.free_vec(&u);
    b.free(compressor_scratch);
    b.free_vec(&pair);
    b.free_vec(&compressed_log);
}
