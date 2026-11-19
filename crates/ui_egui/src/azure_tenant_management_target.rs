/// Distinguishes local tenant operations from account-bound directory lookups.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum AzureTenantManagementTarget {
    Catalogue,
    Directory,
}
