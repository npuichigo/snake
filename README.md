# Snake
Snake is a high-performance data loading framework built on asynchronous Rust 
and Python bindings. Like the segments of a snake in the classic game, we treat
data pipes as interconnected async streams, providing an intuitive and flexible way 
to build data pipelines.


<img src="images/snake.png" width="250" height="250">

## Idea
Let's say we want to apply Fibonacci calculation on slow IO.
In order to maximize performance, we should:
1. spawn async read in parallel to overlap the IO time window.
2. use an `tf.data.map` like mapping which supports parallel transformation.

Rust stream is a kind of async iterator which is a good abstraction of data flow.
After equipping it with parallel ability, we mimic slow IO with `par_then` and `async sleep`,
and perform parallel calculation with `par_map`
```rust
#[pymodule]
fn snake(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parallel_stream, m)?)?;
    Ok(())
}

fn tokio() -> &'static tokio::runtime::Runtime {
    use std::sync::OnceLock;
    static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RT.get_or_init(|| tokio::runtime::Runtime::new().unwrap())
}

fn fib(n: u64) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fib(n - 1) + fib(n - 2),
    }
}

async fn sleep(seconds: u64) {
    let sleep = async move { tokio::time::sleep(std::time::Duration::from_secs(seconds)).await };
    tokio().spawn(sleep).await.unwrap();
}

fn map() -> impl Stream<Item = PyResult<u64>> + Send {
    let _guard = tokio().enter();
    futures::stream::iter(0..100)
        .par_then(None, |i| async move {
            sleep(i % 3).await;
            i
        })
        .par_map(None, |i| move || Ok(fib(i)))
}

#[pyfunction]
fn parallel_stream() -> pyo3_async::asyncio::AsyncGenerator {
    pyo3_async::asyncio::AsyncGenerator::from_stream(map())
}

```
and export to a Python binding, we get
```
import snake
import asyncio

async def parallel_fib():
    async for i in snake.parallel_stream():
        print(i)

asyncio.run(parallel_fib())
```

Now we want to polish it to be a high performance data loading pipeline for deep learning, with native async + parallel (aka structured parallel) support.

## Dependency
[pyo3 async fn tracking issue](https://github.com/PyO3/pyo3/issues/1632)

## References
- [mlx-data](https://github.com/ml-explore/mlx-data)
