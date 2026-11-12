use crate::HclProject;
use crate::reflow::HclReflower;
use crate::reflow::HclUuidCollector;
use crate::reflow::ReflowAzureDevOpsGitRepositoryInitializationAttributes;
use crate::reflow::ReflowBlockDecorations;
use crate::reflow::ReflowByBlockIdentifier;
use crate::reflow::ReflowExpressionsUseImportedResourceBlocks;
use crate::reflow::ReflowJsonAttributes;
use crate::reflow::ReflowPrincipalIdComments;
use crate::reflow::ReflowRemoveDefaultAttributes;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure::EntraDirectoryObjectsByIdsRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use eyre::OptionExt;
use std::path::PathBuf;
use tracing::info;

#[must_use = "This is a future request, you must .execute it"]
pub struct ReflowHclRequest {
    pub authentication: Option<AzureBearerAuthenticationSelection>,
    pub hcl: HclProject,
    pub include_principal_id_comments: bool,
    pub single_file_path: Option<PathBuf>,
    pub mixed: bool,
}

impl ReflowHclRequest {
    pub fn new(
        hcl: HclProject,
        authentication: Option<AzureBearerAuthenticationSelection>,
    ) -> ReflowHclRequest {
        Self {
            authentication,
            hcl,
            include_principal_id_comments: false,
            single_file_path: None,
            mixed: false,
        }
    }

    pub fn include_principal_id_comments(mut self, include: bool) -> Self {
        self.include_principal_id_comments = include;
        self
    }

    pub fn single_file_path(mut self, path: Option<PathBuf>) -> Self {
        self.single_file_path = path;
        self
    }

    pub fn mixed(mut self, mixed: bool) -> Self {
        self.mixed = mixed;
        self
    }

    async fn reflow(self, executor: &CloudTerrastodonExecutor) -> eyre::Result<HclProject> {
        let Self {
            authentication,
            mut hcl,
            include_principal_id_comments,
            single_file_path,
            mixed,
        } = self;

        let mut reflowers: Vec<Box<dyn HclReflower>> = vec![
            Box::new(ReflowJsonAttributes),
            Box::new(ReflowAzureDevOpsGitRepositoryInitializationAttributes),
            Box::new(ReflowRemoveDefaultAttributes),
            Box::new(ReflowByBlockIdentifier::new(single_file_path, mixed)),
            Box::new(ReflowExpressionsUseImportedResourceBlocks::default()),
            Box::new(ReflowBlockDecorations),
        ];
        let principal_ids = if include_principal_id_comments {
            HclUuidCollector::collect(&hcl)
        } else {
            Default::default()
        };
        if include_principal_id_comments && !principal_ids.is_empty() {
            info!("Fetching principals");
            let authentication = authentication.ok_or_eyre(
                "An explicit Azure account selection is required to fetch principals",
            )?;
            let principals = EntraDirectoryObjectsByIdsRequest {
                ids: (principal_ids).into_iter().collect(),
                authentication,
            }
            .execute(executor)
            .await?;
            reflowers.insert(
                3,
                Box::new(ReflowPrincipalIdComments::from_directory_objects(
                    principals.iter().map(|principal| principal.as_ref()),
                )),
            );
        }
        for mut reflower in reflowers {
            executor.cancellation().bail_if_cancelled()?;
            hcl = executor
                .cancellation()
                .cancellable_scope(|_| reflower.reflow(hcl))
                .await?;
        }
        Ok(hcl)
    }
}

impl ExecutableRequest for ReflowHclRequest {
    type Error = eyre::Report;
    type Output = HclProject;

    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move { self.reflow(&executor).await })
    }
}
