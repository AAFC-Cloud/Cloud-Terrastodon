use clap::Args;
use cloud_terrastodon_azure_devops::AzureDevOpsLicenseType;
use cloud_terrastodon_azure_devops::AzureDevOpsUserArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsUserId;
use cloud_terrastodon_azure_devops::AzureDevOpsUserLicenseEntitlement;
use eyre::bail;

#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsLicenseEntitlementUserMatcher {
    /// The GUID that corresponds to the user in Azure DevOps, not to be confused with their Entra object id.
    #[arg(long)]
    pub user_devops_id: Option<AzureDevOpsUserId>,

    /// The email that corresponds to the user in Azure DevOps
    #[arg(long)]
    pub user_email: Option<String>,

    /// The license that we expect the user to have
    #[arg(long)]
    pub has_license: Option<AzureDevOpsLicenseType>,
}

pub type AzureDevOpsLicenseEntitlementPredicate<'a> =
    Box<dyn Fn(&AzureDevOpsUserLicenseEntitlement) -> bool + 'a>;
impl AzureDevOpsLicenseEntitlementUserMatcher {
    pub fn as_predicate<'a>(&'a self) -> eyre::Result<AzureDevOpsLicenseEntitlementPredicate<'a>> {
        if self.user_devops_id.is_none() && self.user_email.is_none() && self.has_license.is_none()
        {
            bail!("No user filter was provided");
        }

        Ok(Box::new(move |e| {
            self.user_devops_id
                .as_ref()
                .is_none_or(|devops_user_id| &e.user_id == devops_user_id)
                && self
                    .user_email
                    .as_ref()
                    .is_none_or(|user_email| e.user.unique_name.eq_ignore_ascii_case(user_email))
                && self
                    .has_license
                    .as_ref()
                    .is_none_or(|license| &e.license == license)
        }))
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
