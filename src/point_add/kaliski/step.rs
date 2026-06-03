
#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

pub(crate) fn kal_step2_lowq_cmp_enabled() -> bool {
    std::env::var("KAL_STEP2_LOWQ_CMP").ok().as_deref() == Some("1")
}

pub(crate) fn kal_step2_with_gt<F: FnOnce(&mut B)>(
    b: &mut B,
    u: &[QubitId],
    v: &[QubitId],
    flag: QubitId,
    body: F,
) {
    if kal_step2_lowq_cmp_enabled() {
        with_gt_lowq(b, u, v, flag, body);
    } else {
        with_gt(b, u, v, flag, body);
    }
}

pub(crate) fn kal_vent_modadd_enabled() -> bool {
    std::env::var("KAL_VENT_MODADD").ok().as_deref() == Some("1")
}

pub(crate) fn kal_vent_halve_enabled() -> bool {
    std::env::var("KAL_VENT_HALVE").ok().as_deref() == Some("1")
}

pub(crate) fn kal_vent_modadd_fast_width() -> Option<usize> {
    std::env::var("KAL_VENT_MODADD_FAST_WIDTH")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
}

pub(crate) fn kal_vent_modadd_borrow_tmp_enabled() -> bool {
    match std::env::var("KAL_VENT_MODADD_BORROW_TMP") {
        Ok(v) => v != "0",
        Err(_) => true,
    }
}

pub(crate) fn kal_step4_use_fast_modadd(width: usize) -> bool {
    if !kal_vent_modadd_enabled() {
        return true;
    }
    kal_vent_modadd_fast_width().is_some_and(|max_width| width <= max_width)
}

pub(crate) fn kal_step4_can_borrow_fast_modadd(width: usize, scratch_len: usize) -> bool {
    kal_vent_modadd_enabled()
        && kal_vent_modadd_borrow_tmp_enabled()
        && width > 1
        && scratch_len >= width - 1
}

pub(crate) fn kal_step4_add_nbit_qq(b: &mut B, a: &[QubitId], acc: &[QubitId]) {
    kal_step4_add_nbit_qq_with_scratch(b, a, acc, &[]);
}


pub(crate) fn kal_step4_add_nbit_qq_with_scratch(
    b: &mut B,
    a: &[QubitId],
    acc: &[QubitId],
    scratch: &[QubitId],
) {
    debug_assert_eq!(a.len(), acc.len());
    if kal_step4_use_fast_modadd(a.len()) {
        add_nbit_qq_fast(b, a, acc);
    } else if kal_step4_can_borrow_fast_modadd(a.len(), scratch.len()) {
        add_nbit_qq_fast_borrowed_carries(b, a, acc, &scratch[..a.len() - 1]);
    } else {
        add_nbit_qq(b, a, acc);
    }
}

pub(crate) fn kal_step4_sub_nbit_qq_with_scratch(
    b: &mut B,
    a: &[QubitId],
    acc: &[QubitId],
    scratch: &[QubitId],
) {
    debug_assert_eq!(a.len(), acc.len());
    if kal_step4_use_fast_modadd(a.len()) {
        sub_nbit_qq_fast(b, a, acc);
    } else if kal_step4_can_borrow_fast_modadd(a.len(), scratch.len()) {
        sub_nbit_qq_fast_borrowed_carries(b, a, acc, &scratch[..a.len() - 1]);
    } else {
        sub_nbit_qq(b, a, acc);
    }
}
