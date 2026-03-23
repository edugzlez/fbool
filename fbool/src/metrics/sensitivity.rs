use crate::fvalue::FValue;

impl FValue<bool> {
    fn sensitivities(&self) -> impl Iterator<Item = u32> + '_ {
        let max_value = 1 << self.n_vars();
        (0..max_value).map(|x| self.point_sensitivity(x))
    }

    pub fn point_sensitivity(&self, x: usize) -> u32 {
        let value = self.get(x).unwrap();
        (0..self.n_vars())
            .map(move |b| x ^ (1 << b))
            .filter(move |y| self.get(*y).unwrap() != value)
            .count() as u32
    }

    pub fn max_sensitivity(&self) -> u32 {
        self.sensitivities().max().unwrap()
    }

    pub fn min_sensitivity(&self) -> u32 {
        self.sensitivities().min().unwrap()
    }

    pub fn mean_sensitivity(&self) -> f32 {
        let max_value = 1 << self.n_vars();

        self.sensitivities().sum::<u32>() as f32 / max_value as f32
    }

    pub fn entropy_sensitivity(&self) -> f32 {
        let max_value = self.n_vars();
        let mut counts = vec![0u32; max_value];
        let mut total_sensitivity = 0;
        for local_sensitivity in self.sensitivities() {
            counts[local_sensitivity as usize] += 1;
            total_sensitivity += local_sensitivity;
        }

        counts
            .iter()
            .map(|&c| {
                if c == 0 {
                    0.0
                } else {
                    c as f32 / total_sensitivity as f32
                }
            })
            .filter(|&s| s != 0.0)
            .map(|s| -s * s.log2())
            .sum::<f32>()
    }

    pub fn vector_sensitivity(&self) -> Vec<u32> {
        self.sensitivities().collect()
    }

    pub fn var_sensitivity(&self) -> f32 {
        let max_value = 1 << self.n_vars();
        let mean = self.mean_sensitivity();
        self.sensitivities()
            .map(|s| (s as f32 - mean).powi(2))
            .sum::<f32>()
            / max_value as f32
    }
}
