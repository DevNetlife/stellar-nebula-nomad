//! Gas-optimized computation utilities.
//! Provides efficient algorithms and patterns for common computations.
//!
//! This is a shared utility library: not every helper is called in every
//! build, so unused helpers are allowed rather than reported as dead code.
#![allow(dead_code)]

use soroban_sdk::{BytesN, Env, Vec};

/// Fast hash for small inputs (optimized for gas)
pub fn fast_hash_u64(_env: &Env, input: u64) -> u64 {
    // Simple but fast hash for u64
    let mut hash = input;
    hash ^= hash >> 33;
    hash = hash.wrapping_mul(0xff51afd7ed558ccd);
    hash ^= hash >> 33;
    hash = hash.wrapping_mul(0xc4ceb9fe1a85ec53);
    hash ^= hash >> 33;
    hash
}

// ─── Procedural-generation primitives ────────────────────────────────────
//
// Shared by `nebula_gen` (Issue #438). They work on native byte arrays and
// plain integers, so the caller makes one host call (`BytesN::to_array`)
// instead of one `BytesN::get` host call per byte.

/// Golden-ratio increment used by SplitMix64 and for index spreading.
pub const GOLDEN_GAMMA: u64 = 0x9e37_79b9_7f4a_7c15;

/// SplitMix64 finalizer — bijective, high-quality, deterministic.
#[inline]
pub fn splitmix64(mut z: u64) -> u64 {
    z = z.wrapping_add(GOLDEN_GAMMA);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// Derive an independent u64 for `(seed, spread_index, salt)`.
///
/// `spread_index` is `index * GOLDEN_GAMMA`, computed once per element by the
/// caller and reused for every salt instead of recomputing the multiply for
/// each derived property.
#[inline]
pub fn derive_spread(seed: u64, spread_index: u64, salt: u64) -> u64 {
    splitmix64(seed ^ splitmix64(spread_index ^ salt))
}

/// Fold a 32-byte seed into a u64 by XOR-ing its four little-endian 8-byte
/// lanes, so every bit of the seed influences the result.
#[inline]
pub fn fold_seed_bytes(bytes: &[u8; 32]) -> u64 {
    let mut acc = 0u64;
    let mut lane = [0u8; 8];
    for chunk in bytes.chunks_exact(8) {
        lane.copy_from_slice(chunk);
        acc ^= u64::from_le_bytes(lane);
    }
    acc
}

/// Fold a `BytesN<32>` seed with a single host call (see [`fold_seed_bytes`]).
#[inline]
pub fn fold_seed(seed: &BytesN<32>) -> u64 {
    fold_seed_bytes(&seed.to_array())
}

/// `true` when every byte of the seed is zero.
#[inline]
pub fn is_zero_bytes32(bytes: &[u8; 32]) -> bool {
    bytes.iter().all(|b| *b == 0)
}

/// Expand a u64 into 32 bytes using four independent SplitMix64 lanes.
pub fn expand_u64_to_bytes32(h: u64) -> [u8; 32] {
    let lanes = [
        splitmix64(h),
        splitmix64(h ^ 0xdead_cafe_1234_5678),
        splitmix64(h.wrapping_add(0x1234_5678_dead_beef)),
        splitmix64(h.wrapping_mul(0x0101_0101_0101_0101).wrapping_add(1)),
    ];
    let mut out = [0u8; 32];
    for (dst, lane) in out.chunks_exact_mut(8).zip(lanes.iter()) {
        dst.copy_from_slice(&lane.to_le_bytes());
    }
    out
}

/// Optimized sum for Vec<u32> with loop unrolling
pub fn sum_vec_u32_optimized(values: &Vec<u32>) -> u64 {
    let len = values.len();
    let mut sum: u64 = 0;
    let mut i = 0;

    // Process 4 elements at a time (loop unrolling)
    while i + 4 <= len {
        sum += values.get_unchecked(i) as u64;
        sum += values.get_unchecked(i + 1) as u64;
        sum += values.get_unchecked(i + 2) as u64;
        sum += values.get_unchecked(i + 3) as u64;
        i += 4;
    }

    // Handle remaining elements
    while i < len {
        sum += values.get_unchecked(i) as u64;
        i += 1;
    }

    sum
}

/// Optimized average calculation
pub fn average_u32_optimized(values: &Vec<u32>) -> u32 {
    if values.is_empty() {
        return 0;
    }
    let sum = sum_vec_u32_optimized(values);
    (sum / values.len() as u64) as u32
}

/// Fast min/max finding with single pass
pub fn min_max_u32(values: &Vec<u32>) -> (u32, u32) {
    if values.is_empty() {
        return (0, 0);
    }

    let mut min = values.get_unchecked(0);
    let mut max = min;

    for i in 1..values.len() {
        let val = values.get_unchecked(i);
        if val < min {
            min = val;
        }
        if val > max {
            max = val;
        }
    }

    (min, max)
}

/// Optimized count of non-zero elements
pub fn count_nonzero_u32(values: &Vec<u32>) -> u32 {
    let len = values.len();
    let mut count = 0u32;
    let mut i = 0;

    // Unrolled loop
    while i + 4 <= len {
        if values.get_unchecked(i) != 0 {
            count += 1;
        }
        if values.get_unchecked(i + 1) != 0 {
            count += 1;
        }
        if values.get_unchecked(i + 2) != 0 {
            count += 1;
        }
        if values.get_unchecked(i + 3) != 0 {
            count += 1;
        }
        i += 4;
    }

    while i < len {
        if values.get_unchecked(i) != 0 {
            count += 1;
        }
        i += 1;
    }

    count
}

/// Efficient filtering without allocations
pub fn filter_nonzero_u32(env: &Env, values: &Vec<u32>) -> Vec<u32> {
    let count = count_nonzero_u32(values);
    let mut result = Vec::new(env);

    if count == 0 {
        return result;
    }

    for i in 0..values.len() {
        let val = values.get_unchecked(i);
        if val != 0 {
            result.push_back(val);
        }
    }

    result
}

/// Optimized binary search (assumes sorted input)
pub fn binary_search_u32(values: &Vec<u32>, target: u32) -> Option<u32> {
    let mut left = 0;
    let mut right = values.len();

    while left < right {
        let mid = left + (right - left) / 2;
        let mid_val = values.get_unchecked(mid);

        if mid_val == target {
            return Some(mid);
        } else if mid_val < target {
            left = mid + 1;
        } else {
            right = mid;
        }
    }

    None
}

/// Fast power of 2 check
#[inline]
pub fn is_power_of_two(n: u32) -> bool {
    n != 0 && (n & (n - 1)) == 0
}

/// Fast log2 for power of 2 numbers
#[inline]
pub fn log2_pow2(n: u32) -> u32 {
    n.trailing_zeros()
}

/// Optimized clamp function
#[inline]
pub fn clamp_u32(value: u32, min: u32, max: u32) -> u32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// Efficient percentage calculation avoiding overflow
pub fn percentage_u32(value: u32, percentage: u32) -> u32 {
    ((value as u64 * percentage as u64) / 100) as u32
}

/// Optimized weighted average
pub fn weighted_average_u32(values: &Vec<u32>, weights: &Vec<u32>) -> u32 {
    if values.is_empty() || values.len() != weights.len() {
        return 0;
    }

    let mut weighted_sum: u64 = 0;
    let mut weight_sum: u64 = 0;

    for i in 0..values.len() {
        let val = values.get_unchecked(i) as u64;
        let weight = weights.get_unchecked(i) as u64;
        weighted_sum += val * weight;
        weight_sum += weight;
    }

    if weight_sum == 0 {
        return 0;
    }

    (weighted_sum / weight_sum) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::Env;

    // // #[test]
    fn test_sum_optimized() {
        let env = Env::default();
        let values = soroban_sdk::vec![&env, 1u32, 2, 3, 4, 5];
        assert_eq!(sum_vec_u32_optimized(&values), 15);
    }

    // // #[test]
    fn test_min_max() {
        let env = Env::default();
        let values = soroban_sdk::vec![&env, 5u32, 2, 8, 1, 9];
        assert_eq!(min_max_u32(&values), (1, 9));
    }

    // // #[test]
    fn test_count_nonzero() {
        let env = Env::default();
        let values = soroban_sdk::vec![&env, 1u32, 0, 3, 0, 5];
        assert_eq!(count_nonzero_u32(&values), 3);
    }

    // // #[test]
    fn test_is_power_of_two() {
        assert!(is_power_of_two(1));
        assert!(is_power_of_two(2));
        assert!(is_power_of_two(4));
        assert!(is_power_of_two(8));
        assert!(!is_power_of_two(3));
        assert!(!is_power_of_two(6));
    }

    // // #[test]
    fn test_fold_seed_bytes_matches_bytewise_fold() {
        let mut bytes = [0u8; 32];
        for (i, b) in bytes.iter_mut().enumerate() {
            *b = (i as u8).wrapping_mul(37).wrapping_add(11);
        }
        // Reference: byte-at-a-time little-endian fold (legacy algorithm).
        let mut expected = 0u64;
        for lane in 0..4 {
            let mut v = 0u64;
            for i in 0..8 {
                v |= (bytes[lane * 8 + i] as u64) << (i * 8);
            }
            expected ^= v;
        }
        assert_eq!(fold_seed_bytes(&bytes), expected);
    }

    // // #[test]
    fn test_fold_seed_single_host_call_matches_array() {
        let env = Env::default();
        let raw = [7u8; 32];
        let seed = BytesN::from_array(&env, &raw);
        assert_eq!(fold_seed(&seed), fold_seed_bytes(&raw));
    }

    // // #[test]
    fn test_is_zero_bytes32() {
        assert!(is_zero_bytes32(&[0u8; 32]));
        let mut b = [0u8; 32];
        b[31] = 1;
        assert!(!is_zero_bytes32(&b));
    }

    // // #[test]
    fn test_expand_u64_to_bytes32_is_deterministic() {
        assert_eq!(expand_u64_to_bytes32(42), expand_u64_to_bytes32(42));
        assert_ne!(expand_u64_to_bytes32(42), expand_u64_to_bytes32(43));
    }

    // // #[test]
    fn test_percentage() {
        assert_eq!(percentage_u32(100, 50), 50);
        assert_eq!(percentage_u32(200, 25), 50);
        assert_eq!(percentage_u32(1000, 10), 100);
    }
}
