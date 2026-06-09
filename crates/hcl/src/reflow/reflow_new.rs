use crate::reflow::HclReflower;
use crate::reflow::ReflowAzureDevOpsGitRepositoryInitializationAttributes;
use crate::reflow::ReflowBlockDecorations;
use crate::reflow::ReflowByBlockIdentifier;
use crate::reflow::ReflowExpressionsUseImportedResourceBlocks;
use crate::reflow::ReflowJsonAttributes;
use crate::reflow::ReflowPrincipalIdComments;
use crate::reflow::ReflowRemoveDefaultAttributes;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_all_principals;
use hcl::edit::structure::Body;
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::info;

pub async fn reflow_hcl(
    tenant: AzureTenantArgument<'_>,
    mut hcl: HashMap<PathBuf, Body>,
    include_principal_id_comments: bool,
    single_file_path: Option<PathBuf>,
    mixed: bool,
) -> eyre::Result<HashMap<PathBuf, Body>> {
    let mut reflowers: Vec<Box<dyn HclReflower>> = vec![
        Box::new(ReflowJsonAttributes),
        Box::new(ReflowAzureDevOpsGitRepositoryInitializationAttributes),
        Box::new(ReflowRemoveDefaultAttributes),
        Box::new(ReflowByBlockIdentifier::new(single_file_path, mixed)),
        Box::new(ReflowExpressionsUseImportedResourceBlocks::default()),
        Box::new(ReflowBlockDecorations),
    ];
    if include_principal_id_comments {
        info!("Fetching principals");
        let tenant_id = tenant.resolve().await?;
        let principals = fetch_all_principals(tenant_id).await?;
        reflowers.insert(3, Box::new(ReflowPrincipalIdComments::new(principals)));
    }
    for mut reflower in reflowers {
        hcl = reflower.reflow(hcl).await?;
    }
    Ok(hcl)
}
