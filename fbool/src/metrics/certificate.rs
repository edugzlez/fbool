//! Certificate complexity for boolean functions.
//!
//! This module provides functionality to compute the certificate complexity
//! of boolean functions. The certificate complexity C(f) of a boolean function f
//! is the maximum over all inputs x of the minimum size certificate needed to
//! prove f(x).
//!
//! A certificate for an input x is a subset S of variables such that for any
//! input y that agrees with x on S, we have f(y) = f(x).
//!
//! ## Optimization
//!
//! Instead of checking all 2^(n-k) possible inputs for each subset, we use a
//! conflict-based approach:
//! 1. Pre-compute all "conflicts" - inputs y where f(y) ≠ f(x)
//! 2. Store the XOR (x ^ y) for each conflict, representing differing bit positions
//! 3. A subset S is a valid certificate iff it "hits" every conflict
//!    (i.e., for every conflict diff, diff & mask(S) ≠ 0)
//!
//! This reduces certificate verification from O(2^(n-k)) to O(|conflicts|).
//!
//! Additional optimizations:
//! - Greedy upper bound to limit search space
//! - Parallel processing for large inputs
//! - Early termination when possible

use rayon::prelude::*;

use crate::fvalue::FValue;

/// Trait for computing certificate complexity of boolean functions.
pub trait CertificateComplexity {
    /// Computes the certificate complexity of a single input x.
    ///
    /// This is the minimum number of bits needed to certify the output f(x).
    /// A certificate is a subset of variables that uniquely determines the output.
    ///
    /// # Arguments
    /// * `x` - The input to compute the certificate complexity for
    ///
    /// # Returns
    /// The minimum certificate size for input x
    fn point_certificate_complexity(&self, x: usize) -> u32;

    /// Computes the certificate complexity of the function.
    ///
    /// This is the maximum certificate complexity over all inputs,
    /// representing the worst-case number of bits needed to certify any output.
    ///
    /// # Returns
    /// The certificate complexity C(f)
    fn certificate_complexity(&self) -> u32;

    /// Computes the 1-certificate complexity of the function.
    ///
    /// This is the maximum certificate complexity over all inputs x where f(x) = true.
    ///
    /// # Returns
    /// The 1-certificate complexity C¹(f)
    fn certificate_complexity_1(&self) -> u32;

    /// Computes the 0-certificate complexity of the function.
    ///
    /// This is the maximum certificate complexity over all inputs x where f(x) = false.
    ///
    /// # Returns
    /// The 0-certificate complexity C⁰(f)
    fn certificate_complexity_0(&self) -> u32;

    /// Computes the mean certificate complexity over all inputs.
    ///
    /// # Returns
    /// The average certificate complexity
    fn mean_certificate_complexity(&self) -> f32;
}

/// Computes a greedy upper bound for the minimum hitting set.
///
/// Uses a greedy algorithm that repeatedly selects the variable that hits
/// the most remaining conflicts until all conflicts are hit.
///
/// # Arguments
/// * `conflicts` - The set of conflict masks (XOR differences)
/// * `n` - The number of variables
///
/// # Returns
/// An upper bound on the minimum certificate size
fn greedy_upper_bound(conflicts: &[usize], n: usize) -> usize {
    if conflicts.is_empty() {
        return 0;
    }

    let mut remaining: Vec<usize> = conflicts.to_vec();
    let mut count = 0;

    while !remaining.is_empty() {
        // Find the variable that hits the most remaining conflicts
        let best_var = (0..n)
            .max_by_key(|&var| {
                let mask = 1 << var;
                remaining.iter().filter(|&&c| c & mask != 0).count()
            })
            .unwrap();

        let mask = 1 << best_var;
        count += 1;

        // Remove conflicts that are now hit
        remaining.retain(|&c| c & mask == 0);
    }

    count
}

/// Computes the minimum certificate size for a given set of conflicts.
///
/// Uses BFS over subset sizes with greedy upper bound pruning.
fn minimum_certificate_size(conflicts: &[usize], n: usize) -> u32 {
    if conflicts.is_empty() {
        return 0;
    }

    // Get greedy upper bound to limit search
    let upper_bound = greedy_upper_bound(conflicts, n);

    // If greedy found size 1, that's optimal
    if upper_bound == 1 {
        return 1;
    }

    // Search for exact minimum starting from size 1
    for size in 1..upper_bound {
        for subset_mask in SubsetMaskIterator::new(n, size) {
            // Check if this subset hits all conflicts
            if conflicts.iter().all(|&diff| diff & subset_mask != 0) {
                return size as u32;
            }
        }
    }

    upper_bound as u32
}

impl CertificateComplexity for FValue<bool> {
    fn point_certificate_complexity(&self, x: usize) -> u32 {
        let n = self.n_vars();
        let f_x = *self.get(x).unwrap();

        // Pre-compute all conflicts: XOR differences with inputs having different output
        let conflicts: Vec<usize> = (0..(1 << n))
            .filter(|&y| *self.get(y).unwrap() != f_x)
            .map(|y| x ^ y)
            .collect();

        minimum_certificate_size(&conflicts, n)
    }

    fn certificate_complexity(&self) -> u32 {
        let n = self.n_vars();
        let num_inputs = 1 << n;

        // Use parallel processing for larger functions
        if n >= 4 {
            (0..num_inputs)
                .into_par_iter()
                .map(|x| self.point_certificate_complexity(x))
                .max()
                .unwrap_or(0)
        } else {
            (0..num_inputs)
                .map(|x| self.point_certificate_complexity(x))
                .max()
                .unwrap_or(0)
        }
    }

    fn certificate_complexity_1(&self) -> u32 {
        let n = self.n_vars();
        let num_inputs = 1 << n;

        if n >= 4 {
            (0..num_inputs)
                .into_par_iter()
                .filter(|&x| *self.get(x).unwrap())
                .map(|x| self.point_certificate_complexity(x))
                .max()
                .unwrap_or(0)
        } else {
            (0..num_inputs)
                .filter(|&x| *self.get(x).unwrap())
                .map(|x| self.point_certificate_complexity(x))
                .max()
                .unwrap_or(0)
        }
    }

    fn certificate_complexity_0(&self) -> u32 {
        let n = self.n_vars();
        let num_inputs = 1 << n;

        if n >= 4 {
            (0..num_inputs)
                .into_par_iter()
                .filter(|&x| !*self.get(x).unwrap())
                .map(|x| self.point_certificate_complexity(x))
                .max()
                .unwrap_or(0)
        } else {
            (0..num_inputs)
                .filter(|&x| !*self.get(x).unwrap())
                .map(|x| self.point_certificate_complexity(x))
                .max()
                .unwrap_or(0)
        }
    }

    fn mean_certificate_complexity(&self) -> f32 {
        let n = self.n_vars();
        let num_inputs = 1 << n;

        if n >= 4 {
            (0..num_inputs)
                .into_par_iter()
                .map(|x| self.point_certificate_complexity(x))
                .sum::<u32>() as f32
                / num_inputs as f32
        } else {
            (0..num_inputs)
                .map(|x| self.point_certificate_complexity(x))
                .sum::<u32>() as f32
                / num_inputs as f32
        }
    }
}

/// Iterator that generates all subsets of {0, ..., n-1} of size k as bitmasks.
///
/// Uses Gosper's hack for efficient generation of next subset with same popcount.
struct SubsetMaskIterator {
    current: usize,
    max: usize,
    done: bool,
}

impl SubsetMaskIterator {
    fn new(n: usize, k: usize) -> Self {
        if k == 0 {
            return Self {
                current: 0,
                max: (1 << n) - 1,
                done: false,
            };
        }
        if k > n {
            return Self {
                current: 0,
                max: 0,
                done: true,
            };
        }

        // First subset: k lowest bits set
        let current = (1 << k) - 1;
        let max = (1 << n) - 1;

        Self {
            current,
            max,
            done: false,
        }
    }

    /// Gosper's hack: compute next subset with same number of bits
    #[inline]
    fn next_subset(v: usize) -> usize {
        let c = v & v.wrapping_neg();
        let r = v + c;
        (((r ^ v) >> 2) / c) | r
    }
}

impl Iterator for SubsetMaskIterator {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let result = self.current;

        // Special case: empty set
        if self.current == 0 {
            self.done = true;
            return Some(result);
        }

        // Use Gosper's hack to get next subset
        let next = Self::next_subset(self.current);

        if next > self.max {
            self.done = true;
        } else {
            self.current = next;
        }

        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_function() {
        // Constant true function: certificate complexity is 0
        let f_true = FValue::<bool>::constant(3, true);
        assert_eq!(f_true.certificate_complexity(), 0);

        // Constant false function: certificate complexity is 0
        let f_false = FValue::<bool>::constant(3, false);
        assert_eq!(f_false.certificate_complexity(), 0);
    }

    #[test]
    fn test_single_variable() {
        // f(x) = x_0 (identity on first variable)
        // Certificate complexity is 1
        let f = FValue::new(vec![false, true]);
        assert_eq!(f.certificate_complexity(), 1);
    }

    #[test]
    fn test_and_function() {
        // f(x0, x1) = x0 AND x1
        // Truth table: 00->0, 01->0, 10->0, 11->1
        let f = FValue::new(vec![false, false, false, true]);

        // For input 11 (index 3), we need both bits (size 2)
        // For inputs 00, 01, 10, we need 1 bit each (any 0 bit suffices)
        assert_eq!(f.point_certificate_complexity(3), 2); // 11 needs both bits
        assert_eq!(f.point_certificate_complexity(0), 1); // 00 needs one 0
        assert_eq!(f.certificate_complexity(), 2);
    }

    #[test]
    fn test_or_function() {
        // f(x0, x1) = x0 OR x1
        // Truth table: 00->0, 01->1, 10->1, 11->1
        let f = FValue::new(vec![false, true, true, true]);

        // For input 00 (index 0), we need both bits to prove output is 0
        // For inputs with a 1, we only need that 1 bit
        assert_eq!(f.point_certificate_complexity(0), 2); // 00 needs both bits
        assert_eq!(f.point_certificate_complexity(1), 1); // 01 needs bit 0
        assert_eq!(f.certificate_complexity(), 2);
    }

    #[test]
    fn test_parity_function() {
        // Parity on 3 variables: need all bits
        let f = FValue::parity(3);
        assert_eq!(f.certificate_complexity(), 3);

        // Parity on 2 variables
        let f2 = FValue::parity(2);
        assert_eq!(f2.certificate_complexity(), 2);
    }

    #[test]
    fn test_majority_function() {
        // Majority on 3 variables
        let f = FValue::majority(3);

        // For majority, to certify 1, we need at least 2 ones
        // To certify 0, we need at least 2 zeros
        // Certificate complexity should be 2
        assert_eq!(f.certificate_complexity(), 2);
    }

    #[test]
    fn test_certificate_complexity_bounds() {
        // Certificate complexity should be between 0 and n
        let f = FValue::random(4);
        let cc = f.certificate_complexity();
        assert!(cc <= 4);
    }

    #[test]
    fn test_certificate_0_and_1() {
        // OR function
        let f = FValue::new(vec![false, true, true, true]);

        // C^0 is max over inputs where f(x)=0, only input 00
        assert_eq!(f.certificate_complexity_0(), 2);

        // C^1 is max over inputs where f(x)=1
        assert_eq!(f.certificate_complexity_1(), 1);
    }

    #[test]
    fn test_subset_mask_iterator() {
        // Test that we generate correct number of subsets
        let subsets: Vec<_> = SubsetMaskIterator::new(4, 2).collect();
        assert_eq!(subsets.len(), 6); // C(4,2) = 6

        let subsets: Vec<_> = SubsetMaskIterator::new(5, 3).collect();
        assert_eq!(subsets.len(), 10); // C(5,3) = 10

        let subsets: Vec<_> = SubsetMaskIterator::new(3, 0).collect();
        assert_eq!(subsets.len(), 1); // Empty set
        assert_eq!(subsets[0], 0);
    }

    #[test]
    fn test_subset_mask_values() {
        // Verify that all generated masks have the correct popcount
        for n in 1..=6 {
            for k in 0..=n {
                for mask in SubsetMaskIterator::new(n, k) {
                    assert_eq!(
                        mask.count_ones() as usize,
                        k,
                        "Mask {} should have {} bits set",
                        mask,
                        k
                    );
                    assert!(mask < (1 << n), "Mask {} should be less than 2^{}", mask, n);
                }
            }
        }
    }

    #[test]
    fn test_larger_function() {
        // Test with 5 variables to ensure performance is acceptable
        let f = FValue::majority(5);
        let cc = f.certificate_complexity();
        // Majority on 5 vars needs 3 bits to certify
        assert_eq!(cc, 3);
    }

    #[test]
    fn test_greedy_upper_bound() {
        // Test the greedy algorithm
        let conflicts = vec![0b001, 0b010, 0b100];
        // Each conflict has only one bit set, need 3 variables
        assert_eq!(greedy_upper_bound(&conflicts, 3), 3);

        // All conflicts share a common bit
        let conflicts2 = vec![0b101, 0b111, 0b011];
        // Bit 0 (value 1) hits all conflicts
        assert_eq!(greedy_upper_bound(&conflicts2, 3), 1);
    }

    #[test]
    fn test_performance_6_vars() {
        // Test with 6 variables - should complete quickly with optimizations
        let f = FValue::majority(6);
        let cc = f.certificate_complexity();
        assert_eq!(cc, 4); // Majority on 6 vars needs 4 bits
    }
}
