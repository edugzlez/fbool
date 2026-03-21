use crate::fvalue::FValue;

impl FValue<bool> {
    /// Calculates the influence of the i-th variable on the boolean function.
    /// The influence measures how much the function's output depends on the i-th input variable.
    /// It is computed as the sum of squares of Fourier coefficients for sets containing variable i.
    pub fn influence(&self, i: usize) -> f32 {
        let coefs = self.fourier_coeficients();

        coefs
            .iter()
            .enumerate()
            .filter(|(index, _)| (index & (1 << i)) != 0)
            .map(|(_, coef)| coef * coef)
            .sum()
    }

    /// Calculates the total influence of the boolean function.
    /// This is the sum of all individual variable influences, weighted by the size of each subset.
    /// It provides a measure of the overall sensitivity of the function to its inputs.
    pub fn total_influence(&self) -> f32 {
        let coefs = self.fourier_coeficients();

        coefs
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != 0)
            .map(|(index, coef)| coef * coef * index.count_ones() as f32)
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use crate::fvalue::FValue;

    #[test]
    fn test_total_influence() {
        let test_cases = &[
            FValue::from_usize(0b1100, 2),
            FValue::from_usize(0b1010, 2),
            FValue::from_usize(0b1111, 2),
            FValue::from_usize(0b1001, 2),
            FValue::from_usize(0b10101010, 3),
            FValue::from_usize(0b11110000, 3),
            FValue::from_usize(245431, 5),
            FValue::from_usize(766583, 4),
            FValue::parity(10),
        ];

        for f in test_cases {
            let n_vars = f.n_vars();

            assert_eq!(
                (0..n_vars)
                    .map(|i| {
                        let inf = f.influence(i);
                        assert!(inf >= 0.0, "Influence should be non-negative");
                        inf
                    })
                    .sum::<f32>(),
                f.total_influence()
            );
        }
    }
}
