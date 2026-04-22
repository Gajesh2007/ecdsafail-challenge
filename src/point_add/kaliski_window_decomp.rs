//! Decompose the hybrid Kaliski-jump local class family into bulk 4-step
//! windows and rare termination tails.
//!
//! This module answers two practical design questions for the moonshot:
//!
//! 1. How much of the 125-class `t=4` family is really bulk behavior versus
//!    just the last few cleanup windows near `v = 0`?
//! 2. Under the best cheap key so far, `(u_low, v_low, cmp0, cmp1, cmp2)`, what
//!    does the residual ambiguity actually look like?
//!
//! The answer is encouraging:
//! - 99%+ of windows are full 4-step windows;
//! - the 125 observed classes split into 108 bulk 4-step classes plus only 17
//!   short tail classes;
//! - on full windows, the same key determines the first 3-step transform
//!   exactly, and the only remaining 4th-step ambiguity is the final odd/odd
//!   direction `UG` vs `VG`.
//!
//! That points to a concrete prototype path: an exact 3-step batched core plus
//! a normal final micro-step, with the last 3 windows handled by a tiny tail
//! fallback.

use std::collections::{BTreeMap, BTreeSet};

use alloy_primitives::U256;
use sha3::digest::{ExtendableOutput, Update, XofReader};

use super::SECP256K1_P;
use super::kaliski_jump::{kaliski_step_uv, observe_window, KCase};
use super::test_timeout::{check_deadline, two_min_deadline};

pub struct Sampler {
    reader: Box<dyn XofReader>,
    p: U256,
}

impl Sampler {
    pub fn new(seed: &[u8], p: U256) -> Self {
        let mut hasher = sha3::Shake128::default();
        hasher.update(seed);
        Self { reader: Box::new(hasher.finalize_xof()), p }
    }

    pub fn next(&mut self) -> U256 {
        loop {
            let mut buf = [0u8; 32];
            self.reader.read(&mut buf);
            let x = U256::from_le_slice(&buf);
            if x < self.p && !x.is_zero() {
                return x;
            }
        }
    }
}

fn case_bits(k: KCase) -> u16 {
    match k {
        KCase::UEven => 0,
        KCase::VEven => 1,
        KCase::UGtV  => 2,
        KCase::VGtU  => 3,
    }
}

fn encode_cases(cases: &[KCase], t: usize) -> u16 {
    let mut sig = (cases.len() as u16) << (2 * t);
    for (i, kc) in cases.iter().enumerate() {
        sig |= case_bits(*kc) << (2 * i);
    }
    sig
}

fn decode_sig(sig: u16, t: usize) -> Vec<u8> {
    let len = (sig >> (2 * t)) as usize;
    (0..len)
        .map(|i| ((sig >> (2 * i)) & 0b11) as u8)
        .collect()
}

fn prefix_sig(sig: u16, t: usize, prefix_len: usize) -> u16 {
    let seq = decode_sig(sig, t);
    let len = seq.len().min(prefix_len);
    let mut out = (len as u16) << (2 * prefix_len);
    for i in 0..len {
        out |= (seq[i] as u16) << (2 * i);
    }
    out
}

fn seq_string(sig: u16, t: usize) -> String {
    let seq = decode_sig(sig, t);
    let mut out = String::new();
    for (i, c) in seq.iter().enumerate() {
        if i > 0 { out.push('-'); }
        out.push_str(match c {
            0 => "UE",
            1 => "VE",
            2 => "UG",
            3 => "VG",
            _ => "??",
        });
    }
    out
}

#[derive(Debug, Clone)]
pub struct AmbiguousPair {
    pub seq_a: String,
    pub seq_b: String,
    pub key_classes: usize,
    pub windows: usize,
}

#[derive(Debug, Clone)]
pub struct WindowDecompStats {
    pub windows: usize,
    pub full_windows: usize,
    pub short_windows: usize,
    pub full_window_fraction: f64,

    pub len1_windows: usize,
    pub len2_windows: usize,
    pub len3_windows: usize,
    pub len4_windows: usize,

    pub distinct_sequences: usize,
    pub distinct_len1: usize,
    pub distinct_len2: usize,
    pub distinct_len3: usize,
    pub distinct_len4: usize,

    pub compare3_key_classes: usize,
    pub compare3_mean_seq_per_key: f64,
    pub compare3_max_seq_per_key: usize,
    pub compare3_ambiguous_keys: usize,
    pub compare3_ambiguous_windows: usize,
    pub compare3_ambiguous_window_fraction: f64,

    pub compare3_same_prefix3_keys: usize,
    pub compare3_only_last_ug_vg_keys: usize,
    pub compare3_tail_ambiguous_keys: usize,

    pub distinct_prefix3_all: usize,
    pub distinct_prefix3_full: usize,
    pub prefix3_exact_on_full_windows: bool,

    pub top_ambiguous_pairs: Vec<AmbiguousPair>,
}

pub fn analyze_window_decomposition(seed: &[u8], n_inputs: usize, w: usize) -> WindowDecompStats {
    let deadline = two_min_deadline();
    let mut sampler = Sampler::new(seed, SECP256K1_P);

    let mut windows = 0usize;
    let mut len_hist = [0usize; 5];
    let mut distinct_all: BTreeSet<u16> = BTreeSet::new();
    let mut distinct_by_len: [BTreeSet<u16>; 5] = std::array::from_fn(|_| BTreeSet::new());
    let mut distinct_prefix3_all: BTreeSet<u16> = BTreeSet::new();
    let mut distinct_prefix3_full: BTreeSet<u16> = BTreeSet::new();

    let mut key_to_seq4: BTreeMap<(u16, u16, u8, u8, u8), BTreeSet<u16>> = BTreeMap::new();
    let mut key_to_prefix3_full: BTreeMap<(u16, u16, u8, u8, u8), BTreeSet<u16>> = BTreeMap::new();
    let mut key_window_count: BTreeMap<(u16, u16, u8, u8, u8), usize> = BTreeMap::new();

    for input_idx in 0..n_inputs {
        if (input_idx & 31) == 0 { check_deadline(deadline, "kaliski_window_decomp::analyze_window_decomposition"); }
        let mut u = SECP256K1_P;
        let mut v = sampler.next();
        for _ in 0..742 {
            if v.is_zero() { break; }
            let (_u4, _v4, obs) = observe_window(u, v, w, 4);
            let sig4 = encode_cases(&obs.cases, 4);
            let sig3 = prefix_sig(sig4, 4, 3);
            let cmp0 = (u > v) as u8;
            let (u1, v1, _kc1) = kaliski_step_uv(u, v);
            let cmp1 = (u1 > v1) as u8;
            let cmp2 = if !v1.is_zero() {
                let (u2, v2, _kc2) = kaliski_step_uv(u1, v1);
                (u2 > v2) as u8
            } else {
                0
            };
            let key = (obs.low_u, obs.low_v, cmp0, cmp1, cmp2);

            let len = obs.cases.len();
            windows += 1;
            len_hist[len] += 1;
            distinct_all.insert(sig4);
            distinct_by_len[len].insert(sig4);
            distinct_prefix3_all.insert(sig3);
            if len == 4 {
                distinct_prefix3_full.insert(sig3);
                key_to_prefix3_full.entry(key).or_default().insert(sig3);
            }
            key_to_seq4.entry(key).or_default().insert(sig4);
            *key_window_count.entry(key).or_default() += 1;

            u = u1;
            v = v1;
        }
    }

    let key_classes = key_to_seq4.len();
    let mut total_seq_per_key = 0usize;
    let mut max_seq_per_key = 0usize;
    let mut ambiguous_keys = 0usize;
    let mut ambiguous_windows = 0usize;
    let mut same_prefix3_keys = 0usize;
    let mut only_last_ug_vg_keys = 0usize;
    let mut tail_ambiguous_keys = 0usize;
    let mut pair_stats: BTreeMap<(u16, u16), (usize, usize)> = BTreeMap::new();

    for (key, seqs) in &key_to_seq4 {
        let c = seqs.len();
        total_seq_per_key += c;
        if c > max_seq_per_key { max_seq_per_key = c; }
        if c > 1 {
            ambiguous_keys += 1;
            ambiguous_windows += key_window_count[key];

            let mut prefix3s = BTreeSet::new();
            let mut all_full = true;
            for sig in seqs {
                let len = (sig >> 8) as usize;
                if len != 4 { all_full = false; }
                prefix3s.insert(prefix_sig(*sig, 4, 3));
            }
            if prefix3s.len() == 1 { same_prefix3_keys += 1; }
            if !all_full { tail_ambiguous_keys += 1; }
            if c == 2 {
                let mut it = seqs.iter();
                let a = *it.next().unwrap();
                let b = *it.next().unwrap();
                let (x, y) = if a < b { (a, b) } else { (b, a) };
                let xa = decode_sig(x, 4);
                let ya = decode_sig(y, 4);
                if xa.len() == 4 && ya.len() == 4 && xa[..3] == ya[..3] && xa[3] == 2 && ya[3] == 3 {
                    only_last_ug_vg_keys += 1;
                }
                let entry = pair_stats.entry((x, y)).or_default();
                entry.0 += 1;
                entry.1 += key_window_count[key];
            }
        }
    }

    let mut top_ambiguous_pairs = pair_stats.into_iter()
        .map(|((a, b), (key_classes, windows))| AmbiguousPair {
            seq_a: seq_string(a, 4),
            seq_b: seq_string(b, 4),
            key_classes,
            windows,
        })
        .collect::<Vec<_>>();
    top_ambiguous_pairs.sort_by(|x, y| y.windows.cmp(&x.windows).then_with(|| y.key_classes.cmp(&x.key_classes)));
    top_ambiguous_pairs.truncate(12);

    let prefix3_exact_on_full_windows = key_to_prefix3_full.values().all(|s| s.len() == 1);

    WindowDecompStats {
        windows,
        full_windows: len_hist[4],
        short_windows: len_hist[1] + len_hist[2] + len_hist[3],
        full_window_fraction: len_hist[4] as f64 / windows as f64,
        len1_windows: len_hist[1],
        len2_windows: len_hist[2],
        len3_windows: len_hist[3],
        len4_windows: len_hist[4],
        distinct_sequences: distinct_all.len(),
        distinct_len1: distinct_by_len[1].len(),
        distinct_len2: distinct_by_len[2].len(),
        distinct_len3: distinct_by_len[3].len(),
        distinct_len4: distinct_by_len[4].len(),
        compare3_key_classes: key_classes,
        compare3_mean_seq_per_key: total_seq_per_key as f64 / key_classes as f64,
        compare3_max_seq_per_key: max_seq_per_key,
        compare3_ambiguous_keys: ambiguous_keys,
        compare3_ambiguous_windows: ambiguous_windows,
        compare3_ambiguous_window_fraction: ambiguous_windows as f64 / windows as f64,
        compare3_same_prefix3_keys: same_prefix3_keys,
        compare3_only_last_ug_vg_keys: only_last_ug_vg_keys,
        compare3_tail_ambiguous_keys: tail_ambiguous_keys,
        distinct_prefix3_all: distinct_prefix3_all.len(),
        distinct_prefix3_full: distinct_prefix3_full.len(),
        prefix3_exact_on_full_windows,
        top_ambiguous_pairs,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_decomposition_test() {
        let s = analyze_window_decomposition(b"kaliski-window-decomp-seed-v1", 10_000, 8);
        eprintln!("=== Kaliski 4-step window decomposition (w=8) ===");
        eprintln!("windows                        : {}", s.windows);
        eprintln!("full 4-step windows            : {} ({:.4}%)", s.full_windows, 100.0 * s.full_window_fraction);
        eprintln!("short windows                  : {}", s.short_windows);
        eprintln!("window length histogram        : len1={} len2={} len3={} len4={}", s.len1_windows, s.len2_windows, s.len3_windows, s.len4_windows);
        eprintln!("distinct sequences total       : {}", s.distinct_sequences);
        eprintln!("distinct sequences by len      : len1={} len2={} len3={} len4={}", s.distinct_len1, s.distinct_len2, s.distinct_len3, s.distinct_len4);
        eprintln!("distinct 3-step prefixes       : all={} full-only={}", s.distinct_prefix3_all, s.distinct_prefix3_full);
        eprintln!("key=(low,cmp0,cmp1,cmp2) classes: {}", s.compare3_key_classes);
        eprintln!("mean seq/key                   : {:.3}", s.compare3_mean_seq_per_key);
        eprintln!("max seq/key                    : {}", s.compare3_max_seq_per_key);
        eprintln!("ambiguous key classes          : {}", s.compare3_ambiguous_keys);
        eprintln!("ambiguous windows              : {} ({:.4}%)", s.compare3_ambiguous_windows, 100.0 * s.compare3_ambiguous_window_fraction);
        eprintln!("ambiguous keys w/ same 3-prefix: {}", s.compare3_same_prefix3_keys);
        eprintln!("ambiguous keys = final UG/VG   : {}", s.compare3_only_last_ug_vg_keys);
        eprintln!("ambiguous keys involving tails : {}", s.compare3_tail_ambiguous_keys);
        eprintln!("exact 3-step prefix on full    : {}", s.prefix3_exact_on_full_windows);
        eprintln!("top residual ambiguous pairs:");
        for p in &s.top_ambiguous_pairs {
            eprintln!("  {:>6} windows / {:>6} keys : {}  <->  {}", p.windows, p.key_classes, p.seq_a, p.seq_b);
        }
        eprintln!("====================================================");

        assert_eq!(s.distinct_sequences, 125);
        assert_eq!(s.distinct_len1, 1);
        assert_eq!(s.distinct_len2, 4);
        assert_eq!(s.distinct_len3, 12);
        assert_eq!(s.distinct_len4, 108);
        assert_eq!(s.len1_windows, 10_000);
        assert_eq!(s.len2_windows, 10_000);
        assert_eq!(s.len3_windows, 10_000);
        assert!(s.full_window_fraction > 0.99);
        assert!(s.prefix3_exact_on_full_windows);
        assert_eq!(s.distinct_prefix3_full, 36);
        assert_eq!(s.compare3_max_seq_per_key, 2);
        assert_eq!(s.compare3_ambiguous_keys - s.compare3_same_prefix3_keys, 4);
        assert!(s.compare3_only_last_ug_vg_keys + s.compare3_tail_ambiguous_keys >= s.compare3_same_prefix3_keys);
    }
}
