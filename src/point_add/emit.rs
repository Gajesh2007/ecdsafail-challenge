//! Generic reversible gate-stream inversion utilities.
//!
//! [`emit_inverse`] records a forward fragment's emitted gates and replays them
//! in reverse to synthesize the fragment's arithmetic inverse, with variants
//! that tolerate clean resets, keep measurement-derived conditions, or stay
//! HMR-safe. [`conjugate`] wraps a compute / body / uncompute sandwich.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;


// ═══════════════════════════════════════════════════════════════════════════
//  emit_inverse: run a closure, pop the ops it emitted, and re-emit them
//  reversed.
//
//  The closure may contain `alloc_qubit` / `free` calls;
//  the R ops that `free` produces are SKIPPED during
//  reverse replay. This relies on the forward being "clean" — i.e. each
//  free lands on a qubit that the forward gates already drove to |0⟩
//  before the R. Under that invariant, the reverse gate sequence brings
//  the same qubit back to |0⟩ at the "alloc" point (pre-forward-allocation),
//  and the R we skipped is unnecessary.
//
//  The forward's internal alloc/free bookkeeping in the B's free
//  pool is NOT undone by the reverse — the pool state at reverse exit
//  equals the pool state at forward exit. Subsequent allocations in the
//  parent scope reuse those qubit IDs, seeing them at |0⟩ (as zeroed by
//  the reverse gate sequence).
// ═══════════════════════════════════════════════════════════════════════════
pub(crate) fn emit_inverse<F: FnOnce(&mut B)>(b: &mut B, f: F) {
    if b.count_only {
        let snap = b.count_snapshot();
        f(b);
        let delta = b.count_delta_since(snap);
        b.restore_count_snapshot(snap);
        add_inverse_count_delta(b, &delta);
        return;
    }
    let start = b.ops.len();
    f(b);
    let end = b.ops.len();
    // Extract the forward slice and drop it from the builder.
    let fwd: Vec<_> = b.ops[start..end].to_vec();
    b.ops.truncate(start);
    emit_inverse_ops_allowing_clean_resets(b, &fwd, "emit_inverse");
}

pub(crate) fn add_inverse_count_delta(b: &mut B, delta: &[usize; 18]) {
    for kind in [
        OperationType::X,
        OperationType::Z,
        OperationType::CX,
        OperationType::CZ,
        OperationType::CCX,
        OperationType::CCZ,
        OperationType::Swap,
    ] {
        b.add_counted_kind(kind, delta[kind as usize]);
    }
}

pub(crate) fn emit_inverse_ops_allowing_clean_resets(b: &mut B, fwd: &[Op], context: &'static str) {
    for op in fwd.iter().rev().copied() {
        match op.kind {
            OperationType::X
            | OperationType::Z
            | OperationType::CX
            | OperationType::CZ
            | OperationType::CCX
            | OperationType::CCZ
            | OperationType::Swap => b.push_op(op),
            // R ops are the free markers. They're not directly reversible
            // as gates, but in a clean forward they're preceded by gates
            // that already zero the qubit. We skip them in reverse.
            OperationType::R => {}
            // Metadata ops (register declarations, debug prints) don't
            // affect state and shouldn't appear inside an emit_inverse
            // closure anyway, but skip them if they do.
            OperationType::Register
            | OperationType::AppendToRegister
            | OperationType::DebugPrint => {}
            _ => panic!(
                "{context}: non-invertible op kind {:?} inside forward block",
                op.kind
            ),
        }
    }
}

pub(crate) fn emit_inverse_ops_measurement_clean_scoped(b: &mut B, fwd: &[Op], context: &'static str) {
    let max_measured_bit = fwd
        .iter()
        .filter(|op| matches!(op.kind, OperationType::Hmr))
        .map(|op| op.c_target.0 as usize)
        .max();
    let mut measured_bits = max_measured_bit
        .map(|idx| vec![false; idx + 1])
        .unwrap_or_default();
    for op in fwd {
        if matches!(op.kind, OperationType::Hmr) {
            measured_bits[op.c_target.0 as usize] = true;
        }
    }

    for op in fwd.iter().rev().copied() {
        if op.c_condition != crate::circuit::NO_BIT
            && measured_bits
                .get(op.c_condition.0 as usize)
                .copied()
                .unwrap_or(false)
        {
            continue;
        }
        match op.kind {
            OperationType::X
            | OperationType::Z
            | OperationType::CX
            | OperationType::CZ
            | OperationType::CCX
            | OperationType::CCZ
            | OperationType::Swap => b.push_op(op),
            OperationType::R
            | OperationType::Hmr
            | OperationType::Register
            | OperationType::AppendToRegister
            | OperationType::DebugPrint => {}
            _ => panic!(
                "{context}: non-invertible op kind {:?} inside forward block",
                op.kind
            ),
        }
    }
}

pub(crate) fn emit_inverse_ops_hmr_safe_keep_conditions(b: &mut B, fwd: &[Op], context: &'static str) {
    for op in fwd.iter().rev().copied() {
        match op.kind {
            OperationType::X
            | OperationType::Z
            | OperationType::CX
            | OperationType::CZ
            | OperationType::CCX
            | OperationType::CCZ
            | OperationType::Swap => b.push_op(op),
            OperationType::R
            | OperationType::Hmr
            | OperationType::Register
            | OperationType::AppendToRegister
            | OperationType::DebugPrint => {}
            _ => panic!(
                "{context}: non-invertible op kind {:?} inside forward block",
                op.kind
            ),
        }
    }
}

/// Runs `compute`, then `body`, then the inverse of `compute` — the
/// "with conjugate" pattern from qrisp. `compute` must emit only
/// reversible gates (no alloc/free/R).
pub(crate) fn conjugate<F, G>(b: &mut B, compute: F, body: G)
where
    F: Fn(&mut B),
    G: FnOnce(&mut B),
{
    compute(b);
    body(b);
    emit_inverse(b, compute);
}

/// Run `body` with `inv` holding `v_in^{-1} mod p`, leaving `v_in`
/// unchanged. Allocates the kaliski state and `inv` register itself, then
/// frees them at the end. The body must NOT touch `st` or `v_in`.
///
/// Implementation keeps `st` live across the body, so we only run
/// `kaliski_forward` ONCE (and its emit_inverse once), instead of the
/// 4-call structure of the previous Bennett-cleaned `kal_compute_into`.
/// Halves the dominant kaliski cost.
pub(crate) fn emit_inverse_hmr_safe<F: FnOnce(&mut B)>(b: &mut B, f: F) {
    let start = b.ops.len();
    f(b);
    let end = b.ops.len();
    let fwd: Vec<_> = b.ops[start..end].to_vec();
    b.ops.truncate(start);
    for op in fwd.into_iter().rev() {
        match op.kind {
            OperationType::X
            | OperationType::Z
            | OperationType::CX
            | OperationType::CZ
            | OperationType::CCX
            | OperationType::CCZ
            | OperationType::Swap => b.push_op(op),
            OperationType::R
            | OperationType::Hmr
            | OperationType::Register
            | OperationType::AppendToRegister
            | OperationType::DebugPrint => {}
            _ => panic!(
                "emit_inverse_hmr_safe: non-invertible op kind {:?} inside forward block",
                op.kind
            ),
        }
    }
}

pub(crate) fn emit_inverse_measurement_clean_scoped<F: FnOnce(&mut B)>(b: &mut B, f: F) {
    let start = b.ops.len();
    let phase_start = b.phase_transitions.len();
    let saved_phase = b.phase;
    f(b);
    let fwd: Vec<_> = b.ops[start..].to_vec();
    let max_measured_bit = fwd
        .iter()
        .filter(|op| matches!(op.kind, OperationType::Hmr))
        .map(|op| op.c_target.0 as usize)
        .max();
    let mut measured_bits = max_measured_bit
        .map(|idx| vec![false; idx + 1])
        .unwrap_or_default();
    for op in &fwd {
        if matches!(op.kind, OperationType::Hmr) {
            measured_bits[op.c_target.0 as usize] = true;
        }
    }
    b.ops.truncate(start);
    b.phase_transitions.truncate(phase_start);
    b.phase = saved_phase;

    for op in fwd.into_iter().rev() {
        if op.c_condition != crate::circuit::NO_BIT
            && measured_bits
                .get(op.c_condition.0 as usize)
                .copied()
                .unwrap_or(false)
        {
            continue;
        }
        match op.kind {
            OperationType::X
            | OperationType::Z
            | OperationType::CX
            | OperationType::CZ
            | OperationType::CCX
            | OperationType::CCZ
            | OperationType::Swap => b.push_op(op),
            OperationType::R
            | OperationType::Hmr
            | OperationType::Register
            | OperationType::AppendToRegister
            | OperationType::DebugPrint => {}
            _ => panic!(
                "emit_inverse_measurement_clean_scoped: non-invertible op kind {:?} inside forward block",
                op.kind
            ),
        }
    }
}
