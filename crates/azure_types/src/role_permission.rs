use crate::prelude::RolePermissionAction;
use ordermap::OrderSet;
use serde::Deserialize;
use serde::Serialize;
use std::cmp::Ordering;

/// See also: `az provider operation list`
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Hash)]
pub struct RolePermissions {
    #[serde(rename = "actions")]
    #[serde(alias = "Actions")]
    pub actions: OrderSet<RolePermissionAction>,
    #[serde(rename = "notActions")]
    #[serde(alias = "NotActions")]
    pub not_actions: OrderSet<RolePermissionAction>,
    #[serde(rename = "dataActions")]
    #[serde(alias = "DataActions")]
    pub data_actions: OrderSet<RolePermissionAction>,
    #[serde(rename = "notDataActions")]
    #[serde(alias = "NotDataActions")]
    pub not_data_actions: OrderSet<RolePermissionAction>,
}

impl RolePermissions {
    pub fn new(
        actions: impl IntoIterator<Item = RolePermissionAction>,
        not_actions: impl IntoIterator<Item = RolePermissionAction>,
        data_actions: impl IntoIterator<Item = RolePermissionAction>,
        not_data_actions: impl IntoIterator<Item = RolePermissionAction>,
    ) -> Self {
        Self {
            actions: actions.into_iter().collect(),
            not_actions: not_actions.into_iter().collect(),
            data_actions: data_actions.into_iter().collect(),
            not_data_actions: not_data_actions.into_iter().collect(),
        }
    }
    /// Returns a Principal of Least Privilege (PoLP) score.
    ///
    /// Lower scores indicate more restrictive permission sets. Scores grow with
    /// the number of allowed actions/data-actions and increase sharply when
    /// wildcard segments are used near the start of the action path.
    pub fn polp_score(&self) -> u64 {
        fn bag_cost<'a>(actions: impl IntoIterator<Item = &'a RolePermissionAction>) -> u64 {
            actions.into_iter().map(RolePermissions::action_cost).sum()
        }

        bag_cost(&self.actions) + bag_cost(&self.data_actions)
    }

    fn action_cost(action: &RolePermissionAction) -> u64 {
        const ACTION_BASE_COST: u64 = 1_000;
        const WILDCARD_COST: u64 = 1_000_000;
        const EARLY_WILDCARD_COST: u64 = 10_000;
        const MAX_WILDCARD_DEPTH: u64 = 16;

        let mut cost = ACTION_BASE_COST;
        if let Some(idx) = action.find('*') {
            cost = cost.saturating_add(WILDCARD_COST);
            let prefix = &action[..idx];
            let segments_before_wildcard = prefix
                .split('/')
                .filter(|segment| !segment.is_empty())
                .count() as u64;
            let depth_penalty = MAX_WILDCARD_DEPTH
                .saturating_sub(segments_before_wildcard)
                .saturating_mul(EARLY_WILDCARD_COST);
            cost = cost.saturating_add(depth_penalty);
        }
        cost
    }

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

impl Ord for RolePermissions {
    fn cmp(&self, other: &Self) -> Ordering {
        self.polp_score().cmp(&other.polp_score())
    }
}

impl PartialOrd for RolePermissions {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod test {
    use super::RolePermissions;
    use crate::prelude::RolePermissionAction;

    #[test]
    pub fn it_works() -> eyre::Result<()> {
        let perm = RolePermissions::new(
            [RolePermissionAction::new(
                "Microsoft.KeyVault/vaults/secrets/*/action",
            )],
            [],
            [RolePermissionAction::new(
                "Microsoft.KeyVault/vaults/secrets/readMetadata/action",
            )],
            [],
        );
        assert!(perm.satisfies(
            &[RolePermissionAction::new(
                "Microsoft.KeyVault/vaults/secrets/list/action"
            )],
            &[RolePermissionAction::new(
                "Microsoft.KeyVault/vaults/secrets/readMetadata/action",
            )]
        ));
        Ok(())
    }
    #[test]
    pub fn denies_not_actions() -> eyre::Result<()> {
        // User asks for read & write, but write is explicitly denied.
        let perm = RolePermissions::new(
            [
                RolePermissionAction::new("Microsoft.Storage/accounts/read/action"),
                RolePermissionAction::new("Microsoft.Storage/accounts/write/action"),
            ],
            [RolePermissionAction::new(
                "Microsoft.Storage/accounts/write/action",
            )],
            [],
            [],
        );
        assert!(!perm.satisfies(
            &[
                RolePermissionAction::new("Microsoft.Storage/accounts/read/action",),
                RolePermissionAction::new("Microsoft.Storage/accounts/write/action",),
            ],
            &[]
        ));
        Ok(())
    }

    #[test]
    pub fn denies_not_data_actions() -> eyre::Result<()> {
        let perm = RolePermissions::new(
            [],
            [],
            [RolePermissionAction::new(
                "Microsoft.KeyVault/vaults/secrets/*/action",
            )],
            [RolePermissionAction::new(
                "Microsoft.KeyVault/vaults/secrets/readMetadata/action",
            )],
        );
        assert!(!perm.satisfies(
            &[],
            &[RolePermissionAction::new(
                "Microsoft.KeyVault/vaults/secrets/readMetadata/action",
            )]
        ));
        Ok(())
    }

    #[test]
    pub fn requires_all_requested_actions_present() -> eyre::Result<()> {
        let perm = RolePermissions::new(
            [RolePermissionAction::new(
                "Microsoft.Storage/accounts/read/action",
            )],
            [],
            [],
            [],
        );
        // Requests read + write but role only grants read.
        assert!(!perm.satisfies(
            &[
                RolePermissionAction::new("Microsoft.Storage/accounts/read/action",),
                RolePermissionAction::new("Microsoft.Storage/accounts/write/action",),
            ],
            &[]
        ));
        Ok(())
    }

    #[test]
    pub fn wildcard_action_satisfies_multiple_requested() -> eyre::Result<()> {
        let perm = RolePermissions::new(
            [RolePermissionAction::new("Microsoft.Storage/accounts/*")],
            [],
            [],
            [],
        );
        assert!(perm.satisfies(
            &[
                RolePermissionAction::new("Microsoft.Storage/accounts/read/action",),
                RolePermissionAction::new("Microsoft.Storage/accounts/write/action",),
            ],
            &[]
        ));
        Ok(())
    }

    #[test]
    pub fn data_actions_independent_from_actions() -> eyre::Result<()> {
        let perm = RolePermissions::new(
            [RolePermissionAction::new(
                "Microsoft.KeyVault/vaults/read/action",
            )],
            [],
            [RolePermissionAction::new(
                "Microsoft.KeyVault/vaults/secrets/list/action",
            )],
            [],
        );
        assert!(perm.satisfies(
            &[RolePermissionAction::new(
                "Microsoft.KeyVault/vaults/read/action",
            )],
            &[RolePermissionAction::new(
                "Microsoft.KeyVault/vaults/secrets/list/action",
            )]
        ));
        Ok(())
    }

    #[test]
    pub fn empty_requests_are_always_satisfied() -> eyre::Result<()> {
        let perm = RolePermissions::new([], [], [], []);
        assert!(perm.satisfies(&[], &[]));
        Ok(())
    }
}
