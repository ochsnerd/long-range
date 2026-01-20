mod norms;
mod union_find;
mod percolation;

#[pyo3::pymodule]
mod testpkg {
    use pyo3::prelude::*;

    #[pyclass]
    struct Observables {
        #[pyo3(get)]
        average_size: f64,
        #[pyo3(get)]
        size_spread: f64,
    }

    impl Observables {
        fn from(o: crate::percolation::Observables) -> Self {
            Observables {
                average_size: o.average_size,
                size_spread: o.size_spread,
            }
        }
    }

    #[pyfunction]
    fn simulate(l: usize, alpha: f64, beta: f64, n_samples: u64, seed: u64) -> Vec<Observables> {
        crate::percolation::simulate(crate::norms::Norm::L1, l, alpha, beta, n_samples, seed)
            .into_iter()
            .map(Observables::from)
            .collect()
    }
}
