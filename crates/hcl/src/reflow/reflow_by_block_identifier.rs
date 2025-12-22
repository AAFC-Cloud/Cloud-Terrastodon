use crate::reflow::HclReflower;
use eyre::bail;
use hcl::edit::structure::Body;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct ReflowByBlockIdentifier;
#[async_trait::async_trait]
impl HclReflower for ReflowByBlockIdentifier {
    async fn reflow(
        &mut self,
        hcl: HashMap<PathBuf, Body>,
    ) -> eyre::Result<HashMap<PathBuf, Body>> {
        let mut reflowed = HashMap::new();
        for (path, body) in hcl {
            let Some(dir) = path.parent() else {
                bail!(
                    "Expected path to have a parent directory: {}",
                    path.display()
                );
            };
            for structure in body.into_iter() {
                match structure.into_block() {
                    Ok(block) => {
                        let new_path = match block.ident.as_str() {
                            "terraform" => dir.join("terraform.tf"),
                            "provider" => {
                                let provider_name = block
                                    .labels
                                    .first()
                                    .map(|s| s.as_str())
                                    .unwrap_or("unknown_provider");
                                let alias = block
                                    .body
                                    .get_attribute("alias")
                                    .and_then(|attribute| attribute.value.as_str());
                                if let Some(alias) = alias {
                                    dir.join(format!("provider.{}.{}.tf", provider_name, alias))
                                } else {
                                    dir.join(format!("provider.{}.tf", provider_name))
                                }
                            }
                            "resource" => {
                                let resource_type = block
                                    .labels
                                    .first()
                                    .map(|s| s.as_str())
                                    .unwrap_or("unknown_resource");
                                let resource_name =
                                    block.labels.get(1).map(|s| s.as_str()).unwrap_or("unnamed");
                                dir.join(format!("resource.{}.{}.tf", resource_type, resource_name))
                            }
                            "data" => {
                                let data_type = block
                                    .labels
                                    .first()
                                    .map(|s| s.as_str())
                                    .unwrap_or("unknown_data");
                                let data_name =
                                    block.labels.get(1).map(|s| s.as_str()).unwrap_or("unnamed");
                                dir.join(format!("data.{}.{}.tf", data_type, data_name))
                            }
                            "variable" => {
                                let variable_name = block
                                    .labels
                                    .first()
                                    .map(|s| s.as_str())
                                    .unwrap_or("unnamed");
                                dir.join(format!("variable.{}.tf", variable_name))
                            }
                            "output" => {
                                let output_name = block
                                    .labels
                                    .first()
                                    .map(|s| s.as_str())
                                    .unwrap_or("unnamed");
                                dir.join(format!("output.{}.tf", output_name))
                            }
                            _ => path.clone(),
                        };
                        reflowed
                            .entry(new_path)
                            .or_insert_with(Body::new)
                            .push(block);
                    }
                    Err(structure) => {
                        reflowed
                            .entry(path.clone())
                            .or_insert_with(Body::new)
                            .push(structure);
                    }
                }
            }
        }
        Ok(reflowed)
    }
}
