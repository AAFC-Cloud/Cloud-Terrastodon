use crate::AzureDevOpsLicenseRule;
use crate::AzureDevOpsUserId;
use crate::LastAccessedDate;
use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;
use cloud_terrastodon_azure_types::ArbitraryJson;
use std::collections::BTreeMap;

/// User entitlement returned after an accepted [7.1 license update](https://learn.microsoft.com/en-us/rest/api/azure/devops/memberentitlementmanagement/user-entitlements/update-user-entitlement?view=azure-devops-rest-7.1#userentitlement).
#[derive(facet::Facet, Debug)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDevOpsLicenseEntitlementUpdateResponse {
    pub access_level: AzureDevOpsLicenseRule,
    /// The documented update response omits this read-only creation timestamp.
    pub date_created: Option<DateTime<Utc>>,
    pub extensions: Vec<ArbitraryJson>,
    pub group_assignments: Vec<ArbitraryJson>,
    pub id: AzureDevOpsUserId,
    pub last_accessed_date: LastAccessedDate,
    pub project_entitlements: Vec<ArbitraryJson>,
    pub user: ArbitraryJson,
}

impl<'a> Arbitrary<'a> for AzureDevOpsLicenseEntitlementUpdateResponse {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        fn objects(
            u: &mut arbitrary::Unstructured<'_>,
            kind: &str,
        ) -> arbitrary::Result<Vec<ArbitraryJson>> {
            (0..u.int_in_range(0..=3usize)?)
                .map(|_| {
                    let id = u32::arbitrary(u)?;
                    ArbitraryJson::try_from_facet(&BTreeMap::from([
                        ("id", format!("{kind}-{id}")),
                        ("displayName", format!("Example {kind} {id}")),
                    ]))
                    .map_err(|_| arbitrary::Error::IncorrectFormat)
                })
                .collect()
        }

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
        // These opaque fields still represent objects. Preserve their shape and
        // the enclosing user identity while delegating JSON encoding to the wrapper.
        let user = ArbitraryJson::try_from_facet(&BTreeMap::from([
            ("id", id.to_string()),
            ("subjectKind", "user".to_owned()),
            ("displayName", format!("Example user {id}")),
            ("principalName", format!("{id}@example.invalid")),
        ]))
        .map_err(|_| arbitrary::Error::IncorrectFormat)?;
        Ok(Self {
            access_level: AzureDevOpsLicenseRule::arbitrary(u)?,
            date_created: Some(date_created),
            extensions: objects(u, "extension")?,
            group_assignments: objects(u, "group")?,
            id,
            last_accessed_date,
            project_entitlements: objects(u, "project")?,
            user,
        })
    }
}

cloud_terrastodon_registry::register_arbitrary!(
    AzureDevOpsLicenseEntitlementUpdateResponse,
    associated = REGISTERED_ARBITRARY
);
