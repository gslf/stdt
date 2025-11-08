//! utils/random.rs
//!
//! Minimal, **non-cryptographic** pseudo-random utilities.
//! Generates integers, decimals, and random choices.
//!
//! # Examples
//! ```
//! use stdt::utils::random::{integer_in, decimal_in, choose, choose_iter};
//!
//! let i = integer_in(-3, 3);
//! assert!((-3..=3).contains(&i));
//!
//! let f = decimal_in(0.0, 1.0);
//! assert!((0.0..=1.0).contains(&f));
//!
//! let v = [1, 2, 3];
//! assert!(choose(&v).is_some());
//! assert!(choose_iter(v).is_some());
//! ```


use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

const SIGN_MASK: u128 = 1u128 << 127;

fn now_ns() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH).unwrap()
        .as_nanos()
}

fn thread_id() -> u64 {
    let tid = std::thread::current().id();
    let mut hasher = DefaultHasher::new();
    tid.hash(&mut hasher);
    hasher.finish()
}

fn mixer(h: &mut u64, x: u64) {
    *h ^= x.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    *h = (*h).rotate_left(27).wrapping_mul(0x94D0_49BB_1331_11EB);
}

// xorshift64*
fn prng(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x >> 12;
    x ^= x << 25;
    x ^= x >> 27;
    *state = x;
    x.wrapping_mul(0x2545_F491_4F6C_DD1D)
}

fn generator_u128(thread_id: u64, ts_ns: u128) -> u128 {
    let mut h: u64 = 0x9E37_79B9_7F4A_7C15;
    mixer(&mut h, thread_id);
    mixer(&mut h, (ts_ns >> 64) as u64);
    mixer(&mut h, ts_ns as u64);
    let mut state = if h == 0 { 1 } else { h };

    // Extend to 16 byte
    let mut bytes = [0u8; 16];
    bytes[..8].copy_from_slice(&prng(&mut state).to_be_bytes());
    bytes[8..].copy_from_slice(&prng(&mut state).to_be_bytes());

    u128::from_be_bytes(bytes)
}

/// Returns a random `i128` uniformly in the **inclusive** range `[min, max]`.
///
/// Panics if `min > max`. Not cryptographically secure.
///
/// # Examples
/// ```
/// use stdt::utils::random::integer_in;
/// let x = integer_in(-2, 2);
/// assert!((-2..=2).contains(&x));
/// ```
pub fn integer_in(min: i128, max: i128) -> i128{
    assert!(min <= max, "min must be <= max");

    if min == max {
        return min;
    }
    
    let seed = generator_u128(thread_id(), now_ns());
    if min == i128::MIN && max == i128::MAX {
        return seed as i128;
    }

    let start = (min as u128) ^ SIGN_MASK;
    let end = (max as u128) ^ SIGN_MASK;
    let width = (end - start) + 1;

    
    let r = start + (seed % width);
    (r ^ SIGN_MASK) as i128
}

/// Returns a random `f64` uniformly in the **inclusive** range `[start, end]` (within FP error).
///
/// Panics if `start > end`. Not cryptographically secure.
///
/// # Examples
/// ```
/// use stdt::utils::random::decimal_in;
/// let x = decimal_in(0.0, 1.0);
/// assert!(x >= 0.0 - f64::EPSILON && x <= 1.0 + f64::EPSILON);
/// ```
pub fn decimal_in(start: f64, end: f64) -> f64{
    assert!(start <= end, "start must be <= end");
    if start == end {
        return start;
    }

    let seed = generator_u128(thread_id(), now_ns());

    let mant: u64 = (seed >> (128 - 53)) as u64;
    let unit: f64 = (mant as f64) * (1.0 / ((1u64 << 53) as f64)); 

    start + (end - start) * unit
}

/// Returns a random reference to an element of `slice`, or `None` if empty.
///
/// # Examples
/// ```
/// use stdt::utils::random::choose;
/// let xs = [10, 20, 30];
/// let pick = choose(&xs);
/// assert!(pick.is_some() && xs.contains(pick.unwrap()));
/// ```
pub fn choose<T>(slice: &[T]) -> Option<&T>{
    if slice.is_empty() {
        return None;
    }
    let idx = integer_in(0, (slice.len() - 1) as i128) as usize;
    slice.get(idx)
}

/// Returns a random item from any iterable by collecting it into a `Vec`.
///
/// Returns `None` if the iterator yields no items. Prefer [`choose`] for slices.
///
/// # Examples
/// ```
/// use stdt::utils::random::choose_iter;
/// let v = vec!["a", "b", "c"];
/// let pick = choose_iter(v.clone());
/// assert!(pick.is_some() && v.contains(&pick.unwrap()));
/// ```
pub fn choose_iter<I>(iter: I) -> Option<I::Item>
where I: IntoIterator,
{
    let v: Vec<I::Item> = iter.into_iter().collect();
    if v.is_empty() {
        return None;
    }
    let idx = integer_in(0, (v.len() - 1) as i128) as usize;
    v.into_iter().nth(idx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer_in_respects_range_small_negative_to_positive() {
        for _ in 0..1_000 {
            let x = integer_in(-10, 10);
            assert!((-10..=10).contains(&x));
        }
    }

    #[test]
    fn integer_in_respects_range_negative_only() {
        for _ in 0..1_000 {
            let x = integer_in(-50, -1);
            assert!((-50..=-1).contains(&x));
        }
    }

    #[test]
    fn integer_in_respects_range_positive_only() {
        for _ in 0..1_000 {
            let x = integer_in(0, 50);
            assert!((0..=50).contains(&x));
        }
    }

    #[test]
    fn integer_in_returns_exact_when_bounds_equal() {
        assert_eq!(integer_in(42, 42), 42);
        assert_eq!(integer_in(-7, -7), -7);
    }

    #[test]
    fn decimal_in_respects_range() {
        for _ in 0..1_000 {
            let x = decimal_in(-0.1, 0.1);
            assert!(x >= -0.1 - f64::EPSILON && x <= 0.1 + f64::EPSILON);
        }
    }

    #[test]
    fn decimal_in_returns_exact_when_bounds_equal() {
        let x = decimal_in(3.14, 3.14);
        assert_eq!(x, 3.14);
    }

    #[test]
    fn choose_empty_returns_none() {
        let s: [i32; 0] = [];
        assert!(choose(&s).is_none());
    }

    #[test]
    fn choose_returns_one_of_elements() {
        let s = [10, 20, 30, 40];
        for _ in 0..100 {
            let v = choose(&s).unwrap();
            assert!(s.contains(v));
        }
    }

    #[test]
    fn choose_iter_empty_returns_none() {
        let v: Vec<i32> = vec![];
        assert!(choose_iter(v).is_none());
    }

    #[test]
    fn choose_iter_returns_from_iterable() {
        let v = vec!["a", "b", "c"];
        for _ in 0..100 {
            let picked = choose_iter(v.clone()).unwrap();
            assert!(v.contains(&picked));
        }
    }
}

