use crate::prelude::RolePermissionAction;
use serde::Deserialize;
use serde::Serialize;

/// See also: `az provider operation list`
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct RolePermissions {
    #[serde(rename = "actions")]
    #[serde(alias = "Actions")]
    pub actions: Vec<RolePermissionAction>,
    #[serde(rename = "notActions")]
    #[serde(alias = "NotActions")]
    pub not_actions: Vec<RolePermissionAction>,
    #[serde(rename = "dataActions")]
    #[serde(alias = "DataActions")]
    pub data_actions: Vec<RolePermissionAction>,
    #[serde(rename = "notDataActions")]
    #[serde(alias = "NotDataActions")]
    pub not_data_actions: Vec<RolePermissionAction>,
}

impl RolePermissions {
    pub fn satisfies(
        &self,
        actions: &[RolePermissionAction],
        data_actions: &[RolePermissionAction],
    ) -> bool {
        for not_action in &self.not_actions {
            for action in actions {
                if not_action.satisfies(action) {
                    return false;
                }
            }
        }
        for not_data_action in &self.not_data_actions {
            for data_action in data_actions {
                if not_data_action.satisfies(data_action) {
                    return false;
                }
            }
        }
        for action in actions {
            let mut satisfied = false;
            for self_action in &self.actions {
                if self_action.satisfies(action) {
                    satisfied = true;
                    break;
                }
            }
            if !satisfied {
                return false;
            }
        }
        for data_action in data_actions {
            let mut satisfied = false;
            for self_data_action in &self.data_actions {
                if self_data_action.satisfies(data_action) {
                    satisfied = true;
                    break;
                }
            }
            if !satisfied {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod test {
    #[test]
    pub fn it_works() -> eyre::Result<()> {
        let perm = super::RolePermissions {
            actions: vec![super::RolePermissionAction::new(
                "Microsoft.KeyVault/vaults/secrets/*/action",
            )],
            not_actions: vec![],
            data_actions: vec![super::RolePermissionAction::new(
                "Microsoft.KeyVault/vaults/secrets/readMetadata/action",
            )],
            not_data_actions: vec![],
        };
        assert!(perm.satisfies(
            &[super::RolePermissionAction::new(
                "Microsoft.KeyVault/vaults/secrets/list/action"
            )],
            &[super::RolePermissionAction::new(
                "Microsoft.KeyVault/vaults/secrets/readMetadata/action",
            )]
        ));
        Ok(())
    }
    #[test]
    pub fn denies_not_actions() -> eyre::Result<()> {
        // User asks for read & write, but write is explicitly denied.
        let perm = super::RolePermissions {
            actions: vec![
                super::RolePermissionAction::new("Microsoft.Storage/accounts/read/action"),
                super::RolePermissionAction::new("Microsoft.Storage/accounts/write/action"),
            ],
            not_actions: vec![super::RolePermissionAction::new(
                "Microsoft.Storage/accounts/write/action",
            )],
            data_actions: vec![],
            not_data_actions: vec![],
        };
        assert!(!perm.satisfies(
            &[
                super::RolePermissionAction::new(
                    "Microsoft.Storage/accounts/read/action",
                ),
                super::RolePermissionAction::new(
                    "Microsoft.Storage/accounts/write/action",
                ),
            ],
            &[]
        ));
        Ok(())
    }

    #[test]
    pub fn denies_not_data_actions() -> eyre::Result<()> {
        let perm = super::RolePermissions {
            actions: vec![],
            not_actions: vec![],
            data_actions: vec![super::RolePermissionAction::new(
                "Microsoft.KeyVault/vaults/secrets/*/action",
            )],
            not_data_actions: vec![super::RolePermissionAction::new(
                "Microsoft.KeyVault/vaults/secrets/readMetadata/action",
            )],
        };
        assert!(!perm.satisfies(
            &[],
            &[super::RolePermissionAction::new(
                "Microsoft.KeyVault/vaults/secrets/readMetadata/action",
            )]
        ));
        Ok(())
    }

    #[test]
    pub fn requires_all_requested_actions_present() -> eyre::Result<()> {
        let perm = super::RolePermissions {
            actions: vec![super::RolePermissionAction::new(
                "Microsoft.Storage/accounts/read/action",
            )],
            not_actions: vec![],
            data_actions: vec![],
            not_data_actions: vec![],
        };
        // Requests read + write but role only grants read.
        assert!(!perm.satisfies(
            &[
                super::RolePermissionAction::new(
                    "Microsoft.Storage/accounts/read/action",
                ),
                super::RolePermissionAction::new(
                    "Microsoft.Storage/accounts/write/action",
                ),
            ],
            &[]
        ));
        Ok(())
    }

    #[test]
    pub fn wildcard_action_satisfies_multiple_requested() -> eyre::Result<()> {
        let perm = super::RolePermissions {
            actions: vec![super::RolePermissionAction::new(
                "Microsoft.Storage/accounts/*",
            )],
            not_actions: vec![],
            data_actions: vec![],
            not_data_actions: vec![],
        };
        assert!(perm.satisfies(
            &[
                super::RolePermissionAction::new(
                    "Microsoft.Storage/accounts/read/action",
                ),
                super::RolePermissionAction::new(
                    "Microsoft.Storage/accounts/write/action",
                ),
            ],
            &[]
        ));
        Ok(())
    }

    #[test]
    pub fn data_actions_independent_from_actions() -> eyre::Result<()> {
        let perm = super::RolePermissions {
            actions: vec![super::RolePermissionAction::new(
                "Microsoft.KeyVault/vaults/read/action",
            )],
            not_actions: vec![],
            data_actions: vec![super::RolePermissionAction::new(
                "Microsoft.KeyVault/vaults/secrets/list/action",
            )],
            not_data_actions: vec![],
        };
        assert!(perm.satisfies(
            &[super::RolePermissionAction::new(
                "Microsoft.KeyVault/vaults/read/action",
            )],
            &[super::RolePermissionAction::new(
                "Microsoft.KeyVault/vaults/secrets/list/action",
            )]
        ));
        Ok(())
    }

    #[test]
    pub fn empty_requests_are_always_satisfied() -> eyre::Result<()> {
        let perm = super::RolePermissions {
            actions: vec![],
            not_actions: vec![],
            data_actions: vec![],
            not_data_actions: vec![],
        };
        assert!(perm.satisfies(&[], &[]));
        Ok(())
    }
}
