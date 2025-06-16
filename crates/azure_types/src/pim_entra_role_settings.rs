use eyre::Result;
use eyre::bail;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::de;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct PimEntraRoleSettings {
    id: Uuid,
    #[serde(rename = "roleDefinitionId")]
    role_definition_id: Uuid,
    #[serde(rename = "userMemberSettings")]
    user_member_settings: Vec<PimEntraRoleSettingsRule>,
}

impl PimEntraRoleSettings {
    pub fn get_maximum_grant_period(&self) -> Result<Duration> {
        for rule in &self.user_member_settings {
            if let PimEntraRoleSettingsRule::ExpirationRule(expiration) = rule {
                return Ok(Duration::from_mins(
                    expiration.maximum_grant_period_in_minutes as u64,
                ));
            }
        }
        bail!("No expiration rule found");
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "ruleIdentifier", content = "setting")]
pub enum PimEntraRoleSettingsRule {
    #[serde(deserialize_with = "json_string_deserializer")]
    ExpirationRule(ExpirationRuleSetting),
    MfaRule(Value),
    JustificationRule(Value),
    ApprovalRule(Value),
    TicketingRule(Value),
    AcrsRule(Value),
    AttributeConditionRule(Value),
}

fn json_string_deserializer<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    serde_json::from_str(&s).map_err(de::Error::custom)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExpirationRuleSetting {
    #[serde(rename = "maximumGrantPeriodInMinutes")]
    maximum_grant_period_in_minutes: u32,
    #[serde(rename = "permanentAssignment")]
    permanent_assignment: bool,
}
