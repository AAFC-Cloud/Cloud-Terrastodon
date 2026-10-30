use crate::InvocationFuture;
use std::any::Any;

pub type AsyncValueFn = fn(Box<dyn Any + Send>) -> InvocationFuture;
pub type SyncValueFn = fn(Box<dyn Any + Send>) -> eyre::Result<Box<dyn Any + Send>>;
pub type SyncRefFn = fn(&(dyn Any + Send)) -> eyre::Result<Box<dyn Any + Send>>;
pub type SyncMutFn = fn(&mut (dyn Any + Send)) -> eyre::Result<Box<dyn Any + Send>>;

#[derive(Clone, Copy)]
pub enum FunctionExecutor {
    AsyncValue(AsyncValueFn),
    SyncValue(SyncValueFn),
    SyncRef(SyncRefFn),
    SyncMut(SyncMutFn),
}
