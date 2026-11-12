use crate::AzureAdminCorrespondenceAuditRequest;
use crate::AzureAuditFinding;
use crate::AzureAuditPasswordCredentialStatus;
use crate::AzureResourceTagAuditRequest;
use crate::AzureServicePrincipalPasswordAuditRequest;
use crate::AzureUnknownPrincipalAuditRequest;
use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure::AzureResourceListRequest;
use cloud_terrastodon_azure::AzureRoleAssignmentListRequest;
use cloud_terrastodon_azure::AzureRoleDefinitionListRequest;
use cloud_terrastodon_azure::AzureRoleDefinitionsAndAssignmentsListRequest;
use cloud_terrastodon_azure::EntraApplicationClientId;
use cloud_terrastodon_azure::EntraPrincipal;
use cloud_terrastodon_azure::EntraPrincipalCollection;
use cloud_terrastodon_azure::EntraPrincipalId;
use cloud_terrastodon_azure::EntraPrincipalListRequest;
use cloud_terrastodon_azure::EntraSecurityGroupListRequest;
use cloud_terrastodon_azure::EntraServicePrincipal;
use cloud_terrastodon_azure::EntraServicePrincipalListRequest;
use cloud_terrastodon_azure::EntraServicePrincipalObjectId;
use cloud_terrastodon_azure::EntraUser;
use cloud_terrastodon_azure::EntraUserId;
use cloud_terrastodon_azure::EntraUserListRequest;
use cloud_terrastodon_azure::Resource;
use cloud_terrastodon_azure::ResourceType;
use cloud_terrastodon_azure::RoleAssignment;
use cloud_terrastodon_azure::RoleDefinition;
use cloud_terrastodon_azure::RoleDefinitionKind;
use cloud_terrastodon_azure::RoleDefinitionsAndAssignments;
use cloud_terrastodon_azure::ServicePrincipalPasswordCredential;
use cloud_terrastodon_cache::ArtifactObservation;
use cloud_terrastodon_cache::CacheKey;
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
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

fn user(id: u128, name: &str, other_mails: &[&str]) -> EntraPrincipal {
    EntraPrincipal::User(Arc::new(EntraUser {
        business_phones: vec![],
        display_name: name.into(),
        external_user_state: None,
        external_user_state_change_date_time: None,
        given_name: None,
        id: EntraUserId::new(uuid::Uuid::from_u128(id)),
        job_title: None,
        mail: None,
        other_mails: other_mails.iter().map(|mail| (*mail).into()).collect(),
        mobile_phone: None,
        office_location: None,
        preferred_language: None,
        surname: None,
        user_principal_name: name.parse().unwrap(),
    }))
}

fn passwords(now: DateTime<Utc>, expiry_days: &[i64]) -> EntraPrincipalCollection {
    EntraPrincipalCollection::new([EntraPrincipal::ServicePrincipal(Arc::new(
        EntraServicePrincipal {
            account_enabled: true,
            add_ins: vec![],
            alternative_names: vec![],
            app_description: None,
            app_display_name: None,
            app_id: EntraApplicationClientId::new(uuid::Uuid::from_u128(1)),
            app_owner_organization_id: None,
            app_role_assignment_required: false,
            app_roles: vec![],
            application_template_id: None,
            created_date_time: now,
            deleted_date_time: None,
            description: None,
            disabled_by_microsoft_status: None,
            display_name: "Synthetic service".into(),
            homepage: None,
            id: EntraServicePrincipalObjectId::new(uuid::Uuid::from_u128(2)),
            info: None,
            key_credentials: vec![],
            login_url: None,
            logout_url: None,
            notes: None,
            notification_email_addresses: vec![],
            oauth2_permission_scopes: vec![],
            password_credentials: expiry_days
                .iter()
                .map(|days| ServicePrincipalPasswordCredential {
                    custom_key_identifier: None,
                    end_date_time: now + Duration::days(*days),
                    key_id: uuid::Uuid::from_u128(3),
                    start_date_time: now - Duration::days(60),
                    secret_text: None,
                    hint: None,
                    display_name: None,
                })
                .collect(),
            preferred_single_sign_on_mode: None,
            preferred_token_signing_key_thumbprint: None,
            reply_urls: vec![],
            resource_specific_application_permissions: vec![],
            saml_single_sign_on_settings: None,
            service_principal_names: vec![],
            service_principal_type: "Application".into(),
            sign_in_audience: None,
            tags: vec![],
            token_encryption_key_id: None,
            verified_publisher: None,
        },
    ))])
}

#[test]
fn audit_policy_tests_preserve_expiry_boundaries_and_replacement_cleanup() -> eyre::Result<()> {
    let now = "2020-01-01T00:00:00Z".parse::<DateTime<Utc>>()?;
    CancellationToken::new().with_scope(|scope| {
        let no_replacement = AzureServicePrincipalPasswordAuditRequest::evaluate(
            &passwords(now, &[-1, 29, 30]),
            now,
            &scope,
        )?;
        assert_eq!(no_replacement.len(), 2);
        assert!(matches!(
            no_replacement[0],
            AzureAuditFinding::ServicePrincipalPassword {
                status: AzureAuditPasswordCredentialStatus::ExpiredWithoutReplacement,
                ..
            }
        ));
        assert!(matches!(
            no_replacement[1],
            AzureAuditFinding::ServicePrincipalPassword {
                status: AzureAuditPasswordCredentialStatus::ExpiringWithoutReplacement,
                ..
            }
        ));
        let replacement = AzureServicePrincipalPasswordAuditRequest::evaluate(
            &passwords(now, &[-1, 29, 31]),
            now,
            &scope,
        )?;
        assert_eq!(replacement.len(), 1);
        assert!(matches!(
            replacement[0],
            AzureAuditFinding::ServicePrincipalPassword {
                status: AzureAuditPasswordCredentialStatus::ExpiredWithReplacement,
                ..
            }
        ));
        eyre::Ok(())
    })
}

#[test]
fn audit_policy_tests_preserve_case_insensitive_admin_correspondence() -> eyre::Result<()> {
    let principals = EntraPrincipalCollection::new([
        user(1, "User@example.invalid", &[]),
        user(2, "Admin.found@example.invalid", &["USER@example.invalid"]),
        user(
            3,
            "admin.missing@example.invalid",
            &["absent@example.invalid"],
        ),
    ]);
    CancellationToken::new().with_scope(|scope| {
        let findings = AzureAdminCorrespondenceAuditRequest::evaluate(&principals, &scope)?;
        assert_eq!(findings.len(), 1);
        assert!(matches!(&findings[0], AzureAuditFinding::AdminWithoutCorrespondingAccount { principal_name, .. } if principal_name == "admin.missing@example.invalid"));
        eyre::Ok(())
    })
}

#[test]
fn audit_policy_tests_compare_only_existing_tags_with_the_nearest_inventoried_parent()
-> eyre::Result<()> {
    let root = "/subscriptions/11111111-1111-1111-1111-111111111111";
    let paths = [
        root.to_owned(),
        format!("{root}/resourceGroups/example"),
        format!(
            "{root}/resourceGroups/example/providers/Microsoft.Storage/storageAccounts/fixture"
        ),
    ];
    let resources = paths
        .iter()
        .zip([
            vec![("a", "root")],
            vec![("a", "parent"), ("b", "parent")],
            vec![("a", "parent")],
        ])
        .map(|(path, tags)| {
            Arc::new(Resource {
                id: path.parse().unwrap(),
                kind: ResourceType::MICROSOFT_DOT_NETWORK_SLASH_VIRTUALNETWORKS,
                name: "fixture".into(),
                tags: tags
                    .into_iter()
                    .map(|(key, value)| (key.into(), value.into()))
                    .collect(),
                properties: HashMap::new(),
            })
        })
        .collect::<Vec<_>>();
    CancellationToken::new().with_scope(|scope| {
        let findings = AzureResourceTagAuditRequest::evaluate(&resources, &scope)?;
        assert_eq!(findings.len(), 1, "child matches nearest parent; absent tag b is not a mismatch");
        assert!(matches!(&findings[0], AzureAuditFinding::ResourceTagMismatch { tag_key, resource_tag_value, parent_tag_value, .. } if tag_key == "a" && resource_tag_value == "parent" && parent_tag_value == "root"));
        eyre::Ok(())
    })
}

#[test]
fn audit_policy_tests_report_only_unknown_assignment_principals() -> eyre::Result<()> {
    let definition = Arc::new(RoleDefinition {
        id: "/providers/Microsoft.Authorization/roleDefinitions/11111111-1111-1111-1111-111111111111".parse()?,
        display_name: "Reader".into(), description: String::new(), assignable_scopes: vec![], permissions: vec![], kind: RoleDefinitionKind::BuiltInRole,
    });
    let assignments = [1_u128, 2].map(|id| {
        Arc::new(RoleAssignment {
            id: format!(
                "/providers/Microsoft.Authorization/roleAssignments/{}",
                uuid::Uuid::from_u128(id)
            )
            .parse()
            .unwrap(),
            scope: "/".parse().unwrap(),
            role_definition_id: definition.id.clone(),
            principal_id: EntraPrincipalId::new(uuid::Uuid::from_u128(id)),
        })
    });
    let rbac = RoleDefinitionsAndAssignments::try_new([definition], assignments)?;
    let principals = EntraPrincipalCollection::new([user(1, "known@example.invalid", &[])]);
    CancellationToken::new().with_scope(|scope| {
        let findings = AzureUnknownPrincipalAuditRequest::evaluate(&principals, &rbac, &scope)?;
        assert_eq!(findings.len(), 1);
        assert!(matches!(&findings[0], AzureAuditFinding::UnknownRoleAssignmentPrincipal { principal_id, .. } if *principal_id == EntraPrincipalId::new(uuid::Uuid::from_u128(2))));
        eyre::Ok(())
    })
}

#[tokio::test]
async fn audit_policy_tests_independent_activities_join_one_shared_principal_fill()
-> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(async |scope| {
            let authentication = AzureBearerAuthenticationSelection {
                account_id: "11111111-1111-1111-1111-111111111111".parse()?,
                tenant_id: "22222222-2222-2222-2222-222222222222".parse()?,
                auth_source: AzureBearerAuthSource::AzureCli,
            };
            let provider = Arc::new(TestAuthenticationProvider::offline("audit-synthetic")?);
            let executor =
                CloudTerrastodonExecutor::new(WorkCache::temporary()?, provider.clone(), scope);
            let selection = authentication.into();
            let request = EntraPrincipalListRequest { authentication };
            let rbac = AzureRoleDefinitionsAndAssignmentsListRequest { authentication };
            executor
                .cached_work(
                    ExecutorKind::InProcess,
                    rbac.cache_key()?,
                    CacheOperationOwner::of::<AzureRoleDefinitionsAndAssignmentsListRequest>(),
                    &selection,
                    NonEmptyString::new(facet_json::to_string(&rbac)?)?,
                    WorkReuse::ReusableRead,
                    |_| async {
                        Ok(WorkOutput::new(
                            RoleDefinitionsAndAssignments::try_new([], [])?,
                            ArtifactObservation::now(),
                        ))
                    },
                )?
                .await?;
            let fills = AtomicUsize::new(0);
            let (_, unknown, admins) = executor
                .join3(
                    async |child| {
                        child
                            .cached_work(
                                ExecutorKind::InProcess,
                                request.cache_key()?,
                                CacheOperationOwner::of::<EntraPrincipalListRequest>(),
                                &selection,
                                NonEmptyString::new(facet_json::to_string(&request)?)?,
                                WorkReuse::ReusableRead,
                                |_| async {
                                    fills.fetch_add(1, Ordering::SeqCst);
                                    tokio::task::yield_now().await;
                                    Ok(WorkOutput::new(
                                        EntraPrincipalCollection::new([user(
                                            1,
                                            "admin.missing@example.invalid",
                                            &[],
                                        )]),
                                        ArtifactObservation::now(),
                                    ))
                                },
                            )?
                            .await
                    },
                    async |child| {
                        AzureUnknownPrincipalAuditRequest { authentication }
                            .execute(&child)
                            .await
                    },
                    async |child| {
                        AzureAdminCorrespondenceAuditRequest { authentication }
                            .execute(&child)
                            .await
                    },
                )
                .await?;
            assert_eq!(fills.load(Ordering::SeqCst), 1);
            assert!(unknown.is_empty());
            assert_eq!(admins.len(), 1);
            assert_eq!(
                provider.authorizations.load(Ordering::SeqCst),
                0,
                "inventory fixtures must prevent any credential or network acquisition"
            );
            Ok(())
        })
        .await
}

async fn inventory_marker(
    executor: &CloudTerrastodonExecutor,
    authentication: AzureBearerAuthenticationSelection,
    key: CacheKey,
    fills: &AtomicUsize,
) -> eyre::Result<Arc<usize>> {
    executor
        .cached_work(
            ExecutorKind::InProcess,
            key,
            CacheOperationOwner::of::<usize>(),
            &authentication.into(),
            NonEmptyString::new("synthetic-audit-inventory")?,
            WorkReuse::ReusableRead,
            |_| async {
                Ok(WorkOutput::new(
                    fills.fetch_add(1, Ordering::SeqCst),
                    ArtifactObservation::now(),
                ))
            },
        )?
        .await
}

async fn assert_audit_refreshes_inventories<R: CacheInvalidatableRequest>(
    request: R,
    authentication: AzureBearerAuthenticationSelection,
    inventories: Vec<CacheKey>,
) -> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(async |scope| {
            let provider = Arc::new(TestAuthenticationProvider::offline("audit-refresh")?);
            let executor =
                CloudTerrastodonExecutor::new(WorkCache::temporary()?, provider.clone(), scope);
            let mut keys = Vec::new();
            for key in inventories {
                keys.push(key.join("pages/1")?);
                keys.push(key);
            }
            let unaffected = EntraUserListRequest {
                authentication: AzureBearerAuthenticationSelection {
                    tenant_id: "33333333-3333-3333-3333-333333333333".parse()?,
                    ..authentication
                },
            }
            .cache_key()?;
            let fills = AtomicUsize::new(0);
            let retained =
                inventory_marker(&executor, authentication, unaffected.clone(), &fills).await?;
            let mut before = Vec::new();
            for key in &keys {
                before
                    .push(inventory_marker(&executor, authentication, key.clone(), &fills).await?);
            }

            request.invalidate(&executor).await?;

            for (key, stale) in keys.into_iter().zip(before) {
                let fresh = inventory_marker(&executor, authentication, key, &fills).await?;
                assert!(
                    !Arc::ptr_eq(&stale, &fresh),
                    "audit must refresh child inventories and pages"
                );
            }
            let still_retained =
                inventory_marker(&executor, authentication, unaffected, &fills).await?;
            assert!(
                Arc::ptr_eq(&retained, &still_retained),
                "refresh must preserve another tenant's inventory"
            );
            assert_eq!(provider.authorizations.load(Ordering::SeqCst), 0);
            Ok(())
        })
        .await
}

#[tokio::test]
async fn audit_policy_tests_refresh_child_inventories_and_preserve_other_tenants()
-> eyre::Result<()> {
    let authentication = AzureBearerAuthenticationSelection {
        account_id: "11111111-1111-1111-1111-111111111111".parse()?,
        tenant_id: "22222222-2222-2222-2222-222222222222".parse()?,
        auth_source: AzureBearerAuthSource::AzureCli,
    };
    let principal_inventories = vec![
        EntraPrincipalListRequest { authentication }.cache_key()?,
        EntraUserListRequest { authentication }.cache_key()?,
        EntraSecurityGroupListRequest { authentication }.cache_key()?,
        EntraServicePrincipalListRequest { authentication }.cache_key()?,
    ];
    assert_audit_refreshes_inventories(
        AzureAdminCorrespondenceAuditRequest { authentication },
        authentication,
        principal_inventories.clone(),
    )
    .await?;
    assert_audit_refreshes_inventories(
        AzureServicePrincipalPasswordAuditRequest {
            authentication,
            observed_at: "2020-01-01T00:00:00Z".parse()?,
        },
        authentication,
        principal_inventories.clone(),
    )
    .await?;
    let mut unknown_principal_inventories = principal_inventories;
    unknown_principal_inventories.extend([
        AzureRoleDefinitionsAndAssignmentsListRequest { authentication }.cache_key()?,
        AzureRoleDefinitionListRequest { authentication }.cache_key()?,
        AzureRoleAssignmentListRequest { authentication }.cache_key()?,
    ]);
    assert_audit_refreshes_inventories(
        AzureUnknownPrincipalAuditRequest { authentication },
        authentication,
        unknown_principal_inventories,
    )
    .await?;
    assert_audit_refreshes_inventories(
        AzureResourceTagAuditRequest { authentication },
        authentication,
        vec![AzureResourceListRequest { authentication }.cache_key()?],
    )
    .await
}
