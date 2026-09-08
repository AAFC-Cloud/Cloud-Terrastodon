use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;
use cloud_terrastodon_azure_devops_types::AzureDevOpsLicenseRule;
use cloud_terrastodon_azure_devops_types::AzureDevOpsLicenseType;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsUserArgument;
use cloud_terrastodon_azure_devops_types::AzureDevOpsUserId;
use cloud_terrastodon_azure_devops_types::LastAccessedDate;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use facet_json::RawJson;
use std::borrow::Cow;
use std::path::PathBuf;
use std::time::Duration;
use tracing::debug;

/// <https://learn.microsoft.com/en-us/rest/api/azure/devops/memberentitlementmanagement/user-entitlements/update-user-entitlement?view=azure-devops-rest-7.1>
#[must_use = "This is an unsent request, you must .await it"]
#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsUserLicenseEntitlementUpdateRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub user: AzureDevOpsUserArgument<'a>,
    pub license_kind: AzureDevOpsLicenseType,
}

pub fn update_azure_devops_user_license_entitlement<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    user: impl Into<AzureDevOpsUserArgument<'a>>,
    license_kind: AzureDevOpsLicenseType,
) -> AzureDevOpsUserLicenseEntitlementUpdateRequest<'a> {
    AzureDevOpsUserLicenseEntitlementUpdateRequest {
        org_url: Cow::Borrowed(org_url),
        user: user.into(),
        license_kind,
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsUserLicenseEntitlementUpdateRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            user: AzureDevOpsUserArgument::arbitrary(u)?.into_owned(),
            license_kind: AzureDevOpsLicenseType::arbitrary(u)?,
        })
    }
}

#[async_trait]
impl<'a> CacheableCommand for AzureDevOpsUserLicenseEntitlementUpdateRequest<'a> {
    type Output = AzureDevOpsLicenseEntitlementUpdateResponse;

    fn cache_key(&self) -> CacheKey {
        CacheKey {
            path: PathBuf::from_iter([
                "az",
                "devops",
                self.org_url.organization_name.as_ref(),
                "license",
                "entitlement",
                "update-user",
                &self.user.to_string(),
            ]),
            valid_for: Duration::ZERO, // this is an update operation, so no caching
        }
    }
    async fn run(self) -> eyre::Result<Self::Output> {
        debug!(
            user = %self.user,
            license_kind = ?self.license_kind,
            "Updating license entitlement for user",
        );
        let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
        cmd.args([
            "devops",
            "user",
            "update",
            "--user",
            &self.user.to_string(),
            "--organization",
            &self.org_url.to_string(),
            "--license-type",
            match &self.license_kind {
                AzureDevOpsLicenseType::AccountExpress => "express",
                AzureDevOpsLicenseType::AccountStakeholder => "stakeholder",
                AzureDevOpsLicenseType::AccountAdvanced => "advanced",
                AzureDevOpsLicenseType::MsdnEligible => "professional",
                AzureDevOpsLicenseType::MsdnEnterprise => "professional",
                AzureDevOpsLicenseType::MsdnProfessional => "professional",
                AzureDevOpsLicenseType::Other(s) => s,
                AzureDevOpsLicenseType::None => "none",
            },
        ]);
        cmd.cache(self.cache_key());
        Ok(cmd.run().await?)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(AzureDevOpsUserLicenseEntitlementUpdateRequest<'a>, 'a);
cloud_terrastodon_registry::register_thing!(
    AzureDevOpsUserLicenseEntitlementUpdateRequest<'static>
);
cloud_terrastodon_registry::register_arbitrary!(
    AzureDevOpsUserLicenseEntitlementUpdateRequest<'static>
);
cloud_terrastodon_registry::register_into_future!(AzureDevOpsUserLicenseEntitlementUpdateRequest<'static> => AzureDevOpsLicenseEntitlementUpdateResponse, effects = [Write]);

#[derive(facet::Facet, Debug)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsLicenseEntitlementUpdateResponse {
    pub access_level: AzureDevOpsLicenseRule,
    pub date_created: DateTime<Utc>,
    pub extensions: Vec<RawJson<'static>>,
    pub group_assignments: Vec<RawJson<'static>>,
    pub id: AzureDevOpsUserId,
    pub last_accessed_date: LastAccessedDate,
    pub project_entitlements: Vec<RawJson<'static>>,
    pub user: RawJson<'static>,
}

impl<'a> Arbitrary<'a> for AzureDevOpsLicenseEntitlementUpdateResponse {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        // Keep generated dates within 2000–2099, where RFC3339 round-trips are
        // supported, and don't generate a last access before account creation.
        const LAST_TIMESTAMP: i64 = 4_102_444_799;
        let created_timestamp = u.int_in_range(946_684_800..=LAST_TIMESTAMP)?;
        let date_created = DateTime::from_timestamp(created_timestamp, 0)
            .ok_or(arbitrary::Error::IncorrectFormat)?;
        let last_accessed_date = if bool::arbitrary(u)? {
            let timestamp = u.int_in_range(created_timestamp..=LAST_TIMESTAMP)?;
            LastAccessedDate::Some(
                DateTime::from_timestamp(timestamp, 0).ok_or(arbitrary::Error::IncorrectFormat)?,
            )
        } else {
            LastAccessedDate::Never
        };
        let id = AzureDevOpsUserId::arbitrary(u)?;

        // RawJson has no Arbitrary implementation. Generate a bounded set of
        // representative, owned JSON objects rather than unvalidated random
        // text. These samples are not fixtures of a particular server response.
        fn objects(
            u: &mut arbitrary::Unstructured<'_>,
            kind: &str,
        ) -> arbitrary::Result<Vec<RawJson<'static>>> {
            (0..u.int_in_range(0..=3)?)
                .map(|_| {
                    let number = u32::arbitrary(u)?;
                    Ok(RawJson::from_owned(format!(
                        r#"{{"id":"{kind}-{number}","displayName":"Example {kind} {number}"}}"#
                    )))
                })
                .collect()
        }

        let extensions = objects(u, "extension")?;
        let group_assignments = objects(u, "group")?;
        let project_entitlements = objects(u, "project")?;
        let user = RawJson::from_owned(format!(
            r#"{{"id":"{id}","subjectKind":"user","displayName":"Example user {id}","principalName":"{id}@example.invalid"}}"#
        ));

        Ok(Self {
            access_level: AzureDevOpsLicenseRule::arbitrary(u)?,
            date_created,
            extensions,
            group_assignments,
            id,
            last_accessed_date,
            project_entitlements,
            user,
        })
    }
}

cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsLicenseEntitlementUpdateResponse);

#[cfg(test)]
mod tests {
    use super::AzureDevOpsLicenseEntitlementUpdateResponse;
    use chrono::DateTime;
    use cloud_terrastodon_azure_devops_types::LastAccessedDate;
    use cloud_terrastodon_registry::ArbitraryBytes;
    use cloud_terrastodon_registry::Function;
    use cloud_terrastodon_registry::FunctionKind;
    use cloud_terrastodon_registry::ProductionKind;
    use cloud_terrastodon_registry::functions_from;
    use facet::Facet;
    use facet_json::RawJson;
    use std::collections::BTreeMap;

    fn registered_constructor() -> &'static Function {
        functions_from(ArbitraryBytes::SHAPE)
            .into_iter()
            .find(|function| {
                function.kind == FunctionKind::Constructor
                    && function.production_kind(AzureDevOpsLicenseEntitlementUpdateResponse::SHAPE)
                        == Some(ProductionKind::Exact)
            })
            .expect("license entitlement update response needs an exact arbitrary constructor")
    }

    #[test]
    fn license_entitlement_update_has_an_arbitrary_response_constructor() {
        let constructor = registered_constructor();
        assert!(constructor.effects.is_empty());
        assert_eq!(constructor.origin, "Arbitrary");
    }

    #[test]
    fn registered_response_constructor_generates_round_trippable_json() -> eyre::Result<()> {
        let constructor = registered_constructor();
        let mut saw_empty = [false; 3];
        let mut saw_populated = [false; 3];
        let mut saw_never_accessed = false;
        let mut saw_accessed = false;

        for seed in 0..=3 {
            // The prefix varies bounded dates, IDs, and collection contents.
            // A zero suffix keeps the arbitrary license-rule strings short.
            let mut bytes = vec![0; 256];
            bytes[..128].fill(seed);
            let mut input = ArbitraryBytes::new(bytes);
            let output = constructor.invoke_mut_boxed(&mut input)?;
            assert!(input.as_slice().len() < 256);
            let response = output
                .downcast::<AzureDevOpsLicenseEntitlementUpdateResponse>()
                .expect("registered constructor must return its exact declared response type");

            let json = facet_json::to_string(response.as_ref())?;
            let decoded: AzureDevOpsLicenseEntitlementUpdateResponse = facet_json::from_str(&json)?;
            assert_eq!(decoded.id, response.id);
            assert_eq!(decoded.date_created, response.date_created);
            assert_eq!(decoded.last_accessed_date, response.last_accessed_date);
            assert_eq!(facet_json::to_string(&decoded)?, json);

            let document: BTreeMap<String, RawJson<'static>> = facet_json::from_str(&json)?;
            let created: String = facet_json::from_str(document["dateCreated"].as_str())?;
            assert_eq!(
                DateTime::parse_from_rfc3339(&created)?.timestamp(),
                response.date_created.timestamp()
            );
            assert!((946_684_800..=4_102_444_799).contains(&response.date_created.timestamp()));
            let accessed: String = facet_json::from_str(document["lastAccessedDate"].as_str())?;
            DateTime::parse_from_rfc3339(&accessed)?;
            match response.last_accessed_date {
                LastAccessedDate::Never => saw_never_accessed = true,
                LastAccessedDate::Some(date) => {
                    saw_accessed = true;
                    assert!(date >= response.date_created);
                    assert!(date.timestamp() <= 4_102_444_799);
                }
            }

            let user: BTreeMap<String, String> = facet_json::from_str(response.user.as_str())?;
            assert_eq!(user["id"], response.id.to_string());
            assert_eq!(user["subjectKind"], "user");
            assert!(!user["displayName"].is_empty());
            for (index, key) in ["extensions", "groupAssignments", "projectEntitlements"]
                .into_iter()
                .enumerate()
            {
                let objects: Vec<BTreeMap<String, String>> =
                    facet_json::from_str(document[key].as_str())?;
                assert!(objects.len() <= 3);
                saw_empty[index] |= objects.is_empty();
                saw_populated[index] |= !objects.is_empty();
                for object in objects {
                    assert!(!object["id"].is_empty());
                    assert!(!object["displayName"].is_empty());
                }
            }
        }

        assert!(saw_empty.into_iter().all(|seen| seen));
        assert!(saw_populated.into_iter().all(|seen| seen));
        assert!(saw_never_accessed && saw_accessed);
        Ok(())
    }
}
