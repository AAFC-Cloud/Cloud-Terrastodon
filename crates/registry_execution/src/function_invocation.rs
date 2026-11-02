use facet::Facet;
use std::any::Any;
use std::any::type_name;
use std::future::Future;
use std::future::IntoFuture;
use std::pin::Pin;

pub type InvocationFuture = Pin<Box<dyn Future<Output = eyre::Result<Box<dyn Any + Send>>> + Send>>;

pub enum FunctionInvocation {
    Pending(InvocationFuture),
    Ready(Box<dyn Any + Send>),
}

pub fn invoke_result_future<T, O, E>(input: Box<dyn Any + Send>) -> InvocationFuture
where
    T: IntoFuture<Output = Result<O, E>> + Any + Send + 'static,
    T::IntoFuture: Send + 'static,
    E: Into<eyre::Error> + Send + 'static,
    O: Facet<'static> + Any + Send + 'static,
{
    Box::pin(async move {
        let request = input.downcast::<T>().map_err(|_| {
            eyre::eyre!(
                "invocation expected input type {}, but received a different boxed value",
                type_name::<T>()
            )
        })?;
        let output = (*request).into_future().await.map_err(Into::into)?;
        Ok(Box::new(output) as Box<dyn Any + Send>)
    })
}
