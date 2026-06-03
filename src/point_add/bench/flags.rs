//! `bench::flags` — verbatim split of the original `bench` module.

#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

pub(crate) fn round190_selector_fused_source_live_residual_bench_enabled() -> bool {
    std::env::var(ROUND190_SELECTOR_FUSED_SOURCE_LIVE_RESIDUAL_BENCH_ENV)
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn direct_centered_branch_sidecar_bench_enabled() -> bool {
    std::env::var(DIRECT_CENTERED_BRANCH_SIDECAR_BENCH_ENV)
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn direct_centered_branch_retained_finalizer_bench_enabled() -> bool {
    std::env::var(DIRECT_CENTERED_BRANCH_RETAINED_FINALIZER_BENCH_ENV)
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn direct_centered_sidecar_finalizer_fit_bench_enabled() -> bool {
    std::env::var(DIRECT_CENTERED_SIDECAR_FINALIZER_FIT_BENCH_ENV)
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn direct_centered_sidecar_fast_finalizer_fit_bench_enabled() -> bool {
    std::env::var(DIRECT_CENTERED_SIDECAR_FAST_FINALIZER_FIT_BENCH_ENV)
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn direct_centered_branch_digit_clean_fit_bench_enabled() -> bool {
    std::env::var(DIRECT_CENTERED_BRANCH_DIGIT_CLEAN_FIT_BENCH_ENV)
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn direct_centered_branch_replay_finalizer_fit_bench_enabled() -> bool {
    std::env::var(DIRECT_CENTERED_BRANCH_REPLAY_FINALIZER_FIT_BENCH_ENV)
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn direct_centered_branch_predicate_step_fit_bench_enabled() -> bool {
    std::env::var(DIRECT_CENTERED_BRANCH_PREDICATE_STEP_FIT_BENCH_ENV)
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn direct_centered_predicate_replay_finalizer_fit_bench_enabled() -> bool {
    std::env::var(DIRECT_CENTERED_PREDICATE_REPLAY_FINALIZER_FIT_BENCH_ENV)
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn direct_centered_qlow_lowp_branch_row_fit_bench_enabled() -> bool {
    std::env::var(DIRECT_CENTERED_QLOW_LOWP_BRANCH_ROW_FIT_BENCH_ENV)
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn direct_centered_qlow_lowp_branch_digit_row_fit_bench_enabled() -> bool {
    std::env::var(DIRECT_CENTERED_QLOW_LOWP_BRANCH_DIGIT_ROW_FIT_BENCH_ENV)
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn direct_centered_row_transition_fit_bench_enabled() -> bool {
    std::env::var(DIRECT_CENTERED_ROW_TRANSITION_FIT_BENCH_ENV)
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn direct_centered_shifted_source_qbit_row_fit_bench_enabled() -> bool {
    std::env::var(DIRECT_CENTERED_SHIFTED_SOURCE_QBIT_ROW_FIT_BENCH_ENV)
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn direct_centered_inline_predicate_finalizer_delta_fit_bench_enabled() -> bool {
    std::env::var(DIRECT_CENTERED_INLINE_PREDICATE_FINALIZER_DELTA_FIT_BENCH_ENV)
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn direct_centered_binary_trie_qrom_bench_enabled() -> bool {
    std::env::var(DIRECT_CENTERED_BINARY_TRIE_QROM_BENCH_ENV)
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn direct_centered_binary_trie_qrom_roundtrip_bench_enabled() -> bool {
    std::env::var(DIRECT_CENTERED_BINARY_TRIE_QROM_ROUNDTRIP_BENCH_ENV)
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn dialog_gcd_raw_apply_fit_bench_enabled() -> bool {
    std::env::var(DIALOG_GCD_RAW_APPLY_FIT_BENCH_ENV)
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn dialog_gcd_raw_tobitvector_fit_bench_enabled() -> bool {
    std::env::var(DIALOG_GCD_RAW_TOBITVECTOR_FIT_BENCH_ENV)
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn dialog_gcd_raw_ipmul_fit_bench_enabled() -> bool {
    std::env::var(DIALOG_GCD_RAW_IPMUL_FIT_BENCH_ENV)
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn dialog_gcd_high_tail_alias_fit_bench_enabled() -> bool {
    std::env::var(DIALOG_GCD_HIGH_TAIL_ALIAS_FIT_BENCH_ENV)
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn dialog_gcd_compressed_block_primitive_fit_bench_enabled() -> bool {
    std::env::var(DIALOG_GCD_COMPRESSED_BLOCK_PRIMITIVE_FIT_BENCH_ENV)
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn dialog_gcd_high_tail_transcript_overhead_bench_enabled() -> bool {
    std::env::var(DIALOG_GCD_HIGH_TAIL_TRANSCRIPT_OVERHEAD_BENCH_ENV)
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn round556_shifted_source_row_bench_enabled() -> bool {
    std::env::var(ROUND556_SHIFTED_SOURCE_ROW_BENCH_ENV)
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn round125_jsf_operator_bench_enabled() -> bool {
    std::env::var(ROUND125_JSF_OPERATOR_BENCH_ENV)
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn round146_halfgcd_decoder_reuse_bench_enabled() -> bool {
    std::env::var(ROUND146_HALFGCD_DECODER_REUSE_BENCH_ENV)
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn round146_halfgcd_decoder_sequence_bench_enabled() -> bool {
    std::env::var(ROUND146_HALFGCD_DECODER_SEQUENCE_BENCH_ENV)
        .ok()
        .as_deref()
        == Some("1")
}
