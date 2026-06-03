//! `dialog::compressed2` — verbatim split of the original `dialog` module.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

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
