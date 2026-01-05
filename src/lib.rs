#[pyo3::pymodule]
mod testpkg {
    use pyo3::prelude::*;

    #[pyfunction]
    fn double(x: usize) -> usize {
        x * 2
    }
}
