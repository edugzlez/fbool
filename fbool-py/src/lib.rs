use fbool::certificate::CertificateComplexity;
use fbool::entanglement::{Entanglement, Entropy, EquanimityImportance};
use fbool::frontier::Frontier;
use fbool::fvalue::FValue;
use fbool::sensitivity::Sensitivity;
use numpy::ndarray::{Array, Dim};
use numpy::{PyArray, ToPyArray};
use optimal5::WithMinimalGates;
use petgraph::graph::UnGraph;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyType;
use pyo3::{pymodule, types::PyModule, Bound, PyResult};

#[pyclass]
struct FBool {
    repr: FValue<bool>,

    buffered_frontier: Option<UnGraph<u32, ()>>,
}

#[pyclass]
struct EntanglementSet {
    #[pyo3(get)]
    entanglement: usize,
    #[pyo3(get)]
    set1: Vec<usize>,
    #[pyo3(get)]
    set2: Vec<usize>,
}

#[pyclass]
struct EntropySets {
    #[pyo3(get)]
    entropy: f32,
    #[pyo3(get)]
    set1: Vec<usize>,
    #[pyo3(get)]
    set2: Vec<usize>,
}

impl FBool {
    fn new_from_fbool(fbool: FValue<bool>) -> Self {
        FBool {
            repr: fbool,
            buffered_frontier: None,
        }
    }

    fn obtain_frontier_graph(&mut self) -> &UnGraph<u32, ()> {
        if self.buffered_frontier.is_none() {
            self.buffered_frontier = Some(self.repr.frontier_graph());
        }
        self.buffered_frontier.as_ref().unwrap()
    }
}

#[pymethods]
impl FBool {
    #[new]
    fn new(repr: Vec<bool>) -> Self {
        FBool {
            repr: FValue::new(repr),
            buffered_frontier: None,
        }
    }

    #[staticmethod]
    fn from_number(number: usize, num_vars: usize) -> Self {
        let mut repr = Vec::with_capacity(1 << num_vars);
        for i in 0..(1 << num_vars) {
            repr.push((number >> i) & 1 == 1);
        }
        FBool {
            repr: FValue::new(repr),
            buffered_frontier: None,
        }
    }
    fn eval(&self, i: usize) -> PyResult<bool> {
        self.repr.get(i).copied().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyIndexError, _>(format!("Index out of range: {i}"))
        })
    }

    fn repr(&self) -> PyResult<Vec<bool>> {
        let repr = self.repr.repr();
        Ok(repr.clone())
    }

    fn size(&self) -> PyResult<usize> {
        Ok(self.repr.repr().len())
    }

    #[classmethod]
    fn majority(_: &Bound<'_, PyType>, n: usize) -> PyResult<Self> {
        let repr = FValue::<bool>::majority(n);
        Ok(FBool::new_from_fbool(repr))
    }

    #[classmethod]
    fn parity(_: &Bound<'_, PyType>, n: usize) -> PyResult<Self> {
        let repr = FValue::<bool>::parity(n);
        Ok(FBool::new_from_fbool(repr))
    }

    #[classmethod]
    fn coprimes(_: &Bound<'_, PyType>, n: usize) -> PyResult<Self> {
        let repr = FValue::<bool>::coprimes(n);
        Ok(FBool::new_from_fbool(repr))
    }

    #[classmethod]
    fn primality(_: &Bound<'_, PyType>, n: usize) -> PyResult<Self> {
        let repr = FValue::<bool>::primality(n);
        Ok(FBool::new_from_fbool(repr))
    }

    #[classmethod]
    fn sum_is_prime(_: &Bound<'_, PyType>, n: usize) -> PyResult<Self> {
        let repr = FValue::<bool>::sum_is_prime(n);
        Ok(FBool::new_from_fbool(repr))
    }

    #[classmethod]
    fn clique(_: &Bound<'_, PyType>, n: usize) -> PyResult<Self> {
        let repr = FValue::<bool>::clique(n);
        Ok(FBool::new_from_fbool(repr))
    }

    fn n_vars(&self) -> PyResult<usize> {
        let n = self.repr.n_vars();
        Ok(n)
    }

    fn entanglement(&self) -> PyResult<usize> {
        let ent = self.repr.entanglement();
        Ok(ent)
    }

    fn entropy(&self) -> PyResult<f32> {
        let ent = self.repr.entropy();
        Ok(ent)
    }

    fn equanimity_importance(&self) -> PyResult<f32> {
        let ent = self.repr.equanimity_importance();
        Ok(ent)
    }

    fn entanglement_sets(&self) -> PyResult<Vec<EntanglementSet>> {
        let ent_sets = self.repr.entanglement_sets();
        let py_sets = ent_sets
            .into_iter()
            .map(|set| EntanglementSet {
                entanglement: set.entanglement,
                set1: set.set1,
                set2: set.set2,
            })
            .collect::<Vec<EntanglementSet>>();

        Ok(py_sets)
    }

    fn entropy_sets(&self) -> PyResult<Vec<EntropySets>> {
        let ent_sets = self.repr.entropy_sets();
        let py_sets = ent_sets
            .into_iter()
            .map(|set| EntropySets {
                entropy: set.entropy,
                set1: set.set1,
                set2: set.set2,
            })
            .collect::<Vec<EntropySets>>();

        Ok(py_sets)
    }

    fn information(&self, vars: Vec<usize>) -> PyResult<usize> {
        let info = self.repr.information(&vars);
        Ok(info)
    }

    fn encode(&self) -> PyResult<Vec<u8>> {
        bincode::encode_to_vec(&self.repr, bincode::config::standard())
            .map_err(|_| PyValueError::new_err("Unable to encode"))
    }

    #[classmethod]
    fn decode(_: &Bound<'_, PyType>, raw: &[u8]) -> PyResult<Self> {
        bincode::decode_from_slice(raw, bincode::config::standard())
            .map(|(repr, _)| FBool::new_from_fbool(repr))
            .map_err(|_| PyValueError::new_err("Unable to decode"))
    }

    fn minimal_gates(&self) -> PyResult<Option<u32>> {
        let minimal_gates = self.repr.minimal_gates();
        Ok(minimal_gates)
    }

    fn minimal_depth(&self) -> PyResult<Option<u32>> {
        let minimal_depth = self.repr.minimal_depth();
        Ok(minimal_depth)
    }

    fn frontier_size(&mut self) -> PyResult<u32> {
        let frontier = self.obtain_frontier_graph();

        Ok(frontier.edge_count() as u32)
    }

    fn max_frontier_size(&mut self) -> PyResult<u32> {
        let b_0 = self.repr.b_0();
        let b_1 = self.repr.b_1();

        Ok(b_0.min(b_1) as u32 * self.repr.n_vars() as u32)
    }

    fn table<'py>(
        &self,
        py: Python<'py>,
        vars: Vec<usize>,
    ) -> Bound<'py, PyArray<bool, Dim<[usize; 2]>>> {
        let m = vars.len();
        let n = self.repr.n_vars() - m;
        let table: Vec<_> = self.repr.table(&vars).into_iter().flatten().collect();
        let table = Array::from_shape_vec((1 << n, 1 << m), table).unwrap();
        let pyarray = table.to_pyarray(py);

        pyarray
    }

    fn npn_representant(&self) -> PyResult<Option<Self>> {
        let representant = self.repr.npn_representant();

        if let Some(repr) = representant {
            Ok(Some(FBool::new(
                (0..32).map(|i| (repr >> i) & 1 == 1).collect(),
            )))
        } else {
            Ok(None)
        }
    }

    fn max_sensitivity(&self) -> PyResult<u32> {
        Ok(self.repr.max_sensitivity())
    }

    fn mean_sensitivity(&self) -> PyResult<f32> {
        Ok(self.repr.mean_sensitivity())
    }

    fn spectral_entropy(&self) -> PyResult<f32> {
        Ok(self.repr.spectral_entropy())
    }

    fn degree(&self) -> PyResult<usize> {
        Ok(self.repr.degree())
    }

    fn no_linearity(&self) -> PyResult<u32> {
        Ok(self.repr.no_linearity())
    }

    fn influence(&self, var: usize) -> PyResult<f32> {
        Ok(self.repr.influence(var))
    }

    fn total_influence(&self) -> PyResult<f32> {
        Ok(self.repr.total_influence())
    }

    fn certificate_complexity(&self) -> PyResult<u32> {
        Ok(self.repr.certificate_complexity())
    }
}

#[pymodule(name = "fbool")]
fn fbool_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<FBool>()?;
    Ok(())
}
