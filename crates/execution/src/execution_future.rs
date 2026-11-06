use eyre::Report;
use std::future::Future;
use std::pin::Pin;

/// An invocation-owned future; borrowed requests and outputs need not be static.
/// Requests can retain their domain error; infrastructure defaults to an eyre report.
pub type ExecutionFuture<'a, T, E = Report> =
    Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'a>>;
