use crate::prelude::KeyVault;
use crate::prelude::KeyVaultAccessPolicySecretManagementOperation;
use crate::prelude::KeyVaultAccessPolicySecretPrivilege;
use crate::prelude::RoleDefinitionsAndAssignments;
use crate::prelude::RoleDefinitionsAndAssignmentsIterTools;
use crate::prelude::RolePermissionAction;
use uuid::Uuid;

impl KeyVault {
    /// Determines if the given principal can list secrets in this Key Vault, either via RBAC or legacy access policies.
    /// Note that this does not yet support transitive permissions; a user in a group with a permission will not be recognized.
    // TODO: Implement transitive permission checks
    pub fn can_list_secrets(
        &self,
        principal: &impl AsRef<Uuid>,
        rbac: &RoleDefinitionsAndAssignments,
    ) -> bool {
        let kv_uses_rbac = self
            .properties
            .enable_rbac_authorization
            .unwrap_or_default();
        if kv_uses_rbac {
            rbac.iter_role_assignments()
                .filter_principal(principal)
                .filter_scope(&self.id)
                .filter_satisfying(
                    &[],
                    &[RolePermissionAction::new(
                        "Microsoft.KeyVault/vaults/secrets/readMetadata/action",
                    )],
                )
                .next()
                .is_some()
        } else {
            self.properties
                .access_policies
                .iter()
                .filter(|policy| policy.object_id == principal)
                .flat_map(|policy| policy.permissions.secrets.iter())
                .any(|privilege| {
                    matches!(
                        privilege,
                        KeyVaultAccessPolicySecretPrivilege::SecretManagementOperation(
                            KeyVaultAccessPolicySecretManagementOperation::List,
                        ) | KeyVaultAccessPolicySecretPrivilege::All(_)
                    )
                })
        }
    }
}
