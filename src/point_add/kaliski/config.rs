
#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

pub(crate) fn r_small_threshold() -> usize {
    std::env::var("KAL_R_SMALL_THRESHOLD")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(R_SMALL_THRESHOLD)
}

pub(crate) fn bulk_prefix_safe_iters() -> usize {
    let centered_roundtrip_hook = std::env::var("BY_CENTERED_CLEAN_ROUNDTRIP_BENCH")
        .ok()
        .as_deref()
        == Some("1")
        || std::env::var("BY_CENTERED_FAST_CLEAN_ROUNDTRIP_BENCH")
            .ok()
            .as_deref()
            == Some("1")
        || std::env::var("BY_CENTERED_DENOM_CONTROLS_BENCH")
            .ok()
            .as_deref()
            == Some("1")
        || std::env::var("BY_CENTERED_LIVE_NUM_BENCH").ok().as_deref() == Some("1")
        || std::env::var("BY_CENTERED_PAIR1_REPLACE").ok().as_deref() == Some("1")
        || std::env::var("BY_CENTERED_PAIR2_REPLACE").ok().as_deref() == Some("1")
        || std::env::var("BY_SCALED_PAIR2_PRODUCT_REPLACE")
            .ok()
            .as_deref()
            == Some("1");
    let centered_q_payload_hook = std::env::var("BY_CENTERED_WINDOW_Q_DENOM_REPLACE")
        .ok()
        .as_deref()
        == Some("1");
    let default = if centered_q_payload_hook {
        // The narrower q-payload history changes the circuit shape enough that
        // the old 370 centered-hook Kaliski prefix hits an altseed phase cliff.
        // This env path is an ugly integration probe; use a conservative prefix
        // rather than letting the remaining Kaliski scaffold dominate the test.
        360
    } else if centered_roundtrip_hook {
        // The huge centered roundtrip hooks change the circuit hash / RNG stream
        // enough that the aggressively tuned 375 bulk-prefix setting can hit a
        // rare phase cliff in the old Kaliski scaffold. Use the previously
        // validated 370 setting for these smoke hooks; normal default remains 378.
        370
    } else {
        BULK_PREFIX_SAFE_ITERS
    };
    std::env::var("KAL_BULK3_ITERS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(default)
}

pub(crate) fn bulk_prefix_enabled() -> bool {
    match std::env::var("KAL_BULK3_EXPERIMENT") {
        Ok(v) => v != "0",
        Err(_) => true,
    }
}
