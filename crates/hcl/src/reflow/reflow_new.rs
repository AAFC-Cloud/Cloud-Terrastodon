use crate::reflow::HclReflower;
use crate::reflow::ReflowAzureDevOpsGitRepositoryInitializationAttributes;
use crate::reflow::ReflowByBlockIdentifier;
use crate::reflow::ReflowExpressionsUseImportedResourceBlocks;
use crate::reflow::ReflowJsonAttributes;
use crate::reflow::ReflowPrincipalIdComments;
use crate::reflow::ReflowRemoveDefaultAttributes;
use cloud_terrastodon_azure::prelude::fetch_all_principals;
use hcl::edit::structure::Body;
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::info;

pub async fn reflow_hcl(mut hcl: HashMap<PathBuf, Body>) -> eyre::Result<HashMap<PathBuf, Body>> {
    info!("Fetching principals");
    let principals = fetch_all_principals().await?;
    let reflowers: Vec<Box<dyn HclReflower>> = vec![
        Box::new(ReflowJsonAttributes),
        Box::new(ReflowAzureDevOpsGitRepositoryInitializationAttributes),
        Box::new(ReflowRemoveDefaultAttributes),
        Box::new(ReflowPrincipalIdComments::new(principals)),
        Box::new(ReflowByBlockIdentifier),
        Box::new(ReflowExpressionsUseImportedResourceBlocks::default()),
    ];
    for mut reflower in reflowers {
        hcl = reflower.reflow(hcl).await?;
    }
    Ok(hcl)
}
