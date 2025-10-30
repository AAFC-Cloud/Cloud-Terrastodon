use clap::{Args, Subcommand};
use eyre::Result;

use crate::interactive::prelude::{pim_activate, pim_activate_azurerm, pim_activate_entra};

/// Arguments for activating Privileged Identity Management roles.
#[derive(Args, Debug, Clone)]
pub struct AzurePimActivateArgs {
    #[command(subcommand)]
    pub target: Option<AzurePimActivateTarget>,
}

/// Target environments that can be activated via PIM.
#[derive(Subcommand, Debug, Clone)]
pub enum AzurePimActivateTarget {
    /// Activate Azure Resource Manager roles.
    #[command(name = "azurerm", alias = "az")]
    AzureRm,
    /// Activate Entra (formerly Azure AD) roles.
    #[command(name = "azuread", aliases = ["entra", "aad", "ad"])]
    AzureAd,
}

impl AzurePimActivateArgs {
    pub async fn invoke(self) -> Result<()> {
        match self.target {
            Some(AzurePimActivateTarget::AzureRm) => pim_activate_azurerm().await,
            Some(AzurePimActivateTarget::AzureAd) => pim_activate_entra().await,
            None => pim_activate().await,
        }
    }
}
