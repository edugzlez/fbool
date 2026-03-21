use crate::fvalue::FValue;
use petgraph::graph::UnGraph;

/// Trait for computing the frontier of a Boolean function.
pub trait Frontier {
    /// Determines whether the pair (x, y) forms a frontier edge.
    ///
    /// Returns true if x and y differ by exactly one bit and their function values are different.
    ///
    /// # Arguments
    ///
    /// * `x` - The first input value.
    /// * `y` - The second input value.
    fn is_frontier(&self, x: usize, y: usize) -> bool;

    /// Constructs the frontier graph of the Boolean function.
    ///
    /// Returns an undirected graph where nodes represent input values and edges connect pairs (x, y)
    /// that form a frontier (i.e., differ by one bit and have different function values).
    fn frontier_graph(&self) -> UnGraph<u32, ()>;

    /// Returns the number of input values for which the function evaluates to false (0).
    fn b_0(&self) -> usize;

    /// Returns the number of input values for which the function evaluates to true (1).
    fn b_1(&self) -> usize;
}

impl Frontier for FValue<bool> {
    /// Determines whether the pair (x, y) forms a frontier edge.
    ///
    /// Returns true if x and y differ by exactly one bit and their function values are different.
    fn is_frontier(&self, x: usize, y: usize) -> bool {
        (x ^ y).count_ones() == 1 && self.get(x).unwrap() != self.get(y).unwrap()
    }

    /// Constructs the frontier graph of the Boolean function.
    ///
    /// Returns an undirected graph where nodes represent input values and edges connect pairs (x, y)
    /// that form a frontier (i.e., differ by one bit and have different function values).
    fn frontier_graph(&self) -> UnGraph<u32, ()> {
        let max_value = 1 << self.n_vars();
        let edges: Vec<_> = (0..max_value)
            .filter(|x| self.get(*x).unwrap() == &false)
            .flat_map(|x| {
                (0..self.n_vars())
                    .map(move |b| x ^ (1 << b))
                    .filter(move |y| self.get(*y).unwrap() == &true)
                    .map(move |y| (x as u32, y as u32))
            })
            .collect();

        UnGraph::<u32, ()>::from_edges(edges)
    }

    /// Returns the number of input values for which the function evaluates to false (0).
    fn b_0(&self) -> usize {
        let max_value = 1 << self.n_vars();
        (0..max_value).filter(|x| !self.get(*x).unwrap()).count()
    }

    /// Returns the number of input values for which the function evaluates to true (1).
    fn b_1(&self) -> usize {
        let max_value = 1 << self.n_vars();
        (0..max_value).filter(|x| *self.get(*x).unwrap()).count()
    }
}
