//! `dialog::misc2` — verbatim split of the original `dialog` module.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

pub(crate) fn dialog_gcd_add_ctrl_chunked_low_to_ext(
    b: &mut B,
    source: &[QubitId],
    acc_ext: &[QubitId],
    ctrl: QubitId,
    c_in: QubitId,
    blocks: usize,
) {
    let n = source.len();
    assert_eq!(acc_ext.len(), n + 1);
    let ext_n = acc_ext.len();
    let blocks = blocks.max(2).min(ext_n);
    let mut carry = c_in;
    let mut lo = 0usize;
    let mut couts: Vec<(QubitId, usize)> = Vec::new();

    for blk in 0..blocks {
        let hi = dialog_gcd_chunk_hi(blocks, blk, ext_n);
        if hi <= lo {
            continue;
        }
        if blk == blocks - 1 || hi == ext_n {
            let f = dialog_gcd_load_controlled_slice(b, ctrl, source, lo.min(n), n);
            cuccaro_add_fast_low_to_ext(b, &f, &acc_ext[lo..hi], carry);
            dialog_gcd_clear_controlled_slice_hmr(b, ctrl, source, lo.min(n), &f);
            b.free_vec(&f);
            break;
        }

        assert!(hi <= n);
        let f = dialog_gcd_load_controlled_slice(b, ctrl, source, lo, hi);
        let owned_zero = if carry == c_in || !dialog_gcd_apply_chunked_f_reuse_cin_zero_enabled() {
            Some(b.alloc_qubit())
        } else {
            None
        };
        let zero = owned_zero.unwrap_or(c_in);
        let cout = b.alloc_qubit();
        let mut a_block = f.clone();
        a_block.push(zero);
        let mut acc_block = acc_ext[lo..hi].to_vec();
        acc_block.push(cout);
        cuccaro_add_fast(b, &a_block, &acc_block, carry);
        if let Some(zero) = owned_zero {
            b.free(zero);
        }
        dialog_gcd_clear_controlled_slice_hmr(b, ctrl, source, lo, &f);
        b.free_vec(&f);
        couts.push((cout, hi));
        carry = cout;
        lo = hi;
    }

    if dialog_gcd_apply_chunked_f_fuse_boundary_clears_enabled() {
        if let Some(&(_, p)) = couts.last() {
            ccx_cmp_lt_into_fast_prefix_targets(
                b,
                &acc_ext[..p],
                &source[..p],
                ctrl,
                &couts,
            );
        }
    } else {
        for &(cout, p) in couts.iter().rev() {
            ccx_cmp_lt_into_fast(b, &acc_ext[..p], &source[..p], ctrl, cout);
        }
    }
    for &(cout, _) in couts.iter().rev() {
        b.free(cout);
    }
}

pub(crate) fn dialog_gcd_sub_ctrl_chunked_low_to_ext(
    b: &mut B,
    source: &[QubitId],
    acc_ext: &[QubitId],
    ctrl: QubitId,
    c_in: QubitId,
    blocks: usize,
) {
    let n = source.len();
    assert_eq!(acc_ext.len(), n + 1);
    let ext_n = acc_ext.len();
    let blocks = blocks.max(2).min(ext_n);
    let mut borrow = c_in;
    let mut lo = 0usize;
    let mut bouts: Vec<(QubitId, usize)> = Vec::new();

    for blk in 0..blocks {
        let hi = dialog_gcd_chunk_hi(blocks, blk, ext_n);
        if hi <= lo {
            continue;
        }
        if blk == blocks - 1 || hi == ext_n {
            let f = dialog_gcd_load_controlled_slice(b, ctrl, source, lo.min(n), n);
            cuccaro_sub_fast_low_to_ext(b, &f, &acc_ext[lo..hi], borrow);
            dialog_gcd_clear_controlled_slice_hmr(b, ctrl, source, lo.min(n), &f);
            b.free_vec(&f);
            break;
        }

        assert!(hi <= n);
        let f = dialog_gcd_load_controlled_slice(b, ctrl, source, lo, hi);
        let owned_zero = if borrow == c_in || !dialog_gcd_apply_chunked_f_reuse_cin_zero_enabled() {
            Some(b.alloc_qubit())
        } else {
            None
        };
        let zero = owned_zero.unwrap_or(c_in);
        let bout = b.alloc_qubit();
        let mut a_block = f.clone();
        a_block.push(zero);
        let mut acc_block = acc_ext[lo..hi].to_vec();
        acc_block.push(bout);
        cuccaro_sub_fast(b, &a_block, &acc_block, borrow);
        if let Some(zero) = owned_zero {
            b.free(zero);
        }
        dialog_gcd_clear_controlled_slice_hmr(b, ctrl, source, lo, &f);
        b.free_vec(&f);
        bouts.push((bout, hi));
        borrow = bout;
        lo = hi;
    }

    if dialog_gcd_apply_chunked_f_fuse_boundary_clears_enabled() {
        if let Some(&(_, p)) = bouts.last() {
            for i in 0..p {
                b.x(source[i]);
            }
            ccx_cmp_lt_into_fast_prefix_targets(
                b,
                &source[..p],
                &acc_ext[..p],
                ctrl,
                &bouts,
            );
            for i in 0..p {
                b.x(source[i]);
            }
        }
    } else {
        for &(bout, p) in bouts.iter().rev() {
            for i in 0..p {
                b.x(source[i]);
            }
            ccx_cmp_lt_into_fast(b, &source[..p], &acc_ext[..p], ctrl, bout);
            for i in 0..p {
                b.x(source[i]);
            }
        }
    }
    for &(bout, _) in bouts.iter().rev() {
        b.free(bout);
    }
}

pub(crate) fn dialog_gcd_cmod_add_materialized_pseudomersenne_chunked(
    b: &mut B,
    acc: &[QubitId],
    a: &[QubitId],
    ctrl: QubitId,
    p: U256,
    blocks: usize,
) {
    assert_eq!(acc.len(), N);
    assert_eq!(a.len(), N);
    let c = U256::MAX.wrapping_sub(p).wrapping_add(U256::from(1u64));

    let (acc_ext, acc_ovf) = ext_reg(b, acc);
    let c_in = b.alloc_qubit();

    b.set_phase("dialog_gcd_materialized_special_chunked_raw_sum");
    dialog_gcd_add_ctrl_chunked_low_to_ext(b, a, &acc_ext, ctrl, c_in, blocks);
    b.free(c_in);

    b.set_phase("dialog_gcd_materialized_special_overflow_fold");
    if let Some(w) = fold_carry_trunc_window() {
        cadd_nbit_const_direct_trunc_fast(b, &acc[..DIALOG_GCD_SPECIAL_ADD_LSBS], c, acc_ovf, w);
    } else {
        cadd_nbit_const_fast(b, &acc[..DIALOG_GCD_SPECIAL_ADD_LSBS], c, acc_ovf);
    }

    b.set_phase("dialog_gcd_materialized_special_overflow_clean");
    let compare_start = N - dialog_gcd_apply_clean_compare_bits();
    ccx_cmp_lt_into_fast(b, &acc[compare_start..], &a[compare_start..], ctrl, acc_ovf);
    unext_reg(b, acc_ovf);
}

pub(crate) fn dialog_gcd_cmod_sub_materialized_pseudomersenne_chunked(
    b: &mut B,
    acc: &[QubitId],
    a: &[QubitId],
    ctrl: QubitId,
    p: U256,
    blocks: usize,
) {
    assert_eq!(acc.len(), N);
    assert_eq!(a.len(), N);
    let c = U256::MAX.wrapping_sub(p).wrapping_add(U256::from(1u64));

    let (acc_ext, acc_ovf) = ext_reg(b, acc);
    let c_in = b.alloc_qubit();

    b.set_phase("dialog_gcd_materialized_special_chunked_raw_difference");
    dialog_gcd_sub_ctrl_chunked_low_to_ext(b, a, &acc_ext, ctrl, c_in, blocks);
    b.free(c_in);

    b.set_phase("dialog_gcd_materialized_special_underflow_fold");
    if let Some(w) = fold_carry_trunc_window() {
        csub_nbit_const_direct_trunc_fast(b, &acc[..DIALOG_GCD_SPECIAL_ADD_LSBS], c, acc_ovf, w);
    } else {
        csub_nbit_const_fast(b, &acc[..DIALOG_GCD_SPECIAL_ADD_LSBS], c, acc_ovf);
    }

    b.set_phase("dialog_gcd_materialized_special_underflow_clean");
    dialog_gcd_clean_truncated_underflow(b, acc, a, ctrl, acc_ovf);
    unext_reg(b, acc_ovf);
}

pub(crate) fn dialog_gcd_cmod_sub_materialized_pseudomersenne(
    b: &mut B,
    acc: &[QubitId],
    a: &[QubitId],
    ctrl: QubitId,
    p: U256,
) {
    assert_eq!(acc.len(), N);
    assert_eq!(a.len(), N);
    if let Some(blocks) = dialog_gcd_apply_chunked_f_blocks()
        .filter(|_| dialog_gcd_raw_apply_truncated_clean_enabled())
        .filter(|_| dialog_gcd_measured_apply_sub_enabled())
    {
        dialog_gcd_cmod_sub_materialized_pseudomersenne_chunked(b, acc, a, ctrl, p, blocks);
        return;
    }
    let c = U256::MAX.wrapping_sub(p).wrapping_add(U256::from(1u64));

    let f = b.alloc_qubits(N);
    b.set_phase("dialog_gcd_materialized_special_load_subtrahend");
    for i in 0..N {
        b.ccx(ctrl, a[i], f[i]);
    }

    let (acc_ext, acc_ovf) = ext_reg(b, acc);

    b.set_phase("dialog_gcd_materialized_special_raw_difference");
    if dialog_gcd_measured_apply_sub_enabled() {
        // Measured (Gidney) difference: ~n Toffoli instead of the ~2n of the
        // non-fast cuccaro_sub uncompute. Peak-safe: the symmetric apply ADD
        // already runs cuccaro_add_fast with its carry lane in this same phase.
        let c_in = b.alloc_qubit();
        if let Some(w) = dialog_gcd_apply_window_blocks() {
            cuccaro_sub_fast_windowed_low_to_ext(b, &f, &acc_ext, c_in, w);
        } else {
            let f_ovf = b.alloc_qubit();
            let mut f_ext = f.clone();
            f_ext.push(f_ovf);
            cuccaro_sub_fast(b, &f_ext, &acc_ext, c_in);
            b.free(f_ovf);
        }
        b.free(c_in);
    } else {
        let f_ovf = b.alloc_qubit();
        let mut f_ext = f.clone();
        f_ext.push(f_ovf);
        sub_nbit_qq(b, &f_ext, &acc_ext);
        b.free(f_ovf);
    }

    b.set_phase("dialog_gcd_materialized_special_underflow_fold");
    if let Some(w) = fold_carry_trunc_window() {
        csub_nbit_const_direct_trunc_fast(b, &acc[..DIALOG_GCD_SPECIAL_ADD_LSBS], c, acc_ovf, w);
    } else {
        csub_nbit_const_fast(b, &acc[..DIALOG_GCD_SPECIAL_ADD_LSBS], c, acc_ovf);
    }

    b.set_phase("dialog_gcd_materialized_special_underflow_clean");
    if dialog_gcd_raw_apply_truncated_clean_enabled() {
        dialog_gcd_clean_truncated_underflow(b, acc, a, ctrl, acc_ovf);
    } else {
        b.x(acc_ovf);
        mod_neg_inplace_fast(b, &f, p);
        cmp_lt_into_fast(b, acc, &f, acc_ovf);
        mod_neg_inplace_fast(b, &f, p);
    }
    unext_reg(b, acc_ovf);

    b.set_phase("dialog_gcd_materialized_special_clear_subtrahend");
    for i in 0..N {
        let m = b.alloc_bit();
        b.hmr(f[i], m);
        b.cz_if(ctrl, a[i], m);
    }
    b.free_vec(&f);
}

pub(crate) fn dialog_gcd_cmod_sub_materialized_pseudomersenne_borrowed_subtrahend(
    b: &mut B,
    acc: &[QubitId],
    a: &[QubitId],
    ctrl: QubitId,
    p: U256,
    f: &[QubitId],
) {
    assert_eq!(acc.len(), N);
    assert_eq!(a.len(), N);
    assert_eq!(f.len(), N);
    let c = U256::MAX.wrapping_sub(p).wrapping_add(U256::from(1u64));

    b.set_phase("dialog_gcd_materialized_special_borrowed_load_subtrahend");
    for i in 0..N {
        b.ccx(ctrl, a[i], f[i]);
    }

    let (acc_ext, acc_ovf) = ext_reg(b, acc);
    let f_ovf = b.alloc_qubit();
    let mut f_ext = f.to_vec();
    f_ext.push(f_ovf);

    b.set_phase("dialog_gcd_materialized_special_borrowed_raw_difference");
    sub_nbit_qq(b, &f_ext, &acc_ext);
    b.free(f_ovf);

    b.set_phase("dialog_gcd_materialized_special_borrowed_underflow_fold");
    if let Some(w) = fold_carry_trunc_window() {
        csub_nbit_const_direct_trunc_fast(b, &acc[..DIALOG_GCD_SPECIAL_ADD_LSBS], c, acc_ovf, w);
    } else {
        csub_nbit_const_fast(b, &acc[..DIALOG_GCD_SPECIAL_ADD_LSBS], c, acc_ovf);
    }

    b.set_phase("dialog_gcd_materialized_special_borrowed_underflow_clean");
    if dialog_gcd_raw_apply_truncated_clean_enabled() {
        dialog_gcd_clean_truncated_underflow(b, acc, a, ctrl, acc_ovf);
    } else {
        b.x(acc_ovf);
        mod_neg_inplace_fast(b, f, p);
        cmp_lt_into_fast(b, acc, f, acc_ovf);
        mod_neg_inplace_fast(b, f, p);
    }
    unext_reg(b, acc_ovf);

    b.set_phase("dialog_gcd_materialized_special_borrowed_clear_subtrahend");
    for i in (0..N).rev() {
        b.ccx(ctrl, a[i], f[i]);
    }
}

pub(crate) fn dialog_gcd_measured_underflow_gate_enabled() -> bool {
    // Measured (Gidney) uncompute of acc_ovf = ctrl & underflow_pred in the
    // materialized_special underflow_clean (HMR + cz_if = 0 Toffoli vs 1 CCX/iter).
    // Exact on the validated reroll island. Default OFF.
    std::env::var("DIALOG_GCD_MEASURED_UNDERFLOW_GATE")
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn dialog_gcd_high_tail_touch_qubits_for_count(b: &mut B, qubits: &[QubitId]) {
    for &q in qubits {
        b.x(q);
        b.x(q);
    }
}

pub(crate) fn dialog_gcd_high_tail_alias_cell_qubit(
    cell: DialogGcdHighTailCell,
    u_ext: &[QubitId],
    v_low: &[QubitId],
    v_tail: &[QubitId],
) -> QubitId {
    assert_eq!(u_ext.len(), DIALOG_GCD_HIGH_TAIL_ALIAS_EXTENDED_BITS);
    assert_eq!(v_low.len(), N);
    assert_eq!(v_tail.len(), DIALOG_GCD_HIGH_TAIL_ALIAS_EXTENSION_BITS);
    match cell.lane {
        DialogGcdHighTailLane::U => {
            assert!(cell.pos >= N);
            u_ext[cell.pos]
        }
        DialogGcdHighTailLane::V => {
            if cell.pos < N {
                v_low[cell.pos]
            } else {
                v_tail[cell.pos - N]
            }
        }
    }
}

pub(crate) fn dialog_gcd_high_tail_block_qubits(
    block: &DialogGcdHighTailBlock,
    u_ext: &[QubitId],
    v_low: &[QubitId],
    v_tail: &[QubitId],
) -> Vec<QubitId> {
    block
        .cells
        .iter()
        .map(|&cell| dialog_gcd_high_tail_alias_cell_qubit(cell, u_ext, v_low, v_tail))
        .collect()
}

pub(crate) fn emit_dialog_gcd_high_tail_transcript_overhead(
    b: &mut B,
    layout: &DialogGcdHighTailLayout,
    u_ext: &[QubitId],
    v_low: &[QubitId],
    v_tail: &[QubitId],
    pair: &[QubitId],
    scratch: QubitId,
) {
    b.set_phase("dialog_gcd_high_tail_transcript_tobitvector_absorb");
    for step in 0..DIALOG_GCD_HIGH_TAIL_ALIAS_ITERATIONS {
        let block = &layout.blocks[step / DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE];
        let slot = step % DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE;
        let block_qubits = dialog_gcd_high_tail_block_qubits(block, u_ext, v_low, v_tail);
        emit_dialog_gcd_round763_compressed_block_swapper(b, pair, &block_qubits, scratch, slot);
    }

    b.set_phase("dialog_gcd_high_tail_transcript_apply_load_unload");
    for step in (0..DIALOG_GCD_HIGH_TAIL_ALIAS_ITERATIONS).rev() {
        let block = &layout.blocks[step / DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE];
        let slot = step % DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE;
        let block_qubits = dialog_gcd_high_tail_block_qubits(block, u_ext, v_low, v_tail);
        emit_dialog_gcd_round763_compressed_block_swapper(b, pair, &block_qubits, scratch, slot);
        emit_dialog_gcd_round763_compressed_block_swapper(b, pair, &block_qubits, scratch, slot);
    }

    b.set_phase("dialog_gcd_high_tail_transcript_reverse_unabsorb");
    for step in (0..DIALOG_GCD_HIGH_TAIL_ALIAS_ITERATIONS).rev() {
        let block = &layout.blocks[step / DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE];
        let slot = step % DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE;
        let block_qubits = dialog_gcd_high_tail_block_qubits(block, u_ext, v_low, v_tail);
        emit_dialog_gcd_round763_compressed_block_swapper(b, pair, &block_qubits, scratch, slot);
    }
}
