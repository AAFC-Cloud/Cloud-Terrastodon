use crate::reflow::HclReflower;
use eyre::bail;
use hcl::edit::Decor;
use hcl::edit::Decorate;
use hcl::edit::structure::Block;
use hcl::edit::structure::Body;
use hcl::edit::structure::Structure;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::mem;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct ReflowByBlockIdentifier {
    single_file_path: Option<PathBuf>,
}

impl ReflowByBlockIdentifier {
    pub fn new(single_file_path: Option<PathBuf>) -> Self {
        Self { single_file_path }
    }
}

#[derive(Debug)]
struct OrderedBlock {
    sort_key: PathBuf,
    block: Block,
}

#[derive(Debug)]
struct OrderedStructure {
    sort_key: PathBuf,
    structure: Structure,
}

#[derive(Debug, Default)]
struct ResourceSection {
    sort_key: PathBuf,
    imports: Vec<Block>,
    moved: Vec<Block>,
    resource: Option<Block>,
}

#[derive(Default)]
struct ReflowingBody {
    terraform: Vec<OrderedBlock>,
    providers: Vec<OrderedBlock>,
    other: Vec<OrderedStructure>,
    variables: Vec<OrderedBlock>,
    data: Vec<OrderedBlock>,
    resources: HashMap<String, ResourceSection>,
    outputs: Vec<OrderedBlock>,
    decor: Decor,
}

impl ReflowingBody {
    fn push_block(&mut self, block: Block, sort_key: PathBuf) {
        match block.ident.as_str() {
            "terraform" => self.terraform.push(OrderedBlock { sort_key, block }),
            "provider" => self.providers.push(OrderedBlock { sort_key, block }),
            "variable" => self.variables.push(OrderedBlock { sort_key, block }),
            "data" => self.data.push(OrderedBlock { sort_key, block }),
            "resource" => self.push_resource(block, sort_key),
            "import" => self.push_resource_reference(block, sort_key, ResourceReferenceKind::Import),
            "moved" => self.push_resource_reference(block, sort_key, ResourceReferenceKind::Moved),
            "output" => self.outputs.push(OrderedBlock { sort_key, block }),
            _ => self.other.push(OrderedStructure {
                sort_key,
                structure: Structure::Block(block),
            }),
        }
    }

    fn push_resource(&mut self, block: Block, sort_key: PathBuf) {
        let Some(resource_key) = resource_key_from_resource_block(&block) else {
            self.other.push(OrderedStructure {
                sort_key,
                structure: Structure::Block(block),
            });
            return;
        };

        let section = self.resource_section(resource_key, sort_key);
        section.resource = Some(block);
    }

    fn push_resource_reference(
        &mut self,
        block: Block,
        sort_key: PathBuf,
        kind: ResourceReferenceKind,
    ) {
        let Some(resource_key) = resource_key_from_reference_block(&block) else {
            self.other.push(OrderedStructure {
                sort_key,
                structure: Structure::Block(block),
            });
            return;
        };

        let section = self.resource_section(resource_key, sort_key);
        match kind {
            ResourceReferenceKind::Import => section.imports.push(block),
            ResourceReferenceKind::Moved => section.moved.push(block),
        }
    }

    fn resource_section(&mut self, resource_key: String, sort_key: PathBuf) -> &mut ResourceSection {
        match self.resources.entry(resource_key) {
            Entry::Occupied(mut occupied) => {
                if compare_paths(&sort_key, &occupied.get().sort_key) == Ordering::Less {
                    occupied.get_mut().sort_key = sort_key;
                }
                occupied.into_mut()
            }
            Entry::Vacant(vacant) => vacant.insert(ResourceSection {
                sort_key,
                ..Default::default()
            }),
        }
    }

    fn push_structure(&mut self, structure: Structure, sort_key: PathBuf) {
        match structure {
            Structure::Block(block) => self.push_block(block, sort_key),
            _ => self.other.push(OrderedStructure {
                sort_key,
                structure,
            }),
        }
    }

    fn into_body(self) -> Body {
        let mut body = Body::new();

        push_ordered_blocks(&mut body, self.terraform);
        push_ordered_blocks(&mut body, self.providers);
        push_ordered_structures(&mut body, self.other);
        push_ordered_blocks(&mut body, self.variables);
        push_ordered_blocks(&mut body, self.data);

        let mut resource_sections = self.resources.into_values().collect::<Vec<_>>();
        resource_sections.sort_by(|left, right| compare_paths(&left.sort_key, &right.sort_key));
        for section in resource_sections {
            for block in section.imports {
                body.push(block);
            }
            for block in section.moved {
                body.push(block);
            }
            if let Some(resource) = section.resource {
                body.push(resource);
            }
        }

        push_ordered_blocks(&mut body, self.outputs);
        body.decorate(self.decor);
        body
    }
}

#[derive(Debug, Clone, Copy)]
enum ResourceReferenceKind {
    Import,
    Moved,
}

fn push_ordered_blocks(body: &mut Body, mut blocks: Vec<OrderedBlock>) {
    blocks.sort_by(|left, right| compare_paths(&left.sort_key, &right.sort_key));
    for OrderedBlock { block, .. } in blocks {
        body.push(block);
    }
}

fn push_ordered_structures(body: &mut Body, mut structures: Vec<OrderedStructure>) {
    structures.sort_by(|left, right| compare_paths(&left.sort_key, &right.sort_key));
    for OrderedStructure { structure, .. } in structures {
        body.push(structure);
    }
}

fn compare_paths(left: &Path, right: &Path) -> Ordering {
    left.to_string_lossy().cmp(&right.to_string_lossy())
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
                reflowed.entry(self.target_path(&path)).or_default().decor = mem::take(decor);
            }
            for structure in body.into_iter() {
                match structure.into_block() {
                    Ok(block) => {
                        let new_path = self.target_path(&get_structure_path(&path, dir, &block));
                        let sort_key = get_structure_path(&path, dir, &block);
                        reflowed
                            .entry(new_path)
                            .or_default()
                            .push_block(block, sort_key);
                    }
                    Err(structure) => {
                        let target_path = self.target_path(&path);
                        reflowed
                            .entry(target_path)
                            .or_default()
                            .push_structure(structure, path.clone());
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

impl ReflowByBlockIdentifier {
    fn target_path(&self, original_path: &Path) -> PathBuf {
        self.single_file_path
            .as_ref()
            .cloned()
            .unwrap_or_else(|| original_path.to_path_buf())
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
    let resource_key = normalize_resource_reference(resource_ref)?;
    Some(resource_key_to_path(dir, &resource_key))
}

fn resource_key_to_path(dir: &Path, resource_key: &str) -> PathBuf {
    let mut parts = resource_key.split('.');
    let resource_type = parts.next().unwrap_or("unknown_resource");
    let resource_name = parts.next().unwrap_or("unnamed");
    dir.join(format!("resource.{}.{}.tf", resource_type, resource_name))
}

fn resource_key_from_reference_block(block: &Block) -> Option<String> {
    let resource_ref = block.body.get_attribute("to")?.value.to_string();
    normalize_resource_reference(resource_ref.trim())
}

fn resource_key_from_resource_block(block: &Block) -> Option<String> {
    let resource_type = block.labels.first()?.as_str();
    let resource_name = block.labels.get(1)?.as_str();
    Some(format!("{}.{}", resource_type, resource_name))
}

fn normalize_resource_reference(resource_ref: &str) -> Option<String> {
    let mut parts = resource_ref.split('.');
    let resource_type = parts.next()?;
    let resource_name = parts.next()?.split('[').next()?;
    if parts.next().is_some() {
        return None;
    }
    Some(format!("{}.{}", resource_type, resource_name))
}
