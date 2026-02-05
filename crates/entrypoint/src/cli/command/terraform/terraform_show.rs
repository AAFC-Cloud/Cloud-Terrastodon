use clap::Args;
use cloud_terrastodon_azure::prelude::PrincipalCollection;
use cloud_terrastodon_azure::prelude::PrincipalId;
use cloud_terrastodon_azure::prelude::fetch_all_principals;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_hcl::prelude::TerraformChangeAction;
use cloud_terrastodon_hcl::prelude::TerraformPlan;
use eyre::OptionExt;
use eyre::Result;
use serde_json;
use serde_json::Value;
use std::cell::RefCell;
use std::collections::HashSet;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use tokio::fs;
use tracing::info;

/// Show a Terraform plan or plan JSON as a parsed `TerraformPlan`.
#[derive(Args, Debug, Clone)]
pub struct TerraformShowArgs {
    /// Path to a Terraform plan (.tfplan) or a JSON plan file (.json)
    pub plan_file: PathBuf,
}

impl TerraformShowArgs {
    pub async fn invoke(self) -> Result<()> {
        // Determine whether the given file is JSON
        let is_json = self.plan_file.extension().and_then(|s| s.to_str()) == Some("json");

        let plan: TerraformPlan = if is_json {
            let content = fs::read_to_string(&self.plan_file).await?;
            serde_json::from_str(&content)?
        } else {
            let path_str = self
                .plan_file
                .to_str()
                .ok_or_else(|| eyre::eyre!("Plan file path is not valid UTF-8"))?;
            let mut cmd = CommandBuilder::new(CommandKind::Terraform);
            cmd.should_announce(true);
            cmd.args(["show", "--json", path_str]);
            cmd.run::<TerraformPlan>().await?
        };

        // Async lazy loader for principals scoped to this function.
        // On first call it fetches and caches the value; subsequent calls return the cached Arc.
        let principals = {
            let cache: Rc<RefCell<Option<Arc<PrincipalCollection>>>> = Rc::new(RefCell::new(None));

            move || {
                let cache = cache.clone();
                async move {
                    // Fast path: return cached clone if present (no await, no borrow across await)
                    if let Some(cached) = cache.borrow().as_ref().cloned() {
                        return eyre::Ok(cached);
                    }

                    // Slow path: fetch, store in cache, and return
                    let fetched = fetch_all_principals().await?;
                    let arc = Arc::new(fetched);
                    *cache.borrow_mut() = Some(arc.clone());
                    eyre::Ok(arc)
                }
            }
        };

        // Usage example: call `principals().await?` where needed
        // let principals = principals().await?;

        // Identify the resource changes for azuread_group
        for change in plan
            .resource_changes
            .iter()
            .filter(|rc| rc.r#type == "azuread_group")
            .filter(|rc| rc.change.actions != [TerraformChangeAction::NoOp])
        {
            let (before_members, before_owners) =
                extract_members_and_owners(change.change.before.as_ref())?;
            let (after_members, after_owners) =
                extract_members_and_owners(change.change.after.as_ref())?;
            let added_members = after_members
                .difference(&before_members)
                .cloned()
                .collect::<HashSet<_>>();
            let removed_members = before_members
                .difference(&after_members)
                .cloned()
                .collect::<HashSet<_>>();
            let added_owners = after_owners
                .difference(&before_owners)
                .cloned()
                .collect::<HashSet<_>>();
            let removed_owners = before_owners
                .difference(&after_owners)
                .cloned()
                .collect::<HashSet<_>>();
            if !added_members.is_empty()
                || !removed_members.is_empty()
                || !added_owners.is_empty()
                || !removed_owners.is_empty()
            {
                // Resolve display names (cached via the local async loader)
                let principals = principals().await?;

                let added_members_display_names = added_members
                    .iter()
                    .map(|id| {
                        principals
                            .get(id)
                            .map(|p| p.name().to_string())
                            .unwrap_or_else(|| id.to_string())
                    })
                    .collect::<Vec<_>>();

                let removed_members_display_names = removed_members
                    .iter()
                    .map(|id| {
                        principals
                            .get(id)
                            .map(|p| p.name().to_string())
                            .unwrap_or_else(|| id.to_string())
                    })
                    .collect::<Vec<_>>();

                let added_owners_display_names = added_owners
                    .iter()
                    .map(|id| {
                        principals
                            .get(id)
                            .map(|p| p.name().to_string())
                            .unwrap_or_else(|| id.to_string())
                    })
                    .collect::<Vec<_>>();

                let removed_owners_display_names = removed_owners
                    .iter()
                    .map(|id| {
                        principals
                            .get(id)
                            .map(|p| p.name().to_string())
                            .unwrap_or_else(|| id.to_string())
                    })
                    .collect::<Vec<_>>();

                info!(
                    change.address,
                    ?added_members,
                    ?removed_members,
                    ?added_owners,
                    ?removed_owners,
                    ?added_members_display_names,
                    ?removed_members_display_names,
                    ?added_owners_display_names,
                    ?removed_owners_display_names,
                    "azuread_group membership/ownership changes detected",
                )
            }
        }

        Ok(())
    }
}

fn extract_members_and_owners(
    azuread_group_data: Option<&Value>,
) -> Result<(HashSet<PrincipalId>, HashSet<PrincipalId>), eyre::Error> {
    Ok(match azuread_group_data {
        None => (HashSet::new(), HashSet::new()),
        Some(group_data) => {
            let mut members = HashSet::new();
            let mut owners = HashSet::new();

            let members_data = group_data
                .get("members")
                .ok_or_eyre("expected members")?
                .as_array()
                .ok_or_eyre("expected array")?;
            for member in members_data {
                let member_str = member.as_str().ok_or_eyre("expected member to be string")?;
                members.insert(member_str.parse()?);
            }

            let owners_data = group_data
                .get("owners")
                .ok_or_eyre("expected owners")?
                .as_array()
                .ok_or_eyre("expected array")?;
            for owner in owners_data {
                let owner_str = owner.as_str().ok_or_eyre("expected owner to be string")?;
                owners.insert(owner_str.parse()?);
            }
            (members, owners)
        }
    })
}
