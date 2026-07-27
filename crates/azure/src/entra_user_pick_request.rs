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
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerEvent;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use facet::Facet;
use std::future::Future;
use std::future::IntoFuture;
use std::path::PathBuf;
use std::pin::Pin;
use tokio::try_join;
use tracing::info;

#[must_use = "This is an interactive future request, you must .await it"]
#[derive(Arbitrary, Facet)]
pub struct EntraUserPickRequest {
    pub tenant_id: AzureTenantId,
}

pub fn pick_entra_users(tenant_id: AzureTenantId) -> EntraUserPickRequest {
    EntraUserPickRequest { tenant_id }
}

#[async_trait]
impl CacheInvalidatable for EntraUserPickRequest {
    async fn invalidate(&self) -> Result<()> {
        let users = fetch_all_entra_users(self.tenant_id).cache_key();
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

impl CacheInvalidatableIntoFuture for EntraUserPickRequest {
    type WithInvalidation = Pin<Box<dyn Future<Output = Self::Output> + Send>>;

    fn with_invalidation(self, invalidate_cache: bool) -> Self::WithInvalidation {
        Box::pin(async move {
            if invalidate_cache {
                self.invalidate().await?;
            }
            self.into_future().await
        })
    }
}

impl IntoFuture for EntraUserPickRequest {
    type Output = Result<Vec<EntraUser>>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send>>;

    fn into_future(self) -> Self::IntoFuture {
        let tenant_id = self.tenant_id;
        Box::pin(async move {
            PickerTui::<EntraUser>::new()
                .set_header("Users")
                .add_event_handler({
                    move |event, sink| async move {
                        if matches!(event.as_ref(), PickerEvent::InitialLoad) {
                            info!("Fetching all Entra users");
                            let users = fetch_all_entra_users(tenant_id).await?;
                            sink.push(users.into_iter().map(user_choice))?;
                            info!("Finished fetching all Entra users");
                        }
                        Ok(())
                    }
                })
                .add_event_handler({
                    move |event, sink| async move {
                        let query = match event.as_ref() {
                            PickerEvent::QueryChanged(query)
                            | PickerEvent::ReloadRequested(query) => query,
                            PickerEvent::InitialLoad | PickerEvent::QueryCleared => return Ok(()),
                        };
                        info!(query = %query, "Searching Entra users");
                        let users = search_entra_users(tenant_id, query.as_ref()).await?;
                        sink.push(users.into_iter().map(user_choice))?;
                        info!(query = %query, "Finished searching Entra users");
                        Ok(())
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

cloud_terrastodon_registry::register_thing!(EntraUserPickRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraUserPickRequest);
cloud_terrastodon_registry::register_into_future!(EntraUserPickRequest => Vec<EntraUser>);
