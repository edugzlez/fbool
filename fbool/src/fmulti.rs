//! Multiple boolean function representation and operations.
//!
//! This module provides the `FMulti` struct which represents multiple boolean functions
//! as a vector of vectors, allowing for operations on collections of boolean functions.

use bincode::{Decode, Encode};
use std::hash::Hash;

use crate::auxiliar::deposit;
use crate::auxiliar::CountUnique;
use crate::fvalue::FValue;

/// A trait for generic values that can be used in multiple boolean function representations.
///
/// This trait requires types to be comparable, have a default value, be cloneable,
/// and have a total ordering.
pub trait GenericValue: Eq + Default + Clone + Ord {}

/// Blanket implementation of GenericValue for any type that satisfies the trait bounds.
impl<Output: Eq + Default + Clone + Ord> GenericValue for Output {}

/// Represents multiple boolean functions as a collection of value vectors.
///
/// The `FMulti` struct stores multiple boolean functions where each function
/// is represented as a vector of values. The outer vector length must be a power of 2
/// (corresponding to 2^n for n external variables), and each inner vector represents
/// the values for one of multiple internal functions.
///
/// # Type Parameters
/// * `Output` - The type of values stored in the function representations, must implement `GenericValue`
///
/// # Examples
/// ```
/// use fbool::fmulti::FMulti;
///
/// // Create multiple boolean functions with 4 entries each (2 variables)
/// let multi_func = FMulti::new(vec![
///     vec![true, false],
///     vec![false, true],
///     vec![true, true],
///     vec![false, false]
/// ]);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FMulti<Output: GenericValue> {
    /// Internal representation as a vector of function vectors
    repr: Vec<Vec<Output>>,
}

impl<Output: GenericValue + Encode> Encode for FMulti<Output> {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        self.repr.encode(encoder)
    }
}

impl<Output: GenericValue + Decode<()>> Decode<()> for FMulti<Output> {
    fn decode<D: bincode::de::Decoder<Context = ()>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        Vec::decode(decoder).map(FMulti::new)
    }
}

fn is_valid_size(n: usize) -> bool {
    n > 0 && n.is_power_of_two()
}

impl<Output: GenericValue> FMulti<Output> {
    /// Creates a new `FMulti` with the given representation.
    ///
    /// # Arguments
    /// * `repr` - A vector of vectors representing multiple boolean functions.
    ///   The outer vector length must be a power of 2.
    ///
    /// # Panics
    /// Panics if the length of the outer vector is not a positive power of 2.
    ///
    /// # Examples
    /// ```
    /// use fbool::fmulti::FMulti;
    ///
    /// let multi_func = FMulti::new(vec![
    ///     vec![true, false],
    ///     vec![false, true],
    ///     vec![true, true],
    ///     vec![false, false]
    /// ]);
    /// ```
    pub fn new(repr: Vec<Vec<Output>>) -> Self {
        if !is_valid_size(repr.len()) {
            panic!("Invalid size");
        }

        FMulti { repr }
    }

    /// Returns a reference to the internal representation.
    ///
    /// # Returns
    /// A reference to the vector of vectors containing the multiple function values.
    pub fn repr(&self) -> &Vec<Vec<Output>> {
        &self.repr
    }
}

impl<Output: Send + Sync + Hash + GenericValue> FMulti<Output> {
    pub fn n_vars(&self) -> usize {
        self.repr.len().ilog2() as usize
    }

    pub fn internal_vars(&self) -> usize {
        self.repr.first().map(Vec::len).unwrap_or(0)
    }

    pub fn get(&self, i: usize) -> Option<&Vec<Output>> {
        self.repr.get(i)
    }

    pub fn fixed(&self, var: usize, value: bool) -> Self {
        let it = crate::auxiliar::binary_numbers(self.n_vars(), var, value as usize);

        let mut new_repr = vec![];

        it.for_each(|x| {
            let bit = self.repr.get(x).cloned().unwrap_or_default();
            new_repr.push(bit.clone());
        });

        FMulti::new(new_repr)
    }

    pub fn multiple_fixed(&self, mut vars: Vec<(usize, bool)>) -> Self {
        let mut new_repr = self.clone();
        vars.sort_by_key(|a| a.0);

        for (i, (var, value)) in vars.iter().enumerate() {
            new_repr = new_repr.fixed(*var - i, *value);
        }

        new_repr
    }

    pub fn list_forms_by_fixed(&self, var: usize) -> Vec<Self> {
        vec![self.fixed(var, false), self.fixed(var, true)]
    }

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

        (0..self.internal_vars())
            .map(|j| {
                (0..(1 << fixed))
                    .map(|v| {
                        let fixed_part = deposit(v, &fixed_positions);
                        let repr = free_deposits
                            .iter()
                            .map(|&free_part| {
                                let index = fixed_part | free_part;
                                self.repr
                                    .get(index)
                                    .and_then(|v| v.get(j))
                                    .cloned()
                                    .unwrap_or_default()
                            })
                            .collect::<Vec<Output>>();

                        FValue::<Output>::new(repr)
                    })
                    .count_unique()
            })
            .max()
            .unwrap_or(usize::MAX)
    }

    pub fn information(&self, vars: &[usize]) -> usize {
        self.count_forms_by_multiple_fixed(vars)
    }

    pub async fn async_information(&self, vars: &[usize]) -> usize {
        self.count_forms_by_multiple_fixed(vars)
    }
}
