use crate::reflow::HclReflower;
use crate::reflow::ReflowAzureDevOpsGitRepositoryInitializationAttributes;
use crate::reflow::ReflowByBlockIdentifier;
use crate::reflow::ReflowExpressionsUseImportedResourceBlocks;
use crate::reflow::ReflowJsonAttributes;
use crate::reflow::ReflowPrincipalIdComments;
use crate::reflow::ReflowRemoveDefaultAttributes;
use cloud_terrastodon_azure::AzureTenantId;
use cloud_terrastodon_azure::fetch_all_principals;
use hcl::edit::structure::Body;
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::info;

pub async fn reflow_hcl(
    tenant_id: AzureTenantId,
    mut hcl: HashMap<PathBuf, Body>,
    include_principal_id_comments: bool,
) -> eyre::Result<HashMap<PathBuf, Body>> {
    let mut reflowers: Vec<Box<dyn HclReflower>> = vec![
        Box::new(ReflowJsonAttributes),
        Box::new(ReflowAzureDevOpsGitRepositoryInitializationAttributes),
        Box::new(ReflowRemoveDefaultAttributes),
        Box::new(ReflowByBlockIdentifier),
        Box::new(ReflowExpressionsUseImportedResourceBlocks::default()),
    ];
    if include_principal_id_comments {
        info!("Fetching principals");
        let principals = fetch_all_principals(tenant_id).await?;
        reflowers.insert(3, Box::new(ReflowPrincipalIdComments::new(principals)));
    }
    for mut reflower in reflowers {
        hcl = reflower.reflow(hcl).await?;
    }
    Ok(hcl)
}
