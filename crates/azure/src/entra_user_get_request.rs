use crate::MicrosoftGraphHelper;
use crate::PercentEncodeExt;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_types::AzurePrincipalArgument;
use cloud_terrastodon_azure_types::EntraUser;
use cloud_terrastodon_azure_types::EntraUserId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AzureTenantAuthContext;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use std::borrow::Cow;
use std::path::PathBuf;
use tracing::debug;

#[derive(Arbitrary, Facet)]
#[repr(C)]
pub enum EntraUserLookup {
    ObjectId(EntraUserId),
    UserPrincipalName(String),
}

impl From<EntraUserId> for EntraUserLookup {
    fn from(value: EntraUserId) -> Self {
        Self::ObjectId(value)
    }
}

impl From<String> for EntraUserLookup {
    fn from(value: String) -> Self {
        Self::UserPrincipalName(value)
    }
}

impl From<&str> for EntraUserLookup {
    fn from(value: &str) -> Self {
        Self::UserPrincipalName(value.to_owned())
    }
}

impl From<AzurePrincipalArgument<'_>> for EntraUserLookup {
    fn from(value: AzurePrincipalArgument<'_>) -> Self {
        match value {
            AzurePrincipalArgument::Id(id) => Self::ObjectId(EntraUserId::new(**id.as_ref())),
            AzurePrincipalArgument::Name(name) => Self::UserPrincipalName(name.into_owned()),
            AzurePrincipalArgument::Principal(principal) => {
                Self::ObjectId(EntraUserId::new(*principal.as_ref().as_ref()))
            }
        }
    }
}

#[must_use = "This is a future request, you must .await it"]
#[derive(Facet)]
pub struct EntraUserGetRequest<'a> {
    pub lookup: EntraUserLookup,
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

impl<'a> Arbitrary<'a> for EntraUserGetRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            lookup: Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(Arbitrary::arbitrary(u)?),
        })
    }
}

pub fn fetch_entra_user<'a, T>(
    lookup: T,
    auth_context: &'a AzureTenantAuthContext,
) -> EntraUserGetRequest<'a>
where
    T: Into<EntraUserLookup>,
{
    EntraUserGetRequest {
        lookup: lookup.into(),
        auth_context: Cow::Borrowed(auth_context),
    }
}

impl EntraUserGetRequest<'_> {
    fn url(&self) -> String {
        let lookup = match &self.lookup {
            EntraUserLookup::ObjectId(user_id) => user_id.to_string(),
            EntraUserLookup::UserPrincipalName(user_principal_name) => {
                user_principal_name.trim().percent_encode()
            }
        };

        format!(
            "https://graph.microsoft.com/v1.0/users/{lookup}?$select={}",
            EntraUser::SELECT
        )
    }
}

#[async_trait]
impl CacheableCommand for EntraUserGetRequest<'_> {
    type Output = EntraUser;

    fn cache_key(&self) -> CacheKey {
        let tenant_id = self.auth_context.tenant_id.to_string();

        match &self.lookup {
            EntraUserLookup::ObjectId(user_id) => {
                let user_id = user_id.to_string();
                CacheKey::new(PathBuf::from_iter([
                    "ms",
                    "graph",
                    "GET",
                    "users",
                    tenant_id.as_str(),
                    user_id.as_str(),
                ]))
            }
            EntraUserLookup::UserPrincipalName(user_principal_name) => {
                let user_principal_name_hash = blake3::hash(user_principal_name.trim().as_bytes())
                    .to_hex()
                    .to_string();
                CacheKey::new(PathBuf::from_iter([
                    "ms",
                    "graph",
                    "GET",
                    "users",
                    "by_user_principal_name",
                    tenant_id.as_str(),
                    user_principal_name_hash.as_str(),
                ]))
            }
        }
    }

    async fn run(self) -> Result<Self::Output> {
        match &self.lookup {
            EntraUserLookup::ObjectId(user_id) => {
                debug!(
                    tenant_id = %self.auth_context.tenant_id,
                    user_id = %user_id,
                    "Fetching user by object id"
                );
            }
            EntraUserLookup::UserPrincipalName(user_principal_name) => {
                let user_principal_name = user_principal_name.trim();
                if user_principal_name.is_empty() {
                    bail!("User principal name cannot be empty.");
                }

                debug!(
                    tenant_id = %self.auth_context.tenant_id,
                    user_principal_name,
                    "Fetching Entra user by user principal name"
                );
            }
        }

        MicrosoftGraphHelper::new(
            self.url(),
            Some(self.cache_key()),
            self.auth_context.as_ref(),
        )
        .fetch_one()
        .await
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(EntraUserGetRequest<'a>, 'a);
cloud_terrastodon_registry::register_thing!(EntraUserGetRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(EntraUserGetRequest<'static>);
cloud_terrastodon_registry::register_into_future!(EntraUserGetRequest<'static> => EntraUser);

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_azure_types::AzureTenantId;
    use cloud_terrastodon_credentials::AuthContext;

    #[test]
    fn url_targets_the_exact_user_principal_name() {
        let request = EntraUserGetRequest {
            lookup: EntraUserLookup::UserPrincipalName("O'Neil@example.com".to_owned()),
            auth_context: Cow::Owned(
                AuthContext::explicit_azure_cli()
                    .bind_to_azure_tenant(AzureTenantId::new(
                        cloud_terrastodon_azure_types::uuid::Uuid::nil(),
                    ))
                    .expect("default auth context should bind to a tenant"),
            ),
        };

        assert!(
            request
                .url()
                .contains("/users/O%27Neil%40example.com?$select=")
        );
    }

    #[test]
    fn azure_principal_argument_converts_to_user_lookup() {
        let lookup: EntraUserLookup = AzurePrincipalArgument::from("jane.doe@example.com").into();

        assert!(matches!(
            lookup,
            EntraUserLookup::UserPrincipalName(name) if name == "jane.doe@example.com"
        ));
    }
}
