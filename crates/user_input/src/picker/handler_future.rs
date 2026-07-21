use std::future::Future;
use std::pin::Pin;

pub(super) type HandlerFuture<'a> = Pin<Box<dyn Future<Output = eyre::Result<()>> + Send + 'a>>;
