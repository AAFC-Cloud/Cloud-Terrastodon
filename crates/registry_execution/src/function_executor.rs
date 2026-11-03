use crate::InvocationFuture;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_registry::SyncMutFn;
use cloud_terrastodon_registry::SyncRefFn;
use cloud_terrastodon_registry::SyncValueFn;
use std::any::Any;

/// Legacy requests retain their owned future while their SDK surface migrates.
pub type AsyncValueFn = fn(Box<dyn Any + Send>) -> InvocationFuture<'static>;

/// An erased executable operation borrowing its caller's explicit resources.
pub type AsyncExecutionFn =
    for<'a> fn(Box<dyn Any + Send>, &'a CloudTerrastodonExecutor) -> InvocationFuture<'a>;

#[derive(Clone, Copy)]
pub enum FunctionExecutor {
    AsyncExecution(AsyncExecutionFn),
    AsyncValue(AsyncValueFn),
    SyncValue(SyncValueFn),
    SyncRef(SyncRefFn),
    SyncMut(SyncMutFn),
}
