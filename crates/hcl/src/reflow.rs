use crate::azuredevops_git_repository_initialization_patcher::AzureDevOpsGitRepositoryInitializationPatcher;
use crate::body_formatter::PrettyBody;
use crate::data_block_creation::create_data_blocks_for_ids;
use crate::data_reference_patcher::DataReferencePatcher;
use crate::default_attribute_cleanup_patcher::DefaultAttributeCleanupPatcher;
use crate::import_lookup_holder::ImportLookupHolder;
use crate::imported_resource_reference_patcher::ImportedResourceReferencePatcher;
use crate::json_patcher::JsonPatcher;
use crate::terraform_block_extracter_patcher::TerraformBlockExtracterPatcher;
use crate::user_id_reference_patcher::UserIdReferencePatcher;
use cloud_terrastodon_azure::prelude::fetch_all_users;
use cloud_terrastodon_hcl_types::prelude::TerraformBlock;
use cloud_terrastodon_hcl_types::prelude::UsersLookupBody;
use eyre::Context;
use eyre::Result;
use hcl::edit::structure::Body;
use hcl::edit::structure::BodyBuilder;
use hcl::edit::visit::Visit;
use hcl::edit::visit_mut::VisitMut;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::Path;
use std::path::PathBuf;
use tokio::fs;
use tracing::debug;
use tracing::instrument;

pub struct ReflowedTFWorkspace {
    pub main: Body,
    pub users: UsersLookupBody,
    pub boilerplate: TerraformBlock,
}
impl ReflowedTFWorkspace {
    pub fn get_file_contents(
        self,
        destination_dir: impl AsRef<Path>,
    ) -> eyre::Result<Vec<(PathBuf, String)>> {
        let dest_dir = destination_dir.as_ref();
        let mut rtn = Vec::new();

        rtn.push((dest_dir.join("main.tf"), self.main.to_string_pretty()?));

        if !self.users.is_empty() {
            let body: Body = self.users.into();
            rtn.push((dest_dir.join("users.tf"), body.to_string_pretty()?));
        }

        if !self.boilerplate.is_empty() {
            let body = BodyBuilder::default().block(self.boilerplate).build();
            rtn.push((dest_dir.join("boilerplate.tf"), body.to_string_pretty()?));
        }

        Ok(rtn)
    }
}

#[instrument(level = "trace")]
pub async fn reflow_workspace(source_dir: &Path) -> Result<ReflowedTFWorkspace> {
    // We use `Box::pin` here to shrink the size of the future on the stack.
    // Without this, we get STATUS_STACK_OVERFLOW when handling big HCL structures.
    // https://hegdenu.net/posts/how-big-is-your-future/
    // archive link: https://web.archive.org/web/20241218031613/https://hegdenu.net/posts/how-big-is-your-future/
    Box::pin(async move {
        debug!("Assembling body for parsing");
        let mut main_body = Box::pin(as_single_body(source_dir)).await?;
        let users_body;
        let boilerplate_body;
        {
            debug!("Extracting terraform config blocks");
            let mut patcher = TerraformBlockExtracterPatcher::default();
            patcher.visit_body_mut(&mut main_body);
            boilerplate_body = patcher.terraform_block;
        }

        {
            debug!("Updating json string literals to use jsonencode");
            let mut json_patcher = JsonPatcher;
            json_patcher.visit_body_mut(&mut main_body);
        }

        {
            debug!("Gathering import blocks to discover IDs");
            let mut lookups = ImportLookupHolder::default();
            lookups.visit_body(&main_body);

            debug!("Updating strings from hardcoded IDs to reference resource blocks instead");
            let mut import_reference_patcher = ImportedResourceReferencePatcher::new(
                lookups,
                ["policy_definition_id", "scope"]
                    .into_iter()
                    .map(|x| x.to_string())
                    .collect(),
            );
            import_reference_patcher.visit_body_mut(&mut main_body);

            debug!("Creating data blocks for hardcoded IDs without a matching resource block");
            let (data_blocks, data_references) =
                create_data_blocks_for_ids(&import_reference_patcher.missing_entries).await?;

            debug!("Adding new data blocks to body");
            for block in data_blocks.into_blocks() {
                main_body.push(block);
            }

            debug!("Updating string from hardcoded IDs to reference data blocks instead");
            let mut data_reference_patcher = DataReferencePatcher {
                lookup: data_references,
            };
            data_reference_patcher.visit_body_mut(&mut main_body);
        }

        {
            debug!("Fetching users to perform user ID substitution");
            let users = fetch_all_users()
                .await?
                .into_iter()
                .map(|user| (user.id, user.user_principal_name))
                .collect();

            debug!("Performing user ID substitution");
            let mut user_reference_patcher = UserIdReferencePatcher {
                user_principal_name_by_user_id: users,
                used: HashSet::default(),
            };
            user_reference_patcher.visit_body_mut(&mut main_body);

            debug!("Building user lookup");
            if let Some(new_users_body) = user_reference_patcher.build_lookup_blocks()? {
                debug!("Appending users.tf to output");
                users_body = new_users_body;
            } else {
                debug!("No users referenced, lookup not needed");
                users_body = Default::default();
            }
        }

        {
            debug!("Pruning default/conflicting properties");
            let mut patcher = DefaultAttributeCleanupPatcher {};
            patcher.visit_body_mut(&mut main_body);
        }

        {
            debug!("Fixing azuredevops_git_repository initialization blocks");
            let mut patcher = AzureDevOpsGitRepositoryInitializationPatcher;
            patcher.visit_body_mut(&mut main_body);
        }

        Ok(ReflowedTFWorkspace {
            main: main_body,
            users: users_body,
            boilerplate: boilerplate_body,
        })
    })
    .await
}

pub async fn as_single_body(source_dir: impl AsRef<Path>) -> Result<Body> {
    let mut body = Body::new();

    // Read all files in source dir and append to body
    let mut found = fs::read_dir(&source_dir)
        .await
        .wrap_err(source_dir.as_ref().display().to_string())?;
    while let Some(entry) = found.next_entry().await? {
        if let Some(found_body) = Box::pin(read_file_into_body(&entry.path())).await? {
            debug!("Appending parsed body to overall body");
            for structure in found_body.into_iter() {
                body.push(structure);
            }
        }
    }
    Ok(body)
}

#[instrument(level = "debug")]
pub async fn read_file_into_body(path: &Path) -> Result<Option<Body>> {
    if !path.is_file() || path.extension() != Some(OsStr::new("tf")) {
        debug!("Path is not a .tf file, skipping");
        return Ok(None);
    }

    debug!("Reading file contents");
    let contents = fs::read(&path)
        .await
        .context(format!("reading {}", path.display()))?;

    debug!("Parsing file contents");
    let text = String::from_utf8(contents).context(format!("utf-8 parsing {}", path.display()))?;

    debug!("Parsing HCL body");
    let body: Body = text
        .parse()
        .context(format!("body parsing {}", path.display()))?;
    Ok(Some(body))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf8_problem() {
        let text = r#"
            import {
                id = "omitted"
                to = azuread_group.écurité
            }
        "#;
        let _body: Body = text.parse().unwrap();
    }
    #[test]
    fn utf8_problem2() {
        let text = r#"
            locals {
                é = 4
            }
            output "ééé" {
            value = local.é
            }
        "#;
        let _body: Body = text.parse().unwrap();
    }
}
