use crate::AzureStorageAccountAuditInconclusiveReason;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure::AzureStorageAccount;
use cloud_terrastodon_azure::AzureStorageAccountBlobContainer;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_command::CancellationScope;
use facet::Facet;
use std::io::Write;
use std::sync::Arc;

/// An account-level anonymous blob access finding and its supporting evidence.
#[derive(Debug, Arbitrary, Facet)]
#[repr(u8)]
pub enum AzureStorageAccountAuditFinding {
    #[facet(rename = "StorageAccountUnusedAnonymousAccess")]
    UnusedAnonymousAccess {
        account: Arc<AzureStorageAccount>,
        containers: Vec<Arc<AzureStorageAccountBlobContainer>>,
    },
    #[facet(rename = "StorageAccountAnonymousAccessInconclusive")]
    Inconclusive {
        account: Arc<AzureStorageAccount>,
        reason: AzureStorageAccountAuditInconclusiveReason,
    },
}

impl AzureStorageAccountAuditFinding {
    pub(crate) const UNUSED_ANONYMOUS_ACCESS_MESSAGE: &str =
        "Storage account allows anonymous blob access but has no containers configured for it";
    pub(crate) const INCONCLUSIVE_MESSAGE: &str =
        "Storage account anonymous-access audit is inconclusive";

    pub fn account(&self) -> &AzureStorageAccount {
        match self {
            Self::UnusedAnonymousAccess { account, .. } | Self::Inconclusive { account, .. } => {
                account
            }
        }
    }

    pub fn message(&self) -> &'static str {
        match self {
            Self::UnusedAnonymousAccess { .. } => Self::UNUSED_ANONYMOUS_ACCESS_MESSAGE,
            Self::Inconclusive { .. } => Self::INCONCLUSIVE_MESSAGE,
        }
    }

    pub fn write_text(
        &self,
        writer: &mut dyn Write,
        cancellation: &CancellationScope,
    ) -> eyre::Result<()> {
        match self {
            Self::UnusedAnonymousAccess {
                account,
                containers,
            } => Self::write_unused_anonymous_access(account, containers, writer, cancellation),
            Self::Inconclusive { account, reason } => {
                Self::write_inconclusive(account, reason, writer)
            }
        }
    }

    pub(crate) fn write_unused_anonymous_access(
        account: &AzureStorageAccount,
        containers: &[Arc<AzureStorageAccountBlobContainer>],
        writer: &mut dyn Write,
        cancellation: &CancellationScope,
    ) -> eyre::Result<()> {
        writeln!(
            writer,
            "{}: {}",
            Self::UNUSED_ANONYMOUS_ACCESS_MESSAGE,
            account.id.expanded_form()
        )?;
        writeln!(
            writer,
            "  {} live containers; all private",
            containers.len()
        )?;
        for container in containers {
            cancellation.bail_if_cancelled()?;
            writeln!(
                writer,
                "    {}: {:?}",
                container.name, container.properties.public_access
            )?;
        }
        Self::write_account_details(account, writer)
    }

    pub(crate) fn write_inconclusive(
        account: &AzureStorageAccount,
        reason: &AzureStorageAccountAuditInconclusiveReason,
        writer: &mut dyn Write,
    ) -> eyre::Result<()> {
        writeln!(
            writer,
            "{}: {}",
            Self::INCONCLUSIVE_MESSAGE,
            account.id.expanded_form()
        )?;
        writeln!(writer, "  {reason}")?;
        Self::write_account_details(account, writer)
    }

    fn write_account_details(
        account: &AzureStorageAccount,
        writer: &mut dyn Write,
    ) -> eyre::Result<()> {
        writeln!(
            writer,
            "  allowBlobPublicAccess: {:?}",
            account.properties.allow_blob_public_access
        )?;
        writeln!(
            writer,
            "  publicNetworkAccess: {:?}",
            account.properties.public_network_access
        )?;
        writeln!(
            writer,
            "  networkAcls: {:?}",
            account.properties.network_acls
        )?;
        writeln!(writer, "  tags: {:?}", account.tags)?;
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountAuditFinding);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountAuditFinding);
