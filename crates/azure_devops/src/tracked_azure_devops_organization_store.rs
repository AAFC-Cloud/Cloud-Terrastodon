use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::TrackedAzureDevOpsOrganization;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_pathing::AppDir;
use eyre::Result;
use eyre::WrapErr;
use std::fs::File;
use std::fs::OpenOptions;
use std::fs::TryLockError;
use std::io::ErrorKind;
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::NamedTempFile;

/// Owns the device's saved Azure DevOps organization URLs and tenant associations.
#[derive(Debug)]
pub struct TrackedAzureDevOpsOrganizationStore {
    directory: PathBuf,
}

impl TrackedAzureDevOpsOrganizationStore {
    /// Use Cloud Terrastodon's local configuration directory.
    pub fn configured() -> Self {
        Self::new(AppDir::Config.join("azure-devops"))
    }

    /// Use an explicit storage directory for an embedded application or test.
    pub fn new(directory: PathBuf) -> Self {
        Self { directory }
    }

    /// Read the local catalogue in URL order. An absent catalogue is empty.
    pub async fn list(&self) -> Result<Vec<TrackedAzureDevOpsOrganization>> {
        let path = self.directory.join("organizations.json");
        let bytes = match tokio::fs::read(&path).await {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => {
                return Err(error).wrap_err_with(|| {
                    format!("Reading tracked organizations from {}", path.display())
                });
            }
        };
        let mut organizations: Vec<TrackedAzureDevOpsOrganization> = facet_json::from_slice(&bytes)
            .wrap_err_with(|| format!("Decoding tracked organizations from {}", path.display()))?;
        organizations.sort_by_cached_key(|org| org.url.expanded_form());
        Ok(organizations)
    }

    /// List the tracked organizations explicitly associated with the tenant.
    pub async fn list_for_tenant(
        &self,
        tenant_id: AzureTenantId,
    ) -> Result<Vec<TrackedAzureDevOpsOrganization>> {
        Ok(self
            .list()
            .await?
            .into_iter()
            .filter(|organization| organization.tenant_id == Some(tenant_id))
            .collect())
    }

    /// Find a saved organization by its parsed URL.
    pub async fn show(
        &self,
        url: &AzureDevOpsOrganizationUrl,
    ) -> Result<Option<TrackedAzureDevOpsOrganization>> {
        Ok(self.list().await?.into_iter().find(|org| org.url == *url))
    }

    /// Track an organization or replace its saved tenant association.
    /// An absent tenant clears any existing association.
    pub async fn add(
        &self,
        organization: TrackedAzureDevOpsOrganization,
        cancellation: &CancellationScope,
    ) -> Result<TrackedAzureDevOpsOrganization> {
        let _lease = self.lock(cancellation).await?;
        let mut organizations = self.list().await?;
        if let Some(existing) = organizations
            .iter_mut()
            .find(|org| org.url == organization.url)
        {
            existing.tenant_id = organization.tenant_id;
        } else {
            organizations.push(organization.clone());
        }
        cancellation.with_deferral("saving tracked Azure DevOps organizations", |_| {
            self.save(&organizations)
        })?;
        Ok(organization)
    }

    /// Remove the saved organization by URL, returning its previous registration.
    pub async fn remove(
        &self,
        url: &AzureDevOpsOrganizationUrl,
        cancellation: &CancellationScope,
    ) -> Result<Option<TrackedAzureDevOpsOrganization>> {
        let _lease = self.lock(cancellation).await?;
        let mut organizations = self.list().await?;
        let Some(index) = organizations.iter().position(|org| org.url == *url) else {
            return Ok(None);
        };
        let organization = organizations.remove(index);
        cancellation.with_deferral("saving tracked Azure DevOps organizations", |_| {
            self.save(&organizations)
        })?;
        Ok(Some(organization))
    }

    async fn lock(&self, cancellation: &CancellationScope) -> Result<File> {
        tokio::fs::create_dir_all(&self.directory)
            .await
            .wrap_err_with(|| format!("Creating {}", self.directory.display()))?;
        let path = self.directory.join("organizations.lock");
        let lease = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)
            .wrap_err_with(|| {
                format!("Opening the organization catalogue lock {}", path.display())
            })?;
        // Keep the same lock file across atomic replacements of the catalogue.
        loop {
            match lease.try_lock() {
                Ok(()) => return Ok(lease),
                Err(TryLockError::WouldBlock) => {
                    cancellation
                        .cancellable_scope(|_| async {
                            tokio::time::sleep(Duration::from_millis(25)).await;
                            eyre::Ok(())
                        })
                        .await?;
                }
                Err(TryLockError::Error(error)) => {
                    return Err(error).wrap_err("Acquiring the organization catalogue lock");
                }
            }
        }
    }

    fn save(&self, organizations: &[TrackedAzureDevOpsOrganization]) -> Result<()> {
        let publish = || {
            let mut temporary = NamedTempFile::new_in(&self.directory)?;
            temporary.write_all(&facet_json::to_vec(organizations)?)?;
            temporary.as_file().sync_all()?;
            temporary
                .persist(self.directory.join("organizations.json"))
                .map_err(|error| error.error)?;
            eyre::Ok(())
        };
        publish().wrap_err_with(|| {
            format!(
                "Saving tracked organizations in {}",
                self.directory.display()
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_command::CancellationToken;

    #[tokio::test]
    async fn tracked_azure_devops_organizations_persist_by_url() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let store = TrackedAzureDevOpsOrganizationStore::new(directory.path().to_path_buf());
        let cloud = TrackedAzureDevOpsOrganization {
            url: "example-org".parse()?,
            tenant_id: Some("ebf6ea7a-dd99-4a15-8a55-05ba333f1937".parse()?),
        };
        let cloud_url = "https://dev.azure.com/example-org/".parse()?;
        let server = TrackedAzureDevOpsOrganization {
            url: "https://ado.example.test/tfs/Collection".parse()?,
            tenant_id: None,
        };

        assert!(store.list().await?.is_empty());
        CancellationToken::new()
            .with_async_scope(|scope| async move {
                store.add(cloud.clone(), &scope).await?;
                store.add(server.clone(), &scope).await?;

                let reopened =
                    TrackedAzureDevOpsOrganizationStore::new(directory.path().to_path_buf());
                assert_eq!(reopened.list().await?, [server.clone(), cloud.clone()]);
                assert_eq!(reopened.show(&cloud_url).await?, Some(cloud.clone()));

                let without_tenant = TrackedAzureDevOpsOrganization {
                    url: cloud_url.clone(),
                    tenant_id: None,
                };
                reopened.add(without_tenant.clone(), &scope).await?;
                assert_eq!(
                    store.remove(&cloud_url, &scope).await?,
                    Some(without_tenant)
                );
                assert_eq!(store.remove(&cloud_url, &scope).await?, None);
                assert_eq!(store.list().await?, [server]);
                eyre::Ok(())
            })
            .await
    }

    #[tokio::test]
    async fn tracked_azure_devops_organization_writers_share_a_lock() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let first = TrackedAzureDevOpsOrganization {
            url: "first-org".parse()?,
            tenant_id: None,
        };
        let second = TrackedAzureDevOpsOrganization {
            url: "second-org".parse()?,
            tenant_id: None,
        };
        let store = TrackedAzureDevOpsOrganizationStore::new(directory.path().to_path_buf());
        let other = TrackedAzureDevOpsOrganizationStore::new(directory.path().to_path_buf());
        CancellationToken::new()
            .with_async_scope(|scope| async move {
                cloud_terrastodon_execution::join2(
                    &scope,
                    async |child| store.add(first.clone(), &child).await,
                    async |child| other.add(second.clone(), &child).await,
                )
                .await?;
                assert_eq!(store.list().await?, [first, second]);
                eyre::Ok(())
            })
            .await
    }
}
