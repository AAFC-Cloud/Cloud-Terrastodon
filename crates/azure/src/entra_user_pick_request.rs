use crate::EntraUserListRequest;
use crate::EntraUserSearchRequest;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::EntraUser;
use cloud_terrastodon_cache::CacheInvalidationKeys;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
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
/// selected tenant. Each event handler uses the picker's linked cancellation scope.
#[must_use = "This is an interactive request; execute it with an executor"]
#[derive(Arbitrary, Facet)]
pub struct EntraUserPickRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl CacheInvalidationKeys for EntraUserPickRequest {
    fn cache_keys_to_invalidate(&self) -> Result<Vec<CacheKey>> {
        Ok(vec![
            EntraUserListRequest {
                authentication: self.authentication,
            }
            .cache_key()?,
            EntraUserSearchRequest {
                search_term: String::new(),
                authentication: self.authentication,
            }
            .tenant_cache_key()?,
        ])
    }
}

impl ExecutableRequest for EntraUserPickRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor
            .with_authentication_lease(&self.authentication.into())
            .map_err(Into::into)
    }
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
            PickerTui::<Arc<EntraUser>>::new()
                .set_header("Users")
                .add_event_handler(async |child, event, sink| {
                    if matches!(event.as_ref(), PickerEvent::InitialLoad) {
                        info!("Fetching all Entra users");
                        let users = EntraUserListRequest {
                            authentication: self.authentication,
                        }
                        .execute(&executor.with_cancellation_scope(child))
                        .await?;
                        sink.push(users.iter().cloned().map(user_choice))?;
                        info!("Finished fetching all Entra users");
                    }
                    Ok(())
                })
                .add_event_handler(async |child, event, sink| {
                    let query = match event.as_ref() {
                        PickerEvent::QueryChanged(query) | PickerEvent::ReloadRequested(query) => {
                            query
                        }
                        PickerEvent::InitialLoad | PickerEvent::QueryCleared => return Ok(()),
                    };
                    info!(query = %query, "Searching Entra users");
                    let users = EntraUserSearchRequest {
                        search_term: (query.as_ref()).into(),
                        authentication: self.authentication,
                    }
                    .execute(&executor.with_cancellation_scope(child))
                    .await?;
                    sink.push(users.into_iter().map(user_choice))?;
                    info!(query = %query, "Finished searching Entra users");
                    Ok(())
                })
                .pick_many_events(executor.cancellation())
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
    use crate::test_authentication::test_authentication;

    #[test]
    fn refresh_covers_all_selected_tenant_searches_without_crossing_tenants() -> Result<()> {
        let selected = test_authentication();
        let other = AzureBearerAuthenticationSelection {
            tenant_id: "33333333-3333-3333-3333-333333333333".parse()?,
            ..selected
        };
        let keys = EntraUserPickRequest {
            authentication: selected,
        }
        .cache_keys_to_invalidate()?;
        for authentication in [&selected, &other] {
            let inventory = EntraUserListRequest {
                authentication: *authentication,
            }
            .cache_key()?;
            assert_eq!(
                keys.iter()
                    .any(|key| inventory.scope.starts_with(&key.scope)),
                authentication.tenant_id == selected.tenant_id
            );
            for term in ["Alice", "Bob", "O'Neil"] {
                let search = EntraUserSearchRequest {
                    search_term: term.to_owned(),
                    authentication: *authentication,
                }
                .cache_key()?;
                assert_eq!(
                    keys.iter().any(|key| search.scope.starts_with(&key.scope)),
                    authentication.tenant_id == selected.tenant_id
                );
            }
        }
        Ok(())
    }
}

cloud_terrastodon_registry_execution::register_runtime_test!(
    EntraUserPickRequest,
    name = "entra-user-pick",
    cached
);
