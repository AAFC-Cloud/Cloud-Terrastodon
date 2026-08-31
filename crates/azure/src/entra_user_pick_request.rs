use crate::fetch_all_entra_users;
use crate::search_entra_users;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraUser;
use cloud_terrastodon_command::CacheInvalidatable;
use cloud_terrastodon_command::CacheInvalidatableIntoFuture;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AuthContext;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerEvent;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use facet::Facet;
use std::borrow::Cow;
use std::future::Future;
use std::future::IntoFuture;
use std::path::PathBuf;
use std::pin::Pin;
use tokio::try_join;
use tracing::info;

#[must_use = "This is an interactive future request, you must .await it"]
#[derive(Facet)]
pub struct EntraUserPickRequest<'a> {
    pub tenant_id: AzureTenantId,
    pub auth_context: Cow<'a, AuthContext>,
}

impl<'a> Arbitrary<'a> for EntraUserPickRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            tenant_id: AzureTenantId::arbitrary(u)?,
            auth_context: Cow::Owned(AuthContext::default()),
        })
    }
}

pub fn pick_entra_users<'a>(
    tenant_id: AzureTenantId,
    auth_context: &'a AuthContext,
) -> EntraUserPickRequest<'a> {
    EntraUserPickRequest {
        tenant_id,
        auth_context: Cow::Borrowed(auth_context),
    }
}

#[async_trait]
impl<'a> CacheInvalidatable for EntraUserPickRequest<'a> {
    async fn invalidate(&self) -> Result<()> {
        let users = fetch_all_entra_users(self.tenant_id, self.auth_context.as_ref()).cache_key();
        let searches = CacheKey::new(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "users",
            "search",
            self.tenant_id.to_string().as_str(),
        ]));

        try_join!(users.invalidate(), searches.invalidate())?;
        Ok(())
    }
}

impl<'a> CacheInvalidatableIntoFuture for EntraUserPickRequest<'a> {
    type WithInvalidation = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

    fn with_invalidation(self, invalidate_cache: bool) -> Self::WithInvalidation {
        Box::pin(async move {
            if invalidate_cache {
                self.invalidate().await?;
            }
            self.into_future().await
        })
    }
}

impl<'a> IntoFuture for EntraUserPickRequest<'a> {
    type Output = Result<Vec<EntraUser>>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        let tenant_id = self.tenant_id;
        let auth_context = self.auth_context;
        Box::pin(async move {
            let list_auth_context = auth_context.clone();
            PickerTui::<EntraUser>::new()
                .set_header("Users")
                .add_event_handler({
                    move |event, sink| {
                        let auth_context = list_auth_context.clone();
                        async move {
                            if matches!(event.as_ref(), PickerEvent::InitialLoad) {
                                info!("Fetching all Entra users");
                                let users =
                                    fetch_all_entra_users(tenant_id, auth_context.as_ref()).await?;
                                sink.push(users.into_iter().map(user_choice))?;
                                info!("Finished fetching all Entra users");
                            }
                            Ok(())
                        }
                    }
                })
                .add_event_handler({
                    move |event, sink| {
                        let auth_context = auth_context.clone();
                        async move {
                            let query = match event.as_ref() {
                                PickerEvent::QueryChanged(query)
                                | PickerEvent::ReloadRequested(query) => query,
                                PickerEvent::InitialLoad | PickerEvent::QueryCleared => {
                                    return Ok(());
                                }
                            };
                            info!(query = %query, "Searching Entra users");
                            let users = search_entra_users(
                                tenant_id,
                                query.as_ref(),
                                auth_context.as_ref(),
                            )
                            .await?;
                            sink.push(users.into_iter().map(user_choice))?;
                            info!(query = %query, "Finished searching Entra users");
                            Ok(())
                        }
                    }
                })
                .pick_many_events()
                .await
                .map_err(Into::into)
        })
    }
}

fn user_choice(user: EntraUser) -> Choice<EntraUser> {
    Choice {
        key: format!(
            "{} {:64} {}",
            user.id, user.display_name, user.user_principal_name
        ),
        value: user,
    }
}

cloud_terrastodon_registry::register_thing!(EntraUserPickRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(EntraUserPickRequest<'static>);
cloud_terrastodon_registry::register_into_future!(EntraUserPickRequest<'static> => Vec<EntraUser>);
