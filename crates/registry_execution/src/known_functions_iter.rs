use crate::Function;
use crate::KNOWN_RUNTIME_FUNCTIONS;
use crate::RuntimeFunction;
use cloud_terrastodon_registry::ArbitraryRegistration;
use cloud_terrastodon_registry::KNOWN_ARBITRARY_VALUES;
use cloud_terrastodon_registry::KNOWN_PURE_FUNCTIONS;
use cloud_terrastodon_registry::PureFunction;
use std::iter::FusedIterator;
use std::slice::Iter;

/// Traverses the original registration slices as copyable operation handles.
#[derive(Clone)]
pub struct KnownFunctionsIter {
    runtime: Iter<'static, RuntimeFunction>,
    arbitrary: Iter<'static, ArbitraryRegistration>,
    pure: Iter<'static, PureFunction>,
}

impl KnownFunctionsIter {
    pub(crate) fn new() -> Self {
        Self {
            runtime: KNOWN_RUNTIME_FUNCTIONS.iter(),
            arbitrary: KNOWN_ARBITRARY_VALUES.iter(),
            pure: KNOWN_PURE_FUNCTIONS.iter(),
        }
    }
}

impl Iterator for KnownFunctionsIter {
    type Item = Function;

    fn next(&mut self) -> Option<Self::Item> {
        self.runtime
            .next()
            .map(Function::Runtime)
            .or_else(|| self.arbitrary.next().map(Function::Arbitrary))
            .or_else(|| self.pure.next().map(Function::Pure))
    }
}

impl DoubleEndedIterator for KnownFunctionsIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.pure
            .next_back()
            .map(Function::Pure)
            .or_else(|| self.arbitrary.next_back().map(Function::Arbitrary))
            .or_else(|| self.runtime.next_back().map(Function::Runtime))
    }
}

impl FusedIterator for KnownFunctionsIter {}
