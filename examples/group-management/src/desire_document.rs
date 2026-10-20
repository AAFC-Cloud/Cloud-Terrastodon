use crate::domain::DesiredGroup;
use crate::domain::DesiredPrincipal;
use crate::domain::ReconciliationMode;
use cloud_terrastodon::azure_types::AzureTenantId;
use eyre::Context;
use eyre::Report;
use eyre::Result;
use eyre::ensure;
use std::collections::HashSet;
use std::path::Path;

mod try_plan;

/// Validated declarative input for multiple groups, with source provenance.
///
/// Declarations stay private so mutation cannot introduce duplicate identities,
/// invalid references, or nonempty ignored relationships after construction.
#[derive(Debug)]
pub struct DesireDocument {
    groups: Vec<DesiredGroup>,
    source_digest: String,
}

impl DesireDocument {
    /// Validate group declarations and associate their supplied source provenance.
    ///
    /// `source_digest` identifies the originating document; this constructor does
    /// not compute it from the parsed groups.
    pub fn try_new(groups: Vec<DesiredGroup>, source_digest: String) -> Result<Self> {
        let document = Self {
            groups,
            source_digest,
        };
        document.validate()?;
        Ok(document)
    }

    /// Borrow the validated group declarations without allowing mutation.
    #[must_use]
    pub fn groups(&self) -> &[DesiredGroup] {
        &self.groups
    }

    /// Identify the original source document, including its formatting.
    ///
    /// File loading computes a BLAKE3 fingerprint of the original bytes. This
    /// provenance is preserved when selecting a tenant; it does not fingerprint
    /// a reserialization of the selected groups.
    #[must_use]
    pub fn source_digest(&self) -> &str {
        &self.source_digest
    }

    /// Read a JSON array of group declarations and validate local policy.
    ///
    /// Loading does not authenticate or read directory data. The source digest
    /// is computed from the same bytes used to parse the declarations.
    pub fn try_from_path(path: &Path) -> Result<Self> {
        let source = std::fs::read_to_string(path)
            .wrap_err_with(|| format!("Reading desired groups from {}", path.display()))?;
        let groups: Vec<DesiredGroup> = facet_json::from_str(&source)
            .wrap_err_with(|| format!("Parsing desired groups in {}", path.display()))?;
        Self::try_new(
            groups,
            format!("blake3:{}", blake3::hash(source.as_bytes())),
        )
        .wrap_err_with(|| format!("Validating desired groups in {}", path.display()))
    }

    /// Select one tenant without changing the original document's provenance.
    ///
    /// Removing declarations preserves validation invariants. An explicitly
    /// selected tenant must have at least one group in this document.
    pub fn select_tenant(mut self, tenant: AzureTenantId) -> Result<Self> {
        self.groups.retain(|group| group.tenant_id == tenant);
        ensure!(
            !self.groups.is_empty(),
            "The document contains no groups for explicitly selected tenant {tenant}"
        );
        Ok(self)
    }

    fn validate(&self) -> Result<()> {
        let mut identities = HashSet::new();
        for group in &self.groups {
            ensure!(!group.tenant_id.is_nil(), "tenantId must not be nil");
            ensure!(!group.group_id.is_nil(), "groupId must not be nil");
            ensure!(
                identities.insert((group.tenant_id, group.group_id)),
                "Duplicate group {} in tenant {}",
                group.group_id,
                group.tenant_id
            );
            ensure!(
                !group.display_name.trim().is_empty(),
                "Group {} displayName must not be blank",
                group.group_id
            );
            validate_relationship(group.members_mode, &group.members)
                .wrap_err_with(|| format!("Invalid members for group {}", group.group_id))?;
            validate_relationship(group.owners_mode, &group.owners)
                .wrap_err_with(|| format!("Invalid owners for group {}", group.group_id))?;
        }
        Ok(())
    }
}

impl TryFrom<&Path> for DesireDocument {
    type Error = Report;

    fn try_from(path: &Path) -> Result<Self> {
        Self::try_from_path(path)
    }
}

fn validate_relationship(mode: ReconciliationMode, principals: &[DesiredPrincipal]) -> Result<()> {
    ensure!(
        mode != ReconciliationMode::Ignore || principals.is_empty(),
        "The ignore mode requires an empty relationship list"
    );
    let mut ids = HashSet::new();
    let mut upns = HashSet::new();
    for principal in principals {
        ensure!(
            principal.id.is_some() || principal.user_principal_name.is_some(),
            "A principal requires id, userPrincipalName, or both"
        );
        if let Some(id) = principal.id {
            ensure!(!id.is_nil(), "A principal id must not be nil");
            ensure!(ids.insert(id), "Duplicate principal id {id}");
        }
        if let Some(upn) = &principal.user_principal_name {
            ensure!(
                upns.insert(upn.comparison_key()),
                "Duplicate userPrincipalName {upn}"
            );
        }
        ensure!(
            principal
                .display_name
                .as_ref()
                .is_none_or(|name| !name.trim().is_empty()),
            "A principal displayName must not be blank"
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::DesireDocument;
    use crate::domain::DesiredGroup;
    use crate::domain::DesiredPrincipal;
    use crate::domain::ReconciliationMode;

    fn group() -> DesiredGroup {
        DesiredGroup {
            tenant_id: "10000000-0000-0000-0000-000000000001".parse().unwrap(),
            group_id: "20000000-0000-0000-0000-000000000001".parse().unwrap(),
            display_name: "Example group".parse().unwrap(),
            members_mode: ReconciliationMode::Ignore,
            members: Vec::new(),
            owners_mode: ReconciliationMode::Ignore,
            owners: Vec::new(),
        }
    }

    #[test]
    fn rejects_principals_in_ignored_relationships() {
        let principal = DesiredPrincipal {
            id: Some("30000000-0000-0000-0000-000000000001".parse().unwrap()),
            user_principal_name: None,
            display_name: None,
        };
        let mut desired = group();
        assert!(DesireDocument::try_new(vec![desired.clone()], "synthetic-input".into()).is_ok());
        desired.members.push(principal.clone());
        let error =
            DesireDocument::try_new(vec![desired.clone()], "synthetic-input".into()).unwrap_err();
        assert!(format!("{error:#}").contains("ignore mode requires an empty"));
        desired.members_mode = ReconciliationMode::Additive;
        assert!(DesireDocument::try_new(vec![desired.clone()], "synthetic-input".into()).is_ok());
        desired.owners.push(principal);
        let error = DesireDocument::try_new(vec![desired], "synthetic-input".into()).unwrap_err();
        assert!(format!("{error:#}").contains("ignore mode requires an empty"));
    }

    #[test]
    fn rejects_whitespace_only_group_annotations() {
        let mut desired = group();
        desired.display_name = "   ".parse().unwrap();
        assert!(DesireDocument::try_new(vec![desired], "synthetic-input".into()).is_err());
    }

    #[test]
    fn rejects_duplicate_groups_within_tenant_but_preserves_tenant_identity() {
        let original = group();
        assert!(
            DesireDocument::try_new(
                vec![original.clone(), original.clone()],
                "synthetic-input".into(),
            )
            .is_err()
        );
        let mut other_tenant = original.clone();
        other_tenant.tenant_id = "10000000-0000-0000-0000-000000000002".parse().unwrap();
        assert!(
            DesireDocument::try_new(vec![original, other_tenant], "synthetic-input".into()).is_ok()
        );
    }

    #[test]
    fn rejects_duplicate_case_insensitive_user_principal_names() {
        let mut desired = group();
        desired.members_mode = ReconciliationMode::Additive;
        desired.members = ["Alex@example.invalid", "alex@example.invalid"]
            .map(|upn| DesiredPrincipal {
                id: None,
                user_principal_name: Some(upn.parse().unwrap()),
                display_name: None,
            })
            .to_vec();
        assert!(DesireDocument::try_new(vec![desired], "synthetic-input".into()).is_err());
    }

    #[test]
    fn rejects_display_name_only_and_nil_principal_references() {
        let mut desired = group();
        desired.owners_mode = ReconciliationMode::Additive;
        desired.owners = vec![DesiredPrincipal {
            id: None,
            user_principal_name: None,
            display_name: Some("Readable but not an identity".to_owned()),
        }];
        assert!(DesireDocument::try_new(vec![desired.clone()], "synthetic-input".into()).is_err());
        desired.owners[0].id = Some("00000000-0000-0000-0000-000000000000".parse().unwrap());
        assert!(DesireDocument::try_new(vec![desired], "synthetic-input".into()).is_err());
    }
}
