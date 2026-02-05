use clap::Args;
use cloud_terrastodon_azure_devops::prelude::AzureDevOpsUserArgument;
use cloud_terrastodon_azure_devops::prelude::AzureDevOpsUserId;
use cloud_terrastodon_azure_devops::prelude::AzureDevOpsUserLicenseEntitlement;
use eyre::bail;

#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsLicenseEntitlementUserMatcher {
    /// The GUID that corresponds to the user in Azure DevOps, not to be confused with their Entra object id.
    #[arg(long)]
    pub user_devops_id: Option<AzureDevOpsUserId>,

    /// The email that corresponds to the user in Azure DevOps
    #[arg(long)]
    pub user_email: Option<String>,
}

pub type AzureDevOpsLicenseEntitlementPredicate<'a> =
    Box<dyn Fn(&AzureDevOpsUserLicenseEntitlement) -> bool + 'a>;
impl AzureDevOpsLicenseEntitlementUserMatcher {
    pub fn as_predicate<'a>(&'a self) -> eyre::Result<AzureDevOpsLicenseEntitlementPredicate<'a>> {
        Ok(match (&self.user_devops_id, &self.user_email) {
            (None, None) => {
                bail!("No user filter was provided");
            }
            (Some(devops_user_id), None) => Box::new(move |e| e.user_id == *devops_user_id),
            (None, Some(user_email)) => {
                Box::new(move |e| e.user.unique_name.eq_ignore_ascii_case(user_email))
            }
            (Some(devops_user_id), Some(user_email)) => Box::new(move |e| {
                e.user_id == *devops_user_id && e.user.unique_name.eq_ignore_ascii_case(user_email)
            }),
        })
    }
    pub fn as_argument<'a>(&'a self) -> eyre::Result<AzureDevOpsUserArgument<'a>> {
        AzureDevOpsUserArgument::try_from(self)
    }
}

impl<'a> TryFrom<&'a AzureDevOpsLicenseEntitlementUserMatcher> for AzureDevOpsUserArgument<'a> {
    type Error = eyre::Error;

    fn try_from(
        matcher: &'a AzureDevOpsLicenseEntitlementUserMatcher,
    ) -> Result<Self, eyre::Report> {
        Ok(match (&matcher.user_devops_id, &matcher.user_email) {
            (Some(devops_user_id), None) => AzureDevOpsUserArgument::IdRef(devops_user_id),
            (None, Some(user_email)) => AzureDevOpsUserArgument::EmailRef(user_email),
            (Some(devops_user_id), Some(_)) => AzureDevOpsUserArgument::IdRef(devops_user_id),
            (None, None) => bail!("No user filter was provided"),
        })
    }
}
