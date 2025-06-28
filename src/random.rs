use crate::current_time::compute_time_in_seconds;
use std::convert::TryFrom;

pub struct RandomNumberGenerator {
    value: u64,
}

const MODULUS: u64 = 0xFFFFFFFFFFFF; // 2^48
const MULTIPLIER: u64 = 0x5DEECE66D;
const INCREMENT: u64 = 11;

fn compute_relevant_bits(value: u64) -> u32 {
    // Convert to u32 using bits 47 through 17 of the 64 bit value
    let bits = (value >> 16) & 0b1111111111111111111111111111111; // 31 bits
    u32::try_from(bits).expect("This should always fit in 32 bits")
}

impl RandomNumberGenerator {
    pub fn new(seed: u64) -> Self {
        Self { value: seed }
    }

    pub fn from_current_time() -> Self {
        let seed = compute_time_in_seconds();
        Self::new(seed)
    }

    pub fn next(&mut self) -> u32 {
        self.value = (MULTIPLIER.wrapping_mul(self.value).wrapping_add(INCREMENT)) % MODULUS;
        compute_relevant_bits(self.value)
    }

    pub fn next_in_range(&mut self, start: usize, end: usize) -> usize {
        if start >= end {
            panic!("Start must be less than end.");
        }
        let range = end - start;
        let random_value = self.next() as usize % range;
        start + random_value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bit_conversion() {
        let value: u64 = 0b1011111010001001110010110111111111100000000001110110111110001101;
        let bits = compute_relevant_bits(value);
        assert_eq!(bits, 0b1001011011111111110000000000111);
    }
}
