use crate::metrics::entanglement::{NVars, WithEntropy};
use itertools::Itertools;
use rayon::prelude::*;

/// Minimum number of variables required to activate parallel computation.
const PARALLEL_THRESHOLD: usize = 10;

/// Represents the peak of a fragmentation spectrum.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FragmentationPeak {
    /// Argmax position of the spectrum.
    pub k_star: usize,
    /// Maximum spectrum value.
    pub s_max: f32,
}

/// Trait for computing fragmentation-profile metrics of boolean functions.
///
/// This extends the entropy view from balanced bipartitions to all subset sizes.
pub trait Fragmentation {
    /// Local fragmentation coefficient F(f, S).
    fn fragmentation_coefficient(&self, vars: &[usize]) -> f32;

    /// Average fragmentation over all subsets of size k, i.e. S_k(f).
    fn fragmentation_k(&self, k: usize) -> f32;

    /// Full spectrum (S_0(f), ..., S_n(f)).
    fn fragmentation_spectrum(&self) -> Vec<f32>;

    /// Alias for the full restriction signature.
    fn restriction_signature(&self) -> Vec<f32> {
        self.fragmentation_spectrum()
    }

    /// Alias for the full fragmentation profile.
    fn fragmentation_profile(&self) -> Vec<f32> {
        self.fragmentation_spectrum()
    }

    /// Peak (k*, Smax) of the spectrum.
    fn fragmentation_peak(&self) -> FragmentationPeak;

    /// First discrete derivative ΔS_k = S_{k+1} - S_k.
    fn fragmentation_delta(&self) -> Vec<f32>;

    /// Second discrete derivative Δ²S_k = ΔS_{k+1} - ΔS_k.
    fn fragmentation_delta2(&self) -> Vec<f32>;
}

impl<T: WithEntropy + NVars + Sync> Fragmentation for T {
    fn fragmentation_coefficient(&self, vars: &[usize]) -> f32 {
        self.get_entropy(vars)
    }

    fn fragmentation_k(&self, k: usize) -> f32 {
        let n = self.num_vars();
        if k > n {
            return 0.0;
        }

        let subsets: Vec<Vec<usize>> = (0..n).combinations(k).collect();
        if subsets.is_empty() {
            return 0.0;
        }

        let sum = if n >= PARALLEL_THRESHOLD {
            subsets
                .par_iter()
                .map(|set| self.fragmentation_coefficient(set))
                .sum::<f32>()
        } else {
            subsets
                .iter()
                .map(|set| self.fragmentation_coefficient(set))
                .sum::<f32>()
        };

        sum / subsets.len() as f32
    }

    fn fragmentation_spectrum(&self) -> Vec<f32> {
        let n = self.num_vars();
        (0..=n).map(|k| self.fragmentation_k(k)).collect()
    }

    fn fragmentation_peak(&self) -> FragmentationPeak {
        let spectrum = self.fragmentation_spectrum();

        let mut k_star = 0usize;
        let mut s_max = f32::NEG_INFINITY;
        for (k, &s) in spectrum.iter().enumerate() {
            if s > s_max {
                k_star = k;
                s_max = s;
            }
        }

        FragmentationPeak { k_star, s_max }
    }

    fn fragmentation_delta(&self) -> Vec<f32> {
        let spectrum = self.fragmentation_spectrum();
        if spectrum.len() < 2 {
            return Vec::new();
        }

        spectrum.windows(2).map(|w| w[1] - w[0]).collect()
    }

    fn fragmentation_delta2(&self) -> Vec<f32> {
        let delta = self.fragmentation_delta();
        if delta.len() < 2 {
            return Vec::new();
        }

        delta.windows(2).map(|w| w[1] - w[0]).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fvalue::FValue;

    fn approx_eq(a: f32, b: f32, tol: f32) {
        assert!((a - b).abs() <= tol, "left={a}, right={b}, tol={tol}");
    }

    #[test]
    fn test_fragmentation_spectrum_parity_is_flat_at_one_inside() {
        let f = FValue::parity(5);
        let spectrum = f.fragmentation_spectrum();

        assert_eq!(spectrum.len(), 6);
        approx_eq(spectrum[0], 0.0, 1e-6);
        for &v in &spectrum[1..] {
            approx_eq(v, 1.0, 1e-6);
        }

        let peak = f.fragmentation_peak();
        assert_eq!(peak.k_star, 1);
        approx_eq(peak.s_max, 1.0, 1e-6);
    }

    #[test]
    fn test_fragmentation_k_for_and_matches_h2_to_minus_k() {
        let f = FValue::from_usize(1usize << 7, 3);

        let s0 = f.fragmentation_k(0);
        let s1 = f.fragmentation_k(1);
        let s2 = f.fragmentation_k(2);
        let s3 = f.fragmentation_k(3);

        approx_eq(s0, 0.0, 1e-6);
        approx_eq(s1, 1.0, 1e-6);
        approx_eq(s2, 0.811_278_1, 1e-5);
        approx_eq(s3, 0.543_564_44, 1e-5);
    }

    #[test]
    fn test_fragmentation_derivatives_lengths() {
        let f = FValue::majority(5);

        let spectrum = f.fragmentation_spectrum();
        let d1 = f.fragmentation_delta();
        let d2 = f.fragmentation_delta2();

        assert_eq!(d1.len(), spectrum.len() - 1);
        assert_eq!(d2.len(), d1.len() - 1);
    }
}
