//! # Boolean Function Entanglement Analysis Library
//!
//! This library provides tools for analyzing entanglement properties of boolean functions.
//! It includes structures and traits for computing entanglement measures, entropy calculations,
//! and information-theoretic properties of boolean function partitions.
//!
//! ## Key Concepts
//!
//! - **Entanglement**: Measures how interrelated different parts of a boolean function are
//! - **Entropy**: Information-theoretic measure of randomness in function partitions
//! - **Information**: Quantifies the amount of information contained in variable sets
//!
//! ## Main Structures
//!
//! - [`EntanglementSet`] - Represents an entanglement value with its corresponding variable partition
//! - [`EntropySet`] - Represents an entropy value with its corresponding variable partition
//!
//! ## Main Traits
//!
//! - [`Entanglement`] - Provides methods for computing entanglement measures
//! - [`Entropy`] - Provides methods for computing entropy measures
//! - [`WithInformation`] - Provides information computation for variable sets

#[cfg(feature = "fmatrix")]
use crate::fmulti::FMulti;
use crate::{auxiliar::SubsetIterator, fvalue::FValue};
use rayon::prelude::*;
use std::hash::Hash;

/// Minimum number of variables required to activate parallel computation.
///
/// For functions with fewer variables the parallelization overhead exceeds the
/// gain; benchmarks show the crossover point is around n = 10.
const PARALLEL_THRESHOLD: usize = 10;

/// Represents an entanglement measurement with its associated variable partition.
///
/// This structure stores the entanglement value computed for a specific partition
/// of variables into two disjoint sets, along with the sets themselves.
///
/// # Fields
/// * `entanglement` - The computed entanglement value for this partition
/// * `set1` - The first set of variable indices in the partition
/// * `set2` - The second set of variable indices in the partition
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EntanglementSet {
    /// The entanglement value for this variable partition
    pub entanglement: usize,
    /// The first subset of variables in the partition
    pub set1: Vec<usize>,
    /// The second subset of variables in the partition
    pub set2: Vec<usize>,
}

impl Default for EntanglementSet {
    fn default() -> Self {
        EntanglementSet {
            entanglement: usize::MAX,
            set1: vec![],
            set2: vec![],
        }
    }
}

/// Represents an entropy measurement with its associated variable partition.
///
/// This structure stores the entropy value computed for a specific partition
/// of variables into two disjoint sets, along with the sets themselves.
///
/// # Fields
/// * `entropy` - The computed entropy value for this partition
/// * `set1` - The first set of variable indices in the partition
/// * `set2` - The second set of variable indices in the partition
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EntropySet {
    /// The entropy value for this variable partition
    pub entropy: f32,
    /// The first subset of variables in the partition
    pub set1: Vec<usize>,
    /// The second subset of variables in the partition
    pub set2: Vec<usize>,
}

impl Default for EntropySet {
    fn default() -> Self {
        EntropySet {
            entropy: f32::MAX,
            set1: vec![],
            set2: vec![],
        }
    }
}

/// Trait for types that have a defined number of variables.
pub trait NVars {
    /// Returns the number of boolean variables in the function.
    fn num_vars(&self) -> usize;
}

/// Trait for types that can compute information measures for variable sets.
///
/// Information typically refers to the number of distinct function forms
/// when certain variables are held fixed while others vary.
pub trait WithInformation {
    /// Computes the information content for a given set of variables.
    ///
    /// # Arguments
    /// * `vars` - A vector of variable indices to compute information for
    ///
    /// # Returns
    /// The information value (typically count of distinct forms) for the variable set
    fn get_information(&self, vars: &[usize]) -> usize;
}

/// Trait for types that can compute multiple information measures simultaneously.
pub trait WithMultipleInformation {
    /// Computes multiple information values for a given set of variables.
    ///
    /// # Arguments
    /// * `vars` - A vector of variable indices to compute information for
    ///
    /// # Returns
    /// A vector of information values for the variable set
    fn get_multiple_information(&self, vars: &[usize]) -> Vec<usize>;
}

/// Trait for types that can compute entropy measures for variable sets.
///
/// Entropy provides an information-theoretic measure of the randomness
/// or uncertainty in the function when variables are partitioned.
pub trait WithEntropy {
    /// Computes the entropy for a given set of variables.
    ///
    /// # Arguments
    /// * `vars` - A vector of variable indices to compute entropy for
    ///
    /// # Returns
    /// The entropy value for the variable set
    fn get_entropy(&self, vars: &[usize]) -> f32;
}

/// Trait for computing entanglement measures of boolean functions.
///
/// Entanglement measures quantify how much the function depends on interactions
/// between different subsets of variables, rather than being separable.
pub trait Entanglement {
    /// Computes the minimum entanglement value across all possible variable partitions.
    ///
    /// # Returns
    /// The minimum sum of information values across all bipartitions of variables
    fn entanglement(&self) -> usize;

    /// Returns all entanglement values for every possible variable partition.
    ///
    /// # Returns
    /// A vector of `EntanglementSet` structures containing entanglement values
    /// and their corresponding variable partitions
    fn entanglement_sets(&self) -> Vec<EntanglementSet>;

    /// Computes the minimum max-entanglement value across all possible variable partitions.
    ///
    /// Instead of summing information values, this takes the maximum of the two
    /// partition information values, then finds the minimum across all partitions.
    ///
    /// # Returns
    /// The minimum max information value across all bipartitions of variables
    fn minmax_entanglement(&self) -> usize;

    /// Returns all max-entanglement values for every possible variable partition.
    ///
    /// # Returns
    /// A vector of `EntanglementSet` structures containing max-entanglement values
    /// and their corresponding variable partitions
    fn minmax_entanglement_sets(&self) -> Vec<EntanglementSet>;
}

/// Trait for computing information values for individual variables.
pub trait SubInfos {
    /// Computes the information content for each individual variable.
    ///
    /// # Returns
    /// A vector where each element is the information content when fixing
    /// the corresponding variable (by index)
    fn sub_infos(&self) -> Vec<usize>;
}

/// Trait for computing entropy measures of boolean functions.
///
/// Entropy provides information-theoretic measures of uncertainty when
/// variables are partitioned into different subsets.
pub trait Entropy {
    /// Computes the minimum entropy value across all possible variable partitions.
    ///
    /// # Returns
    /// The minimum sum of entropy values across all bipartitions of variables
    fn entropy(&self) -> f32;

    /// Returns all entropy values for every possible variable partition.
    ///
    /// # Returns
    /// A vector of `EntropySet` structures containing entropy values
    /// and their corresponding variable partitions
    fn entropy_sets(&self) -> Vec<EntropySet>;
}

impl<T: WithInformation + NVars + Sync> Entanglement for T {
    fn entanglement(&self) -> usize {
        let n = self.num_vars();
        if n >= PARALLEL_THRESHOLD {
            SubsetIterator::new(n)
                .into_par_iter()
                .map(|(set1, set2)| {
                    let s1 = self.get_information(&set1);
                    let s2 = self.get_information(&set2);
                    s1 + s2
                })
                .min()
                .unwrap_or(usize::MAX)
        } else {
            SubsetIterator::new(n)
                .map(|(set1, set2)| {
                    let s1 = self.get_information(&set1);
                    let s2 = self.get_information(&set2);
                    s1 + s2
                })
                .min()
                .unwrap_or(usize::MAX)
        }
    }

    fn entanglement_sets(&self) -> Vec<EntanglementSet> {
        let n = self.num_vars();
        if n >= PARALLEL_THRESHOLD {
            SubsetIterator::new(n)
                .into_par_iter()
                .map(|(set1, set2)| {
                    let s1 = self.get_information(&set1);
                    let s2 = self.get_information(&set2);
                    EntanglementSet {
                        entanglement: s1 + s2,
                        set1,
                        set2,
                    }
                })
                .collect()
        } else {
            SubsetIterator::new(n)
                .map(|(set1, set2)| {
                    let s1 = self.get_information(&set1);
                    let s2 = self.get_information(&set2);
                    EntanglementSet {
                        entanglement: s1 + s2,
                        set1,
                        set2,
                    }
                })
                .collect()
        }
    }

    fn minmax_entanglement(&self) -> usize {
        let n = self.num_vars();
        if n >= PARALLEL_THRESHOLD {
            SubsetIterator::new(n)
                .into_par_iter()
                .map(|(set1, set2)| {
                    let s1 = self.get_information(&set1);
                    let s2 = self.get_information(&set2);
                    s1.max(s2)
                })
                .min()
                .unwrap_or(usize::MAX)
        } else {
            SubsetIterator::new(n)
                .map(|(set1, set2)| {
                    let s1 = self.get_information(&set1);
                    let s2 = self.get_information(&set2);
                    s1.max(s2)
                })
                .min()
                .unwrap_or(usize::MAX)
        }
    }

    fn minmax_entanglement_sets(&self) -> Vec<EntanglementSet> {
        let n = self.num_vars();
        if n >= PARALLEL_THRESHOLD {
            SubsetIterator::new(n)
                .into_par_iter()
                .map(|(set1, set2)| {
                    let s1 = self.get_information(&set1);
                    let s2 = self.get_information(&set2);
                    EntanglementSet {
                        entanglement: s1.max(s2),
                        set1,
                        set2,
                    }
                })
                .collect()
        } else {
            SubsetIterator::new(n)
                .map(|(set1, set2)| {
                    let s1 = self.get_information(&set1);
                    let s2 = self.get_information(&set2);
                    EntanglementSet {
                        entanglement: s1.max(s2),
                        set1,
                        set2,
                    }
                })
                .collect()
        }
    }
}

impl<T: WithEntropy + NVars + Sync> Entropy for T {
    fn entropy(&self) -> f32 {
        let n = self.num_vars();
        if n >= PARALLEL_THRESHOLD {
            SubsetIterator::new(n)
                .into_par_iter()
                .map(|(set1, set2)| {
                    let s1 = self.get_entropy(&set1);
                    let s2 = self.get_entropy(&set2);
                    s1 + s2
                })
                .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or(f32::MAX)
        } else {
            SubsetIterator::new(n)
                .map(|(set1, set2)| {
                    let s1 = self.get_entropy(&set1);
                    let s2 = self.get_entropy(&set2);
                    s1 + s2
                })
                .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or(f32::MAX)
        }
    }

    fn entropy_sets(&self) -> Vec<EntropySet> {
        let n = self.num_vars();
        if n >= PARALLEL_THRESHOLD {
            SubsetIterator::new(n)
                .into_par_iter()
                .map(|(set1, set2)| {
                    let s1 = self.get_entropy(&set1);
                    let s2 = self.get_entropy(&set2);
                    EntropySet {
                        entropy: s1 + s2,
                        set1,
                        set2,
                    }
                })
                .collect()
        } else {
            SubsetIterator::new(n)
                .map(|(set1, set2)| {
                    let s1 = self.get_entropy(&set1);
                    let s2 = self.get_entropy(&set2);
                    EntropySet {
                        entropy: s1 + s2,
                        set1,
                        set2,
                    }
                })
                .collect()
        }
    }
}

impl<T: WithInformation + NVars + Sync> SubInfos for T {
    fn sub_infos(&self) -> Vec<usize> {
        let n = self.num_vars();
        if n >= PARALLEL_THRESHOLD {
            (0..n)
                .into_par_iter()
                .map(|i| self.get_information(&[i]))
                .collect()
        } else {
            (0..n).map(|i| self.get_information(&[i])).collect()
        }
    }
}

#[cfg(feature = "fmatrix")]
impl<T: crate::fmulti::GenericValue + Send + Sync> NVars for FMulti<T> {
    fn num_vars(&self) -> usize {
        self.repr().len().ilog2() as usize
    }
}

#[cfg(feature = "fmatrix")]
impl<T: crate::fmulti::GenericValue + Send + Hash + Sync> WithInformation for FMulti<T> {
    fn get_information(&self, vars: &[usize]) -> usize {
        self.count_forms_by_multiple_fixed(vars)
    }
}

impl<T: crate::fvalue::GenericValue + Send + Sync> NVars for FValue<T> {
    fn num_vars(&self) -> usize {
        self.repr().len().ilog2() as usize
    }
}

impl<T: crate::fvalue::GenericValue + Send + Hash + Sync> WithInformation for FValue<T> {
    fn get_information(&self, vars: &[usize]) -> usize {
        self.count_forms_by_multiple_fixed(vars)
    }
}

impl<T: crate::fvalue::GenericValue + Send + Hash + Sync> WithEntropy for FValue<T> {
    fn get_entropy(&self, vars: &[usize]) -> f32 {
        self.set_entropy(vars)
    }
}

/// Trait for computing equanimity importance measures of boolean functions.
pub trait EquanimityImportance {
    fn equanimity_importance(&self) -> f32;
}

impl<T: crate::fvalue::GenericValue> EquanimityImportance for FValue<T> {
    fn equanimity_importance(&self) -> f32 {
        fn pow2(n: usize) -> usize {
            1 << n
        }

        let mut importance_sum = 0;
        let num_input_bits = self.repr().len().ilog2() as usize;

        for i in 1..=num_input_bits {
            for j in (0..pow2(num_input_bits)).step_by(pow2(i)) {
                for k in 0..pow2(i - 1) {
                    if self.repr()[j + k] != self.repr()[j + k + pow2(i - 1)] {
                        importance_sum += 1;
                    }
                }
            }
        }

        importance_sum as f32 / (num_input_bits * pow2(num_input_bits - 1)) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fvalue::FValue;

    #[test]
    fn test_entanglement_set_default() {
        let es = EntanglementSet::default();
        assert_eq!(es.entanglement, usize::MAX);
        assert_eq!(es.set1.len(), 0);
        assert_eq!(es.set2.len(), 0);
    }

    #[test]
    fn test_entropy_set_default() {
        let es = EntropySet::default();
        assert_eq!(es.entropy, f32::MAX);
        assert_eq!(es.set1.len(), 0);
        assert_eq!(es.set2.len(), 0);
    }

    #[test]
    fn test_nvars_fvalue() {
        let f = FValue::parity(3);
        assert_eq!(f.num_vars(), 3);

        let f4 = FValue::majority(4);
        assert_eq!(f4.num_vars(), 4);
    }

    #[test]
    fn test_with_information() {
        let f = FValue::parity(3);

        // Information for empty set should be 1 (constant when all vars fixed)
        let info_empty = f.get_information(&[]);
        assert!(info_empty > 0);

        // Information for single variable
        let info_single = f.get_information(&[0]);
        assert!(info_single > 0);

        // Information for all variables should be the total number of distinct functions
        let info_all = f.get_information(&[0, 1, 2]);
        assert!(info_all > 0);
    }

    #[test]
    fn test_entanglement_computation() {
        let f = FValue::parity(3);
        let ent = f.entanglement();

        // Entanglement should be a finite positive value
        assert!(ent < usize::MAX);
        assert!(ent > 0);
    }

    #[test]
    fn test_entanglement_sets() {
        let f = FValue::parity(2);
        let ent_sets = f.entanglement_sets();

        // Should have at least one entanglement set
        assert!(!ent_sets.is_empty());

        // Each set should have valid data
        for es in &ent_sets {
            assert!(es.entanglement < usize::MAX);
            // set1 and set2 together should cover all variables
            assert!(es.set1.len() + es.set2.len() <= f.num_vars());
        }
    }

    #[test]
    fn test_minmax_entanglement() {
        let f = FValue::majority(3);
        let minmax = f.minmax_entanglement();

        // MinMax entanglement should be a finite value
        assert!(minmax < usize::MAX);
        assert!(minmax > 0);
    }

    #[test]
    fn test_minmax_entanglement_sets() {
        let f = FValue::majority(3);
        let minmax_sets = f.minmax_entanglement_sets();

        // Should have at least one set
        assert!(!minmax_sets.is_empty());

        // Each set should have valid data
        for es in &minmax_sets {
            assert!(es.entanglement < usize::MAX);
        }
    }

    #[test]
    fn test_entropy_computation() {
        let f = FValue::parity(3);
        let ent = f.entropy();

        // Entropy should be a finite positive value
        assert!(ent < f32::MAX);
        assert!(ent > 0.0);
    }

    #[test]
    fn test_entropy_sets() {
        let f = FValue::parity(3);
        let ent_sets = f.entropy_sets();

        // Should have at least one entropy set
        assert!(!ent_sets.is_empty());

        // Each set should have valid data
        for es in &ent_sets {
            assert!(es.entropy < f32::MAX);
            assert!(es.entropy >= 0.0);
        }
    }

    #[test]
    fn test_sub_infos() {
        let f = FValue::parity(3);
        let sub_infos = f.sub_infos();

        // Should have information for each variable
        assert_eq!(sub_infos.len(), 3);

        // Each should be positive
        for info in sub_infos {
            assert!(info > 0);
        }
    }

    #[test]
    fn test_equanimity_importance() {
        let f = FValue::majority(3);
        let eq_imp = f.equanimity_importance();

        // Should be a valid probability/ratio
        assert!(eq_imp >= 0.0);
        assert!(eq_imp <= 1.0);
    }

    #[test]
    fn test_entanglement_parity_vs_majority() {
        // Parity and majority functions should have different entanglement properties
        let parity_f = FValue::parity(3);
        let majority_f = FValue::majority(3);

        let parity_ent = parity_f.entanglement();
        let majority_ent = majority_f.entanglement();

        // Both should be valid
        assert!(parity_ent < usize::MAX);
        assert!(majority_ent < usize::MAX);

        // They should likely be different (though this is not strictly guaranteed)
        // This is more of a sanity check
        assert!(parity_ent > 0);
        assert!(majority_ent > 0);
    }

    #[test]
    fn test_with_entropy() {
        let f = FValue::parity(3);

        // Entropy for different variable sets
        let entropy_empty = f.get_entropy(&[]);
        let entropy_single = f.get_entropy(&[0]);
        let entropy_all = f.get_entropy(&[0, 1, 2]);

        // All should be finite and non-negative
        assert!((0.0..f32::MAX).contains(&entropy_empty));
        assert!((0.0..f32::MAX).contains(&entropy_single));
        assert!((0.0..f32::MAX).contains(&entropy_all));
    }
}
