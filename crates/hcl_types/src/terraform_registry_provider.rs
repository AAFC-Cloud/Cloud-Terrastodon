use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

use crate::version::SemVer;

#[derive(Debug, Serialize, Deserialize)]
pub struct TerraformProviderInfo {
    pub id: String,
    pub owner: String,
    pub namespace: String,
    pub name: String,
    pub alias: Option<Value>,
    pub version: String,
    pub tag: String,
    pub description: String,
    pub source: String,
    pub published_at: String,
    pub downloads: u64,
    pub tier: String,
    pub logo_url: String,
    pub versions: Vec<SemVer>,
    pub docs: Vec<Value>,
}

#[cfg(test)]
mod test {
    #[test]
    pub fn it_works_devops() -> eyre::Result<()> {
        let json = include_str!("../test_data/azuredevops.json");
        let provider =
            serde_json::from_str::<super::TerraformProviderInfo>(json)?;
        println!("Parsed Azure DevOps provider: {provider:#?}");
        Ok(())
    }
    #[test]
    pub fn it_works_azurerm() -> eyre::Result<()> {
        let json = include_str!("../test_data/azurerm.json");
        let provider =
            serde_json::from_str::<super::TerraformProviderInfo>(json)?;
        println!("Parsed AzureRM provider: {provider:#?}");
        Ok(())
    }
}
