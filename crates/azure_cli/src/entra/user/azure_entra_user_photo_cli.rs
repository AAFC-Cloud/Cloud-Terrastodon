use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_azure::EntraSignedInUserThumbnailGetRequest;
use cloud_terrastodon_azure::EntraUserId;
use cloud_terrastodon_azure::EntraUserThumbnailGetRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::OptionExt;
use eyre::Result;
use eyre::WrapErr;
use facet::Facet;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tempfile::NamedTempFile;

/// Download an Entra user's 48×48 profile photo.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraUserPhotoArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,

    /// Entra user object ID. Omit to download the signed-in user's photo.
    #[facet(figue::named)]
    pub user_id: Option<EntraUserId>,

    /// Destination path for the photo. The file must be new.
    #[facet(figue::named)]
    pub file: PathBuf,
}

impl AzureEntraUserPhotoArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let destination = std::path::absolute(self.file).wrap_err("resolving photo output path")?;
        let file_name = destination
            .file_name()
            .ok_or_eyre("photo output path must include a file name")?;
        let authentication = self.authentication.resolve(executor).await?;
        let photo = match self.user_id {
            Some(user_id) => {
                EntraUserThumbnailGetRequest {
                    user_id,
                    authentication,
                }
                .execute(executor)
                .await?
            }
            None => {
                eyre::ensure!(
                    authentication.auth_source != AzureBearerAuthSource::WorkloadIdentity,
                    "the selected source has no signed-in Entra user; specify --user-id"
                );
                EntraSignedInUserThumbnailGetRequest { authentication }
                    .execute(executor)
                    .await?
            }
        };

        let saved_file = match photo {
            Some(photo) => {
                executor
                    .cancellation()
                    .with_deferral("saving the Entra profile photo", |_| {
                        // Canonicalize the directory for long-path support on Windows.
                        let directory = fs::canonicalize(
                            destination
                                .parent()
                                .ok_or_eyre("photo output path has no parent directory")?,
                        )
                        .wrap_err("resolving photo output directory")?;
                        let mut temporary = NamedTempFile::new_in(&directory)
                            .wrap_err("staging the profile photo")?;
                        temporary
                            .write_all(&photo.bytes)
                            .wrap_err("writing the profile photo")?;
                        temporary
                            .persist_noclobber(directory.join(file_name))
                            .map_err(|error| error.error)
                            .wrap_err("publishing the profile photo to a new destination")?;
                        eyre::Ok(())
                    })?;
                Some(destination)
            }
            None => None,
        };

        Ok(CliOutput::facet_with_text(saved_file, |path, writer, _| {
            match path {
                Some(path) => writeln!(writer, "Saved profile photo to {}", path.display())?,
                None => writeln!(writer, "No profile photo available.")?,
            }
            Ok(())
        }))
    }
}
