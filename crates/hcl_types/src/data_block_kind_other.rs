use crate::prelude::ProviderKind;
use eyre::bail;
use std::any::type_name;
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct OtherDataBlockKind {
    pub provider: ProviderKind,
    pub resource: String,
}
impl FromStr for OtherDataBlockKind {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((provider, resource)) = s.split_once('_') else {
            bail!(
                "Expected at least one underscore in {s:?} to parse as a {:?}",
                type_name::<OtherDataBlockKind>()
            )
        };
        Ok(OtherDataBlockKind {
            provider: provider.parse()?,
            resource: resource.to_owned(),
        })
    }
}
impl std::fmt::Display for OtherDataBlockKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", &self.provider))?;
        f.write_str("_")?;
        f.write_str(&self.resource)
    }
}

#[cfg(test)]
mod tests {
    use crate::resource_block_kind_azuread::AzureAdResourceBlockKind;
    use crate::resource_block_resource_kind::ResourceBlockResourceKind;

    #[test]
    fn parse_azuread_other() -> eyre::Result<()> {
        let kind: ResourceBlockResourceKind = "azuread_thingy".parse()?;
        assert_eq!(
            kind,
            ResourceBlockResourceKind::AzureAD(AzureAdResourceBlockKind::Other(
                "thingy".to_owned()
            ))
        );
        Ok(())
    }
}
