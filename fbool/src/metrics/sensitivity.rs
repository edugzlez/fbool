use crate::fvalue::FValue;

pub trait Sensitivity {
    fn point_sensitivity(&self, x: usize) -> u32;
    fn max_sensitivity(&self) -> u32;
    fn mean_sensitivity(&self) -> f32;
}

impl Sensitivity for FValue<bool> {
    fn point_sensitivity(&self, x: usize) -> u32 {
        let value = self.get(x).unwrap();
        (0..self.n_vars())
            .map(move |b| x ^ (1 << b))
            .filter(move |y| self.get(*y).unwrap() != value)
            .count() as u32
    }

    fn max_sensitivity(&self) -> u32 {
        let max_value = 1 << self.n_vars();

        (0..max_value)
            .map(|x| self.point_sensitivity(x))
            .max()
            .unwrap()
    }

    fn mean_sensitivity(&self) -> f32 {
        let max_value = 1 << self.n_vars();

        (0..max_value)
            .map(|x| self.point_sensitivity(x))
            .sum::<u32>() as f32
            / max_value as f32
    }
}
