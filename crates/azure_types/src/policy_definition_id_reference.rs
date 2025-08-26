use crate::prelude::PolicyDefinitionId;
use crate::prelude::PolicySetDefinitionId;
use crate::scopes::Scope;
use eyre::bail;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use std::str::FromStr;

/// An ID for either a policy definition or a policy set definition
#[derive(Debug, PartialEq, Eq)]
pub enum PolicyDefinitionIdReference {
    PolicyDefinitionId(PolicyDefinitionId),
    PolicySetDefinitionId(PolicySetDefinitionId),
}
impl Scope for PolicyDefinitionIdReference {
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

    fn as_scope_impl(&self) -> crate::prelude::ScopeImpl {
        match self {
            PolicyDefinitionIdReference::PolicyDefinitionId(policy_definition_id) => {
                policy_definition_id.as_scope_impl()
            }
            PolicyDefinitionIdReference::PolicySetDefinitionId(policy_set_definition_id) => {
                policy_set_definition_id.as_scope_impl()
            }
        }
    }

    fn kind(&self) -> crate::prelude::ScopeImplKind {
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

impl Serialize for PolicyDefinitionIdReference {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.expanded_form())
    }
}

impl<'de> Deserialize<'de> for PolicyDefinitionIdReference {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        let id = PolicyDefinitionIdReference::try_from_expanded(expanded.as_str())
            .map_err(|e| serde::de::Error::custom(format!("{e}")))?;
        Ok(id)
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
