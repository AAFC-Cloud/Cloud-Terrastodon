use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use eyre::Result;
use eyre::WrapErr;
use std::path::PathBuf;
use uuid::Uuid;

/// Private credential storage for one immutable account session generation.
#[derive(Clone)]
pub(crate) struct CloudTerrastodonAccountSecretStore {
    pub(crate) directory: PathBuf,
    pub(crate) account_id: CloudTerrastodonAccountId,
    pub(crate) generation: Uuid,
}
impl CloudTerrastodonAccountSecretStore {
    #[cfg(windows)]
    fn target(&self) -> Result<crate::WindowsCredentialManagerTargetName> {
        crate::WindowsCredentialManagerTargetName::try_new(format!(
            "cloud-terrastodon:account:{}:{}",
            self.account_id, self.generation
        ))
    }

    pub(crate) fn read(&self) -> Result<String> {
        #[cfg(windows)]
        return crate::read_credential_from_manager(&self.target()?)?
            .ok_or_else(|| eyre::eyre!("the selected account has no stored credential"));
        #[cfg(not(windows))]
        std::fs::read_to_string(self.directory.join("credential"))
            .wrap_err("reading the selected account credential")
    }

    pub(crate) fn write(&self, value: &str) -> Result<()> {
        eyre::ensure!(
            !self.directory.join("retired").try_exists()?,
            "the account credential session is retired"
        );
        #[cfg(windows)]
        return crate::write_credential_to_manager(&self.target()?, value)
            .wrap_err("writing the selected account credential");
        #[cfg(not(windows))]
        {
            use std::io::Write;
            std::fs::create_dir_all(&self.directory)?;
            let mut file = tempfile::NamedTempFile::new_in(&self.directory)?;
            #[cfg(unix)]
            std::fs::set_permissions(
                file.path(),
                std::os::unix::fs::PermissionsExt::from_mode(0o600),
            )?;
            file.write_all(value.as_bytes())?;
            file.as_file().sync_all()?;
            file.persist(self.directory.join("credential"))
                .map_err(|error| error.error)
                .wrap_err("publishing the selected account credential")?;
            Ok(())
        }
    }

    pub(crate) fn delete(&self) -> Result<()> {
        #[cfg(windows)]
        return crate::delete_credential_from_manager(&self.target()?).map(|_| ());
        #[cfg(not(windows))]
        match std::fs::remove_file(self.directory.join("credential")) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error).wrap_err("removing the retired account credential"),
        }
    }
}
impl std::fmt::Debug for CloudTerrastodonAccountSecretStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CloudTerrastodonAccountSecretStore")
            .field("account_id", &self.account_id)
            .finish_non_exhaustive()
    }
}
