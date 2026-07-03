use arbitrary::Arbitrary;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Arbitrary, facet::Facet)]
#[facet(json::proxy = String)]
pub struct EntraServicePrincipalId(uuid::Uuid);

crate::impl_uuid_newtype!(EntraServicePrincipalId);

cloud_terrastodon_registry::register_thing!(EntraServicePrincipalId);

#[cfg(test)]
mod service_principal_id_facet_json_roundtrip {
    use super::*;

    #[test]
    fn json_roundtrips() -> eyre::Result<()> {
        let id = EntraServicePrincipalId::new(uuid::Uuid::nil());
        crate::facet_json_equivalence::assert_json_serialize_equivalent(&id)?;
        crate::facet_json_equivalence::assert_json_roundtrip_equivalent::<EntraServicePrincipalId>(
            "\"00000000-0000-0000-0000-000000000000\"",
        )?;
        Ok(())
    }
}

