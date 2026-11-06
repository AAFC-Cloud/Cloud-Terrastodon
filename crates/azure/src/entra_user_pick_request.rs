use crate::EntraUserListRequest;
use crate::EntraUserSearchRequest;
use crate::fetch_all_entra_users;
use crate::search_entra_users;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraUser;
use cloud_terrastodon_cache::CacheInvalidationKeys;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickError;
use cloud_terrastodon_user_input::PickerEvent;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use facet::Facet;
use std::sync::Arc;
use tracing::info;

/// Select users from the tenant inventory or an interactive Graph search.
///
/// Refreshing invalidates both the complete inventory and every search for the
/// selected tenant. The executor owns each event handler's cancellation scope.
#[must_use = "This is an interactive request; execute it with an executor"]
#[derive(Arbitrary, Facet)]
pub struct EntraUserPickRequest {
    pub tenant_id: AzureTenantId,
    pub auth_source: AzureBearerAuthSource,
}

#[track_caller]
pub fn pick_entra_users<'a>(
    tenant_id: AzureTenantId,
    auth_source: AzureBearerAuthSource,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, EntraUserPickRequest> {
    executor.execute(EntraUserPickRequest {
        tenant_id,
        auth_source,
    })
}

impl CacheInvalidationKeys for EntraUserPickRequest {
    fn cache_keys_to_invalidate(&self) -> Result<Vec<CacheKey>> {
        Ok(vec![
            EntraUserListRequest {
                tenant_id: self.tenant_id,
                auth_source: self.auth_source,
            }
            .cache_key()?,
            EntraUserSearchRequest {
                search_term: String::new(),
                tenant_id: self.tenant_id,
                auth_source: self.auth_source,
            }
            .tenant_cache_key()?,
        ])
    }
}

impl ExecutableRequest for EntraUserPickRequest {
    type Output = Vec<Arc<EntraUser>>;
    type Error = PickError<Arc<EntraUser>>;

    fn run<'exec>(
        self,
        executor: CloudTerrastodonExecutor,
    ) -> ExecutionFuture<'exec, Self::Output, Self::Error>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let list_executor = executor.clone();
            PickerTui::<Arc<EntraUser>>::new()
                .set_header("Users")
                .add_event_handler(move |event, sink| {
                    let executor = list_executor.clone();
                    async move {
                        if matches!(event.as_ref(), PickerEvent::InitialLoad) {
                            executor
                                .cancellable_scope(|child| async move {
                                    info!("Fetching all Entra users");
                                    let users = fetch_all_entra_users(
                                        self.tenant_id,
                                        self.auth_source,
                                        &child,
                                    )
                                    .await?;
                                    sink.push(users.iter().cloned().map(user_choice))?;
                                    info!("Finished fetching all Entra users");
                                    Ok::<(), eyre::Report>(())
                                })
                                .await?;
                        }
                        Ok(())
                    }
                })
                .add_event_handler(move |event, sink| {
                    let executor = executor.clone();
                    async move {
                        let query = match event.as_ref() {
                            PickerEvent::QueryChanged(query)
                            | PickerEvent::ReloadRequested(query) => query,
                            PickerEvent::InitialLoad | PickerEvent::QueryCleared => return Ok(()),
                        };
                        executor
                            .cancellable_scope(|child| async move {
                                info!(query = %query, "Searching Entra users");
                                let users = search_entra_users(
                                    query.as_ref(),
                                    self.tenant_id,
                                    self.auth_source,
                                    &child,
                                )
                                .await?;
                                sink.push(users.into_iter().map(user_choice))?;
                                info!(query = %query, "Finished searching Entra users");
                                Ok(())
                            })
                            .await
                    }
                })
                .pick_many_events()
                .await
        })
    }
}

fn user_choice(user: Arc<EntraUser>) -> Choice<Arc<EntraUser>> {
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
cloud_terrastodon_registry_execution::register_executable_request!(EntraUserPickRequest => Vec<Arc<EntraUser>>, effects = [Read, Solicit]);

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_auth::AuthContext;

    #[test]
    fn refresh_covers_all_selected_tenant_searches_without_crossing_tenants() -> Result<()> {
        let selected = AuthContext::explicit_azure_cli()
            .bind_to_azure_tenant("11111111-1111-1111-1111-111111111111".parse()?)?;
        let other = AuthContext::explicit_azure_cli()
            .bind_to_azure_tenant("22222222-2222-2222-2222-222222222222".parse()?)?;
        let keys = EntraUserPickRequest {
            tenant_id: selected.tenant_id,
            auth_source: selected.auth_source()?,
        }
        .cache_keys_to_invalidate()?;
        for auth_context in [&selected, &other] {
            let inventory = EntraUserListRequest {
                tenant_id: auth_context.tenant_id,
                auth_source: auth_context.auth_source()?,
            }
            .cache_key()?;
            assert_eq!(
                keys.iter()
                    .any(|key| inventory.scope.starts_with(&key.scope)),
                auth_context.tenant_id == selected.tenant_id
            );
            for term in ["Alice", "Bob", "O'Neil"] {
                let search = EntraUserSearchRequest {
                    search_term: term.to_owned(),
                    tenant_id: auth_context.tenant_id,
                    auth_source: auth_context.auth_source()?,
                }
                .cache_key()?;
                assert_eq!(
                    keys.iter().any(|key| search.scope.starts_with(&key.scope)),
                    auth_context.tenant_id == selected.tenant_id
                );
            }
        }
        Ok(())
    }
}
