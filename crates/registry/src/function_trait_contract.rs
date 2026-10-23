use facet::Facet;
use std::fmt;

/// The trait contract that exposes an operation to the registry.
///
/// Unlike an operation's display label, this vocabulary may guide discovery
/// policy. Custom contracts remain distinct even when their text matches a built-in
/// contract. Static text keeps registrations constant and their metadata copyable.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Facet)]
#[repr(u8)]
pub enum FunctionTraitContract {
    /// An executable request registered through its `IntoFuture` implementation.
    IntoFuture,
    /// An infallible conversion registered through `From`.
    From,
    /// A fallible conversion registered through `TryFrom`.
    TryFrom,
    /// An offline value generator registered through `Arbitrary`.
    Arbitrary,
    /// An explicitly named contract supplied by a custom registration.
    Other(&'static str),
}

impl FunctionTraitContract {
    /// Text for presentation and search; classify contracts using enum variants.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::IntoFuture => "IntoFuture",
            Self::From => "From",
            Self::TryFrom => "TryFrom",
            Self::Arbitrary => "Arbitrary",
            Self::Other(name) => name,
        }
    }
}

impl fmt::Display for FunctionTraitContract {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
