use cloud_terrastodon_azure::EntraGroupId;

/// An independently scheduled inventory used to build an Entra group report.
pub(super) enum AzureEntraGroupShowLookup {
    Members(EntraGroupId),
    Owners(EntraGroupId),
    RoleAssignments,
}
