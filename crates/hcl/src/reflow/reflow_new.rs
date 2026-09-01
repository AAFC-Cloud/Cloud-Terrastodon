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
use cloud_terrastodon_azure::fetch_entra_directory_objects_by_ids;
use cloud_terrastodon_credentials::AzureTenantAuthContext;
use eyre::OptionExt;
use std::borrow::Cow;
use std::future::Future;
use std::future::IntoFuture;
use std::path::PathBuf;
use std::pin::Pin;
use tracing::info;

#[must_use = "This is a future request, you must .await it"]
pub struct ReflowHclRequest<'a> {
    pub auth_context: Option<Cow<'a, AzureTenantAuthContext>>,
    pub hcl: HclProject,
    pub include_principal_id_comments: bool,
    pub single_file_path: Option<PathBuf>,
    pub mixed: bool,
}

impl<'a> ReflowHclRequest<'a> {
    pub fn new(
        hcl: HclProject,
        auth_context: Option<&'a AzureTenantAuthContext>,
    ) -> ReflowHclRequest<'a> {
        Self {
            auth_context: auth_context.map(Cow::Borrowed),
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

    async fn run(self) -> eyre::Result<HclProject> {
        let Self {
            auth_context,
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
            let auth_context = auth_context.as_deref().ok_or_eyre(
                "An Azure tenant authentication context is required to fetch principals",
            )?;
            let principals =
                fetch_entra_directory_objects_by_ids(principal_ids, auth_context).await?;
            reflowers.insert(
                3,
                Box::new(ReflowPrincipalIdComments::from_directory_objects(
                    principals,
                )),
            );
        }
        for mut reflower in reflowers {
            hcl = reflower.reflow(hcl).await?;
        }
        Ok(hcl)
    }
}

impl<'a> IntoFuture for ReflowHclRequest<'a> {
    type Output = eyre::Result<HclProject>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(self.run())
    }
}
