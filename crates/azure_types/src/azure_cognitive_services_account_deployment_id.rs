use crate::AzureCognitiveServicesAccountDeploymentName;
use crate::AzureCognitiveServicesAccountResourceId;
use crate::scopes::Scope;
use eyre::Context;
use eyre::ContextCompat;
use eyre::Result;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use std::str::FromStr;

pub const AZURE_COGNITIVE_SERVICES_ACCOUNT_DEPLOYMENT_ID_SEGMENT: &str = "/deployments/";

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct AzureCognitiveServicesAccountDeploymentId {
    pub account_id: AzureCognitiveServicesAccountResourceId,
    pub deployment_name: AzureCognitiveServicesAccountDeploymentName,
}

impl AzureCognitiveServicesAccountDeploymentId {
    pub fn new(
        account_id: impl Into<AzureCognitiveServicesAccountResourceId>,
        deployment_name: impl Into<AzureCognitiveServicesAccountDeploymentName>,
    ) -> Self {
        Self {
            account_id: account_id.into(),
            deployment_name: deployment_name.into(),
        }
    }

    pub fn try_new<A, N>(account_id: A, deployment_name: N) -> Result<Self>
    where
        A: TryInto<AzureCognitiveServicesAccountResourceId>,
        A::Error: Into<eyre::Error>,
        N: TryInto<AzureCognitiveServicesAccountDeploymentName>,
        N::Error: Into<eyre::Error>,
    {
        let account_id = account_id
            .try_into()
            .map_err(Into::into)
            .wrap_err("Failed to convert account_id")?;
        let deployment_name = deployment_name
            .try_into()
            .map_err(Into::into)
            .wrap_err("Failed to convert deployment_name")?;
        Ok(Self {
            account_id,
            deployment_name,
        })
    }

    pub fn expanded_form(&self) -> String {
        format!(
            "{}{}{}",
            self.account_id.expanded_form(),
            AZURE_COGNITIVE_SERVICES_ACCOUNT_DEPLOYMENT_ID_SEGMENT,
            self.deployment_name
        )
    }
}

impl FromStr for AzureCognitiveServicesAccountDeploymentId {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self> {
        let (account, deployment_name) = s
            .rsplit_once(AZURE_COGNITIVE_SERVICES_ACCOUNT_DEPLOYMENT_ID_SEGMENT)
            .wrap_err("Deployment id must contain /deployments/")?;
        Ok(Self {
            account_id: account.parse::<AzureCognitiveServicesAccountResourceId>()?,
            deployment_name: deployment_name
                .parse::<AzureCognitiveServicesAccountDeploymentName>()?,
        })
    }
}

impl Serialize for AzureCognitiveServicesAccountDeploymentId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.expanded_form())
    }
}

impl<'de> Deserialize<'de> for AzureCognitiveServicesAccountDeploymentId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        expanded
            .parse()
            .map_err(|e: eyre::Error| serde::de::Error::custom(format!("{e:?}")))
    }
}

#[cfg(test)]
mod test {
    use super::AzureCognitiveServicesAccountDeploymentId;

    #[test]
    fn round_trip() -> eyre::Result<()> {
        let expanded = "/subscriptions/11111111-1111-1111-1111-111111111111/resourceGroups/my-rg/providers/Microsoft.CognitiveServices/accounts/my-openai/deployments/gpt-4.1";
        let id: AzureCognitiveServicesAccountDeploymentId = expanded.parse()?;
        assert_eq!(id.expanded_form(), expanded);
        assert_eq!(
            id.account_id
                .azure_cognitive_services_account_resource_name
                .to_string(),
            "my-openai"
        );
        assert_eq!(id.deployment_name.to_string(), "gpt-4.1");
        Ok(())
    }
}
