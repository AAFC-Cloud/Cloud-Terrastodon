use crate::reflected_table_group::ReflectedTableGroup;
use crate::scope_role_assignment_row::ScopeRoleAssignmentRow;
use crate::scope_role_assignment_scope_filter::ScopeRoleAssignmentScopeFilter;
use cloud_terrastodon_azure::HasSlug;
use eframe::egui::Id;

/// How the visible assignment rows are partitioned into expandable groups.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub(crate) enum ScopeRoleAssignmentGrouping {
    #[default]
    Role,
    PrincipalType,
    Scope,
    None,
}

impl ScopeRoleAssignmentGrouping {
    pub const ALL: [Self; 4] = [Self::Role, Self::PrincipalType, Self::Scope, Self::None];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Role => "Role",
            Self::PrincipalType => "Principal type",
            Self::Scope => "Scope",
            Self::None => "None",
        }
    }

    pub fn group(self, row: &ScopeRoleAssignmentRow) -> ReflectedTableGroup {
        let (id, label) = match self {
            Self::Role => (
                Id::new(row.assignment.role_definition_id.name()),
                row.role.clone(),
            ),
            Self::PrincipalType => (Id::new(&row.principal_type), row.principal_type.clone()),
            Self::Scope => (
                Id::new(&row.scope),
                format!(
                    "{} · {}",
                    ScopeRoleAssignmentScopeFilter::of(row).label(),
                    row.scope_label,
                ),
            ),
            Self::None => (Id::new(()), "Assignments".to_owned()),
        };
        ReflectedTableGroup {
            id: id.with(self),
            label,
        }
    }
}
