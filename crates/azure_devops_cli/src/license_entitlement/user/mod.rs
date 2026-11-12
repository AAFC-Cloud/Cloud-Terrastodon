mod azure_devops_license_entitlement_user_cli;
pub use azure_devops_license_entitlement_user_cli::AzureDevOpsLicenseEntitlementUserArgs;
pub mod list;
pub mod revoke;
pub mod show;
pub mod summary;
pub mod update;

pub use list::AzureDevOpsLicenseEntitlementUserListArgs;
pub use revoke::AzureDevOpsLicenseEntitlementUserRevokeArgs;
pub use show::AzureDevOpsLicenseEntitlementUserShowArgs;
pub use summary::AzureDevOpsLicenseEntitlementUserSummaryArgs;
pub use update::AzureDevOpsLicenseEntitlementUserUpdateArgs;
pub use update::AzureDevOpsLicenseEntitlementUserUpdateTuiArgs;
