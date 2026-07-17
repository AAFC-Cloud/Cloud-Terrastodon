use arbitrary::Arbitrary;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Arbitrary, facet::Facet)]
#[facet(json::proxy = String)]
pub struct EntraServicePrincipalObjectId(uuid::Uuid);

crate::impl_uuid_newtype!(EntraServicePrincipalObjectId);

cloud_terrastodon_registry::register_thing!(EntraServicePrincipalObjectId);
cloud_terrastodon_registry::register_arbitrary!(EntraServicePrincipalObjectId);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn json_roundtrips() -> eyre::Result<()> {
        let id = EntraServicePrincipalObjectId::new(uuid::Uuid::nil());
        crate::facet_json_equivalence::assert_json_serialize_equivalent(&id)?;
        crate::facet_json_equivalence::assert_json_roundtrip_equivalent::<EntraServicePrincipalObjectId>(
            "\"00000000-0000-0000-0000-000000000000\"",
        )?;
        Ok(())
    }
}
