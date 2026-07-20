use crate::DecorExtensions;
use crate::HclProject;
use crate::reflow::HclReflower;
use eyre::bail;
use hcl::edit::Decor;
use hcl::edit::Decorate;
use hcl::edit::expr::Expression;
use hcl::edit::expr::ForExpr;
use hcl::edit::expr::Traversal;
use hcl::edit::expr::TraversalOperator;
use hcl::edit::prelude::Span;
use hcl::edit::structure::Block;
use hcl::edit::structure::Body;
use hcl::edit::structure::Structure;
use hcl::edit::visit_mut::VisitMut;
use hcl::edit::visit_mut::visit_expr_mut;
use hcl::edit::visit_mut::visit_for_template_expr_mut;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::HashSet;
use std::mem;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct ReflowByBlockIdentifier {
    single_file_path: Option<PathBuf>,
    mixed: bool,
}

impl ReflowByBlockIdentifier {
    pub fn new(single_file_path: Option<PathBuf>, mixed: bool) -> Self {
        Self {
            single_file_path,
            mixed,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum NodeKey {
    Terraform {
        path: PathBuf,
    },
    Provider {
        path: PathBuf,
    },
    Variable {
        path: PathBuf,
        name: String,
    },
    Local {
        path: PathBuf,
        name: String,
    },
    Data {
        path: PathBuf,
        data_type: String,
        name: String,
    },
    Resource {
        path: PathBuf,
        resource_type: String,
        name: String,
    },
    Output {
        path: PathBuf,
        name: String,
    },
    Misc {
        path: PathBuf,
        ordinal: usize,
    },
}

impl NodeKey {
    fn path(&self) -> &PathBuf {
        match self {
            NodeKey::Terraform { path }
            | NodeKey::Provider { path }
            | NodeKey::Variable { path, .. }
            | NodeKey::Local { path, .. }
            | NodeKey::Data { path, .. }
            | NodeKey::Resource { path, .. }
            | NodeKey::Output { path, .. }
            | NodeKey::Misc { path, .. } => path,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum NodeKind {
    Terraform,
    Provider,
    Variable,
    Local,
    Data,
    Resource,
    Output,
    Misc,
}

impl NodeKind {
    fn is_support(self) -> bool {
        matches!(self, NodeKind::Variable | NodeKind::Local | NodeKind::Data)
    }

    fn is_fixed(self) -> bool {
        matches!(
            self,
            NodeKind::Terraform | NodeKind::Provider | NodeKind::Output | NodeKind::Misc
        )
    }

    fn is_anchor(self) -> bool {
        matches!(self, NodeKind::Resource | NodeKind::Output)
    }

    fn support_priority(self) -> usize {
        match self {
            NodeKind::Variable => 0,
            NodeKind::Local => 1,
            NodeKind::Data => 2,
            NodeKind::Terraform => 3,
            NodeKind::Provider => 4,
            NodeKind::Resource => 5,
            NodeKind::Output => 6,
            NodeKind::Misc => 7,
        }
    }

    fn fixed_priority(self) -> usize {
        match self {
            NodeKind::Terraform => 0,
            NodeKind::Provider => 1,
            NodeKind::Misc => 2,
            NodeKind::Variable => 3,
            NodeKind::Local => 4,
            NodeKind::Data => 5,
            NodeKind::Resource => 6,
            NodeKind::Output => 7,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct SortKey {
    path: PathBuf,
    ordinal: usize,
}

impl SortKey {
    fn new(path: PathBuf, ordinal: usize) -> Self {
        Self { path, ordinal }
    }
}

impl Ord for SortKey {
    fn cmp(&self, other: &Self) -> Ordering {
        compare_paths(&self.path, &other.path).then(self.ordinal.cmp(&other.ordinal))
    }
}

impl PartialOrd for SortKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone)]
struct AttachedBlock {
    sort_key: SortKey,
    block: Block,
}

#[derive(Debug, Clone)]
struct BlockPayload {
    block: Block,
    imports: Vec<AttachedBlock>,
    moved: Vec<AttachedBlock>,
}

#[derive(Debug, Clone)]
enum NodePayload {
    Block(BlockPayload),
    Structure(Structure),
}

#[derive(Debug, Clone)]
struct Node {
    key: NodeKey,
    kind: NodeKind,
    source_path: PathBuf,
    source_location: LocationWithinFile,
    sort_key: SortKey,
    deps: HashSet<NodeKey>,
    resource_parent: Option<NodeKey>,
    payload: NodePayload,
}

#[derive(Debug, Clone)]
struct LocationWithinFile {
    path: PathBuf,
    line: usize,
    column: usize,
}

impl std::fmt::Display for LocationWithinFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.path.display(), self.line, self.column)
    }
}

#[derive(Debug, Clone)]
struct BodyDecorFragment {
    source_path: PathBuf,
    sort_key: SortKey,
    decor: Decor,
}

#[derive(Debug, Clone)]
struct PendingAttachment {
    target: NodeKey,
    block: AttachedBlock,
    kind: AttachmentKind,
}

#[derive(Debug, Clone, Copy)]
enum AttachmentKind {
    Import,
    Moved,
}

#[derive(Default)]
struct CollectedGraph {
    nodes: HashMap<NodeKey, Node>,
    pending_attachments: Vec<PendingAttachment>,
    body_decor_fragments: Vec<BodyDecorFragment>,
}

impl CollectedGraph {
    fn push_body_decor(&mut self, source_path: PathBuf, decor: Decor) {
        self.body_decor_fragments.push(BodyDecorFragment {
            sort_key: SortKey::new(source_path.clone(), 0),
            source_path,
            decor,
        });
    }

    fn push_node(&mut self, node: Node) -> eyre::Result<()> {
        if let Some(existing) = self.nodes.get(&node.key) {
            bail!(
                "Duplicate Terraform block identity {:?} encountered while reflowing. First block at {} conflicts with duplicate at {}. Reflow aborted to avoid data loss.",
                node.key,
                existing.source_location,
                node.source_location
            );
        }
        self.nodes.insert(node.key.clone(), node);
        Ok(())
    }

    fn finalize(mut self) -> HashMap<NodeKey, Node> {
        for pending in self.pending_attachments {
            let Some(target) = self.nodes.get_mut(&pending.target) else {
                let key = NodeKey::Misc {
                    path: pending.target.path().clone(),
                    ordinal: pending.block.sort_key.ordinal,
                };
                self.nodes.insert(
                    key.clone(),
                    Node {
                        key,
                        kind: NodeKind::Misc,
                        source_path: pending.block.sort_key.path.clone(),
                        source_location: fallback_location(&pending.block.sort_key.path),
                        sort_key: pending.block.sort_key,
                        deps: HashSet::new(),
                        resource_parent: None,
                        payload: NodePayload::Structure(Structure::Block(pending.block.block)),
                    },
                );
                continue;
            };

            let NodePayload::Block(payload) = &mut target.payload else {
                continue;
            };

            match pending.kind {
                AttachmentKind::Import => payload.imports.push(pending.block),
                AttachmentKind::Moved => payload.moved.push(pending.block),
            }
        }

        for node in self.nodes.values_mut() {
            if node.kind != NodeKind::Resource {
                continue;
            }
            let resource_deps = node
                .deps
                .iter()
                .filter(|dep| matches!(dep, NodeKey::Resource { .. }))
                .cloned()
                .collect::<HashSet<_>>();
            if resource_deps.len() == 1 {
                node.resource_parent = resource_deps.into_iter().next();
            }
        }

        self.nodes
    }
}

#[derive(Default)]
struct DependencyCollector {
    dir: PathBuf,
    deps: HashSet<NodeKey>,
    scoped_variables: HashMap<String, usize>,
}

impl DependencyCollector {
    fn collect_from_block(block: &mut Block, dir: &Path) -> HashSet<NodeKey> {
        let mut collector = Self {
            dir: dir.to_path_buf(),
            deps: HashSet::new(),
            scoped_variables: HashMap::new(),
        };
        collector.visit_body_mut(&mut block.body);
        collector.deps
    }
}

impl VisitMut for DependencyCollector {
    fn visit_expr_mut(&mut self, node: &mut Expression) {
        if let Some(traversal) = node.as_traversal()
            && let Some(dep) =
                dependency_from_traversal(&self.dir, traversal, &self.scoped_variables)
        {
            self.deps.insert(dep);
        }
        visit_expr_mut(self, node);
    }

    fn visit_for_expr_mut(&mut self, node: &mut ForExpr) {
        self.visit_expr_mut(&mut node.intro.collection_expr);

        if let Some(key_var) = &node.intro.key_var {
            self.push_scoped_variable(key_var.as_str());
        }
        self.push_scoped_variable(node.intro.value_var.as_str());

        if let Some(key_expr) = &mut node.key_expr {
            self.visit_expr_mut(key_expr);
        }
        self.visit_expr_mut(&mut node.value_expr);
        if let Some(cond) = &mut node.cond {
            self.visit_expr_mut(&mut cond.expr);
        }

        if let Some(key_var) = &node.intro.key_var {
            self.pop_scoped_variable(key_var.as_str());
        }
        self.pop_scoped_variable(node.intro.value_var.as_str());
    }

    fn visit_for_template_expr_mut(&mut self, node: &mut hcl::edit::template::ForTemplateExpr) {
        self.visit_expr_mut(&mut node.collection_expr);
        if let Some(key_var) = &node.key_var {
            self.push_scoped_variable(key_var.as_str());
        }
        self.push_scoped_variable(node.value_var.as_str());
        visit_for_template_expr_mut(self, node);
        if let Some(key_var) = &node.key_var {
            self.pop_scoped_variable(key_var.as_str());
        }
        self.pop_scoped_variable(node.value_var.as_str());
    }
}

impl DependencyCollector {
    fn push_scoped_variable(&mut self, variable: &str) {
        *self
            .scoped_variables
            .entry(variable.to_owned())
            .or_default() += 1;
    }

    fn pop_scoped_variable(&mut self, variable: &str) {
        let Some(count) = self.scoped_variables.get_mut(variable) else {
            return;
        };
        *count = count.saturating_sub(1);
        if *count == 0 {
            self.scoped_variables.remove(variable);
        }
    }
}

#[derive(Debug, Clone)]
struct Bucket {
    id: NodeKey,
    nodes: Vec<NodeKey>,
}

#[derive(Debug, Clone)]
struct BucketLayout<'a> {
    bucket: Bucket,
    nodes: Vec<&'a Node>,
}

#[async_trait::async_trait]
impl HclReflower for ReflowByBlockIdentifier {
    async fn reflow(&mut self, hcl: HclProject) -> eyre::Result<HclProject> {
        let mut collected = CollectedGraph::default();

        for (path, mut body) in hcl {
            let Some(dir) = path.parent() else {
                bail!(
                    "Expected path to have a parent directory: {}",
                    path.display()
                );
            };
            let source_content = body.to_string();

            let decor = body.decor_mut();
            if !decor.is_empty() {
                collected.push_body_decor(path.clone(), mem::take(decor));
            }

            for (ordinal, structure) in body.into_iter().enumerate() {
                collect_structure(
                    &mut collected,
                    &path,
                    &source_content,
                    dir,
                    ordinal,
                    structure,
                )?;
            }
        }

        let body_decor_fragments = collected.body_decor_fragments.clone();
        let nodes = collected.finalize();
        let buckets = if self.mixed {
            collapse_buckets(&nodes)
        } else {
            flat_buckets(&nodes)
        };

        if let Some(single_file_path) = self.single_file_path.as_ref() {
            let body = render_layout(
                BucketLayout {
                    bucket: Bucket {
                        id: NodeKey::Misc {
                            path: single_file_path.clone(),
                            ordinal: 0,
                        },
                        nodes: nodes.keys().cloned().collect(),
                    },
                    nodes: nodes.values().collect(),
                },
                &collected_decor_for_layout(&body_decor_fragments, nodes.values()),
            );
            return Ok(HclProject::from([(single_file_path.clone(), body)]));
        }

        let mut rendered = HclProject::new();
        for layout in bucket_layouts(&nodes, &buckets) {
            let output_path = bucket_output_path(&layout.nodes);
            let body = render_layout(
                layout.clone(),
                &collected_decor_for_layout(&body_decor_fragments, layout.nodes.iter().copied()),
            );
            rendered.insert(output_path, body);
        }

        let node_source_paths = nodes
            .values()
            .map(|node| node.source_path.clone())
            .collect::<HashSet<_>>();
        for fragment in body_decor_fragments {
            if node_source_paths.contains(&fragment.source_path) {
                continue;
            }
            let mut body = Body::new();
            body.decorate(fragment.decor);
            rendered.insert(fragment.source_path, body);
        }

        Ok(rendered)
    }
}

fn collect_structure(
    collected: &mut CollectedGraph,
    source_path: &Path,
    source_content: &str,
    dir: &Path,
    ordinal: usize,
    structure: Structure,
) -> eyre::Result<()> {
    match structure.into_block() {
        Ok(block) => collect_block(collected, source_path, source_content, dir, ordinal, block),
        Err(structure) => {
            let path = source_path.to_path_buf();
            let key = NodeKey::Misc {
                path: path.clone(),
                ordinal,
            };
            collected.push_node(Node {
                key,
                kind: NodeKind::Misc,
                source_path: path.clone(),
                source_location: fallback_location(&path),
                sort_key: SortKey::new(path, ordinal),
                deps: HashSet::new(),
                resource_parent: None,
                payload: NodePayload::Structure(structure),
            })?;
            Ok(())
        }
    }
}

fn collect_block(
    collected: &mut CollectedGraph,
    source_path: &Path,
    source_content: &str,
    dir: &Path,
    ordinal: usize,
    block: Block,
) -> eyre::Result<()> {
    let source_path = source_path.to_path_buf();
    let sort_key = SortKey::new(source_path.clone(), ordinal);
    match block.ident.as_str() {
        "terraform" => push_standard_block_node(
            collected,
            NodeKind::Terraform,
            source_path,
            source_content,
            sort_key,
            block,
        ),
        "provider" => push_standard_block_node(
            collected,
            NodeKind::Provider,
            source_path,
            source_content,
            sort_key,
            block,
        ),
        "variable" => push_standard_block_node(
            collected,
            NodeKind::Variable,
            source_path,
            source_content,
            sort_key,
            block,
        ),
        "data" => push_standard_block_node(
            collected,
            NodeKind::Data,
            source_path,
            source_content,
            sort_key,
            block,
        ),
        "resource" => push_standard_block_node(
            collected,
            NodeKind::Resource,
            source_path,
            source_content,
            sort_key,
            block,
        ),
        "output" => push_standard_block_node(
            collected,
            NodeKind::Output,
            source_path,
            source_content,
            sort_key,
            block,
        ),
        "locals" => {
            for local_node in split_locals_block(source_path, source_content, sort_key, block) {
                collected.push_node(local_node)?;
            }
            Ok(())
        }
        "import" | "moved" => {
            let Some(target) = resource_target_from_reference_block(dir, &block) else {
                let key = NodeKey::Misc {
                    path: source_path.clone(),
                    ordinal,
                };
                collected.push_node(Node {
                    key,
                    kind: NodeKind::Misc,
                    source_path: source_path.clone(),
                    source_location: block_location(&source_path, source_content, &block),
                    sort_key,
                    deps: HashSet::new(),
                    resource_parent: None,
                    payload: NodePayload::Structure(Structure::Block(block)),
                })?;
                return Ok(());
            };

            let attachment_kind = if block.ident.as_str() == "import" {
                AttachmentKind::Import
            } else {
                AttachmentKind::Moved
            };
            collected.pending_attachments.push(PendingAttachment {
                target,
                block: AttachedBlock { sort_key, block },
                kind: attachment_kind,
            });
            Ok(())
        }
        _ => {
            let key = NodeKey::Misc {
                path: source_path.clone(),
                ordinal,
            };
            collected.push_node(Node {
                key,
                kind: NodeKind::Misc,
                source_path: source_path.clone(),
                source_location: block_location(&source_path, source_content, &block),
                sort_key,
                deps: HashSet::new(),
                resource_parent: None,
                payload: NodePayload::Structure(Structure::Block(block)),
            })?;
            Ok(())
        }
    }
}

fn push_standard_block_node(
    collected: &mut CollectedGraph,
    kind: NodeKind,
    source_path: PathBuf,
    source_content: &str,
    sort_key: SortKey,
    mut block: Block,
) -> eyre::Result<()> {
    let Some(dir) = source_path.parent() else {
        return Ok(());
    };
    let key = node_key_from_block(dir, &block, kind, sort_key.ordinal);
    let source_location = block_location(&source_path, source_content, &block);
    let mut deps = if matches!(
        kind,
        NodeKind::Terraform | NodeKind::Provider | NodeKind::Misc
    ) {
        HashSet::new()
    } else {
        DependencyCollector::collect_from_block(&mut block, dir)
    };
    deps.remove(&key);

    collected.push_node(Node {
        key,
        kind,
        source_path,
        source_location,
        sort_key,
        deps,
        resource_parent: None,
        payload: NodePayload::Block(BlockPayload {
            block,
            imports: Vec::new(),
            moved: Vec::new(),
        }),
    })
}

fn split_locals_block(
    source_path: PathBuf,
    source_content: &str,
    sort_key: SortKey,
    block: Block,
) -> Vec<Node> {
    let Some(dir) = source_path.parent() else {
        return Vec::new();
    };

    let original_block = block.clone();
    let source_location = block_location(&source_path, source_content, &original_block);
    let body_decor = block.body.decor().clone();
    let attrs = block.body.into_attributes().collect::<Vec<_>>();

    if attrs.is_empty() {
        return vec![Node {
            key: NodeKey::Misc {
                path: source_path.clone(),
                ordinal: sort_key.ordinal,
            },
            kind: NodeKind::Misc,
            source_path: source_path.clone(),
            source_location: source_location.clone(),
            sort_key,
            deps: HashSet::new(),
            resource_parent: None,
            payload: NodePayload::Structure(Structure::Block(original_block)),
        }];
    }

    attrs
        .into_iter()
        .enumerate()
        .map(|(index, attr)| {
            let name = attr.key.as_str().to_owned();
            let key = NodeKey::Local {
                path: dir.join(format!("local.{}.tf", name)),
                name,
            };
            let mut local_block = original_block.clone();
            if index > 0 {
                *local_block.decor_mut() = Decor::default();
            }
            let mut body = Body::new();
            if index == 0 {
                body.decorate(body_decor.clone());
            }
            body.push(attr);
            local_block.body = body;

            let deps = DependencyCollector::collect_from_block(&mut local_block, dir)
                .into_iter()
                .filter(|dep| dep != &key)
                .collect::<HashSet<_>>();

            Node {
                key,
                kind: NodeKind::Local,
                source_path: source_path.clone(),
                source_location: source_location.clone(),
                sort_key: SortKey::new(sort_key.path.clone(), sort_key.ordinal + index),
                deps,
                resource_parent: None,
                payload: NodePayload::Block(BlockPayload {
                    block: local_block,
                    imports: Vec::new(),
                    moved: Vec::new(),
                }),
            }
        })
        .collect()
}

fn node_key_from_block(dir: &Path, block: &Block, kind: NodeKind, ordinal: usize) -> NodeKey {
    match kind {
        NodeKind::Terraform => NodeKey::Terraform {
            path: dir.join("terraform.tf"),
        },
        NodeKind::Provider => NodeKey::Provider {
            path: provider_path(dir, block),
        },
        NodeKind::Variable => NodeKey::Variable {
            path: variable_path(dir, block),
            name: block
                .labels
                .first()
                .map(|label| label.as_str().to_owned())
                .unwrap_or_else(|| "unnamed".to_owned()),
        },
        NodeKind::Local => NodeKey::Misc {
            path: dir.join("locals.tf"),
            ordinal,
        },
        NodeKind::Data => NodeKey::Data {
            path: data_path(dir, block),
            data_type: block
                .labels
                .first()
                .map(|label| label.as_str().to_owned())
                .unwrap_or_else(|| "unknown_data".to_owned()),
            name: block
                .labels
                .get(1)
                .map(|label| label.as_str().to_owned())
                .unwrap_or_else(|| "unnamed".to_owned()),
        },
        NodeKind::Resource => NodeKey::Resource {
            path: resource_path(dir, block),
            resource_type: block
                .labels
                .first()
                .map(|label| label.as_str().to_owned())
                .unwrap_or_else(|| "unknown_resource".to_owned()),
            name: block
                .labels
                .get(1)
                .map(|label| label.as_str().to_owned())
                .unwrap_or_else(|| "unnamed".to_owned()),
        },
        NodeKind::Output => NodeKey::Output {
            path: output_path(dir, block),
            name: block
                .labels
                .first()
                .map(|label| label.as_str().to_owned())
                .unwrap_or_else(|| "unnamed".to_owned()),
        },
        NodeKind::Misc => NodeKey::Misc {
            path: dir.join("misc.tf"),
            ordinal,
        },
    }
}

fn dependency_from_traversal(
    dir: &Path,
    traversal: &Traversal,
    scoped_variables: &HashMap<String, usize>,
) -> Option<NodeKey> {
    let root = traversal.expr.as_variable()?.as_str();
    if scoped_variables.contains_key(root) {
        return None;
    }
    match root {
        "data" => {
            let [kind, name, ..] = traversal.operators.as_slice() else {
                return None;
            };
            let TraversalOperator::GetAttr(data_type) = kind.value() else {
                return None;
            };
            let TraversalOperator::GetAttr(name) = name.value() else {
                return None;
            };
            Some(NodeKey::Data {
                path: dir.join(format!("data.{}.{}.tf", data_type.as_str(), name.as_str())),
                data_type: data_type.as_str().to_owned(),
                name: name.as_str().to_owned(),
            })
        }
        "var" => {
            let [name, ..] = traversal.operators.as_slice() else {
                return None;
            };
            let TraversalOperator::GetAttr(name) = name.value() else {
                return None;
            };
            Some(NodeKey::Variable {
                path: dir.join(format!("variable.{}.tf", name.as_str())),
                name: name.as_str().to_owned(),
            })
        }
        "local" => {
            let [name, ..] = traversal.operators.as_slice() else {
                return None;
            };
            let TraversalOperator::GetAttr(name) = name.value() else {
                return None;
            };
            Some(NodeKey::Local {
                path: dir.join(format!("local.{}.tf", name.as_str())),
                name: name.as_str().to_owned(),
            })
        }
        "count" | "each" | "self" | "path" | "module" | "terraform" => None,
        resource_type => {
            let [name, ..] = traversal.operators.as_slice() else {
                return None;
            };
            let TraversalOperator::GetAttr(name) = name.value() else {
                return None;
            };
            Some(NodeKey::Resource {
                path: dir.join(format!("resource.{}.{}.tf", resource_type, name.as_str())),
                resource_type: resource_type.to_owned(),
                name: name.as_str().to_owned(),
            })
        }
    }
}

fn resource_target_from_reference_block(dir: &Path, block: &Block) -> Option<NodeKey> {
    let resource_ref = block.body.get_attribute("to")?.value.to_string();
    let resource_key = normalize_resource_reference(resource_ref.trim())?;
    let mut parts = resource_key.split('.');
    let resource_type = parts.next()?.to_owned();
    let name = parts.next()?.to_owned();
    Some(NodeKey::Resource {
        path: dir.join(format!("resource.{}.{}.tf", resource_type, name)),
        resource_type,
        name,
    })
}

fn collapse_buckets(nodes: &HashMap<NodeKey, Node>) -> Vec<Bucket> {
    let mut bucket_of = nodes
        .keys()
        .map(|key| (key.clone(), key.clone()))
        .collect::<HashMap<_, _>>();

    loop {
        let buckets = bucket_members(nodes, &bucket_of);
        if let Some((source, target)) = buckets
            .iter()
            .filter_map(|bucket| {
                child_resource_merge_target(bucket, nodes, &bucket_of)
                    .map(|target| (bucket.id.clone(), target))
            })
            .min_by_key(|(source, _)| source.path().clone())
        {
            for bucket in bucket_of.values_mut() {
                if *bucket == source {
                    *bucket = target.clone();
                }
            }
            continue;
        }

        let buckets = bucket_members(nodes, &bucket_of);
        if let Some((source, target)) = buckets
            .iter()
            .filter_map(|bucket| {
                support_merge_target(bucket, nodes, &bucket_of)
                    .map(|target| (bucket.id.clone(), target))
            })
            .min_by_key(|(source, _)| source.path().clone())
        {
            for bucket in bucket_of.values_mut() {
                if *bucket == source {
                    *bucket = target.clone();
                }
            }
            continue;
        }

        break;
    }

    bucket_members(nodes, &bucket_of)
}

fn flat_buckets(nodes: &HashMap<NodeKey, Node>) -> Vec<Bucket> {
    let mut buckets = nodes
        .keys()
        .cloned()
        .map(|key| Bucket {
            id: key.clone(),
            nodes: vec![key],
        })
        .collect::<Vec<_>>();
    buckets.sort_by_key(bucket_sort_key);
    buckets
}

fn bucket_members(
    nodes: &HashMap<NodeKey, Node>,
    bucket_of: &HashMap<NodeKey, NodeKey>,
) -> Vec<Bucket> {
    let mut members = HashMap::<NodeKey, Vec<NodeKey>>::new();
    for key in nodes.keys() {
        if let Some(bucket) = bucket_of.get(key) {
            members.entry(bucket.clone()).or_default().push(key.clone());
        }
    }
    let mut buckets = members
        .into_iter()
        .map(|(id, mut nodes_in_bucket)| {
            nodes_in_bucket.sort_by_key(|key| nodes[key].sort_key.clone());
            Bucket {
                id,
                nodes: nodes_in_bucket,
            }
        })
        .collect::<Vec<_>>();
    buckets.sort_by_key(bucket_sort_key);
    buckets
}

fn bucket_sort_key(bucket: &Bucket) -> SortKey {
    bucket
        .nodes
        .first()
        .map(|key| SortKey::new(key.path().clone(), 0))
        .unwrap_or_else(|| SortKey::new(PathBuf::new(), 0))
}

fn child_resource_merge_target(
    bucket: &Bucket,
    nodes: &HashMap<NodeKey, Node>,
    bucket_of: &HashMap<NodeKey, NodeKey>,
) -> Option<NodeKey> {
    if bucket
        .nodes
        .iter()
        .any(|key| nodes[key].kind.is_fixed() && nodes[key].kind != NodeKind::Misc)
    {
        return None;
    }

    let root_resources = bucket
        .nodes
        .iter()
        .filter(|key| nodes[*key].kind == NodeKind::Resource)
        .filter(|key| {
            nodes[*key]
                .resource_parent
                .as_ref()
                .and_then(|parent| bucket_of.get(parent))
                .is_none_or(|parent_bucket| parent_bucket != &bucket.id)
        })
        .cloned()
        .collect::<Vec<_>>();

    if root_resources.len() != 1 {
        return None;
    }

    let root = nodes.get(&root_resources[0])?;
    let parent = root.resource_parent.as_ref()?;
    let target = bucket_of.get(parent)?;
    if *target == bucket.id {
        None
    } else {
        Some(target.clone())
    }
}

fn support_merge_target(
    bucket: &Bucket,
    nodes: &HashMap<NodeKey, Node>,
    bucket_of: &HashMap<NodeKey, NodeKey>,
) -> Option<NodeKey> {
    if bucket.nodes.is_empty() || bucket.nodes.iter().any(|key| !nodes[key].kind.is_support()) {
        return None;
    }

    let member_keys = bucket.nodes.iter().cloned().collect::<HashSet<_>>();
    let mut consumer_buckets = HashSet::new();
    for consumer in nodes.values() {
        let Some(consumer_bucket) = bucket_of.get(&consumer.key) else {
            continue;
        };
        if *consumer_bucket == bucket.id {
            continue;
        }
        if consumer.deps.iter().any(|dep| member_keys.contains(dep)) {
            consumer_buckets.insert(consumer_bucket.clone());
        }
    }

    if consumer_buckets.len() == 1 {
        consumer_buckets.into_iter().next()
    } else {
        None
    }
}

fn bucket_layouts<'a>(
    nodes: &'a HashMap<NodeKey, Node>,
    buckets: &'a [Bucket],
) -> Vec<BucketLayout<'a>> {
    buckets
        .iter()
        .map(|bucket| BucketLayout {
            bucket: bucket.clone(),
            nodes: bucket
                .nodes
                .iter()
                .filter_map(|key| nodes.get(key))
                .collect(),
        })
        .collect()
}

fn bucket_output_path(nodes: &[&Node]) -> PathBuf {
    let nodes_by_key = nodes
        .iter()
        .map(|node| (node.key.clone(), *node))
        .collect::<HashMap<_, _>>();
    let root_resource = nodes
        .iter()
        .filter(|node| node.kind == NodeKind::Resource)
        .filter(|node| {
            node.resource_parent
                .as_ref()
                .is_none_or(|parent| !nodes_by_key.contains_key(parent))
        })
        .min_by_key(|node| node.sort_key.clone())
        .map(|node| node.key.path().clone());
    root_resource.unwrap_or_else(|| {
        nodes
            .iter()
            .min_by_key(|node| node.sort_key.clone())
            .map(|node| node.key.path().clone())
            .unwrap_or_default()
    })
}

fn collected_decor_for_layout<'a>(
    fragments: &[BodyDecorFragment],
    nodes: impl IntoIterator<Item = &'a Node>,
) -> Option<Decor> {
    let source_paths = nodes
        .into_iter()
        .map(|node| node.source_path.clone())
        .collect::<HashSet<_>>();
    fragments
        .iter()
        .filter(|fragment| source_paths.contains(&fragment.source_path))
        .min_by_key(|fragment| fragment.sort_key.clone())
        .map(|fragment| fragment.decor.clone())
}

fn render_layout(layout: BucketLayout<'_>, decor: &Option<Decor>) -> Body {
    let _ = &layout.bucket;
    let nodes_by_key = layout
        .nodes
        .iter()
        .map(|node| (node.key.clone(), *node))
        .collect::<HashMap<_, _>>();
    let consumer_map = build_consumer_map(layout.nodes.iter().copied());
    let attachment_map =
        build_support_attachment_map(layout.nodes.iter().copied(), &consumer_map, &nodes_by_key);
    let resource_children = build_resource_children(layout.nodes.iter().copied(), &nodes_by_key);
    let output_keys = layout
        .nodes
        .iter()
        .filter(|node| node.kind == NodeKind::Output)
        .map(|node| node.key.clone())
        .collect::<Vec<_>>();
    let root_resources = layout
        .nodes
        .iter()
        .filter(|node| node.kind == NodeKind::Resource)
        .filter(|node| {
            node.resource_parent
                .as_ref()
                .is_none_or(|parent| !nodes_by_key.contains_key(parent))
        })
        .map(|node| node.key.clone())
        .collect::<Vec<_>>();

    let mut body = Body::new();

    let mut fixed_nodes = layout
        .nodes
        .iter()
        .filter(|node| {
            matches!(
                node.kind,
                NodeKind::Terraform | NodeKind::Provider | NodeKind::Misc
            )
        })
        .collect::<Vec<_>>();
    fixed_nodes.sort_by(|left, right| {
        left.kind
            .fixed_priority()
            .cmp(&right.kind.fixed_priority())
            .then(left.sort_key.cmp(&right.sort_key))
    });
    for node in fixed_nodes {
        push_non_resource_node(&mut body, node);
    }

    let mut emitted_support = HashSet::new();
    emit_support_group(
        &mut body,
        None,
        &attachment_map,
        &nodes_by_key,
        &mut emitted_support,
    );

    let mut root_resources = root_resources;
    root_resources.sort_by_key(|key| nodes_by_key[key].sort_key.clone());
    for resource in root_resources {
        emit_resource_subtree(
            &mut body,
            &resource,
            &resource_children,
            &attachment_map,
            &nodes_by_key,
            &mut emitted_support,
        );
    }

    let mut outputs = output_keys;
    outputs.sort_by_key(|key| nodes_by_key[key].sort_key.clone());
    for output in outputs {
        emit_support_group(
            &mut body,
            Some(output.clone()),
            &attachment_map,
            &nodes_by_key,
            &mut emitted_support,
        );
        push_non_resource_node(&mut body, nodes_by_key[&output]);
    }

    if let Some(decor) = decor {
        body.decorate(decor.clone());
    }

    body
}

fn build_consumer_map<'a>(
    nodes: impl IntoIterator<Item = &'a Node>,
) -> HashMap<NodeKey, Vec<NodeKey>> {
    let all_nodes = nodes
        .into_iter()
        .map(|node| (node.key.clone(), node))
        .collect::<HashMap<_, _>>();
    let mut consumers = HashMap::<NodeKey, Vec<NodeKey>>::new();
    for node in all_nodes.values() {
        for dep in &node.deps {
            if all_nodes.contains_key(dep) {
                consumers
                    .entry(dep.clone())
                    .or_default()
                    .push(node.key.clone());
            }
        }
    }
    consumers
}

fn build_resource_children<'a>(
    nodes: impl IntoIterator<Item = &'a Node>,
    nodes_by_key: &HashMap<NodeKey, &Node>,
) -> HashMap<NodeKey, Vec<NodeKey>> {
    let mut children = HashMap::<NodeKey, Vec<NodeKey>>::new();
    for node in nodes {
        if node.kind != NodeKind::Resource {
            continue;
        }
        let Some(parent) = node.resource_parent.as_ref() else {
            continue;
        };
        if nodes_by_key.contains_key(parent) {
            children
                .entry(parent.clone())
                .or_default()
                .push(node.key.clone());
        }
    }
    for child_nodes in children.values_mut() {
        child_nodes.sort_by_key(|key| nodes_by_key[key].sort_key.clone());
    }
    children
}

fn build_support_attachment_map<'a>(
    nodes: impl IntoIterator<Item = &'a Node>,
    consumers: &HashMap<NodeKey, Vec<NodeKey>>,
    nodes_by_key: &HashMap<NodeKey, &Node>,
) -> HashMap<Option<NodeKey>, Vec<NodeKey>> {
    let mut attachments = HashMap::<Option<NodeKey>, Vec<NodeKey>>::new();
    let mut memo = HashMap::<NodeKey, HashSet<NodeKey>>::new();

    for node in nodes {
        if !node.kind.is_support() {
            continue;
        }
        let anchors = anchor_consumers(
            &node.key,
            consumers,
            nodes_by_key,
            &mut memo,
            &mut HashSet::new(),
        );
        let attachment = classify_attachment(&anchors, nodes_by_key);
        attachments
            .entry(attachment)
            .or_default()
            .push(node.key.clone());
    }

    for keys in attachments.values_mut() {
        keys.sort_by(|left, right| {
            let left_node = nodes_by_key[left];
            let right_node = nodes_by_key[right];
            left_node
                .kind
                .support_priority()
                .cmp(&right_node.kind.support_priority())
                .then(left_node.sort_key.cmp(&right_node.sort_key))
        });
    }

    attachments
}

fn anchor_consumers(
    node: &NodeKey,
    consumers: &HashMap<NodeKey, Vec<NodeKey>>,
    nodes_by_key: &HashMap<NodeKey, &Node>,
    memo: &mut HashMap<NodeKey, HashSet<NodeKey>>,
    visiting: &mut HashSet<NodeKey>,
) -> HashSet<NodeKey> {
    if let Some(found) = memo.get(node) {
        return found.clone();
    }
    if !visiting.insert(node.clone()) {
        return HashSet::new();
    }

    let mut anchors = HashSet::new();
    for consumer in consumers.get(node).into_iter().flatten() {
        let Some(consumer_node) = nodes_by_key.get(consumer) else {
            continue;
        };
        if consumer_node.kind.is_anchor() {
            anchors.insert(consumer.clone());
            continue;
        }
        anchors.extend(anchor_consumers(
            consumer,
            consumers,
            nodes_by_key,
            memo,
            visiting,
        ));
    }

    visiting.remove(node);
    memo.insert(node.clone(), anchors.clone());
    anchors
}

fn classify_attachment(
    anchors: &HashSet<NodeKey>,
    nodes_by_key: &HashMap<NodeKey, &Node>,
) -> Option<NodeKey> {
    if anchors.is_empty() {
        return None;
    }

    let mut resource_anchors = anchors
        .iter()
        .filter(|anchor| nodes_by_key[anchor].kind == NodeKind::Resource)
        .cloned()
        .collect::<Vec<_>>();
    let output_anchors = anchors
        .iter()
        .filter(|anchor| nodes_by_key[anchor].kind == NodeKind::Output)
        .cloned()
        .collect::<Vec<_>>();

    if !resource_anchors.is_empty() && output_anchors.is_empty() {
        resource_anchors.sort_by_key(|key| nodes_by_key[key].sort_key.clone());
        return lowest_common_resource_ancestor(&resource_anchors, nodes_by_key);
    }
    if resource_anchors.is_empty() && output_anchors.len() == 1 {
        return output_anchors.into_iter().next();
    }

    None
}

fn lowest_common_resource_ancestor(
    resources: &[NodeKey],
    nodes_by_key: &HashMap<NodeKey, &Node>,
) -> Option<NodeKey> {
    let first = resources.first()?.clone();
    let first_chain = resource_ancestor_chain(&first, nodes_by_key);
    first_chain.into_iter().find(|candidate| {
        resources
            .iter()
            .all(|resource| resource_ancestor_chain(resource, nodes_by_key).contains(candidate))
    })
}

fn resource_ancestor_chain(
    resource: &NodeKey,
    nodes_by_key: &HashMap<NodeKey, &Node>,
) -> Vec<NodeKey> {
    let mut chain = Vec::new();
    let mut current = Some(resource.clone());
    while let Some(node_key) = current {
        chain.push(node_key.clone());
        current = nodes_by_key
            .get(&node_key)
            .and_then(|node| node.resource_parent.as_ref())
            .filter(|parent| nodes_by_key.contains_key(*parent))
            .cloned();
    }
    chain
}

fn emit_resource_subtree(
    body: &mut Body,
    resource: &NodeKey,
    children: &HashMap<NodeKey, Vec<NodeKey>>,
    attachment_map: &HashMap<Option<NodeKey>, Vec<NodeKey>>,
    nodes_by_key: &HashMap<NodeKey, &Node>,
    emitted_support: &mut HashSet<NodeKey>,
) {
    emit_support_group(
        body,
        Some(resource.clone()),
        attachment_map,
        nodes_by_key,
        emitted_support,
    );

    let resource_node = nodes_by_key[resource];
    let NodePayload::Block(payload) = &resource_node.payload else {
        return;
    };

    let mut imports = payload.imports.clone();
    imports.sort_by_key(|attached| attached.sort_key.clone());
    for import in imports {
        push_block_for_emission(body, import.block);
    }

    let mut moved = payload.moved.clone();
    moved.sort_by_key(|attached| attached.sort_key.clone());
    for moved in moved {
        push_block_for_emission(body, moved.block);
    }

    push_block_for_emission(body, payload.block.clone());

    for child in children.get(resource).into_iter().flatten() {
        emit_resource_subtree(
            body,
            child,
            children,
            attachment_map,
            nodes_by_key,
            emitted_support,
        );
    }
}

fn emit_support_group(
    body: &mut Body,
    anchor: Option<NodeKey>,
    attachment_map: &HashMap<Option<NodeKey>, Vec<NodeKey>>,
    nodes_by_key: &HashMap<NodeKey, &Node>,
    emitted_support: &mut HashSet<NodeKey>,
) {
    let Some(group) = attachment_map.get(&anchor) else {
        return;
    };

    for key in group {
        emit_support_node(body, key, group, nodes_by_key, emitted_support);
    }
}

fn emit_support_node(
    body: &mut Body,
    key: &NodeKey,
    group: &[NodeKey],
    nodes_by_key: &HashMap<NodeKey, &Node>,
    emitted_support: &mut HashSet<NodeKey>,
) {
    if !emitted_support.insert(key.clone()) {
        return;
    }

    let group_keys = group.iter().cloned().collect::<HashSet<_>>();
    let node = nodes_by_key[key];
    let mut deps = node
        .deps
        .iter()
        .filter(|dep| group_keys.contains(*dep))
        .cloned()
        .collect::<Vec<_>>();
    deps.sort_by(|left, right| {
        let left_node = nodes_by_key[left];
        let right_node = nodes_by_key[right];
        left_node
            .kind
            .support_priority()
            .cmp(&right_node.kind.support_priority())
            .then(left_node.sort_key.cmp(&right_node.sort_key))
    });
    for dep in deps {
        emit_support_node(body, &dep, group, nodes_by_key, emitted_support);
    }

    push_non_resource_node(body, node);
}

fn push_non_resource_node(body: &mut Body, node: &Node) {
    match &node.payload {
        NodePayload::Block(payload) => push_block_for_emission(body, payload.block.clone()),
        NodePayload::Structure(Structure::Block(block)) => {
            push_block_for_emission(body, block.clone())
        }
        NodePayload::Structure(structure) => body.push(structure.clone()),
    }
}

fn push_block_for_emission(body: &mut Body, mut block: Block) {
    if block
        .decor()
        .prefix()
        .is_some_and(|prefix| prefix.trim().is_empty())
    {
        block.decor_mut().set_prefix("");
    }
    body.push(block);
}

fn block_location(path: &Path, source_content: &str, block: &Block) -> LocationWithinFile {
    let resolved_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

    block
        .span()
        .and_then(|span| find_line_column(source_content, span.start))
        .map(|(line, column)| LocationWithinFile {
            path: resolved_path.clone(),
            line,
            column,
        })
        .unwrap_or_else(|| LocationWithinFile {
            path: resolved_path,
            line: 1,
            column: 1,
        })
}

fn fallback_location(path: &Path) -> LocationWithinFile {
    LocationWithinFile {
        path: path.canonicalize().unwrap_or_else(|_| path.to_path_buf()),
        line: 1,
        column: 1,
    }
}

fn find_line_column(content: &str, byte_index: usize) -> Option<(usize, usize)> {
    let mut line = 1;
    let mut column = 1;
    let mut current_index = 0;

    for character in content.chars() {
        if current_index == byte_index {
            return Some((line, column));
        }

        if character == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }

        current_index += character.len_utf8();
    }

    if current_index == byte_index {
        Some((line, column))
    } else {
        None
    }
}

fn provider_path(dir: &Path, block: &Block) -> PathBuf {
    let provider_name = block
        .labels
        .first()
        .map(|label| label.as_str())
        .unwrap_or("unknown_provider");
    let alias = block
        .body
        .get_attribute("alias")
        .and_then(|attribute| attribute.value.as_str());
    match alias {
        Some(alias) => dir.join(format!("provider.{}.{}.tf", provider_name, alias)),
        None => dir.join(format!("provider.{}.tf", provider_name)),
    }
}

fn resource_path(dir: &Path, block: &Block) -> PathBuf {
    let resource_type = block
        .labels
        .first()
        .map(|label| label.as_str())
        .unwrap_or("unknown_resource");
    let name = block
        .labels
        .get(1)
        .map(|label| label.as_str())
        .unwrap_or("unnamed");
    dir.join(format!("resource.{}.{}.tf", resource_type, name))
}

fn data_path(dir: &Path, block: &Block) -> PathBuf {
    let data_type = block
        .labels
        .first()
        .map(|label| label.as_str())
        .unwrap_or("unknown_data");
    let name = block
        .labels
        .get(1)
        .map(|label| label.as_str())
        .unwrap_or("unnamed");
    dir.join(format!("data.{}.{}.tf", data_type, name))
}

fn variable_path(dir: &Path, block: &Block) -> PathBuf {
    let name = block
        .labels
        .first()
        .map(|label| label.as_str())
        .unwrap_or("unnamed");
    dir.join(format!("variable.{}.tf", name))
}

fn output_path(dir: &Path, block: &Block) -> PathBuf {
    let name = block
        .labels
        .first()
        .map(|label| label.as_str())
        .unwrap_or("unnamed");
    dir.join(format!("output.{}.tf", name))
}

fn compare_paths(left: &Path, right: &Path) -> Ordering {
    left.to_string_lossy().cmp(&right.to_string_lossy())
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
