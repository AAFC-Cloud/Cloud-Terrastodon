use arbitrary::Arbitrary;
use compact_str::CompactString;

#[derive(Debug, PartialEq, Eq, Clone, Arbitrary, facet::Facet)]
#[repr(C)]
pub enum EntraUserExternalUserState {
    Accepted,
    PendingAcceptance,
    #[facet(other)]
    Other(CompactString),
}

cloud_terrastodon_registry::register_thing!(EntraUserExternalUserState);
cloud_terrastodon_registry::register_arbitrary!(EntraUserExternalUserState);

#[cfg(test)]
mod tests {
    use super::EntraUserExternalUserState;
    use compact_str::CompactString;

    #[test]
    fn supports_known_and_unknown_values() -> eyre::Result<()> {
        assert_eq!(
            facet_json::from_str::<EntraUserExternalUserState>("\"Accepted\"")?,
            EntraUserExternalUserState::Accepted
        );
        assert_eq!(
            facet_json::from_str::<EntraUserExternalUserState>("\"PendingAcceptance\"")?,
            EntraUserExternalUserState::PendingAcceptance
        );

        let other = facet_json::from_str::<EntraUserExternalUserState>("\"FutureState\"")?;
        assert_eq!(
            other,
            EntraUserExternalUserState::Other(CompactString::from("FutureState"))
        );
        assert_eq!(facet_json::to_string(&other)?, "\"FutureState\"");
        Ok(())
    }
}
