use futures::stream::{self, BoxStream, StreamExt};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3_async::asyncio::AsyncGenerator;
use pyo3_async::AllowThreads;
use std::cell::RefCell;

type SnakeStream = BoxStream<'static, PyResult<usize>>;

#[pyclass]
pub(crate) struct Dataset {
    stream: RefCell<Option<SnakeStream>>,
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
        self.and_then(|stream| {
            stream
                .map(move |x| {
                    Python::with_gil(|py| {
                        let y = f.call1(py, (x?,))?;
                        let y = y.extract::<usize>(py)?;
                        Ok(y)
                    })
                })
                .boxed()
        })
    }

    fn __aiter__(slf: PyRef<'_, Self>) -> PyResult<AsyncGenerator> {
        Ok(AsyncGenerator::from_stream(AllowThreads(
            slf.stream
                .borrow_mut()
                .take()
                .ok_or_else(|| PyRuntimeError::new_err("Stream can only be consumed once"))?,
        )))
    }
}

impl Dataset {
    fn and_then<F>(&self, func: F) -> PyResult<Self>
    where
        F: FnOnce(SnakeStream) -> SnakeStream,
    {
        let stream = func(
            self.stream
                .borrow_mut()
                .take()
                .ok_or_else(|| PyRuntimeError::new_err("Dataset can only be transformed once"))?,
        );
        Ok(Dataset {
            stream: RefCell::new(Some(stream)),
        })
    }
}
