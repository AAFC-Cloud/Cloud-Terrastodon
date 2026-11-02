use std::any::Any;

pub type SyncValueFn = fn(Box<dyn Any + Send>) -> eyre::Result<Box<dyn Any + Send>>;
pub type SyncRefFn = fn(&(dyn Any + Send)) -> eyre::Result<Box<dyn Any + Send>>;
pub type SyncMutFn = fn(&mut (dyn Any + Send)) -> eyre::Result<Box<dyn Any + Send>>;

/// Synchronous invocation contracts that receive no execution capabilities.
#[derive(Clone, Copy)]
pub enum PureFunctionExecutor {
    SyncValue(SyncValueFn),
    SyncRef(SyncRefFn),
    SyncMut(SyncMutFn),
}
