use cloud_terrastodon_azure::AzureDenyAssignmentResource;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::ScopeImpl;
use facet::Facet;
use std::sync::Arc;

#[derive(Facet)]
pub(crate) struct ScopeDenyAssignmentRow {
    pub assignment: Arc<AzureDenyAssignmentResource>,
    pub name: String,
    pub inheritance: String,
    pub scope: ScopeImpl,
    pub principal_count: usize,
}

impl ScopeDenyAssignmentRow {
    pub fn applicable(
        assignment: Arc<AzureDenyAssignmentResource>,
        selected_scope: &str,
    ) -> Option<Self> {
        let properties = &assignment.properties;
        let scope = properties.scope.clone();
        let direct = scope.expanded_form().eq_ignore_ascii_case(selected_scope);
        if !direct && properties.do_not_apply_to_child_scopes {
            return None;
        }
        Some(Self {
            name: properties.deny_assignment_name.clone(),
            inheritance: if direct { "Direct" } else { "Inherited" }.into(),
            scope,
            principal_count: properties.principals.len(),
            assignment,
        })
    }

    pub fn matches_principal_id(&self, search: &str) -> bool {
        self.assignment
            .properties
            .principals
            .iter()
            .any(|principal| principal.id.to_string().contains(search))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_azure::AzureDenyAssignmentResourceProperties;
    use cloud_terrastodon_azure::ResourceType;
    use eyre::Result;

    #[test]
    fn deny_marked_for_this_scope_only_is_never_shown_as_inherited() -> Result<()> {
        const SUBSCRIPTION: &str = "/subscriptions/00000000-0000-0000-0000-000000000001";
        const GROUP: &str =
            "/subscriptions/00000000-0000-0000-0000-000000000001/resourceGroups/example";
        let assignment = Arc::new(AzureDenyAssignmentResource {
            id: format!("{SUBSCRIPTION}/providers/Microsoft.Authorization/denyAssignments/00000000-0000-0000-0000-000000000002").parse()?,
            name: "00000000-0000-0000-0000-000000000002".parse()?,
            resource_type: ResourceType::MICROSOFT_DOT_AUTHORIZATION_SLASH_DENYASSIGNMENTS,
            properties: AzureDenyAssignmentResourceProperties {
                deny_assignment_name: "Example".into(),
                description: String::new(),
                do_not_apply_to_child_scopes: true,
                exclude_principals: Vec::new(),
                is_system_protected: true,
                permissions: Vec::new(),
                principals: Vec::new(),
                scope: SUBSCRIPTION.parse()?,
                condition: None,
                condition_version: None,
                created_by: None,
                created_on: None,
                updated_by: None,
                updated_on: None,
            },
        });
        let direct = ScopeDenyAssignmentRow::applicable(Arc::clone(&assignment), SUBSCRIPTION);
        assert!(direct.is_some_and(|row| row.inheritance == "Direct"));
        assert!(ScopeDenyAssignmentRow::applicable(assignment, GROUP).is_none());
        Ok(())
    }
}
