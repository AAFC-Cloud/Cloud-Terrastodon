use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::EntraApplicationClientId;
use cloud_terrastodon_azure::EntraApplicationRegistration;
use cloud_terrastodon_azure::EntraApplicationRegistrationGetRequest;
use cloud_terrastodon_azure::EntraApplicationRegistrationListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use tracing::info;

/// Show a single Entra (Azure AD) application registration.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraApplicationRegistrationShowArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,

    /// Application registration object id, app id, display name, identifier URI, or unique name.
    #[facet(figue::positional)]
    pub application_registration: String,
}

impl AzureEntraApplicationRegistrationShowArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        info!(needle = %self.application_registration, tenant_id = %authentication.tenant_id, "Fetching application registrations");
        let needle = self.application_registration.trim();

        if let Ok(app_id) = needle.parse::<EntraApplicationClientId>() {
            match (EntraApplicationRegistrationGetRequest {
                application_id: (app_id),
                authentication,
            })
            .execute(executor)
            .await
            {
                Ok(application) => {
                    return Ok(CliOutput::facet(application));
                }
                Err(error) => {
                    info!(
                        %error,
                        "Object-id lookup did not match; checking alternate application identifiers"
                    );
                }
            }
        }

        let applications = EntraApplicationRegistrationListRequest { authentication }
            .execute(executor)
            .await?;
        info!(
            count = applications.len(),
            "Fetched application registrations"
        );

        let mut matches = applications
            .into_iter()
            .filter(|application| matches_application_registration(application, needle))
            .collect::<Vec<_>>();

        match matches.len() {
            0 => {
                bail!("No application registration found matching '{}'.", needle);
            }
            1 => Ok(CliOutput::facet(matches.remove(0))),
            _ => {
                matches.sort_by_key(|application| application.id.to_string());
                let ids = matches
                    .iter()
                    .map(|application| application.id.to_string())
                    .collect::<Vec<_>>()
                    .join("\n  ");
                bail!(
                    "Multiple application registrations matched '{}'. Use a full object id.\n  {}",
                    needle,
                    ids
                );
            }
        }
    }
}

fn matches_application_registration(
    application: &EntraApplicationRegistration,
    needle: &str,
) -> bool {
    application.id.to_string() == needle
        || application.app_id.to_string() == needle
        || application.display_name.eq_ignore_ascii_case(needle)
        || application
            .unique_name
            .as_deref()
            .map(|value| value.eq_ignore_ascii_case(needle))
            .unwrap_or(false)
        || application
            .identifier_uris
            .iter()
            .any(|uri| uri.eq_ignore_ascii_case(needle))
}

#[cfg(test)]
mod tests {
    use super::matches_application_registration;
    use arbitrary::Arbitrary;
    use arbitrary::Unstructured;
    use cloud_terrastodon_azure::EntraApplicationRegistration;

    fn sample_application_registration() -> EntraApplicationRegistration {
        let data = (0u8..=255).cycle().take(4096).collect::<Vec<_>>();
        let mut unstructured = Unstructured::new(&data);
        EntraApplicationRegistration::arbitrary(&mut unstructured)
            .expect("sample application registration should be generated from arbitrary")
    }

    #[test]
    fn matches_by_identifier_uri() {
        let mut application = sample_application_registration();
        application.identifier_uris = vec!["api://contoso/example-app".to_string()];

        assert!(matches_application_registration(
            &application,
            "api://contoso/example-app"
        ));
    }

    #[test]
    fn matches_by_unique_name() {
        let mut application = sample_application_registration();
        application.unique_name = Some("contoso-my-app".to_string());

        assert!(matches_application_registration(
            &application,
            "contoso-my-app"
        ));
    }
}
