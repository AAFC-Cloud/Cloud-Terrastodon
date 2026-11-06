#![allow(
    clippy::disallowed_types,
    reason = "Provider metadata needs raw JSON without an azure_types dependency cycle; the module scope includes generated Facet codecs."
)]

use crate::version::SemVer;
use facet::Facet;
use facet_json::RawJson;

#[derive(Debug, Facet)]
pub struct TerraformProviderInfo {
    pub id: String,
    pub owner: String,
    pub namespace: String,
    pub name: String,
    pub alias: Option<RawJson<'static>>,
    pub version: String,
    pub tag: String,
    pub description: String,
    pub source: String,
    pub published_at: String,
    pub downloads: u64,
    pub tier: String,
    pub logo_url: String,
    pub versions: Vec<SemVer>,
    pub docs: Vec<RawJson<'static>>,
}

#[cfg(test)]
mod test {
    #[test]
    pub fn it_works_devops() -> eyre::Result<()> {
        let json = include_str!("../test_data/azuredevops.json");
        let provider = facet_json::from_str::<super::TerraformProviderInfo>(json)?;
        println!("Parsed Azure DevOps provider: {provider:#?}");
        Ok(())
    }
    #[test]
    pub fn it_works_azurerm() -> eyre::Result<()> {
        let json = include_str!("../test_data/azurerm.json");
        let provider = facet_json::from_str::<super::TerraformProviderInfo>(json)?;
        println!("Parsed AzureRM provider: {provider:#?}");
        Ok(())
    }
}
