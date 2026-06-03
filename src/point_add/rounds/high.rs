
#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;


pub(crate) fn round763_dedup_enabled() -> bool {
    // EXACT rewrite: the pair ccx(1,3->4) ... ccx(1,3->4) bracketing cx(1->0)
    // cancels (nothing between them touches 1/3/4), so it reduces to bare cx(1->0).
    // 2 CCX -> 0 per direction x ~1064 sites. Default OFF (op-stream reseed).
    std::env::var("DIALOG_GCD_ROUND763_DEDUP").ok().as_deref() == Some("1")
}

pub(crate) fn round763_compress_lever_enabled() -> bool {
    // Reachable-support rewrite of the round763 6->5 sidecar packer. Each raw
    // slot is (b0, b0_and_b1), with b0_and_b1 = b0 & (v<u), so state (0,1) is
    // unreachable on the verifier support. On that support, three CCX collapse
    // to CX and the compressor drops from 9 CCX to 4 CCX per direction.
    std::env::var("DIALOG_GCD_ROUND763_COMPRESS_LEVER")
        .ok()
        .as_deref()
        == Some("1")
}
