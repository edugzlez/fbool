//! Boolean function value representation and operations.
//!
//! This module provides the `FValue` struct which represents a boolean function
//! as a vector of values, along with various operations for manipulating and
//! analyzing boolean functions.

use bincode::{Decode, Encode};
use itertools::Itertools;
use std::hash::Hash;

use crate::auxiliar::deposit;
use crate::auxiliar::CountUnique;

/// A trait for generic values that can be used in boolean function representations.
///
/// This trait requires types to be comparable, have a default value, be cloneable,
/// and have a total ordering.
pub trait GenericValue: Eq + Default + Clone + Ord {}

/// Blanket implementation of GenericValue for any type that satisfies the trait bounds.
impl<Output: Eq + Default + Clone + Ord> GenericValue for Output {}

/// Represents a boolean function as a vector of values.
///
/// The `FValue` struct stores a boolean function using a truth table representation
/// where the length must be a power of 2 (corresponding to 2^n for n variables).
///
/// # Type Parameters
/// * `Output` - The type of values stored in the function representation, must implement `GenericValue`
///
/// # Examples
/// ```
/// use fbool::fvalue::FValue;
///
/// // Create a boolean function with 4 entries (2 variables)
/// let func = FValue::new(vec![true, false, true, false]);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FValue<Output: GenericValue> {
    /// Internal representation as a vector of values
    repr: Vec<Output>,
}

impl<Output: GenericValue + Encode> Encode for FValue<Output> {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        self.repr.encode(encoder)
    }
}

impl<Output: GenericValue + Decode<()>> Decode<()> for FValue<Output> {
    fn decode<D: bincode::de::Decoder<Context = ()>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        Vec::decode(decoder).map(FValue::new)
    }
}

/// Checks if a given size is valid for a boolean function representation.
///
/// A valid size must be positive and a power of 2.
///
/// # Arguments
/// * `n` - The size to check
///
/// # Returns
/// * `true` if the size is valid (positive and power of 2), `false` otherwise
fn is_valid_size(n: usize) -> bool {
    n > 0 && n.is_power_of_two()
}

impl<Output: GenericValue> FValue<Output> {
    /// Creates a new `FValue` with the given representation vector.
    ///
    /// # Arguments
    /// * `repr` - A vector representing the boolean function values. Length must be a power of 2.
    ///
    /// # Panics
    /// Panics if the length of `repr` is not a positive power of 2.
    ///
    /// # Examples
    /// ```
    /// use fbool::fvalue::FValue;
    ///
    /// let func = FValue::new(vec![true, false, true, false]);
    /// ```
    pub fn new(repr: Vec<Output>) -> Self {
        if !is_valid_size(repr.len()) {
            panic!("Invalid size");
        }

        FValue { repr }
    }

    /// Returns a reference to the internal representation vector.
    ///
    /// # Returns
    /// A reference to the vector containing the function values.
    pub fn repr(&self) -> &Vec<Output> {
        &self.repr
    }
}

impl<Output: Send + Sync + Hash + GenericValue> FValue<Output> {
    /// Returns the number of variables in this boolean function.
    ///
    /// # Returns
    /// The number of boolean variables (log2 of the representation length).
    pub fn n_vars(&self) -> usize {
        self.repr.len().ilog2() as usize
    }

    /// Gets the value at the specified index in the truth table.
    ///
    /// # Arguments
    /// * `i` - The index in the truth table
    ///
    /// # Returns
    /// `Some(&Output)` if the index is valid, `None` otherwise.
    pub fn get(&self, i: usize) -> Option<&Output> {
        self.repr.get(i)
    }

    /// Creates a new function by fixing a specific variable to a given value.
    ///
    /// This operation reduces the number of variables by one by setting the specified
    /// variable to either true or false and creating a new function with the resulting values.
    ///
    /// # Arguments
    /// * `var` - The index of the variable to fix (0-based)
    /// * `value` - The boolean value to assign to the variable
    ///
    /// # Returns
    /// A new `FValue` with one fewer variable where `var` is fixed to `value`.
    pub fn fixed(&self, var: usize, value: bool) -> Self {
        let it = crate::auxiliar::binary_numbers(self.n_vars(), var, value as usize);

        let mut new_repr = vec![];

        it.for_each(|x| {
            let bit = self.repr.get(x).cloned().unwrap_or_default();
            new_repr.push(bit.clone());
        });

        FValue::new(new_repr)
    }

    pub fn multiple_fixed(&self, mut vars: Vec<(usize, bool)>) -> Self {
        let mut new_repr = self.clone();
        vars.sort_by(|a, b| a.0.cmp(&b.0));

        for (i, (var, value)) in vars.iter().enumerate() {
            new_repr = new_repr.fixed(*var - i, *value);
        }

        new_repr
    }

    pub fn list_forms_by_fixed(&self, var: usize) -> Vec<Self> {
        vec![self.fixed(var, false), self.fixed(var, true)]
    }

    /// Counts the number of unique function forms when multiple variables are fixed.
    ///
    /// This method calculates how many distinct boolean functions result when
    /// the specified variables are allowed to take all possible combinations of values,
    /// while the remaining variables are treated as the function's arguments.
    ///
    /// # Arguments
    /// * `vars` - A vector of variable indices to be fixed
    ///
    /// # Returns
    /// The number of unique function forms when the specified variables are fixed
    /// across all possible value assignments.
    pub fn count_forms_by_multiple_fixed(&self, vars: &[usize]) -> usize {
        let mut fixed_positions = vars.to_vec();
        fixed_positions.sort();

        let free_positions = (0..self.repr().len().ilog2() as usize)
            .filter(|i| !fixed_positions.contains(i))
            .collect::<Vec<_>>();

        let (fixed, free) = (fixed_positions.len(), free_positions.len());

        let free_deposits: Vec<_> = (0..(1 << free))
            .map(|w| deposit(w, &free_positions))
            .collect();

        (0..(1 << fixed))
            .map(|v| {
                let fixed_part = deposit(v, &fixed_positions);
                let repr = free_deposits
                    .iter()
                    .map(|&free_part| {
                        let index = fixed_part | free_part;
                        self.repr.get(index).cloned().unwrap_or_default()
                    })
                    .collect::<Vec<Output>>();

                FValue::new(repr)
            })
            .count_unique()
    }

    pub fn table(&self, vars: &[usize]) -> Vec<Vec<Output>> {
        let mut fixed_positions = vars.to_vec();
        fixed_positions.sort();

        let free_positions = (0..self.repr().len().ilog2() as usize)
            .filter(|i| !fixed_positions.contains(i))
            .collect::<Vec<_>>();

        let (fixed, free) = (fixed_positions.len(), free_positions.len());

        let free_deposits: Vec<_> = (0..(1 << free))
            .map(|w| deposit(w, &free_positions))
            .collect();

        (0..(1 << fixed))
            .map(|v| {
                let fixed_part = deposit(v, &fixed_positions);
                free_deposits
                    .iter()
                    .map(|&free_part| {
                        let index = fixed_part | free_part;
                        self.repr.get(index).cloned().unwrap_or_default()
                    })
                    .collect::<Vec<Output>>()
            })
            .collect()
    }

    pub fn set_entropy(&self, vars: &[usize]) -> f32 {
        let mut fixed_positions = vars.to_vec();
        fixed_positions.sort();

        let free_positions = (0..self.repr().len().ilog2() as usize)
            .filter(|i| !fixed_positions.contains(i))
            .collect::<Vec<_>>();

        let (fixed, free) = (fixed_positions.len(), free_positions.len());

        let free_deposits: Vec<_> = (0..(1 << free))
            .map(|w| deposit(w, &free_positions))
            .collect();

        let map = (0..(1 << fixed))
            .map(|v| {
                let fixed_part = deposit(v, &fixed_positions);
                let repr = free_deposits
                    .iter()
                    .map(|&free_part| {
                        let index = fixed_part | free_part;
                        self.repr.get(index).cloned().unwrap_or_default()
                    })
                    .collect::<Vec<Output>>();

                FValue::new(repr)
            })
            .counts();

        let k2 = 1 << fixed;

        let sigmas = map
            .values()
            .map(|&x| x as f32 / k2 as f32)
            .collect::<Vec<f32>>();

        -sigmas.iter().map(|x| x * x.log2()).sum::<f32>()
    }

    pub fn information(&self, vars: &[usize]) -> usize {
        self.count_forms_by_multiple_fixed(vars)
    }

    pub async fn async_information(&self, vars: &[usize]) -> usize {
        self.count_forms_by_multiple_fixed(vars)
    }

    pub fn negate_var(&self, i: usize) -> Self {
        let mut repr: Vec<Output> = vec![Output::default(); self.repr.len()];
        let n = self.n_vars();
        for j in 0..self.repr.len() {
            repr[j ^ (1 << (n - 1 - i))] = self.repr[j].clone();
        }

        FValue::new(repr)
    }

    pub fn permutate_var(&self, i: usize, j: usize) -> Self {
        let n_vars = self.n_vars();
        let repr: Vec<Output> = (0..self.repr.len())
            .map(|k| {
                let bit_i = (k >> (n_vars - 1 - i)) & 1;
                let bit_j = (k >> (n_vars - 1 - j)) & 1;

                let mut k_prima = k & !(1 << (n_vars - 1 - i)) & !(1 << (n_vars - 1 - j));
                k_prima |= bit_j << (n_vars - 1 - i);
                k_prima |= bit_i << (n_vars - 1 - j);

                self.repr[k_prima].clone()
            })
            .collect();

        FValue::new(repr)
    }
}

impl FValue<bool> {
    pub fn negate(&self) -> Self {
        let repr: Vec<bool> = self.repr.iter().map(|x| !x).collect();
        FValue::new(repr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_and_decode() {
        let n = 5;
        let f1 = FValue::<bool>::primality(n);
        let raw = bincode::encode_to_vec(&f1, bincode::config::standard()).unwrap();
        let (f2, _): (FValue<bool>, _) =
            bincode::decode_from_slice(&raw, bincode::config::standard()).unwrap();

        for i in 0..1 << n {
            assert_eq!(f1.get(i), f2.get(i));
        }
    }
}
