use compact_str::CompactString;
use std::fmt::Display;
use std::ops::Deref;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash, facet::Facet)]
#[facet(json::proxy = String)]
pub struct ComputeSkuName(CompactString);
crate::impl_facet_string_proxy!(ComputeSkuName, value => value.to_string());
impl ComputeSkuName {
    pub fn try_new(value: impl Into<CompactString>) -> eyre::Result<Self> {
        let value = value.into();
        if value.is_empty() {
            Err(eyre::eyre!("Compute SKU name cannot be empty"))
        } else {
            Ok(Self(value))
        }
    }
}

impl Display for ComputeSkuName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl Deref for ComputeSkuName {
    type Target = CompactString;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl FromStr for ComputeSkuName {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ComputeSkuName::try_new(s)
    }
}
impl TryFrom<CompactString> for ComputeSkuName {
    type Error = eyre::Error;

    fn try_from(value: CompactString) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}
impl From<ComputeSkuName> for CompactString {
    fn from(value: ComputeSkuName) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use super::ComputeSkuName;

    #[test]
    fn json_round_trips_through_facet() -> eyre::Result<()> {
        let name = facet_json::from_str::<ComputeSkuName>("\"Standard_D2s_v5\"")?;
        assert_eq!(name.as_str(), "Standard_D2s_v5");
        let reparsed = facet_json::from_str::<ComputeSkuName>(&facet_json::to_string(&name)?)?;
        assert_eq!(name, reparsed);
        Ok(())
    }
}
