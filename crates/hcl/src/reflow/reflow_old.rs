use crate::azuredevops_git_repository_initialization_patcher::AzureDevOpsGitRepositoryInitializationPatcher;
use crate::body_formatter::PrettyBody;
use crate::discovery::DiscoveryDepth;
use crate::discovery::discover_hcl;
use crate::user_id_reference_patcher::UserIdReferencePatcher;
use cloud_terrastodon_azure::prelude::fetch_all_users;
use cloud_terrastodon_hcl_types::prelude::TerraformBlock;
use cloud_terrastodon_hcl_types::prelude::UsersLookupBody;
use eyre::Result;
use hcl::edit::structure::Body;
use hcl::edit::structure::BodyBuilder;
use hcl::edit::visit_mut::VisitMut;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
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
        let mut main_body = discover_hcl(source_dir, DiscoveryDepth::Shallow)
            .await?
            .into_values()
            .flatten()
            .collect::<Body>();
        let users_body;
        let boilerplate_body = Default::default();

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
