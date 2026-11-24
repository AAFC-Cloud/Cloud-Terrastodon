use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure::AzureActivityLogListRequest;
use cloud_terrastodon_azure::AzureActivityLogScope;
use eyre::Result;
use std::time::Duration;

/// A relative Activity Log time window whose end is fixed when submitted.
#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub(crate) enum ScopeActivityPeriod {
    Hour,
    SixHours,
    #[default]
    Day,
    Week,
    Month,
}

impl ScopeActivityPeriod {
    pub const ALL: [Self; 5] = [
        Self::Hour,
        Self::SixHours,
        Self::Day,
        Self::Week,
        Self::Month,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Hour => "Last hour",
            Self::SixHours => "Last 6 hours",
            Self::Day => "Last 24 hours",
            Self::Week => "Last 7 days",
            Self::Month => "Last 30 days",
        }
    }

    pub fn request(
        self,
        authentication: AzureBearerAuthenticationSelection,
        scope: AzureActivityLogScope,
    ) -> Result<AzureActivityLogListRequest> {
        let mut request = AzureActivityLogListRequest::last_day(authentication, scope)?;
        let hours = match self {
            Self::Hour => 1,
            Self::SixHours => 6,
            Self::Day => 24,
            Self::Week => 7 * 24,
            Self::Month => 30 * 24,
        };
        request.from = request.until - Duration::from_secs(60 * 60 * hours);
        Ok(request)
    }
}
