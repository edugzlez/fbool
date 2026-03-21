use std::sync::Arc;

use clique_solver::find_clique;
use futures::{stream, StreamExt};
use kdam::{tqdm, BarExt};
use primality::IsPrime;
use tokio::sync::Mutex;

#[cfg(feature = "fmatrix")]
use crate::fmulti::FMulti;
use crate::fvalue::FValue;

impl FValue<bool> {
    pub fn majority(n_vars: usize) -> Self {
        let repr = (0usize..(1 << n_vars))
            .map(|n| {
                let number_of_ones = n.count_ones() as usize;
                number_of_ones > n_vars / 2
            })
            .collect::<Vec<_>>();

        FValue::new(repr)
    }

    pub fn random(n_vars: usize) -> Self {
        let repr = (0usize..(1 << n_vars))
            .map(|_| rand::random::<bool>())
            .collect::<Vec<_>>();

        FValue::new(repr)
    }

    pub fn parity(n_vars: usize) -> Self {
        let repr = (0usize..(1 << n_vars))
            .map(|n| {
                let number_of_ones = n.count_ones() as usize;
                number_of_ones.is_multiple_of(2)
            })
            .collect::<Vec<_>>();

        FValue::new(repr)
    }

    pub fn primality(n_vars: usize) -> Self {
        let repr = (0usize..(1 << n_vars))
            .map(|n| n == 1 || n.is_prime())
            .collect::<Vec<_>>();

        FValue::new(repr)
    }

    pub fn clique(n_vars: usize) -> Self {
        let n = n_vars * (n_vars + 1) / 2; // Number of edges in a complete graph with n nodes
        let repr = (0usize..(1 << n))
            .map(|v| {
                let v = (0..n).map(|i| (v >> i) & 1 == 1).collect::<Vec<_>>();

                find_clique(&v, n_vars, n_vars / 2).is_some()
            })
            .collect::<Vec<bool>>();

        FValue::new(repr)
    }

    pub fn equality(n_vars: usize) -> Self {
        let total_nvars = 2 * n_vars;

        let repr = (0..(1 << total_nvars))
            .map(|x| {
                let a = x >> n_vars;
                let b = x & ((1 << n_vars) - 1);
                a == b
            })
            .collect::<Vec<_>>();

        FValue::new(repr)
    }

    pub fn constant(n_vars: usize, value: bool) -> Self {
        let repr = vec![value; 1 << n_vars];

        FValue::new(repr)
    }

    pub fn find_zero(vector_size: usize, element_size: usize) -> Self {
        let n_vars = vector_size * element_size;

        let repr = (0..(1 << n_vars) as usize)
            .map(|n| {
                let mut found = false;
                for i in (0..n_vars).step_by(element_size) {
                    let is_zero = (i..(i + element_size))
                        .map(|x| (n >> x) & 1 == 1)
                        .all(|x| !x);

                    if is_zero {
                        found = true;
                        break;
                    }
                }
                found
            })
            .collect();

        FValue::new(repr)
    }

    pub fn sum_is_prime(n_vars: usize) -> Self {
        let total_nvars = 2 * n_vars;

        let repr = (0..(1 << total_nvars))
            .map(|x| {
                let a = x >> n_vars;
                let b = x & ((1 << n_vars) - 1);
                (a + b).is_prime()
            })
            .collect::<Vec<_>>();

        FValue::new(repr)
    }

    pub fn product_is_multiple_of(n_vars: usize, multiple: usize) -> Self {
        let total_nvars = 2 * n_vars;

        let repr = (0..(1 << total_nvars))
            .map(|x: usize| {
                let a = x >> n_vars;
                let b = x & ((1 << n_vars) - 1);
                (a + b).is_multiple_of(multiple)
            })
            .collect::<Vec<_>>();

        FValue::new(repr)
    }

    pub fn coprimes(n_vars: usize) -> Self {
        let total_nvars = 2 * n_vars;

        let repr = (0..(1 << total_nvars))
            .map(|x| {
                let a = x >> n_vars;
                let b = x & ((1 << n_vars) - 1);
                gcd(a, b) == 1
            })
            .collect::<Vec<_>>();

        FValue::new(repr)
    }

    pub fn ordered(n_vars: usize) -> Self {
        let total_nvars = 2 * n_vars;

        let repr = (0..(1 << total_nvars))
            .map(|x| {
                let a = x >> n_vars;
                let b = x & ((1 << n_vars) - 1);
                a <= b
            })
            .collect::<Vec<_>>();

        FValue::new(repr)
    }

    pub fn from_usize(fun: usize, n_vars: usize) -> Self {
        let repr = (0..(1 << n_vars))
            .map(|idx| (fun >> idx) & 1 == 1)
            .collect();

        FValue::new(repr)
    }
}

#[cfg(feature = "fmatrix")]
impl FMulti<bool> {
    pub fn product(n_vars: usize) -> Self {
        let total_nvars = 2 * n_vars;

        let repr = (0..(1 << total_nvars))
            .map(|x| {
                let a = x >> n_vars;
                let b = x & ((1 << n_vars) - 1);
                let c = a * b;
                (0..total_nvars).map(|i| (c >> i) & 1 == 1).collect()
            })
            .collect::<Vec<_>>();

        FMulti::new(repr)
    }
}

impl FValue<usize> {
    pub fn multiply(n_vars: usize, factor: usize) -> Self {
        let repr = (0..(1 << n_vars))
            .rev()
            .map(|x| x * factor)
            .collect::<Vec<_>>();

        FValue::new(repr)
    }

    pub fn constant(n_vars: usize, value: usize) -> Self {
        let repr = (0..(1 << n_vars)).map(|_| value).collect::<Vec<_>>();

        FValue::new(repr)
    }
}

pub fn gcd(a: usize, b: usize) -> usize {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

impl FValue<usize> {
    pub fn sum(n_vars: usize) -> Self {
        let total_nvars = 2 * n_vars;

        let repr = (0..(1 << total_nvars))
            .map(|x| {
                let a = x >> n_vars;
                let b = x & ((1 << n_vars) - 1);
                a + b
            })
            .collect::<Vec<_>>();

        FValue::new(repr)
    }

    pub fn product(n_vars: usize) -> Self {
        let total_nvars = 2 * n_vars;

        let repr = (0..(1 << total_nvars))
            .map(|x| {
                let a = x >> n_vars;
                let b = x & ((1 << n_vars) - 1);
                a * b
            })
            .collect::<Vec<_>>();

        FValue::new(repr)
    }

    pub fn max(n_vars: usize) -> Self {
        let total_nvars = 2 * n_vars;

        let repr = (0..(1 << total_nvars))
            .map(|x| {
                let a = x >> n_vars;
                let b = x & ((1 << n_vars) - 1);
                a.max(b)
            })
            .collect::<Vec<_>>();

        FValue::new(repr)
    }

    pub fn gcd(n_vars: usize) -> Self {
        let total_nvars = 2 * n_vars;

        let repr = (0..(1 << total_nvars))
            .map(|x| {
                let a = x >> n_vars;
                let b = x & ((1 << n_vars) - 1);
                gcd(a, b)
            })
            .collect::<Vec<_>>();

        FValue::new(repr)
    }

    pub fn sum_some(n_vars: usize, elements: usize) -> Self {
        let total_nvars = elements * n_vars;

        let repr = (0..(1 << total_nvars))
            .map(|x| {
                let mut sum = 0;
                for i in (0..total_nvars).step_by(n_vars) {
                    sum += (x >> i) & ((1 << n_vars) - 1);
                }
                sum
            })
            .collect::<Vec<_>>();

        FValue::new(repr)
    }

    pub async fn meta_entanglement(n_vars: usize) -> Self {
        let total_nvars = 1usize << n_vars;
        let pb = Arc::new(Mutex::new(tqdm!(total = 1 << total_nvars)));

        let repr = stream::iter((0..(1 << total_nvars)).map(|_| {
            let pb = Arc::clone(&pb);
            async move {
                // let f = FValue::from_usize(x, total_nvars);
                // let e = f.entanglement();
                let e = usize::MAX;
                let mut pb = pb.lock().await;
                pb.update(1).unwrap();
                e
            }
        }))
        .buffered(16)
        .collect::<Vec<_>>()
        .await;

        FValue::new(repr)
    }
}
