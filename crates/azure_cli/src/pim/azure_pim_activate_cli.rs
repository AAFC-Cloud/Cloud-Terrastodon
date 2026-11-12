use crate::pim_activate;
use crate::pim_activate_azurerm;
use crate::pim_activate_entra;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Arguments for activating Privileged Identity Management roles.
#[derive(Facet, Debug, Clone)]
pub struct AzurePimActivateArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,

    #[facet(figue::subcommand)]
    pub target: Option<AzurePimActivateTarget>,
}

/// Target environments that can be activated via PIM.
#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzurePimActivateTarget {
    /// Activate Azure Resource Manager roles.
    #[facet(rename = "azurerm", figue::alias = "az")]
    AzureRm,
    /// Activate Entra (formerly Azure AD) roles.
    #[facet(
        rename = "azuread",
        figue::alias = "entra",
        figue::alias = "aad",
        figue::alias = "ad"
    )]
    AzureAd,
}

impl AzurePimActivateArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        match self.target {
            Some(AzurePimActivateTarget::AzureRm) => {
                pim_activate_azurerm(authentication, executor).await
            }
            Some(AzurePimActivateTarget::AzureAd) => {
                pim_activate_entra(authentication, executor).await
            }
            None => pim_activate(authentication, executor).await,
        }?;
        Ok(CliOutput::none())
    }
}
