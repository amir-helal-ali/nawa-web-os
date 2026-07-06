//! Bloom filter — probabilistic set membership test.
//!
//! Used to skip SSTable reads when the key is definitely not present.
//! False positive rate is configurable; false negatives are impossible.

use xxhash_rust::xxh3::Xxh3;

/// A bloom filter for fast key-existence checks.
pub struct BloomFilter {
    bits: Vec<u64>, // each u64 holds 64 bits
    num_hashes: usize,
    num_bits: usize,
}

impl BloomFilter {
    /// Create a new bloom filter sized for `expected_items` with target FP rate.
    pub fn new(expected_items: usize, false_positive_rate: f64) -> Self {
        let num_bits = optimal_num_bits(expected_items, false_positive_rate);
        let num_hashes = optimal_num_hashes(num_bits, expected_items);
        let num_words = (num_bits + 63) / 64;
        Self {
            bits: vec![0u64; num_words],
            num_hashes,
            num_bits,
        }
    }

    /// Insert a key.
    pub fn insert(&mut self, key: &[u8]) {
        let (h1, h2) = double_hash(key);
        for i in 0..self.num_hashes {
            let bit = (h1.wrapping_add(h2.wrapping_mul(i as u64))) as usize % self.num_bits;
            self.bits[bit / 64] |= 1u64 << (bit % 64);
        }
    }

    /// Check if a key might be present (false positive possible).
    pub fn might_contain(&self, key: &[u8]) -> bool {
        let (h1, h2) = double_hash(key);
        for i in 0..self.num_hashes {
            let bit = (h1.wrapping_add(h2.wrapping_mul(i as u64))) as usize % self.num_bits;
            if (self.bits[bit / 64] & (1u64 << (bit % 64))) == 0 {
                return false;
            }
        }
        true
    }

    /// Number of bits in the filter.
    pub fn num_bits(&self) -> usize {
        self.num_bits
    }

    /// Memory usage in bytes.
    pub fn mem_size(&self) -> usize {
        self.bits.len() * 8
    }
}

fn double_hash(key: &[u8]) -> (u64, u64) {
    let mut h1 = Xxh3::new();
    h1.update(key);
    let h1 = h1.digest();

    let mut h2 = Xxh3::with_seed(0xDEAD_BEEF);
    h2.update(key);
    let h2 = h2.digest();

    (h1, h2)
}

fn optimal_num_bits(items: usize, fp_rate: f64) -> usize {
    if items == 0 {
        return 64;
    }
    let ln2 = std::f64::consts::LN_2;
    let m = (-(items as f64) * fp_rate.ln() / (ln2 * ln2)).ceil() as usize;
    m.max(64)
}

fn optimal_num_hashes(bits: usize, items: usize) -> usize {
    if items == 0 {
        return 1;
    }
    let k = ((bits as f64 / items as f64) * std::f64::consts::LN_2).ceil() as usize;
    k.max(1).min(30)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_check() {
        let mut bf = BloomFilter::new(1000, 0.01);
        for i in 0..1000u32 {
            bf.insert(&i.to_le_bytes());
        }
        // All inserted keys should be present.
        for i in 0..1000u32 {
            assert!(bf.might_contain(&i.to_le_bytes()), "key {i} should be present");
        }
    }

    #[test]
    fn false_positive_rate() {
        let mut bf = BloomFilter::new(1000, 0.01);
        for i in 0..1000u32 {
            bf.insert(&i.to_le_bytes());
        }
        // Check 1000 keys that were NOT inserted.
        let mut false_positives = 0;
        for i in 1000..2000u32 {
            if bf.might_contain(&i.to_le_bytes()) {
                false_positives += 1;
            }
        }
        // Should be well under 5% (target is 1%).
        let rate = false_positives as f64 / 1000.0;
        assert!(rate < 0.05, "false positive rate too high: {rate}");
    }

    #[test]
    fn empty_filter_never_contains() {
        let bf = BloomFilter::new(100, 0.01);
        assert!(!bf.might_contain(b"hello"));
        assert!(!bf.might_contain(b"world"));
    }
}
