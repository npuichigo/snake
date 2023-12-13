use pyo3::prelude::*;

mod dataset;

#[pymodule]
fn snakedata(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<dataset::Dataset>()?;
    Ok(())
}
