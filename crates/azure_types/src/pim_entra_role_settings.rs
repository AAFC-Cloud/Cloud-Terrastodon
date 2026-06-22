use eyre::Result;
use eyre::bail;
use facet_json::RawJson;
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug, PartialEq, facet::Facet)]
pub struct PimEntraRoleSettings {
    id: Uuid,
    #[facet(rename = "roleDefinitionId")]
    role_definition_id: Uuid,
    #[facet(rename = "userMemberSettings")]
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

#[derive(Debug, PartialEq, facet::Facet)]
#[facet(opaque, proxy = PimEntraRoleSettingsRuleProxy)]
#[repr(C)]
pub enum PimEntraRoleSettingsRule {
    ExpirationRule(ExpirationRuleSetting),
    MfaRule(RawJson<'static>),
    JustificationRule(RawJson<'static>),
    ApprovalRule(RawJson<'static>),
    TicketingRule(RawJson<'static>),
    AcrsRule(RawJson<'static>),
    AttributeConditionRule(RawJson<'static>),
}

#[derive(Debug, PartialEq, Eq, facet::Facet)]
pub struct PimEntraRoleSettingsRuleProxy {
    #[facet(rename = "ruleIdentifier")]
    rule_identifier: String,
    setting: RawJson<'static>,
}

impl TryFrom<PimEntraRoleSettingsRuleProxy> for PimEntraRoleSettingsRule {
    type Error = eyre::Error;

    fn try_from(value: PimEntraRoleSettingsRuleProxy) -> Result<Self, Self::Error> {
        let raw_setting = value.setting.as_str();
        Ok(match value.rule_identifier.as_str() {
            "ExpirationRule" => {
                let setting_json = facet_json::from_str::<String>(raw_setting)
                    .unwrap_or_else(|_| raw_setting.to_string());
                Self::ExpirationRule(facet_json::from_str(&setting_json)?)
            }
            "MfaRule" => Self::MfaRule(value.setting),
            "JustificationRule" => Self::JustificationRule(value.setting),
            "ApprovalRule" => Self::ApprovalRule(value.setting),
            "TicketingRule" => Self::TicketingRule(value.setting),
            "AcrsRule" => Self::AcrsRule(value.setting),
            "AttributeConditionRule" => Self::AttributeConditionRule(value.setting),
            _ => Self::AttributeConditionRule(value.setting),
        })
    }
}

impl TryFrom<&PimEntraRoleSettingsRule> for PimEntraRoleSettingsRuleProxy {
    type Error = eyre::Error;

    fn try_from(value: &PimEntraRoleSettingsRule) -> Result<Self, Self::Error> {
        let (rule_identifier, setting) = match value {
            PimEntraRoleSettingsRule::ExpirationRule(setting) => (
                "ExpirationRule",
                RawJson::from_owned(facet_json::to_string(setting)?),
            ),
            PimEntraRoleSettingsRule::MfaRule(setting) => ("MfaRule", setting.clone()),
            PimEntraRoleSettingsRule::JustificationRule(setting) => {
                ("JustificationRule", setting.clone())
            }
            PimEntraRoleSettingsRule::ApprovalRule(setting) => ("ApprovalRule", setting.clone()),
            PimEntraRoleSettingsRule::TicketingRule(setting) => ("TicketingRule", setting.clone()),
            PimEntraRoleSettingsRule::AcrsRule(setting) => ("AcrsRule", setting.clone()),
            PimEntraRoleSettingsRule::AttributeConditionRule(setting) => {
                ("AttributeConditionRule", setting.clone())
            }
        };
        Ok(Self {
            rule_identifier: rule_identifier.to_string(),
            setting,
        })
    }
}

#[derive(Debug, PartialEq, facet::Facet)]
pub struct ExpirationRuleSetting {
    #[facet(rename = "maximumGrantPeriodInMinutes")]
    maximum_grant_period_in_minutes: u32,
    #[facet(rename = "permanentAssignment")]
    permanent_assignment: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_json_round_trips() -> eyre::Result<()> {
        let json = r#"
        {
            "id": "00000000-0000-0000-0000-000000000000",
            "roleDefinitionId": "11111111-1111-1111-1111-111111111111",
            "userMemberSettings": [
                {
                    "ruleIdentifier": "ExpirationRule",
                    "setting": "{\"maximumGrantPeriodInMinutes\":120,\"permanentAssignment\":false}"
                }
            ]
        }
        "#;

        let settings = facet_json::from_str::<PimEntraRoleSettings>(json)?;
        assert_eq!(
            settings.get_maximum_grant_period()?,
            Duration::from_mins(120)
        );
        let reparsed =
            facet_json::from_str::<PimEntraRoleSettings>(&facet_json::to_string(&settings)?)?;
        assert_eq!(settings, reparsed);
        Ok(())
    }
}
