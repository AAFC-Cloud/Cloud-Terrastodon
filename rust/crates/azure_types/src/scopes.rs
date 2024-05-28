use crate::management_groups::ManagementGroupId;
use crate::policy_assignments::PolicyAssignmentId;
use crate::policy_definitions::PolicyDefinitionId;
use crate::policy_set_definitions::PolicySetDefinitionId;
use crate::prelude::ResourceGroupId;
use crate::prelude::RoleAssignmentId;
use anyhow::Error;
use anyhow::Result;
use std::str::FromStr;

pub trait Scope: Sized {
    fn expanded_form(&self) -> &str;
    fn short_name(&self) -> &str;
    fn from_expanded(expanded: &str) -> Result<Self>;
}

#[derive(Debug)]
pub enum ScopeError {
    Malformed,
    InvalidName,
    Unrecognized,
}
impl std::error::Error for ScopeError {}
impl std::fmt::Display for ScopeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ScopeError::Malformed => "malformed scope",
            ScopeError::InvalidName => "invalid name in scope",
            ScopeError::Unrecognized => "unrecognized scope kind",
        })
    }
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum ScopeImpl {
    ManagementGroup(ManagementGroupId),
    PolicyDefinition(PolicyDefinitionId),
    PolicySetDefinition(PolicySetDefinitionId),
    PolicyAssignment(PolicyAssignmentId),
    ResourceGroup(ResourceGroupId),
    RoleAssignment(RoleAssignmentId),
}
impl Scope for ScopeImpl {
    fn expanded_form(&self) -> &str {
        match self {
            ScopeImpl::ManagementGroup(m) => m.expanded_form(),
            ScopeImpl::PolicyDefinition(p) => p.expanded_form(),
            ScopeImpl::PolicySetDefinition(p) => p.expanded_form(),
            ScopeImpl::PolicyAssignment(p) => p.expanded_form(),
            ScopeImpl::ResourceGroup(r) => r.expanded_form(),
            ScopeImpl::RoleAssignment(r) => r.expanded_form(),
        }
    }

    fn short_name(&self) -> &str {
        match self {
            ScopeImpl::ManagementGroup(m) => m.short_name(),
            ScopeImpl::PolicyDefinition(p) => p.short_name(),
            ScopeImpl::PolicySetDefinition(p) => p.short_name(),
            ScopeImpl::PolicyAssignment(p) => p.short_name(),
            ScopeImpl::ResourceGroup(r) => r.short_name(),
            ScopeImpl::RoleAssignment(r) => r.short_name(),
        }
    }

    fn from_expanded(expanded: &str) -> Result<Self> {
        if let Ok(scope) = ManagementGroupId::from_expanded(expanded) {
            return Ok(ScopeImpl::ManagementGroup(scope));
        }
        if let Ok(scope) = PolicyDefinitionId::from_expanded(expanded) {
            return Ok(ScopeImpl::PolicyDefinition(scope));
        }
        if let Ok(scope) = PolicySetDefinitionId::from_expanded(expanded) {
            return Ok(ScopeImpl::PolicySetDefinition(scope));
        }
        if let Ok(scope) = PolicyAssignmentId::from_expanded(expanded) {
            return Ok(ScopeImpl::PolicyAssignment(scope));
        }
        Err(ScopeError::Unrecognized.into())
    }
}

impl FromStr for ScopeImpl {
    type Err = Error;

    fn from_str(s: &str) -> Result<ScopeImpl> {
        Self::from_expanded(s)
    }
}

impl std::fmt::Display for ScopeImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScopeImpl::ManagementGroup(x) => {
                f.write_fmt(format_args!("ManagementGroup({})", x.expanded_form()))
            }
            ScopeImpl::PolicyDefinition(x) => {
                f.write_fmt(format_args!("PolicyDefinition({})", x.expanded_form()))
            }
            ScopeImpl::PolicySetDefinition(x) => {
                f.write_fmt(format_args!("PolicySetDefinition({})", x.expanded_form()))
            }
            ScopeImpl::PolicyAssignment(x) => {
                f.write_fmt(format_args!("PolicyAssignment({})", x.expanded_form()))
            }
            ScopeImpl::ResourceGroup(x) => {
                f.write_fmt(format_args!("ResourceGroup({})", x.expanded_form()))
            },
            ScopeImpl::RoleAssignment(x) => {
                f.write_fmt(format_args!("RoleAssignment({})", x.expanded_form()))
            },
        }
    }
}

/*
fn is_valid_guid(name: &str) -> bool {
    let sections: Vec<&str> = name.split('-').collect();

    if sections.len() != 5 {
        return false;
    }

    let expected_lengths = [8, 4, 4, 4, 12];
    for (section, &expected_length) in sections.iter().zip(&expected_lengths) {
        if section.len() != expected_length {
            return false;
        }
        if !section.chars().all(|c| c.is_ascii_hexdigit()) {
            return false;
        }
    }

    true
}
*/

// #[derive(Debug, Clone)]
// pub enum Scoped<T> where T: Scope {
//     Unscoped(T),
//     ManagementGroup(T),
//     Subscription(T),
//     ResourceGroup(T)
// }
