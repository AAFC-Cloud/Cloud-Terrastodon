pub mod list;
pub mod update;
pub mod show;
pub mod revoke;

pub use list::AzureDevOpsLicenseEntitlementUserListArgs;
pub use update::{
    AzureDevOpsLicenseEntitlementUserUpdateArgs,
    AzureDevOpsLicenseEntitlementUserUpdateTuiArgs,
};
pub use show::AzureDevOpsLicenseEntitlementUserShowArgs;
pub use revoke::AzureDevOpsLicenseEntitlementUserRevokeArgs;
