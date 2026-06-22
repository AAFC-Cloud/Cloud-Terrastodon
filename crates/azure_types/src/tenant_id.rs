use arbitrary::Arbitrary;
use std::ops::Deref;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Eq, PartialEq, Clone, Copy, Arbitrary, Hash, PartialOrd, Ord, facet::Facet)]
#[facet(json::proxy = String)]
pub struct AzureTenantId(Uuid);
crate::impl_facet_string_proxy!(AzureTenantId, value => value.to_string());

impl AzureTenantId {
    pub fn new(uuid: Uuid) -> Self {
        AzureTenantId(uuid)
    }
}
impl Deref for AzureTenantId {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for AzureTenantId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.hyphenated().fmt(f)
    }
}

impl FromStr for AzureTenantId {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(AzureTenantId(uuid::Uuid::parse_str(s)?))
    }
}

#[cfg(test)]
mod test {
    use super::AzureTenantId;

    #[test]
    fn json_roundtrips() -> eyre::Result<()> {
        crate::facet_json_equivalence::assert_json_roundtrip_equivalent::<AzureTenantId>(
            "\"00000000-0000-0000-0000-000000000000\"",
        )?;
        Ok(())
    }
}
