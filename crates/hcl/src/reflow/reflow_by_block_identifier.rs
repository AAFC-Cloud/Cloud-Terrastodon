use crate::reflow::HclReflower;
use eyre::bail;
use hcl::edit::Decor;
use hcl::edit::Decorate;
use hcl::edit::structure::Block;
use hcl::edit::structure::Body;
use hcl::edit::structure::Structure;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct ReflowByBlockIdentifier;

#[derive(Default)]
struct ReflowingBody {
    imports: Vec<Block>,
    moved: Vec<Block>,
    resources: Vec<Block>,
    other: Vec<Structure>,
    decor: Decor,
}

impl ReflowingBody {
    fn push_block(&mut self, block: Block) {
        match block.ident.as_str() {
            "import" => self.imports.push(block),
            "moved" => self.moved.push(block),
            "resource" => self.resources.push(block),
            _ => self.other.push(Structure::Block(block)),
        }
    }

    fn push_structure(&mut self, structure: Structure) {
        match structure {
            Structure::Block(block) => self.push_block(block),
            _ => self.other.push(structure),
        }
    }

    fn into_body(self) -> Body {
        let mut body = Body::new();
        for block in self.imports {
            body.push(block);
        }
        for block in self.moved {
            body.push(block);
        }
        for block in self.resources {
            body.push(block);
        }
        for structure in self.other {
            body.push(structure);
        }
        body.decorate(self.decor);
        body
    }
}

#[async_trait::async_trait]
impl HclReflower for ReflowByBlockIdentifier {
    async fn reflow(
        &mut self,
        hcl: HashMap<PathBuf, Body>,
    ) -> eyre::Result<HashMap<PathBuf, Body>> {
        let mut reflowed: HashMap<PathBuf, ReflowingBody> = HashMap::new();
        for (path, mut body) in hcl {
            let Some(dir) = path.parent() else {
                bail!(
                    "Expected path to have a parent directory: {}",
                    path.display()
                );
            };
            let decor = body.decor_mut();
            use crate::DecorExtensions;
            if !decor.is_empty() {
                reflowed.entry(path.clone()).or_default().decor = std::mem::take(decor);
            }
            for structure in body.into_iter() {
                match structure.into_block() {
                    Ok(block) => {
                        let new_path = get_structure_path(&path, dir, &block);
                        reflowed.entry(new_path).or_default().push_block(block);
                    }
                    Err(structure) => {
                        reflowed
                            .entry(path.clone())
                            .or_default()
                            .push_structure(structure);
                    }
                }
            }
        }
        Ok(reflowed
            .into_iter()
            .map(|(path, body)| (path, body.into_body()))
            .collect())
    }
}

fn get_structure_path(original_path: &PathBuf, dir: &std::path::Path, block: &Block) -> PathBuf {
    match block.ident.as_str() {
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
            let resource_name = block.labels.get(1).map(|s| s.as_str()).unwrap_or("unnamed");
            dir.join(format!("resource.{}.{}.tf", resource_type, resource_name))
        }
        "data" => {
            let data_type = block
                .labels
                .first()
                .map(|s| s.as_str())
                .unwrap_or("unknown_data");
            let data_name = block.labels.get(1).map(|s| s.as_str()).unwrap_or("unnamed");
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
        "import" | "moved" => block
            .body
            .get_attribute("to")
            .map(|attribute| attribute.value.to_string())
            .and_then(|resource_ref| resource_ref_to_resource_path(dir, resource_ref.trim()))
            .unwrap_or_else(|| original_path.clone()),
        _ => original_path.clone(),
    }
}

fn resource_ref_to_resource_path(dir: &std::path::Path, resource_ref: &str) -> Option<PathBuf> {
    let mut parts = resource_ref.split('.');
    let resource_type = parts.next()?;
    let resource_name = parts.next()?.split('[').next()?;
    if parts.next().is_some() {
        return None;
    }
    Some(dir.join(format!("resource.{}.{}.tf", resource_type, resource_name)))
}
