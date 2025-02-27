use std::hash::Hash;
use std::collections::HashMap;
use crate::coding::Coding;

// Returns `conf` if it is greater than `0`, or `max(1, available parallelism + conf)` otherwise.
/*pub fn threads_count(conf: isize) -> NonZeroUsize {
    if conf > 0 {
        unsafe { NonZeroUsize::new_unchecked(conf as usize) }
    } else {
        unsafe { available_parallelism().map_or(NonZeroUsize::new_unchecked(1), |v| {
            NonZeroUsize::new_unchecked(v.get().saturating_sub((-conf) as usize).max(1))
        }) }
    }
}*/

// Calls `f` in `threads_count` threads and returns vector of `threads_count` results returned by the callings.
/*pub fn threads_run<F, T>(threads_count: NonZeroUsize, mut f: F) -> std::thread::Result<Vec<T>>
where F: FnMut() -> T + Send + Clone, T: Send {
    thread::scope(|scope| {
        let mut extra_results = Vec::<ScopedJoinHandle::<T>>::with_capacity(threads_count.get()-1);
        for _ in 0..threads_count.get() - 1 {
            let mut f_clone = f.clone();
            extra_results.push(scope.spawn(move |_| { f_clone() }));
        }
        let mut results = Vec::with_capacity(threads_count.get());
        results.push(f());
        for e in extra_results {
            results.push(e.join().unwrap());
        }
        results
    })
}*/

/// Encodes all `values` using `value_coding`.
/// Returns pair that consists of: values fragments and their sizes.
pub fn encode_all<C: Coding>(value_coding: &C, values: &[C::Value]) -> Vec::<C::Codeword>
    //where V: Hash + Eq + Clone
{
    let encoder = value_coding.encoder();
    values.iter().map(|v| value_coding.code_of(&encoder, v)).collect()
}

pub fn encode_all_from_map<C: Coding, K, H>(value_coding: &C, map: &HashMap<K, C::Value, H>) -> (Vec<K>, Vec::<C::Codeword>)
    where K: Hash + Clone//, C::Value: Hash + Eq + Clone
{
    let mut keys = Vec::<K>::with_capacity(map.len());
    let mut values = Vec::<C::Codeword>::with_capacity(map.len());
    let encoder = value_coding.encoder();
    for (k, v) in map {
        keys.push(k.clone());
        values.push(value_coding.code_of(&encoder, v));
    }
    (keys, values)
}

/// Calculates the number of bits needed to store values from `0` up to given one (works only for non-negative integers).
///
/// # Example
///
/// ```
/// use csf::bits_to_store;
///
/// assert_eq!(bits_to_store!(0u32), 0);
/// assert_eq!(bits_to_store!(1u32), 1);
/// assert_eq!(bits_to_store!(7u32), 3);
/// assert_eq!(bits_to_store!(8u32), 4);
/// ```
#[macro_export]
macro_rules! bits_to_store {
    ($value:expr) => {{
        let v = $value;
        (if v.is_power_of_two() {
            v.trailing_zeros()+1
        } else {
            v.checked_next_power_of_two().unwrap_or(0).trailing_zeros()
        }) as u8
    }};
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_bits_to_store() {
        assert_eq!(bits_to_store!(0u32), 0);
        assert_eq!(bits_to_store!(1u32), 1);
        assert_eq!(bits_to_store!(2u32), 2);
        assert_eq!(bits_to_store!(3u32), 2);
        assert_eq!(bits_to_store!(4u32), 3);
        assert_eq!(bits_to_store!(7u32), 3);
        assert_eq!(bits_to_store!(8u32), 4);
        assert_eq!(bits_to_store!(u32::MAX-1), 32);
        assert_eq!(bits_to_store!(u32::MAX), 32);
    }
}

