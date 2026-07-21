use super::handler_completion::HandlerCompletion;
use std::future::Future;
use std::pin::Pin;

pub(super) type HandlerTask<'a> = Pin<Box<dyn Future<Output = HandlerCompletion> + Send + 'a>>;
