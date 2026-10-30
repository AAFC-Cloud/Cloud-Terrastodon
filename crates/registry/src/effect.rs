use arbitrary::Arbitrary;
use facet::Facet;

/// Observable effects declared by a registered operation.
///
/// These describe service, filesystem, and application state, plus interaction
/// with people. Ordinary query caching and diagnostic artifacts do not make a
/// read a domain mutation.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Facet, Arbitrary)]
#[facet(rename_all = "kebab-case")]
#[repr(u8)]
pub enum Effect {
    /// Observe state without mutating its domain data.
    Read,
    /// Mutate domain state, configuration, or an explicitly requested output.
    /// Workflows that also query state can declare both effects.
    Write,
    /// Ask a person for input, selection, confirmation, or another response.
    /// Unattended discovery must exclude operations declaring this effect.
    Solicit,
}
