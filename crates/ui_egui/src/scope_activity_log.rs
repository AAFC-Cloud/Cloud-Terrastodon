use crate::scope_activity_period::ScopeActivityPeriod;
use cloud_terrastodon_azure::AzureActivityLogEvent;
use cloud_terrastodon_azure::AzureActivityLogScope;
use cloud_terrastodon_command::app_work::Loadable;
use std::sync::Arc;

/// The last Activity Log query submitted from a resource's Activity Log page.
pub(crate) struct ScopeActivityLog {
    pub scope: AzureActivityLogScope,
    pub period: ScopeActivityPeriod,
    pub utc_window: Option<String>,
    pub response: Loadable<Arc<Vec<Arc<AzureActivityLogEvent>>>>,
}

impl ScopeActivityLog {
    pub fn is_loading(&self) -> bool {
        matches!(self.response, Loadable::Loading { .. })
    }
}
