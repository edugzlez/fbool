use crate::fvalue::FValue;

/// Structural handcrafted metrics described in the paper.
pub trait StructuralMetrics {
    /// Counting metric: popcount(Z) + popcount(U), where
    /// Z is the number of zeros and U is the number of ones in the truth table.
    fn counting(&self) -> u32;

    /// Repetitiveness metric for n=5 using fixed partition (4,4,4,4,4,4,4,4).
    ///
    /// The truth table is split into consecutive 4-bit blocks; each block that appears
    /// at least twice contributes its full length (4) for every occurrence.
    fn repetitiveness(&self) -> u32;
}

impl StructuralMetrics for FValue<bool> {
    fn counting(&self) -> u32 {
        let u = self.repr().iter().filter(|&&b| b).count() as u32;
        let z = self.repr().len() as u32 - u;
        z.count_ones() + u.count_ones()
    }

    fn repetitiveness(&self) -> u32 {
        const BLOCK_SIZE: usize = 4;

        let repr = self.repr();
        if !repr.len().is_multiple_of(BLOCK_SIZE) {
            return 0;
        }

        // Encode each 4-bit block as a nibble in [0, 15] and count frequencies.
        let mut blocks: Vec<u8> = Vec::with_capacity(repr.len() / BLOCK_SIZE);
        let mut freq = [0u32; 16];

        for chunk in repr.chunks_exact(BLOCK_SIZE) {
            let mut nibble = 0u8;
            for (i, &bit) in chunk.iter().enumerate() {
                if bit {
                    nibble |= 1u8 << i;
                }
            }
            blocks.push(nibble);
            freq[nibble as usize] += 1;
        }

        blocks
            .iter()
            .map(|&nibble| {
                if freq[nibble as usize] >= 2 {
                    BLOCK_SIZE as u32
                } else {
                    0
                }
            })
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::StructuralMetrics;
    use crate::fvalue::FValue;

    #[test]
    fn counting_constant_zero_for_n5_is_one() {
        let f = FValue::<bool>::constant(5, false);
        assert_eq!(f.counting(), 1);
    }

    #[test]
    fn counting_balanced_for_n5_is_two() {
        let f = FValue::parity(5);
        // U = 16 (10000b -> popcount 1), Z = 16 (10000b -> popcount 1)
        assert_eq!(f.counting(), 2);
    }

    #[test]
    fn repetitiveness_all_equal_blocks_for_n5_is_32() {
        let f = FValue::<bool>::constant(5, false);
        // Eight blocks of size 4, all repeated -> 8 * 4 = 32.
        assert_eq!(f.repetitiveness(), 32);
    }

    #[test]
    fn repetitiveness_unique_blocks_for_n5_is_zero() {
        // Build 8 unique 4-bit blocks: 0..7.
        let mut repr = Vec::with_capacity(32);
        for block in 0u8..8 {
            for i in 0..4 {
                repr.push(((block >> i) & 1) == 1);
            }
        }
        let f = FValue::new(repr);
        assert_eq!(f.repetitiveness(), 0);
    }
}
