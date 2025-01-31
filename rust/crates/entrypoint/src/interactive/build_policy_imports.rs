use cloud_terrastodon_core_azure::prelude::fetch_all_policy_assignments;
use cloud_terrastodon_core_azure::prelude::fetch_all_policy_definitions;
use cloud_terrastodon_core_azure::prelude::fetch_all_policy_set_definitions;
use cloud_terrastodon_core_azure::prelude::PolicyAssignment;
use cloud_terrastodon_core_pathing::AppDir;
use cloud_terrastodon_core_tofu::prelude::Sanitizable;
use cloud_terrastodon_core_tofu::prelude::TofuImportBlock;
use cloud_terrastodon_core_tofu::prelude::TofuWriter;
use eyre::eyre;
use eyre::Result;
use std::collections::HashSet;

pub async fn build_policy_imports() -> Result<()> {
    let policy_definitions = fetch_all_policy_definitions().await?;
    let policy_set_definitions = fetch_all_policy_set_definitions().await?;
    let policy_assignments = fetch_all_policy_assignments().await?;

    let mut imports: Vec<TofuImportBlock> = Default::default();
    let mut seen_ids: HashSet<String> = HashSet::new();

    for policy_definition in policy_definitions {
        if policy_definition.policy_type == "Custom" {
            let block: TofuImportBlock = policy_definition.into();
            if seen_ids.insert(block.id.clone()) {
                imports.push(block);
            }
        }
    }

    policy_set_definitions
        .into_iter()
        .filter(|def| def.policy_type == "Custom")
        .map(|x| x.into())
        .for_each(|block: TofuImportBlock| {
            if seen_ids.insert(block.id.clone()) {
                imports.push(block);
            }
        });

    for (management_group, policy_assignments) in policy_assignments {
        policy_assignments
            .into_iter()
            .map(|policy_assignment: PolicyAssignment| {
                //todo: filter out inherited assignments that cause the terraform block label to contain a mismatched management group name
                let import_block: TofuImportBlock = policy_assignment.into();
                let provider = import_block.provider;
                let id = import_block.id;
                let mut to = import_block.to;
                to.use_name(|name| format!("{}_{}", name, management_group.name()).sanitize());
                TofuImportBlock { provider, id, to }
            })
            .for_each(|block: TofuImportBlock| {
                if seen_ids.insert(block.id.clone()) {
                    imports.push(block);
                }
            });
    }

    if imports.is_empty() {
        return Err(eyre!("Imports should not be empty"));
    }

    TofuWriter::new(AppDir::Imports.join("policy_imports.tf"))
        .overwrite(imports)
        .await?;

    Ok(())
}
