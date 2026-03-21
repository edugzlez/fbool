use fwht::FWHT;

use crate::fvalue::FValue;

impl FValue<bool> {
    /// Computes the Walsh coefficients of the boolean function.
    ///
    /// This function converts the truth table representation to a Walsh spectrum
    /// by applying the Fast Walsh-Hadamard Transform (FWHT). The boolean values
    /// are mapped to {-1, 1} before transformation.
    ///
    /// Returns a vector of Walsh coefficients as i8 values.
    // TODO: try to cache this function
    pub fn walsh_coeficients(&self) -> Vec<isize> {
        let mut truth_table = self
            .repr()
            .iter()
            .map(|x| if *x { 1 } else { -1 })
            .collect::<Vec<isize>>();

        truth_table.fwht_mut().unwrap();

        truth_table
    }

    /// Computes the Fourier coefficients of the boolean function.
    ///
    /// This function normalizes the Walsh coefficients by dividing each
    /// coefficient by the total number of possible inputs (2^n_vars).
    ///
    /// Returns a vector of normalized Fourier coefficients as f32 values.
    pub fn fourier_coeficients(&self) -> Vec<f32> {
        let n = (1 << self.n_vars()) as f32;
        self.walsh_coeficients()
            .iter()
            .map(|x| *x as f32 / n)
            .collect()
    }

    /// Computes the algebraic degree of the boolean function.
    ///
    /// The degree is determined by finding the highest Hamming weight
    /// (number of 1s) among the indices of non-zero Walsh coefficients.
    /// This corresponds to the highest degree monomial in the algebraic
    /// normal form of the function.
    ///
    /// Returns the degree as a usize value.
    pub fn degree(&self) -> usize {
        let coefs = self.walsh_coeficients();

        let mut degree = 0;
        for (i, &coef) in coefs.iter().enumerate() {
            if coef != 0 {
                let order = i.count_ones() as usize;
                if order > degree {
                    degree = order;
                }
            }
        }

        degree
    }

    /// Computes the spectral entropy of the boolean function.
    ///
    /// This function calculates the Shannon entropy based on the squared
    /// Fourier coefficients, which represent probability distributions.
    /// The entropy measures the randomness or unpredictability of the
    /// function's spectrum.
    ///
    /// Returns the spectral entropy as an f32 value.
    pub fn spectral_entropy(&self) -> f32 {
        self.fourier_coeficients()
            .iter()
            .filter(|&&coef| coef != 0f32)
            .map(|coef| {
                let prob = coef.powi(2);
                -prob * prob.ln()
            })
            .sum::<f32>()
    }

    /// Computes the nonlinearity of the boolean function.
    ///
    /// Nonlinearity measures the minimum Hamming distance between the
    /// function and all affine functions. It is calculated using the
    /// maximum absolute Walsh coefficient. Higher nonlinearity indicates
    /// better cryptographic properties.
    ///
    /// Returns the nonlinearity as an u32 value.
    pub fn no_linearity(&self) -> u32 {
        let n = (1 << (self.n_vars() - 1)) as f32;
        let max_walsh = self
            .walsh_coeficients()
            .iter()
            .map(|&x| x.abs())
            .max()
            .unwrap() as f32;

        (n - (max_walsh / 2.0)) as u32
    }
}

#[cfg(test)]
mod tests {
    use crate::fvalue::FValue;

    /// Tests the Parseval identity for Fourier coefficients.
    ///
    /// The Parseval identity states that the sum of squares of
    /// Fourier coefficients should equal 1 for normalized coefficients.
    /// This test verifies this property holds for various boolean functions.
    #[test]
    fn test_parseval_identity() {
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
            let coefs = f.fourier_coeficients();

            let sum_of_squares: f32 = coefs.iter().map(|x| x.powi(2)).sum();

            assert_eq!(sum_of_squares, 1.0);
        }
    }
}
