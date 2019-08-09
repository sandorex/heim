use std::fmt::Debug;
use std::sync::Mutex;

use futures::executor::ThreadPool;
use futures::channel::oneshot;
use futures::future::{self, Future};
use futures::task::Spawn;

use heim_common::prelude::*;

lazy_static::lazy_static! {
    pub(crate) static ref THREAD_POOL: FuturePool = FuturePool::new();
}

#[derive(Debug)]
pub struct FuturePool(Mutex<ThreadPool>);

impl FuturePool {
    pub fn new() -> FuturePool {
        let inner = ThreadPool::builder()
            .name_prefix("heim-")
            .create()
            .expect("Misconfigured thread pool");

        FuturePool(Mutex::new(inner))
    }

    pub fn spawn<F, T>(&self, f: F) -> impl Future<Output = T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + Debug + 'static,
    {

        let (tx, rx) = oneshot::channel();
        let fut: future::FutureObj<()> = future::lazy(|_| {
                let _ = tx.send(f());
            })
            .boxed()
            .into();

        {
            let mut pool = self.0.lock().expect("Futures pool mutex is poisoned");
            let _ = pool.spawn_obj(fut);
        }

        rx.map(|res| res.expect("Runtime future was canceled"))
    }
}
