use crate::AzureResourceGroupId;
use crate::ManagementGroupId;
use crate::Scope;
use crate::ScopeImpl;
use arbitrary::Arbitrary;
use eyre::bail;
use facet::Facet;
use std::fmt::Display;
use std::fmt::Formatter;
use std::str::FromStr;

/// Scopes supported by the remediation creation workflow.
///
/// ARM supports creation at [resource group](https://learn.microsoft.com/en-us/rest/api/policyinsights/remediations/create-or-update-at-resource-group?view=rest-policyinsights-2024-10-01)
/// and [management group](https://learn.microsoft.com/en-us/rest/api/policyinsights/remediations/create-or-update-at-management-group?view=rest-policyinsights-2024-10-01)
/// scopes. Other platform scopes require separate SDK support.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum PolicyRemediationScope {
    ManagementGroup(ManagementGroupId),
    ResourceGroup(AzureResourceGroupId),
}

impl Display for PolicyRemediationScope {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ManagementGroup(id) => Display::fmt(id, f),
            Self::ResourceGroup(id) => Display::fmt(id, f),
        }
    }
}

impl TryFrom<&ScopeImpl> for PolicyRemediationScope {
    type Error = eyre::Report;

    fn try_from(scope: &ScopeImpl) -> eyre::Result<Self> {
        match scope {
            ScopeImpl::ManagementGroup(id) => Ok(Self::ManagementGroup(id.clone())),
            ScopeImpl::ResourceGroup(id) => Ok(Self::ResourceGroup(id.clone())),
            _ => {
                bail!(
                    "remediation creation supports management group and resource group scopes, got {scope:?}"
                );
            }
        }
    }
}

impl FromStr for PolicyRemediationScope {
    type Err = eyre::Report;

    fn from_str(value: &str) -> eyre::Result<Self> {
        if let Ok(id) = value.parse::<AzureResourceGroupId>() {
            return Ok(Self::ResourceGroup(id));
        }
        let id = ManagementGroupId::try_from_expanded(value)?;
        Ok(Self::ManagementGroup(id))
    }
}

crate::impl_facet_string_proxy!(PolicyRemediationScope, value => value.to_string());

cloud_terrastodon_registry::register_thing!(PolicyRemediationScope);
cloud_terrastodon_registry::register_arbitrary!(PolicyRemediationScope);
