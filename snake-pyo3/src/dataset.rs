use futures::stream::{self, BoxStream, StreamExt};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3_async::asyncio::AsyncGenerator;
use std::cell::RefCell;

#[pyclass]
pub(crate) struct Dataset {
    stream: RefCell<Option<BoxStream<'static, PyResult<usize>>>>,
}

#[pymethods]
impl Dataset {
    #[staticmethod]
    fn range(start: usize, end: usize) -> PyResult<Self> {
        let stream = stream::iter(start..end).map(|x| Ok(x)).boxed();
        Ok(Dataset {
            stream: RefCell::new(Some(stream)),
        })
    }

    fn map(&self, f: PyObject) -> PyResult<Self> {
        let stream = self
            .stream
            .borrow_mut()
            .take()
            .ok_or_else(|| PyRuntimeError::new_err("Dataset is already transformed before"))?
            .map(move |x| {
                Python::with_gil(|py| {
                    let y = f.call1(py, (x?,))?;
                    let y = y.extract::<usize>(py)?;
                    Ok(y)
                })
            });
        Ok(Dataset {
            stream: RefCell::new(Some(stream.boxed())),
        })
    }

    fn __aiter__(slf: PyRef<'_, Self>) -> PyResult<AsyncGenerator> {
        let stream = slf
            .stream
            .borrow_mut()
            .take()
            .ok_or_else(|| PyRuntimeError::new_err("Stream can only be consumed once"))?;
        Ok(AsyncGenerator::from_stream(stream))
    }
}
