use crate::FunctionExecutor;
use crate::FunctionInvocation;
use crate::RuntimeFunction;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_registry::ArbitraryRegistration;
use cloud_terrastodon_registry::FunctionDescriptor;
use cloud_terrastodon_registry::PureFunction;
use cloud_terrastodon_registry::PureFunctionExecutor;
use cloud_terrastodon_registry::describe_function;
use std::any::Any;
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;
use std::mem;
use std::ops::Deref;
use std::ptr;

/// A copyable handle to one authoritative operation registration.
///
/// Wrapping a pure registration requires no catalogue lookup, allocation or
/// copied metadata. Identity belongs to the source registration, so copying
/// this handle does not create another operation.
///
/// When invoked with runtime-borrowed input, operations must keep that input
/// and its shallow clones within the invocation future or returned value.
/// They must not let borrowed data escape into detached tasks, blocking jobs,
/// globals or independently retained state. Clone the underlying data into
/// owned storage before an escape; awaiting a spawned task is insufficient
/// because cancelling its parent can detach it. The registry's erased borrowed
/// pointer bridge cannot enforce those lifetime obligations.
#[derive(Clone, Copy)]
pub enum Function {
    Runtime(&'static RuntimeFunction),
    Pure(&'static PureFunction),
    Arbitrary(&'static ArbitraryRegistration),
}

impl Function {
    /// Borrow the source descriptor, preserving identity across catalogues.
    pub const fn descriptor(self) -> &'static FunctionDescriptor {
        match self {
            Self::Runtime(registration) => registration.descriptor(),
            Self::Pure(registration) => registration.descriptor(),
            Self::Arbitrary(registration) => registration.descriptor(),
        }
    }

    /// Inspect callback timing and receiver mode without invoking the operation.
    pub const fn executor(self) -> FunctionExecutor {
        match self {
            Self::Runtime(registration) => registration.executor(),
            Self::Pure(registration) => match registration.executor() {
                PureFunctionExecutor::SyncValue(invoke) => FunctionExecutor::SyncValue(invoke),
                PureFunctionExecutor::SyncRef(invoke) => FunctionExecutor::SyncRef(invoke),
                PureFunctionExecutor::SyncMut(invoke) => FunctionExecutor::SyncMut(invoke),
            },
            Self::Arbitrary(registration) => FunctionExecutor::SyncMut(registration.invocation()),
        }
    }

    #[track_caller]
    pub fn invoke_value_boxed<'a>(
        self,
        input: Box<dyn Any + Send>,
        executor: &'a CloudTerrastodonExecutor,
    ) -> eyre::Result<FunctionInvocation<'a>> {
        match self {
            Self::Runtime(registration) => registration.invoke_value_boxed(input, executor),
            Self::Pure(registration) => Ok(FunctionInvocation::Ready(
                registration.invoke_value_boxed(input)?,
            )),
            Self::Arbitrary(_) => Err(eyre::eyre!(
                "{} requires ByMut",
                describe_function(self.descriptor())
            )),
        }
    }

    pub fn invoke_ref_boxed(self, input: &(dyn Any + Send)) -> eyre::Result<Box<dyn Any + Send>> {
        match self {
            Self::Runtime(registration) => registration.invoke_ref_boxed(input),
            Self::Pure(registration) => registration.invoke_ref_boxed(input),
            Self::Arbitrary(_) => Err(eyre::eyre!(
                "{} requires ByMut",
                describe_function(self.descriptor())
            )),
        }
    }

    pub fn invoke_mut_boxed(
        self,
        input: &mut (dyn Any + Send),
    ) -> eyre::Result<Box<dyn Any + Send>> {
        match self {
            Self::Runtime(registration) => registration.invoke_mut_boxed(input),
            Self::Pure(registration) => registration.invoke_mut_boxed(input),
            Self::Arbitrary(registration) => (registration.invocation())(input),
        }
    }
}

impl Deref for Function {
    type Target = FunctionDescriptor;

    fn deref(&self) -> &Self::Target {
        self.descriptor()
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        match (*self, *other) {
            (Self::Runtime(left), Self::Runtime(right)) => ptr::eq(left, right),
            (Self::Pure(left), Self::Pure(right)) => ptr::eq(left, right),
            (Self::Arbitrary(left), Self::Arbitrary(right)) => ptr::eq(left, right),
            _ => false,
        }
    }
}

impl Eq for Function {}

impl Hash for Function {
    fn hash<H: Hasher>(&self, state: &mut H) {
        mem::discriminant(self).hash(state);
        match *self {
            Self::Runtime(registration) => ptr::from_ref(registration).hash(state),
            Self::Pure(registration) => ptr::from_ref(registration).hash(state),
            Self::Arbitrary(registration) => ptr::from_ref(registration).hash(state),
        }
    }
}

impl fmt::Debug for Function {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let variant = match self {
            Self::Runtime(_) => "Runtime",
            Self::Pure(_) => "Pure",
            Self::Arbitrary(_) => "Arbitrary",
        };
        formatter
            .debug_tuple(variant)
            .field(&describe_function(self.descriptor()))
            .field(&self.registration_site)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_cache::WorkCache;
    use cloud_terrastodon_execution::AnonymousAuthenticationProvider;
    use cloud_terrastodon_registry::FunctionKind;
    use cloud_terrastodon_registry::FunctionTraitContract;
    use cloud_terrastodon_registry::RegistrationSite;
    use cloud_terrastodon_registry::runtime_from_boxed;
    use facet::Facet;
    use std::sync::Arc;
    use teamy_cancellation::CancellationToken;

    fn increment(input: Box<dyn Any + Send>) -> eyre::Result<Box<dyn Any + Send>> {
        let input = input.downcast::<u16>().expect("fixture receives u16");
        Ok(Box::new(*input + 1))
    }

    // Deliberately absent from every distributed slice: wrapping an explicit
    // registration must not require catalogue membership or initialization.
    static UNLINKED: PureFunction = PureFunction::sync_value(
        u16::SHAPE,
        u16::SHAPE,
        FunctionKind::Conversion,
        "increment",
        FunctionTraitContract::Other("fixture"),
        increment,
        runtime_from_boxed::<u16>,
        RegistrationSite::new(file!(), line!()),
    );

    #[test]
    fn explicit_pure_handle_invokes_without_catalogue_membership() -> eyre::Result<()> {
        let function = Function::Pure(&UNLINKED);
        assert!(ptr::eq(function.descriptor(), UNLINKED.descriptor()));
        CancellationToken::new().with_scope(|scope| {
            let executor = CloudTerrastodonExecutor::new(
                WorkCache::temporary()?,
                Arc::new(AnonymousAuthenticationProvider),
                scope,
            );
            let invocation = function.invoke_value_boxed(Box::new(7_u16), &executor)?;
            let FunctionInvocation::Ready(output) = invocation else {
                panic!("a pure operation must remain synchronous");
            };
            assert_eq!(*output.downcast::<u16>().unwrap(), 8);
            Ok(())
        })
    }
}
