use crate::prelude::KeyVault;
use crate::prelude::RoleDefinitionsAndAssignments;
use crate::prelude::RoleDefinitionsAndAssignmentsIterTools;
use crate::prelude::RolePermissionAction;
use tracing::warn;
use uuid::Uuid;

impl KeyVault {
    pub fn can_list_secrets(
        &self,
        principal: &impl AsRef<Uuid>,
        rbac: &RoleDefinitionsAndAssignments,
    ) -> bool {
        let kv_uses_rbac = self
            .properties
            .enable_rbac_authorization
            .unwrap_or_default();
        if !kv_uses_rbac {
            warn!(
                "Key Vault {} does not use RBAC authorization, cannot determine access (not yet implemented)",
                self.name
            );
            return false;
        }
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
    }
}
