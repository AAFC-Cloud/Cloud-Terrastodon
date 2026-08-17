#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InvocationMode {
    /// Invoke from a reflected clone and retain the input root.
    Retain,
    /// Move the input root into the invocation and mark it Consumed.
    Consume,
}
