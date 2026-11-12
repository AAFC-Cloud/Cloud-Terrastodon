use crate::auth_logout_error::AuthLogoutError;
use crate::azure_cli_account_identity::AzureCliAccountIdentity;
use crate::azure_cli_session::AzureCliSession;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_pathing::AppDir;
use eyre::Result;
use eyre::WrapErr;
use eyre::ensure;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use uuid::Uuid;

/// Persistent login generations and the tenant's selected generation.
///
/// Identity-bearing files are private authentication state, never work artifacts.
/// Publishing a fresh generation does not change an existing provider's binding.
#[derive(Clone)]
pub(crate) struct AzureCliSessionStore {
    root: PathBuf,
}

impl AzureCliSessionStore {
    pub(crate) fn at(root: PathBuf) -> Self {
        Self { root }
    }

    /// Explicitly adopt one application-owned legacy tenant session. This never
    /// reads the Azure CLI ambient configuration directory.
    pub(crate) async fn import_legacy(
        &self,
        tenant: AzureTenantId,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        let legacy = Self::configured().load(tenant, cancellation).await?;
        let _legacy_lease = legacy.lease(cancellation).await?;
        let generation = Uuid::new_v4();
        let destination = self.root.join("generations").join(generation.to_string());
        Self::prepare_directory(&destination)?;
        let mut pending = vec![(legacy.directory.clone(), destination)];
        while let Some((source, target)) = pending.pop() {
            cancellation.bail_if_cancelled()?;
            for child in std::fs::read_dir(source)? {
                let child = child?;
                let kind = child.file_type()?;
                ensure!(
                    !kind.is_symlink(),
                    "legacy credential storage contains a symbolic link; cannot import it"
                );
                if child.file_name() == "session.lock" {
                    continue;
                }
                let output = target.join(child.file_name());
                if kind.is_dir() {
                    Self::prepare_directory(&output)?;
                    pending.push((child.path(), output));
                } else if kind.is_file() {
                    std::fs::copy(child.path(), output)?;
                }
            }
        }
        cancellation.with_deferral("publishing imported Azure CLI credentials", |_| {
            Self::prepare_directory(&self.root.join("tenants"))?;
            std::fs::write(
                self.root.join("tenants").join(tenant.to_string()),
                generation.to_string(),
            )?;
            eyre::Ok(())
        })
    }
    pub(crate) fn configured() -> Self {
        Self {
            root: AppDir::Config.join("azure-cli-sessions"),
        }
    }

    fn prepare_directory(path: &Path) -> Result<()> {
        std::fs::create_dir_all(path)
            .wrap_err("creating the isolated Azure CLI session directory")?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))
                .wrap_err("restricting the isolated Azure CLI session directory")?;
        }
        Ok(())
    }

    pub(crate) async fn load(
        &self,
        tenant: AzureTenantId,
        cancellation: &CancellationScope,
    ) -> Result<AzureCliSession> {
        ensure!(
            self.root.try_exists()?,
            "no Cloud Terrastodon Azure CLI session exists; run ct account login for the registered Azure CLI account"
        );
        let _lease = AzureCliSession::lock(&self.root.join("store.lock"), cancellation).await?;
        let generation = std::fs::read_to_string(
            self.root.join("tenants").join(tenant.to_string()),
        )
        .wrap_err(
            "no isolated Azure CLI session is selected for this tenant; explicitly log in first",
        )?;
        // Parse before using stored text as a path component.
        let generation = Uuid::parse_str(generation.trim())
            .wrap_err("invalid Azure CLI session generation identifier")?;
        let directory = self.root.join("generations").join(generation.to_string());
        let bytes = std::fs::read(directory.join("identity.json"))
            .wrap_err("reading the bound Azure CLI session identity")?;
        let identity: AzureCliAccountIdentity = AzureCliSession::decode(&bytes)?;
        identity.validate(tenant)?;
        Ok(AzureCliSession::new(directory, identity))
    }

    pub(crate) async fn login(
        &self,
        tenant: AzureTenantId,
        cancellation: &CancellationScope,
    ) -> Result<AzureCliSession> {
        Self::prepare_directory(&self.root)?;
        // Login/publish and logout serialize; existing generations can continue
        // their own work without consulting this selector.
        let _store_lease =
            AzureCliSession::lock(&self.root.join("store.lock"), cancellation).await?;
        let generation = Uuid::new_v4();
        let directory = self.root.join("generations").join(generation.to_string());
        Self::prepare_directory(&directory)?;
        let _generation_lease =
            AzureCliSession::lock(&directory.join("session.lock"), cancellation).await?;
        AzureCliSession::credential_command(
            &directory,
            &[
                "login",
                "--tenant",
                &tenant.to_string(),
                "--allow-no-subscriptions",
                "--output",
                "none",
            ],
            cancellation,
        )
        .await?;
        let identity = AzureCliSession::read_identity(&directory, tenant, cancellation).await?;
        // A fresh directory contains only this login. Its recorded account is
        // checked again before and after token subprocesses; login and
        // account-changing commands cannot execute through a bound session.
        let serialized = facet_json::to_vec(&identity).map_err(|_| {
            eyre::eyre!("serializing the bound Azure CLI identity failed; details suppressed")
        })?;
        cancellation
            .deferred("publishing the explicit Azure CLI login", |_| async {
                std::fs::write(directory.join("identity.json"), serialized)
                    .wrap_err("recording the isolated Azure CLI identity")?;
                Self::prepare_directory(&self.root.join("tenants"))?;
                let mut selected = tempfile::NamedTempFile::new_in(self.root.join("tenants"))
                    .wrap_err("creating the Azure CLI session selector")?;
                selected
                    .write_all(generation.to_string().as_bytes())
                    .wrap_err("writing the Azure CLI session selector")?;
                selected
                    .as_file()
                    .sync_all()
                    .wrap_err("flushing the Azure CLI session selector")?;
                selected
                    .persist(self.root.join("tenants").join(tenant.to_string()))
                    .map_err(|error| error.error)
                    .wrap_err("publishing the Azure CLI session selector")?;
                eyre::Ok(())
            })
            .await?;
        Ok(AzureCliSession::new(directory, identity))
    }

    /// Revoke each generation under its lease, then ask CLI to clear its tokens.
    /// Keep revoked directories so existing providers fail closed on next use.
    pub(crate) async fn logout_all(&self, cancellation: &CancellationScope) -> Result<()> {
        if !self.root.try_exists()? {
            return Ok(());
        }
        let _store_lease =
            AzureCliSession::lock(&self.root.join("store.lock"), cancellation).await?;
        let generations = self.root.join("generations");
        if !generations.try_exists()? {
            return Ok(());
        }
        let mut failures = Vec::new();
        for entry in
            std::fs::read_dir(generations).wrap_err("enumerating isolated Azure CLI sessions")?
        {
            // A damaged generation must not prevent revoking the remaining
            // sessions. Retain each source error and finish the enumeration.
            let result: Result<()> = async {
                let entry = entry.wrap_err("reading an isolated Azure CLI session entry")?;
                if !entry.file_type()?.is_dir()
                    || Uuid::parse_str(&entry.file_name().to_string_lossy()).is_err()
                {
                    return Ok(());
                }
                let _lease =
                    AzureCliSession::lock(&entry.path().join("session.lock"), cancellation).await?;
                std::fs::write(entry.path().join("revoked"), b"explicit logout")
                    .wrap_err("revoking an isolated Azure CLI session")?;
                AzureCliSession::credential_command(
                    &entry.path(),
                    &["account", "clear", "--output", "none"],
                    cancellation,
                )
                .await?;
                Ok(())
            }
            .await;
            if let Err(error) = result {
                failures.push(error);
            }
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(AuthLogoutError { errors: failures }.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_command::CancellationToken;

    #[tokio::test]
    async fn generation_selection_does_not_replace_an_existing_binding() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let store = AzureCliSessionStore {
            root: directory.path().to_path_buf(),
        };
        let tenant = AzureTenantId::new(Uuid::from_u128(1));
        let first = Uuid::from_u128(2);
        let second = Uuid::from_u128(3);
        for (id, account) in [(first, "synthetic-first"), (second, "synthetic-second")] {
            let path = store.root.join("generations").join(id.to_string());
            AzureCliSessionStore::prepare_directory(&path)?;
            std::fs::write(
                path.join("identity.json"),
                facet_json::to_vec(&AzureCliAccountIdentity {
                    tenant,
                    account: account.into(),
                    kind: "user".into(),
                })?,
            )?;
        }
        AzureCliSessionStore::prepare_directory(&store.root.join("tenants"))?;
        let selector = store.root.join("tenants").join(tenant.to_string());
        std::fs::write(&selector, first.to_string())?;
        CancellationToken::new()
            .with_async_scope(|scope| async move {
                let bound = store.load(tenant, &scope).await?;
                std::fs::write(&selector, second.to_string())?;
                let next = store.load(tenant, &scope).await?;
                assert_ne!(bound.directory, next.directory);
                assert_eq!(bound.identity.account, "synthetic-first");
                assert_eq!(next.identity.account, "synthetic-second");
                std::fs::write(bound.directory.join("revoked"), b"synthetic logout")?;
                assert!(bound.lease(&scope).await.is_err());
                assert!(next.lease(&scope).await.is_ok());
                std::fs::write(&selector, "../../outside")?;
                assert!(store.load(tenant, &scope).await.is_err());
                Ok(())
            })
            .await
    }

    #[tokio::test]
    async fn logout_reports_each_unreadable_generation_without_stopping_early() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let store = AzureCliSessionStore {
            root: directory.path().to_path_buf(),
        };
        for id in [Uuid::from_u128(1), Uuid::from_u128(2)] {
            // A directory cannot be opened as the read/write lease file on
            // either supported platform. Every path stops before CLI/config access.
            std::fs::create_dir_all(
                store
                    .root
                    .join("generations")
                    .join(id.to_string())
                    .join("session.lock"),
            )?;
        }
        CancellationToken::new()
            .with_async_scope(|scope| async move {
                let error = store
                    .logout_all(&scope)
                    .await
                    .expect_err("both damaged leases must be reported");
                let failures = error
                    .downcast_ref::<AuthLogoutError>()
                    .expect("logout aggregates generation failures");
                assert_eq!(failures.errors.len(), 2);
                for error in &failures.errors {
                    assert!(
                        error
                            .to_string()
                            .contains("opening the Azure CLI session lease")
                    );
                }
                Ok(())
            })
            .await
    }
}
