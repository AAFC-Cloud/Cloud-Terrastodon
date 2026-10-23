use crate::pim_activate;
use crate::pim_activate_azurerm;
use crate::pim_activate_entra;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_credentials::AuthContext;
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
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        let auth_context = self
            .tenant
            .bind_auth_context(auth_context, cancellation)
            .await?;
        match self.target {
            Some(AzurePimActivateTarget::AzureRm) => {
                pim_activate_azurerm(&auth_context, cancellation).await
            }
            Some(AzurePimActivateTarget::AzureAd) => {
                pim_activate_entra(auth_context.tenant_id, cancellation).await
            }
            None => pim_activate(&auth_context, cancellation).await,
        }
    }
}
