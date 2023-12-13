use futures::stream::{self, BoxStream, StreamExt};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3_async::asyncio::AsyncGenerator;
use std::cell::RefCell;

#[pyclass]
pub(crate) struct Dataset {
    stream: RefCell<Option<BoxStream<'static, usize>>>,
}

#[pymethods]
impl Dataset {
    #[staticmethod]
    fn range(start: usize, end: usize) -> PyResult<Self> {
        Ok(Dataset {
            stream: RefCell::new(Some(stream::iter(start..end).boxed())),
        })
    }

    fn __aiter__(slf: PyRef<'_, Self>) -> PyResult<AsyncGenerator> {
        let stream = slf
            .stream
            .borrow_mut()
            .take()
            .ok_or_else(|| PyRuntimeError::new_err("Stream can only be consumed once"))?
            .map(|x| PyResult::Ok(x));
        Ok(AsyncGenerator::from_stream(stream))
    }
}
