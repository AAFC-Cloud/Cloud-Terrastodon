use eyre::Result;
use std::future::Future;
use std::pin::Pin;

/// An invocation-owned future; borrowed requests and outputs need not be static.
pub type ExecutionFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T>> + Send + 'a>>;
