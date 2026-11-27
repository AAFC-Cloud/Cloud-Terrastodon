use crate::HclProject;
use crate::reflow::HclReflower;
use cloud_terrastodon_hcl_types::AzureDevOpsResourceBlockKind;
use cloud_terrastodon_hcl_types::ResourceBlockResourceKind;
use hcl_edit::Ident;
use hcl_edit::expr::Array;
use hcl_edit::expr::Expression;
use hcl_edit::structure::Attribute;
use hcl_edit::structure::Block;
use hcl_edit::visit_mut::VisitMut;
use tracing::warn;

pub struct ReflowAzureDevOpsGitRepositoryInitializationAttributes;
#[async_trait::async_trait]
impl HclReflower for ReflowAzureDevOpsGitRepositoryInitializationAttributes {
    async fn reflow(&mut self, hcl: HclProject) -> eyre::Result<HclProject> {
        let mut reflowed = HclProject::new();
        for (path, mut body) in hcl {
            self.visit_body_mut(&mut body);
            reflowed.insert(path, body);
        }
        Ok(reflowed)
    }
}
impl VisitMut for ReflowAzureDevOpsGitRepositoryInitializationAttributes {
    fn visit_block_mut(&mut self, node: &mut hcl_edit::structure::Block) {
        if node.ident.as_str() != "resource" {
            return;
        }
        let Some(resource_kind_str) = node.labels.first().map(|x| x.as_str()) else {
            return;
        };
        let Ok(resource_kind) = resource_kind_str.parse() else {
            warn!("Failed to identify resource kind for {resource_kind_str:?}");
            return;
        };

        if let ResourceBlockResourceKind::AzureDevOps(AzureDevOpsResourceBlockKind::Repo) =
            resource_kind
        {
            let mut initialization_block = Block::builder(Ident::new("initialization"));
            initialization_block = initialization_block
                .attribute(Attribute::new(Ident::new("init_type"), "Clean".to_string()));
            if let Err(e) = node
                .body
                .try_insert(node.body.len(), initialization_block.build())
            {
                warn!(
                    "Failed to insert initialization block for resource {:?}: {:?}",
                    node.labels, e
                );
            };

            let mut lifecycle_block = Block::builder(Ident::new("lifecycle"));
            lifecycle_block = lifecycle_block.attribute(Attribute::new(
                Ident::new("ignore_changes"),
                Expression::Array({
                    let mut array = Array::new();
                    array.push(Ident::new("initialization"));
                    array
                }),
            ));
            if let Err(e) = node
                .body
                .try_insert(node.body.len(), lifecycle_block.build())
            {
                warn!(
                    "Failed to insert lifecycle block for resource {:?}: {:?}",
                    node.labels, e
                );
            };
        }
    }
}
