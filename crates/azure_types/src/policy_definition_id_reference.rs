use crate::PolicyDefinitionId;
use crate::PolicySetDefinitionId;
use crate::scopes::Scope;
use eyre::bail;
use std::str::FromStr;

/// An ID for either a policy definition or a policy set definition
#[derive(Debug, PartialEq, Eq, facet::Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum PolicyDefinitionIdReference {
    PolicyDefinitionId(PolicyDefinitionId),
    PolicySetDefinitionId(PolicySetDefinitionId),
}
crate::impl_facet_string_proxy!(PolicyDefinitionIdReference, value => value.expanded_form());
impl Scope for PolicyDefinitionIdReference {
    type Err = <Self as std::str::FromStr>::Err;
    fn expanded_form(&self) -> String {
        match self {
            PolicyDefinitionIdReference::PolicyDefinitionId(policy_definition_id) => {
                policy_definition_id.expanded_form()
            }
            PolicyDefinitionIdReference::PolicySetDefinitionId(policy_set_definition_id) => {
                policy_set_definition_id.expanded_form()
            }
        }
    }

    fn try_from_expanded(expanded: &str) -> eyre::Result<Self> {
        Self::from_str(expanded)
    }

    fn as_scope_impl(&self) -> crate::ScopeImpl {
        match self {
            PolicyDefinitionIdReference::PolicyDefinitionId(policy_definition_id) => {
                policy_definition_id.as_scope_impl()
            }
            PolicyDefinitionIdReference::PolicySetDefinitionId(policy_set_definition_id) => {
                policy_set_definition_id.as_scope_impl()
            }
        }
    }

    fn kind(&self) -> crate::ScopeImplKind {
        match self {
            PolicyDefinitionIdReference::PolicyDefinitionId(policy_definition_id) => {
                policy_definition_id.kind()
            }
            PolicyDefinitionIdReference::PolicySetDefinitionId(policy_set_definition_id) => {
                policy_set_definition_id.kind()
            }
        }
    }
}

impl FromStr for PolicyDefinitionIdReference {
    type Err = eyre::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match (
            PolicySetDefinitionId::try_from_expanded(s),
            PolicyDefinitionId::try_from_expanded(s),
        ) {
            (Ok(a), Ok(b)) => {
                bail!(
                    "Matched both PolicyDefinitionId and PolicySetDefinitionId, this shouldnt happen. Got {} and {}",
                    a.expanded_form(),
                    b.expanded_form()
                );
            }
            (Ok(a), Err(_)) => Ok(PolicyDefinitionIdReference::PolicySetDefinitionId(a)),
            (Err(_), Ok(b)) => Ok(PolicyDefinitionIdReference::PolicyDefinitionId(b)),
            (Err(a), Err(b)) => {
                bail!(
                    "Failed to determine policy definition id kind. PolicySetDefinitionIdError={a}, PolicyDefinitionIdError={b}"
                )
            }
        }
    }
}

impl PartialEq<PolicyDefinitionId> for PolicyDefinitionIdReference {
    fn eq(&self, other: &PolicyDefinitionId) -> bool {
        match self {
            PolicyDefinitionIdReference::PolicyDefinitionId(policy_definition_id) => {
                policy_definition_id == other
            }
            _ => false,
        }
    }
}
impl PartialEq<PolicySetDefinitionId> for PolicyDefinitionIdReference {
    fn eq(&self, other: &PolicySetDefinitionId) -> bool {
        match self {
            PolicyDefinitionIdReference::PolicySetDefinitionId(policy_set_definition_id) => {
                policy_set_definition_id == other
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_round_trips_through_facet() -> eyre::Result<()> {
        for expanded in [
            "/providers/Microsoft.Authorization/policyDefinitions/55555555-5555-5555-5555-555555555555",
            "/providers/Microsoft.Authorization/policySetDefinitions/55555555-5555-5555-5555-555555555555",
        ] {
            let json = facet_json::to_string(expanded)?;
            let id: PolicyDefinitionIdReference = facet_json::from_str(&json)?;
            assert_eq!(facet_json::to_string(&id)?, json);
            assert_eq!(id.expanded_form(), expanded);
        }
        Ok(())
    }
}
