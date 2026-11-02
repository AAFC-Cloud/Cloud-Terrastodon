use crate::InvocationFuture;
use cloud_terrastodon_registry::SyncMutFn;
use cloud_terrastodon_registry::SyncRefFn;
use cloud_terrastodon_registry::SyncValueFn;
use std::any::Any;

pub type AsyncValueFn = fn(Box<dyn Any + Send>) -> InvocationFuture;

#[derive(Clone, Copy)]
pub enum FunctionExecutor {
    AsyncValue(AsyncValueFn),
    SyncValue(SyncValueFn),
    SyncRef(SyncRefFn),
    SyncMut(SyncMutFn),
}
