//! Auxiliary utilities for boolean function analysis.
//!
//! This module provides various helper functions and iterators for working with
//! boolean functions, binary representations, and combinatorial operations.

#![allow(dead_code)]
use std::ops::Range;

use itertools::{Combinations, Itertools};
use rand::seq::SliceRandom;
use rayon::iter::{IntoParallelIterator, ParallelBridge};

/// Iterator over binary numbers that have a specific bit value at a given position.
///
/// This iterator generates binary numbers where a specified variable (bit position)
/// has a particular value (0 or 1), useful for boolean function analysis.
#[derive(Debug, Clone)]
pub struct BinaryIterator {
    current: usize,
    max: usize,
    size: usize,
    variable: usize,
    value: usize,
}

impl Iterator for BinaryIterator {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current <= self.max {
            let num = self.current;
            self.current += 1;
            if (num >> self.variable) & 1 == self.value {
                return Some(num);
            }
        }
        None
    }
}

/// Iterator over all possible bipartitions of variables.
///
/// This iterator generates all possible ways to split `n` variables into two
/// complementary subsets, useful for entanglement analysis.
pub struct SubsetIterator {
    _n: usize,
    _comb: Combinations<Range<usize>>,
}

impl SubsetIterator {
    pub fn new(n: usize) -> Self {
        let m = n / 2;
        if n % 2 == 0 {
            Self {
                _n: n,
                _comb: (1..n).combinations(m - 1),
            }
        } else {
            Self {
                _n: n,
                _comb: (0..n).combinations(m),
            }
        }
    }

    #[allow(dead_code)]
    pub fn length(&self) -> usize {
        let n = self._n;
        let m = n / 2;

        (1..m).fold(1, |acc, i| acc * (n - i) / (i + 1)) // Se ajusta por el elemento fijo 0
    }
}

impl Iterator for SubsetIterator {
    type Item = (Vec<usize>, Vec<usize>);

    fn next(&mut self) -> Option<Self::Item> {
        match self._comb.next() {
            Some(mut set1) => {
                if self._n % 2 == 0 {
                    set1.insert(0, 0);
                }
                let set2 = (0..self._n)
                    .filter(|x| !set1.contains(x))
                    .collect::<Vec<_>>();
                Some((set1, set2))
            }
            None => None,
        }
    }
}

impl IntoParallelIterator for SubsetIterator {
    type Item = (Vec<usize>, Vec<usize>);
    type Iter = rayon::iter::IterBridge<SubsetIterator>;

    fn into_par_iter(self) -> Self::Iter {
        self.par_bridge()
    }
}

pub fn binary_numbers(size: usize, variable: usize, value: usize) -> BinaryIterator {
    BinaryIterator {
        current: 0,
        max: (1 << size) - 1,
        size,
        variable,
        value,
    }
}

pub fn usize_to_string(n: usize, size: usize) -> String {
    let mut s = String::with_capacity(size);
    for i in (0..size).rev() {
        let bit = (n >> i) & 1;
        s.push_str(&bit.to_string());
    }
    s
}

pub fn is_power_of_two(n: usize) -> bool {
    n & (n - 1) == 0
}

pub fn shuffle<T>(v: &mut [T]) {
    v.shuffle(&mut rand::thread_rng());
}

pub fn separate<T: Clone>(v: Vec<T>, n: usize) -> (Vec<T>, Vec<T>) {
    let mut v = v;
    shuffle(&mut v);
    let (a, b) = v.split_at(n);
    (a.to_vec(), b.to_vec())
}

pub fn vec_to_string(vec: &Vec<usize>) -> String {
    let mut s = String::new();
    for e in vec {
        s.push_str(&e.to_string());
        s.push(' ');
    }
    s
}

pub fn deposit(assignment: usize, positions: &[usize]) -> usize {
    let mut result = 0;
    for (i, &pos) in positions.iter().enumerate() {
        let bit = (assignment >> i) & 1;
        result |= bit << pos;
    }
    result
}

pub trait CountUnique {
    fn count_unique(self) -> usize;
}

impl<I, T> CountUnique for I
where
    I: Iterator<Item = T>,
    T: Eq + ::std::hash::Hash,
{
    fn count_unique(self) -> usize {
        self.collect::<::std::collections::HashSet<_>>().len()
    }
}
