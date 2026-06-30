use crate::interactive::pim_activate;
use crate::interactive::pim_activate_azurerm;
use crate::interactive::pim_activate_entra;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use eyre::Result;

/// Arguments for activating Privileged Identity Management roles.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzurePimActivateArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,

    #[facet(figue::subcommand)]
    pub target: Option<AzurePimActivateTarget>,
}

/// Target environments that can be activated via PIM.
#[derive(facet::Facet, Debug, Clone)]
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
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        match self.target {
            Some(AzurePimActivateTarget::AzureRm) => pim_activate_azurerm(tenant_id).await,
            Some(AzurePimActivateTarget::AzureAd) => pim_activate_entra(tenant_id).await,
            None => pim_activate(tenant_id).await,
        }
    }
}
