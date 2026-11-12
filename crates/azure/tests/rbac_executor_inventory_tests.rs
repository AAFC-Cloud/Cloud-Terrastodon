#![cfg(test)]

//! Exercise RBAC composition against typed in-memory child inventories only.
//! Authorization always fails, so an accidental cache miss cannot access Azure.

use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure::AzureRoleAssignmentListRequest;
use cloud_terrastodon_azure::AzureRoleDefinitionListRequest;
use cloud_terrastodon_azure::AzureRoleDefinitionsAndAssignmentsListRequest;
use cloud_terrastodon_azure_types::RoleAssignment;
use cloud_terrastodon_azure_types::RoleDefinition;
use cloud_terrastodon_azure_types::RoleDefinitionKind;
use cloud_terrastodon_cache::ArtifactObservation;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::ExecutorKind;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::NonEmptyString;
use cloud_terrastodon_cache::WorkCache;
use cloud_terrastodon_cache::WorkOutput;
use cloud_terrastodon_cache::WorkReuse;
use cloud_terrastodon_command::CancellationToken;
use cloud_terrastodon_execution::CacheInvalidatableRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::TestAuthenticationProvider;
use eyre::Result;
use std::sync::Arc;
use std::sync::atomic::Ordering;

fn request(source: AzureBearerAuthSource) -> AzureRoleDefinitionsAndAssignmentsListRequest {
    AzureRoleDefinitionsAndAssignmentsListRequest {
        authentication: AzureBearerAuthenticationSelection {
            account_id: "11111111-1111-1111-1111-111111111111".parse().unwrap(),
            tenant_id: "00000000-0000-0000-0000-000000000001".parse().unwrap(),
            auth_source: source,
        },
    }
}

fn definition(name: &str) -> Arc<RoleDefinition> {
    Arc::new(RoleDefinition {
        id: "/providers/Microsoft.Authorization/roleDefinitions/00000000-0000-0000-0000-000000000002"
            .parse()
            .unwrap(),
        display_name: name.to_owned(),
        description: String::new(),
        assignable_scopes: vec!["/".to_owned()],
        permissions: Vec::new(),
        kind: RoleDefinitionKind::CustomRole,
    })
}

fn assignment(definition: &RoleDefinition, principal: u8) -> Arc<RoleAssignment> {
    Arc::new(RoleAssignment {
        id: "/providers/Microsoft.Authorization/roleAssignments/00000000-0000-0000-0000-000000000003"
            .parse()
            .unwrap(),
        scope: "/subscriptions/00000000-0000-0000-0000-000000000004".parse().unwrap(),
        role_definition_id: definition.id.clone(),
        principal_id: format!("00000000-0000-0000-0000-{principal:012}").parse().unwrap(),
    })
}

// Seed the existing executor boundary using the public request's canonical
// identity. The actual combined request still owns composition and validation.
async fn seed<R, T>(
    executor: &CloudTerrastodonExecutor,
    request: &R,
    authentication: AzureBearerAuthenticationSelection,
    value: T,
) -> Result<Arc<T>>
where
    R: HasCacheKey + facet::Facet<'static> + 'static,
    T: Send + Sync + 'static,
{
    executor
        .cached_work(
            ExecutorKind::InProcess,
            request.cache_key()?,
            CacheOperationOwner::of::<R>(),
            &authentication.into(),
            NonEmptyString::new(facet_json::to_string(request)?)?,
            WorkReuse::ReusableRead,
            |_| async move { Ok(WorkOutput::new(value, ArtifactObservation::now())) },
        )?
        .await
}

async fn seed_definitions(
    executor: &CloudTerrastodonExecutor,
    request: &AzureRoleDefinitionsAndAssignmentsListRequest,
    values: Vec<Arc<RoleDefinition>>,
) -> Result<Arc<Vec<Arc<RoleDefinition>>>> {
    seed(
        executor,
        &AzureRoleDefinitionListRequest {
            authentication: request.authentication,
        },
        request.authentication,
        values,
    )
    .await
}

async fn seed_assignments(
    executor: &CloudTerrastodonExecutor,
    request: &AzureRoleDefinitionsAndAssignmentsListRequest,
    values: Vec<Arc<RoleAssignment>>,
) -> Result<Arc<Vec<Arc<RoleAssignment>>>> {
    seed(
        executor,
        &AzureRoleAssignmentListRequest {
            authentication: request.authentication,
        },
        request.authentication,
        values,
    )
    .await
}

#[tokio::test]
async fn concurrent_composition_shares_one_validated_inventory_and_child_records() -> Result<()> {
    CancellationToken::new()
        .cancellable_scope(|scope| async move {
            let authentication = Arc::new(TestAuthenticationProvider::offline(
                "synthetic-test-session",
            )?);
            let executor = CloudTerrastodonExecutor::new(
                WorkCache::temporary()?,
                authentication.clone(),
                scope,
            );
            let request = request(AzureBearerAuthSource::AzureCli);
            let role = definition("reader");
            let assigned = assignment(&role, 5);
            seed_definitions(&executor, &request, vec![Arc::clone(&role)]).await?;
            seed_assignments(&executor, &request, vec![Arc::clone(&assigned)]).await?;

            let (first, concurrent) = executor
                .join2(
                    async |child| child.execute(request.clone()).await,
                    async |child| child.execute(request.clone()).await,
                )
                .await?;
            let warm = request.execute(&executor).await?;
            assert!(Arc::ptr_eq(&first, &concurrent));
            assert!(Arc::ptr_eq(&first, &warm));
            assert!(Arc::ptr_eq(&first.role_definitions[&role.id], &role));
            assert!(Arc::ptr_eq(
                &first.role_assignments[&assigned.id],
                &assigned
            ));
            assert_eq!(authentication.authorizations.load(Ordering::SeqCst), 0);
            Ok(())
        })
        .await
}

#[tokio::test]
async fn either_child_invalidation_expires_the_combined_inventory() -> Result<()> {
    CancellationToken::new()
        .cancellable_scope(|scope| async move {
            let authentication = Arc::new(TestAuthenticationProvider::offline(
                "synthetic-test-session",
            )?);
            let executor = CloudTerrastodonExecutor::new(
                WorkCache::temporary()?,
                authentication.clone(),
                scope,
            );
            let request = request(AzureBearerAuthSource::AzureCli);
            let original = definition("before");
            let assigned = assignment(&original, 5);
            seed_definitions(&executor, &request, vec![Arc::clone(&original)]).await?;
            seed_assignments(&executor, &request, vec![Arc::clone(&assigned)]).await?;
            let first = request.clone().execute(&executor).await?;

            AzureRoleDefinitionListRequest {
                authentication: request.authentication,
            }
            .invalidate(&executor)
            .await?;
            let changed = definition("after");
            seed_definitions(&executor, &request, vec![Arc::clone(&changed)]).await?;
            let second = request.clone().execute(&executor).await?;
            assert!(!Arc::ptr_eq(&first, &second));
            assert_eq!(first.role_definitions[&original.id].display_name, "before");
            assert!(Arc::ptr_eq(&second.role_definitions[&changed.id], &changed));

            AzureRoleAssignmentListRequest {
                authentication: request.authentication,
            }
            .invalidate(&executor)
            .await?;
            let reassigned = assignment(&changed, 6);
            seed_assignments(&executor, &request, vec![Arc::clone(&reassigned)]).await?;
            let third = request.clone().execute(&executor).await?;
            assert!(!Arc::ptr_eq(&second, &third));
            assert!(Arc::ptr_eq(
                &third.role_assignments[&reassigned.id],
                &reassigned
            ));

            // Refreshing the combined request must refresh both child inventories,
            // not merely evict the assembled map and immediately reuse its inputs.
            request.invalidate(&executor).await?;
            let refreshed_role = definition("explicit refresh");
            let refreshed_assignment = assignment(&refreshed_role, 7);
            seed_definitions(&executor, &request, vec![Arc::clone(&refreshed_role)]).await?;
            seed_assignments(&executor, &request, vec![Arc::clone(&refreshed_assignment)]).await?;
            let refreshed = request.execute(&executor).await?;
            assert!(!Arc::ptr_eq(&third, &refreshed));
            assert!(Arc::ptr_eq(
                &refreshed.role_definitions[&refreshed_role.id],
                &refreshed_role
            ));
            assert!(Arc::ptr_eq(
                &refreshed.role_assignments[&refreshed_assignment.id],
                &refreshed_assignment
            ));
            assert_eq!(authentication.authorizations.load(Ordering::SeqCst), 0);
            Ok(())
        })
        .await
}

#[tokio::test]
async fn inconsistent_children_fail_validation_and_allow_a_corrected_retry() -> Result<()> {
    CancellationToken::new()
        .cancellable_scope(|scope| async move {
            let authentication = Arc::new(TestAuthenticationProvider::offline(
                "synthetic-test-session",
            )?);
            let executor = CloudTerrastodonExecutor::new(
                WorkCache::temporary()?,
                authentication.clone(),
                scope,
            );
            let request = request(AzureBearerAuthSource::AzureCli);
            let missing_role = definition("missing");
            let assigned = assignment(&missing_role, 5);
            seed_definitions(&executor, &request, Vec::new()).await?;
            seed_assignments(&executor, &request, vec![assigned]).await?;
            for _ in 0..2 {
                let error = request.clone().execute(&executor).await.unwrap_err();
                assert!(
                    error
                        .to_string()
                        .contains("references unknown role definition")
                );
            }

            AzureRoleDefinitionListRequest {
                authentication: request.authentication,
            }
            .invalidate(&executor)
            .await?;
            seed_definitions(&executor, &request, vec![missing_role]).await?;
            let corrected = request.execute(&executor).await?;
            assert_eq!(corrected.iter_role_assignments().count(), 1);
            assert_eq!(authentication.authorizations.load(Ordering::SeqCst), 0);
            Ok(())
        })
        .await
}

#[tokio::test]
async fn authentication_selection_keeps_inventories_separate() -> Result<()> {
    CancellationToken::new()
        .cancellable_scope(|scope| async move {
            let authentication = Arc::new(TestAuthenticationProvider::offline(
                "synthetic-test-session",
            )?);
            let executor = CloudTerrastodonExecutor::new(
                WorkCache::temporary()?,
                authentication.clone(),
                scope,
            );
            let cli_request = request(AzureBearerAuthSource::AzureCli);
            let browser_request = request(AzureBearerAuthSource::Browser);
            for (request, name) in [(&cli_request, "cli"), (&browser_request, "browser")] {
                let role = definition(name);
                let assigned = assignment(&role, 5);
                seed_definitions(&executor, request, vec![role]).await?;
                seed_assignments(&executor, request, vec![assigned]).await?;
            }
            let cli = cli_request.execute(&executor).await?;
            let browser = browser_request.execute(&executor).await?;
            assert!(!Arc::ptr_eq(&cli, &browser));
            assert_eq!(
                cli.role_definitions.values().next().unwrap().display_name,
                "cli"
            );
            assert_eq!(
                browser
                    .role_definitions
                    .values()
                    .next()
                    .unwrap()
                    .display_name,
                "browser"
            );
            assert_eq!(authentication.authorizations.load(Ordering::SeqCst), 0);
            Ok(())
        })
        .await
}
