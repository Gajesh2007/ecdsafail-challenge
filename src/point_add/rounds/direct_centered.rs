//! "Direct centered" branch experiments.
//!
//! Sidecar / finalizer fits, binary-trie QROM addressing, predicate-step rows,
//! and low-path branch row steps.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;


pub(crate) fn emit_round190_direct_centered_signed_digit_body(
    b: &mut B,
    op_sign: QubitId,
    addend: &[QubitId],
    target: &[QubitId],
    carry: QubitId,
) {
    debug_assert_eq!(addend.len(), target.len());
    debug_assert!(addend.len() >= 2);

    b.x(op_sign);
    for &wire in addend {
        b.cx(op_sign, wire);
    }
    b.cx(op_sign, carry);
    b.x(op_sign);

    cuccaro_add(b, addend, target, carry);

    b.x(op_sign);
    b.cx(op_sign, carry);
    for &wire in addend.iter().rev() {
        b.cx(op_sign, wire);
    }
    b.x(op_sign);
}

pub(crate) fn direct_centered_qlow_lowp_branch_row_q_bits_from_env() -> usize {
    std::env::var(DIRECT_CENTERED_QLOW_LOWP_BRANCH_ROW_Q_BITS_ENV)
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|&bits| (1..=N).contains(&bits))
        .unwrap_or(13)
}

pub(crate) fn direct_centered_binary_trie_qrom_address_bits_from_env() -> usize {
    std::env::var(DIRECT_CENTERED_BINARY_TRIE_QROM_ADDRESS_BITS_ENV)
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|&bits| (1..=N).contains(&bits))
        .unwrap_or(13)
}

pub(crate) fn direct_centered_binary_trie_qrom_rows_from_env(address_bits: usize) -> usize {
    let max_rows = 1usize << address_bits;
    std::env::var(DIRECT_CENTERED_BINARY_TRIE_QROM_ROWS_ENV)
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|&rows| rows > 0 && rows <= max_rows)
        .unwrap_or(4_934.min(max_rows))
}

pub(crate) fn direct_centered_binary_trie_qrom_target_bits_from_env() -> usize {
    std::env::var(DIRECT_CENTERED_BINARY_TRIE_QROM_TARGET_BITS_ENV)
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|&bits| (1..=N).contains(&bits))
        .unwrap_or(16)
}

pub(crate) fn emit_direct_centered_low_path_branch_toggle(
    b: &mut B,
    low_path: &[QubitId],
    divisor: &[QubitId],
    branch: QubitId,
    shifted_low_path: &[QubitId],
    divisor_high_zero: QubitId,
    cmp_cin: QubitId,
) {
    assert_eq!(low_path.len(), divisor.len());
    assert_eq!(low_path.len() + 1, shifted_low_path.len());
    assert!(low_path.len() >= 2);

    b.set_phase("direct_centered_branch_predicate_shift_low_path");
    for i in 1..=low_path.len() {
        b.cx(low_path[i - 1], shifted_low_path[i]);
    }

    let mut divisor_ext = Vec::with_capacity(divisor.len() + 1);
    divisor_ext.extend_from_slice(divisor);
    divisor_ext.push(divisor_high_zero);

    b.set_phase("direct_centered_branch_predicate_compare");
    b.x(branch);
    cmp_lt_into_borrowed_cin(b, shifted_low_path, &divisor_ext, branch, cmp_cin);

    b.set_phase("direct_centered_branch_predicate_clear_shift_low_path");
    for i in (1..=low_path.len()).rev() {
        b.cx(low_path[i - 1], shifted_low_path[i]);
    }
}

pub(crate) fn direct_centered_binary_trie_prefix_has_row(
    row_count: usize,
    prefix: usize,
    depth: usize,
    address_bits: usize,
) -> bool {
    if row_count == 0 {
        return false;
    }
    if depth == 0 {
        return true;
    }
    let remaining = address_bits - depth;
    (prefix << remaining) < row_count
}

pub(crate) fn direct_centered_binary_trie_qrom_data_bit(row: usize, bit: usize) -> bool {
    let mut x = (row as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    x ^= (bit as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x.rotate_left(((7 * bit + 3) & 63) as u32);
    ((x >> ((11 * bit + 5) & 63)) & 1) != 0
}

pub(crate) fn direct_centered_binary_trie_qrom_payload_bit(
    table: Option<(&[u64], usize)>,
    row: usize,
    bit: usize,
) -> bool {
    if let Some((words, words_per_row)) = table {
        let word = words[row * words_per_row + bit / 64];
        ((word >> (bit % 64)) & 1) != 0
    } else {
        direct_centered_binary_trie_qrom_data_bit(row, bit)
    }
}

pub(crate) fn direct_centered_binary_trie_qrom_node_count(row_count: usize, address_bits: usize) -> usize {
    fn rec(row_count: usize, address_bits: usize, prefix: usize, depth: usize) -> usize {
        if depth == address_bits {
            return 0;
        }
        let mut total = 0usize;
        for bit in 0..=1usize {
            let child_prefix = (prefix << 1) | bit;
            if direct_centered_binary_trie_prefix_has_row(
                row_count,
                child_prefix,
                depth + 1,
                address_bits,
            ) {
                total += 1 + rec(row_count, address_bits, child_prefix, depth + 1);
            }
        }
        total
    }

    assert!(address_bits < usize::BITS as usize);
    assert!(row_count <= (1usize << address_bits));
    rec(row_count, address_bits, 0, 0)
}

pub(crate) fn emit_direct_centered_binary_trie_qrom_rec(
    b: &mut B,
    address: &[QubitId],
    target: &[QubitId],
    row_count: usize,
    prefix: usize,
    depth: usize,
    parent_flag: QubitId,
    table: Option<(&[u64], usize)>,
) {
    if depth == address.len() {
        if prefix < row_count {
            for bit in 0..target.len() {
                if direct_centered_binary_trie_qrom_payload_bit(table, prefix, bit) {
                    b.cx(parent_flag, target[bit]);
                }
            }
        }
        return;
    }

    for bit in 0..=1usize {
        let child_prefix = (prefix << 1) | bit;
        if !direct_centered_binary_trie_prefix_has_row(
            row_count,
            child_prefix,
            depth + 1,
            address.len(),
        ) {
            continue;
        }

        let selector = address[address.len() - 1 - depth];
        if bit == 0 {
            b.x(selector);
        }
        let child_flag = b.alloc_qubit();
        b.ccx(parent_flag, selector, child_flag);
        if bit == 0 {
            b.x(selector);
        }

        emit_direct_centered_binary_trie_qrom_rec(
            b,
            address,
            target,
            row_count,
            child_prefix,
            depth + 1,
            child_flag,
            table,
        );

        let m = b.alloc_bit();
        b.hmr(child_flag, m);
        if bit == 0 {
            b.x(selector);
        }
        b.cz_if(parent_flag, selector, m);
        if bit == 0 {
            b.x(selector);
        }
        b.bit_store0(m);
        b.free(child_flag);
    }
}

pub(crate) fn emit_direct_centered_binary_trie_qrom_xor(
    b: &mut B,
    address: &[QubitId],
    target: &[QubitId],
    row_count: usize,
) {
    assert!(!address.is_empty());
    assert!(!target.is_empty());
    assert!(address.len() < usize::BITS as usize);
    assert!(row_count > 0);
    assert!(row_count <= (1usize << address.len()));

    let root_flag = b.alloc_qubit();
    b.x(root_flag);
    b.set_phase("direct_centered_binary_trie_qrom_unary_walk");
    emit_direct_centered_binary_trie_qrom_rec(b, address, target, row_count, 0, 0, root_flag, None);

    b.set_phase("direct_centered_binary_trie_qrom_clear_root");
    b.x(root_flag);
    b.free(root_flag);
}

pub(crate) fn emit_direct_centered_binary_trie_qrom_xor_table(
    b: &mut B,
    address: &[QubitId],
    target: &[QubitId],
    row_count: usize,
    table_words: &[u64],
) {
    emit_direct_centered_binary_trie_qrom_xor_table_phased(
        b,
        address,
        target,
        row_count,
        table_words,
        "direct_centered_binary_trie_qrom_unary_walk",
        "direct_centered_binary_trie_qrom_clear_root",
    );
}

pub(crate) fn emit_direct_centered_binary_trie_qrom_xor_table_phased(
    b: &mut B,
    address: &[QubitId],
    target: &[QubitId],
    row_count: usize,
    table_words: &[u64],
    walk_phase: &'static str,
    clear_root_phase: &'static str,
) {
    assert!(!address.is_empty());
    assert!(!target.is_empty());
    assert!(address.len() < usize::BITS as usize);
    assert!(row_count > 0);
    assert!(row_count <= (1usize << address.len()));
    let words_per_row = (target.len() + 63) / 64;
    assert_eq!(table_words.len(), row_count * words_per_row);

    let root_flag = b.alloc_qubit();
    b.x(root_flag);
    b.set_phase(walk_phase);
    emit_direct_centered_binary_trie_qrom_rec(
        b,
        address,
        target,
        row_count,
        0,
        0,
        root_flag,
        Some((table_words, words_per_row)),
    );

    b.set_phase(clear_root_phase);
    b.x(root_flag);
    b.free(root_flag);
}

pub(crate) fn direct_centered_binary_trie_qrom_table_words(row_count: usize, target_bits: usize) -> Vec<u64> {
    let words_per_row = (target_bits + 63) / 64;
    let mut words = vec![0u64; row_count * words_per_row];
    for row in 0..row_count {
        for bit in 0..target_bits {
            if direct_centered_binary_trie_qrom_data_bit(row, bit) {
                words[row * words_per_row + bit / 64] |= 1u64 << (bit % 64);
            }
        }
    }
    words
}

pub(crate) fn emit_direct_centered_branch_sidecar_component(b: &mut B, tx: &[QubitId], ty: &[QubitId]) {
    let digit_lane = b.alloc_qubits(DIRECT_CENTERED_LOW_BRANCH_DIGIT_LANE_BITS);
    let meta = b.alloc_qubits(DIRECT_CENTERED_LOW_BRANCH_META_BITS);
    let prefix = b.alloc_qubits(DIRECT_CENTERED_LOW_BRANCH_PREFIX_BITS);
    let branch = b.alloc_qubits(DIRECT_CENTERED_EXPLICIT_BRANCH_HISTORY_BITS);
    let touch = b.alloc_qubits(DIRECT_CENTERED_BRANCH_SIDECAR_TOUCH_BITS);

    b.set_phase("direct_centered_sidecar_seed_digit_lane");
    for i in 0..digit_lane.len() {
        b.cx(tx[i % tx.len()], digit_lane[i]);
        if i % 2 == 0 {
            b.cx(ty[(3 * i + 1) % ty.len()], digit_lane[i]);
        }
    }

    b.set_phase("direct_centered_sidecar_seed_metadata");
    for i in 0..meta.len() {
        b.cx(tx[(17 * i + 5) % tx.len()], meta[i]);
        b.cx(ty[(19 * i + 7) % ty.len()], meta[i]);
    }

    b.set_phase("direct_centered_sidecar_seed_prefix");
    for i in 0..prefix.len() {
        b.cx(digit_lane[(11 * i + 3) % digit_lane.len()], prefix[i]);
        if i % 3 == 0 {
            b.cx(meta[i % meta.len()], prefix[i]);
        }
    }

    b.set_phase("direct_centered_sidecar_emit_branch_history");
    for i in 0..branch.len() {
        b.ccx(
            prefix[(7 * i + 2) % prefix.len()],
            digit_lane[(13 * i + 1) % digit_lane.len()],
            branch[i],
        );
        b.ccx(
            meta[i % meta.len()],
            ty[(29 * i + 11) % ty.len()],
            branch[i],
        );
    }

    b.set_phase("direct_centered_sidecar_consume_branch_history");
    for i in 0..branch.len() {
        b.ccx(
            branch[i],
            digit_lane[(17 * i + 9) % digit_lane.len()],
            touch[i % touch.len()],
        );
        b.ccx(
            branch[i],
            prefix[(19 * i + 4) % prefix.len()],
            touch[(5 * i + 3) % touch.len()],
        );
    }

    b.set_phase("direct_centered_sidecar_unconsume_branch_history");
    for i in (0..branch.len()).rev() {
        b.ccx(
            branch[i],
            prefix[(19 * i + 4) % prefix.len()],
            touch[(5 * i + 3) % touch.len()],
        );
        b.ccx(
            branch[i],
            digit_lane[(17 * i + 9) % digit_lane.len()],
            touch[i % touch.len()],
        );
    }

    b.set_phase("direct_centered_sidecar_clear_branch_history");
    for i in (0..branch.len()).rev() {
        b.ccx(
            meta[i % meta.len()],
            ty[(29 * i + 11) % ty.len()],
            branch[i],
        );
        b.ccx(
            prefix[(7 * i + 2) % prefix.len()],
            digit_lane[(13 * i + 1) % digit_lane.len()],
            branch[i],
        );
    }

    b.set_phase("direct_centered_sidecar_clear_prefix");
    for i in (0..prefix.len()).rev() {
        if i % 3 == 0 {
            b.cx(meta[i % meta.len()], prefix[i]);
        }
        b.cx(digit_lane[(11 * i + 3) % digit_lane.len()], prefix[i]);
    }

    b.set_phase("direct_centered_sidecar_clear_metadata");
    for i in (0..meta.len()).rev() {
        b.cx(ty[(19 * i + 7) % ty.len()], meta[i]);
        b.cx(tx[(17 * i + 5) % tx.len()], meta[i]);
    }

    b.set_phase("direct_centered_sidecar_clear_digit_lane");
    for i in (0..digit_lane.len()).rev() {
        if i % 2 == 0 {
            b.cx(ty[(3 * i + 1) % ty.len()], digit_lane[i]);
        }
        b.cx(tx[i % tx.len()], digit_lane[i]);
    }

    b.set_phase("direct_centered_sidecar_free");
    b.free_vec(&touch);
    b.free_vec(&branch);
    b.free_vec(&prefix);
    b.free_vec(&meta);
    b.free_vec(&digit_lane);
}

pub(crate) fn emit_direct_centered_branch_retained_finalizer(
    b: &mut B,
    remainder: &[QubitId],
    divisor: &[QubitId],
    branch: QubitId,
    gated_divisor: &[QubitId],
    carry: QubitId,
) {
    assert_eq!(remainder.len(), divisor.len());
    assert_eq!(remainder.len(), gated_divisor.len());

    b.set_phase("direct_centered_branch_retained_finalizer_gate_divisor");
    for i in 0..divisor.len() {
        b.ccx(branch, divisor[i], gated_divisor[i]);
    }

    b.set_phase("direct_centered_branch_retained_finalizer_subtract");
    for &q in gated_divisor {
        b.x(q);
    }
    b.x(carry);
    cuccaro_add(b, gated_divisor, remainder, carry);
    b.x(carry);
    for &q in gated_divisor.iter().rev() {
        b.x(q);
    }

    b.set_phase("direct_centered_branch_retained_finalizer_clear_gated_divisor");
    for i in (0..divisor.len()).rev() {
        b.ccx(branch, divisor[i], gated_divisor[i]);
    }
}

pub(crate) fn emit_direct_centered_branch_retained_finalizer_fast(
    b: &mut B,
    remainder: &[QubitId],
    divisor: &[QubitId],
    branch: QubitId,
    gated_divisor: &[QubitId],
    nonnegative: QubitId,
    carries: &[QubitId],
) {
    assert_eq!(remainder.len(), divisor.len());
    assert_eq!(remainder.len(), gated_divisor.len());
    assert!(carries.len() >= remainder.len().saturating_sub(1));

    b.set_phase("direct_centered_branch_retained_fast_finalizer_gate_divisor");
    for i in 0..divisor.len() {
        b.ccx(branch, divisor[i], gated_divisor[i]);
    }

    b.set_phase("direct_centered_branch_retained_fast_finalizer_subtract");
    b.x(nonnegative);
    for &q in gated_divisor {
        b.cx(nonnegative, q);
    }
    cuccaro_add_fast_borrowed_carries(
        b,
        gated_divisor,
        remainder,
        nonnegative,
        &carries[..remainder.len().saturating_sub(1)],
    );
    for &q in gated_divisor.iter().rev() {
        b.cx(nonnegative, q);
    }
    b.x(nonnegative);

    b.set_phase("direct_centered_branch_retained_fast_finalizer_clear_gated_divisor");
    for i in (0..divisor.len()).rev() {
        b.ccx(branch, divisor[i], gated_divisor[i]);
    }
}

pub(crate) fn emit_direct_centered_branch_digit_update_clean(
    b: &mut B,
    coeff_acc: &[QubitId],
    coeff_v: &[QubitId],
    branch: QubitId,
    sign: QubitId,
    gated_coeff_v: &[QubitId],
    carry: QubitId,
) {
    assert_eq!(coeff_acc.len(), coeff_v.len());
    assert_eq!(coeff_acc.len(), gated_coeff_v.len());

    b.set_phase("direct_centered_branch_digit_clean_gate_coeff");
    for i in 0..coeff_v.len() {
        b.ccx(branch, coeff_v[i], gated_coeff_v[i]);
    }

    // sign=1 adds branch*coeff_v; sign=0 subtracts it.  This clean Cuccaro
    // variant avoids a 255-qubit carry lane, so the digit can execute while
    // the branch history is still live.
    b.set_phase("direct_centered_branch_digit_clean_addsub");
    b.x(carry);
    b.cx(sign, carry);
    for &wire in gated_coeff_v {
        b.cx(carry, wire);
    }
    cuccaro_add(b, gated_coeff_v, coeff_acc, carry);
    for &wire in gated_coeff_v.iter().rev() {
        b.cx(carry, wire);
    }
    b.cx(sign, carry);
    b.x(carry);

    b.set_phase("direct_centered_branch_digit_clean_clear_coeff");
    for i in (0..coeff_v.len()).rev() {
        let m = b.alloc_bit();
        b.hmr(gated_coeff_v[i], m);
        b.cz_if(branch, coeff_v[i], m);
        b.bit_store0(m);
    }
}

pub(crate) fn emit_direct_centered_remainder_abs_swap_transition(
    b: &mut B,
    low_path: &[QubitId],
    divisor: &[QubitId],
    branch: QubitId,
    gated_divisor: &[QubitId],
    carries: &[QubitId],
) {
    assert_eq!(low_path.len(), divisor.len());
    assert_eq!(low_path.len(), gated_divisor.len());
    assert!(low_path.len() >= 2);
    assert!(carries.len() >= low_path.len() - 1);

    b.set_phase("direct_centered_row_transition_branch_negate_low_path");
    for &wire in low_path {
        b.cx(branch, wire);
    }

    b.set_phase("direct_centered_row_transition_gate_divisor");
    for i in 0..divisor.len() {
        b.ccx(branch, divisor[i], gated_divisor[i]);
    }

    b.set_phase("direct_centered_row_transition_abs_add");
    cuccaro_add_fast_borrowed_carries(
        b,
        gated_divisor,
        low_path,
        branch,
        &carries[..low_path.len() - 1],
    );

    b.set_phase("direct_centered_row_transition_clear_gated_divisor");
    for i in (0..divisor.len()).rev() {
        let m = b.alloc_bit();
        b.hmr(gated_divisor[i], m);
        b.cz_if(branch, divisor[i], m);
        b.bit_store0(m);
    }

    b.set_phase("direct_centered_row_transition_swap_next_state");
    for i in 0..low_path.len() {
        b.swap(low_path[i], divisor[i]);
    }
}

pub(crate) fn direct_centered_touch_qubits_for_count(b: &mut B, qubits: &[QubitId]) {
    for &q in qubits {
        b.x(q);
        b.x(q);
    }
}

pub(crate) fn emit_direct_centered_qlow_lowpath_branch_row_step(
    b: &mut B,
    numerator_low_path: &[QubitId],
    divisor: &[QubitId],
    q_low: &[QubitId],
    branch: QubitId,
    decoder_scratch: &[QubitId],
    shifted_low_path: &[QubitId],
    decoder_lt_tmp: QubitId,
    decoder_cmp_cin: QubitId,
    divisor_high_zero: QubitId,
    branch_cmp_cin: QubitId,
) {
    assert_eq!(numerator_low_path.len(), N);
    assert_eq!(divisor.len(), N);
    assert!(!q_low.is_empty());
    assert!(q_low.len() <= N);
    assert_eq!(decoder_scratch.len(), N);
    assert_eq!(shifted_low_path.len(), N + 1);

    b.set_phase("direct_centered_qlow_lowpath_row_decode_q_low");
    halfgcd_coeff_decoder::emit_halfgcd_coeff_quotient_decoder_with_scratch(
        b,
        numerator_low_path,
        divisor,
        q_low,
        decoder_scratch,
        decoder_lt_tmp,
        decoder_cmp_cin,
    );

    b.set_phase("direct_centered_qlow_lowpath_row_branch_predicate");
    emit_direct_centered_low_path_branch_toggle(
        b,
        numerator_low_path,
        divisor,
        branch,
        shifted_low_path,
        divisor_high_zero,
        branch_cmp_cin,
    );
}

pub(crate) fn emit_direct_centered_shifted_source_qbit_row(
    b: &mut B,
    rem: &[QubitId],
    rem_divisor: &[QubitId],
    coeff: &[QubitId],
    coeff_divisor: &[QubitId],
    qbits: &[QubitId],
    gated: &[QubitId],
    lt_tmp: QubitId,
    sign_one: QubitId,
    nonnegative: QubitId,
    carries: &[QubitId],
) {
    let width = rem.len();
    assert_eq!(rem_divisor.len(), width);
    assert_eq!(coeff.len(), width);
    assert_eq!(coeff_divisor.len(), width);
    assert_eq!(gated.len(), width);
    assert!(!qbits.is_empty());
    assert!(qbits.len() <= width);
    assert!(carries.len() >= width - 1);
    let mut remainder_cmp_carries = Vec::with_capacity(width);
    remainder_cmp_carries.extend_from_slice(&carries[..width - 1]);
    remainder_cmp_carries.push(sign_one);
    let mut coeff_cmp_carries = Vec::with_capacity(width);
    coeff_cmp_carries.extend_from_slice(&carries[..width - 1]);
    coeff_cmp_carries.push(lt_tmp);

    b.set_phase("direct_centered_shifted_source_qbit_remainder_digits");
    for q_index in (0..qbits.len()).rev() {
        round556_emit_forward_remainder_digit_borrowed(
            b,
            rem,
            rem_divisor,
            gated,
            lt_tmp,
            nonnegative,
            &remainder_cmp_carries,
            carries,
            qbits[q_index],
            q_index,
        );
    }

    b.set_phase("direct_centered_shifted_source_qbit_coeff_digits");
    b.x(sign_one);
    for q_index in 0..qbits.len() {
        round556_emit_coeff_update_erase_digit_borrowed(
            b,
            coeff,
            coeff_divisor,
            gated,
            sign_one,
            nonnegative,
            &coeff_cmp_carries,
            carries,
            qbits[q_index],
            q_index,
        );
    }
    b.x(sign_one);
}
