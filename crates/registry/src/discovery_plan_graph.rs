use crate::CancellationScope;
use crate::DiscoveryPlan;
use crate::DiscoveryPlanEdge;
use crate::DiscoveryPlanField;
use crate::DiscoveryPlanNode;
use crate::DiscoveryPlanNodeKind;
use crate::Function;
use crate::FunctionKind;
use crate::RuntimeValue;
use crate::describe_function;
use crate::describe_shape;
use facet::Def;
use facet::Facet;
use facet::KnownPointer;
use facet::OpaqueBorrow;
use facet::Shape;
use facet::Type;
use facet::UserType;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ptr;

/// Owns the relationship between internal node indices, reflected shape identity,
/// and the first derivation establishing each shape's structural availability.
pub(crate) struct DiscoveryPlanGraph {
    nodes: Vec<DiscoveryPlanNode>,
    edges: Vec<DiscoveryPlanEdge>,
    shapes: HashMap<&'static Shape, usize>,
    functions: Vec<(&'static Function, usize)>,
    witnesses: HashMap<usize, usize>,
}

impl DiscoveryPlanGraph {
    pub(crate) fn new(
        functions: impl IntoIterator<Item = &'static Function>,
        seeds: impl IntoIterator<Item = &'static Shape>,
        targets: impl IntoIterator<Item = &'static Shape>,
        cancellation: &CancellationScope,
    ) -> eyre::Result<Self> {
        let mut seen = HashSet::new();
        let mut functions: Vec<_> = functions
            .into_iter()
            .filter(|function| eligible(function) && seen.insert(ptr::from_ref(*function)))
            .collect();
        // Link order is not a scheduling policy. Registration locations break
        // ties between distinct registrations with otherwise equal signatures.
        functions.sort_by_cached_key(|function| {
            (
                describe_function(function),
                function.registration_site.file,
                function.registration_site.line,
            )
        });
        let mut graph = Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            shapes: HashMap::new(),
            functions: Vec::new(),
            witnesses: HashMap::new(),
        };
        let mut seeds: Vec<_> = seeds.into_iter().collect();
        seeds.sort_by_cached_key(|shape| describe_shape(shape));
        for shape in seeds {
            cancellation.bail_if_cancelled()?;
            let id = graph.shape_node(shape);
            graph.nodes[id].seed = true;
            graph.nodes[id].reachable = true;
        }
        let mut targets: Vec<_> = targets.into_iter().collect();
        targets.sort_by_cached_key(|shape| describe_shape(shape));
        for shape in targets {
            let id = graph.shape_node(shape);
            graph.nodes[id].target = true;
        }
        let mut reflection_roots = Vec::new();
        for function in functions {
            cancellation.bail_if_cancelled()?;
            graph.shape_node(function.input_shape);
            if function.kind == FunctionKind::AsyncInvoke
                && let Type::User(UserType::Struct(record)) = function.input_shape.ty
            {
                for field in record.fields {
                    graph.shape_node(field.shape());
                    if let Some(pointee) = owning_input_shape(field.shape()) {
                        graph.shape_node(pointee);
                    }
                }
            }
            let output = graph.shape_node(function.output_shape);
            let id = graph.nodes.len();
            graph.nodes.push(DiscoveryPlanNode {
                id,
                label: if function.kind == FunctionKind::AsyncInvoke {
                    describe_shape(function.input_shape)
                } else {
                    describe_function(function)
                },
                kind: match function.kind {
                    FunctionKind::AsyncInvoke => DiscoveryPlanNodeKind::Request,
                    FunctionKind::Conversion => DiscoveryPlanNodeKind::Conversion,
                    FunctionKind::Projection => DiscoveryPlanNodeKind::Projection,
                    FunctionKind::Constructor => unreachable!("constructors are excluded"),
                },
                reachable: false,
                seed: false,
                target: false,
                level: 0,
                effects: function.effects.to_vec(),
                fields: DiscoveryPlanField::for_shape(function.input_shape),
                missing_inputs: Vec::new(),
            });
            graph.functions.push((function, id));
            graph.edges.push(DiscoveryPlanEdge {
                source: id,
                target: output,
                label: "output".to_owned(),
            });
            reflection_roots.push(function.output_shape);
        }
        // Only outputs expose descendants. In particular, seeding an auth enum
        // must not authorize contexts from all its alternative variants, and
        // assembling a request must not make its input fields look like outputs.
        let mut visited = HashSet::new();
        let mut pending = reflection_roots;
        while let Some(shape) = pending.pop() {
            cancellation.bail_if_cancelled()?;
            if !visited.insert(shape) {
                continue;
            }
            let source = graph.shape_node(shape);
            for (child, label) in children(shape) {
                let target = graph.shape_node(child);
                graph.edges.push(DiscoveryPlanEdge {
                    source,
                    target,
                    label,
                });
                pending.push(child);
            }
        }
        Ok(graph)
    }

    fn shape_node(&mut self, shape: &'static Shape) -> usize {
        if let Some(id) = self.shapes.get(shape) {
            return *id;
        }
        let id = self.nodes.len();
        self.shapes.insert(shape, id);
        self.nodes.push(DiscoveryPlanNode {
            id,
            label: describe_shape(shape),
            kind: DiscoveryPlanNodeKind::Shape,
            reachable: false,
            seed: false,
            target: false,
            level: 0,
            effects: Vec::new(),
            fields: DiscoveryPlanField::for_shape(shape),
            missing_inputs: Vec::new(),
        });
        id
    }

    fn inputs(&self, function: &Function) -> Vec<(usize, String)> {
        let input = self.shapes[function.input_shape];
        if self.nodes[input].reachable || function.kind != FunctionKind::AsyncInvoke {
            return vec![(input, "input".to_owned())];
        }
        let Type::User(UserType::Struct(record)) = function.input_shape.ty else {
            return vec![(input, "input".to_owned())];
        };
        let inputs: Vec<_> = record
            .fields
            .iter()
            .filter(|field| !host_supplied_or_optional(field.shape()))
            .map(|field| {
                let field_id = self.shapes[field.shape()];
                let source = if self.nodes[field_id].reachable {
                    field_id
                } else {
                    owning_input_shape(field.shape())
                        .map(|pointee| self.shapes[pointee])
                        .unwrap_or(field_id)
                };
                (source, field.name.to_owned())
            })
            .collect();
        // An input-free read may hide ambient scope/configuration lookup. Require
        // its complete request shape rather than manufacturing an implicit root.
        // Cancellation and omitted filters do not establish a discovery scope.
        if inputs.is_empty() {
            vec![(input, "input".to_owned())]
        } else {
            inputs
        }
    }

    pub(crate) fn finish(
        mut self,
        cancellation: &CancellationScope,
    ) -> eyre::Result<DiscoveryPlan> {
        // Reachability grows monotonically over a finite metadata graph. A shape
        // needs any producer; an operation needs ALL selected required inputs.
        // Witnesses are recorded only at first availability, preventing cyclic
        // conversions from inventing a starting value or a cyclic derivation.
        loop {
            cancellation.bail_if_cancelled()?;
            let mut changed = false;
            for index in 0..self.functions.len() {
                let (function, id) = self.functions[index];
                if self.nodes[id].reachable {
                    continue;
                }
                cancellation.bail_if_cancelled()?;
                let inputs = self.inputs(function);
                if inputs.iter().all(|(id, _)| self.nodes[*id].reachable) {
                    self.nodes[id].level = inputs
                        .iter()
                        .map(|(id, _)| self.nodes[*id].level)
                        .max()
                        .unwrap_or(0)
                        + 1;
                    self.nodes[id].reachable = true;
                    for (source, label) in inputs {
                        self.edges.push(DiscoveryPlanEdge {
                            source,
                            target: id,
                            label,
                        });
                    }
                    changed = true;
                }
            }
            for (index, edge) in self.edges.iter().enumerate() {
                cancellation.bail_if_cancelled()?;
                if self.nodes[edge.target].kind == DiscoveryPlanNodeKind::Shape
                    && !self.nodes[edge.target].reachable
                    && self.nodes[edge.source].reachable
                {
                    self.nodes[edge.target].reachable = true;
                    self.nodes[edge.target].level = self.nodes[edge.source].level + 1;
                    self.witnesses.insert(edge.target, index);
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        for index in 0..self.functions.len() {
            let (function, id) = self.functions[index];
            if self.nodes[id].reachable {
                continue;
            }
            for (source, label) in self.inputs(function) {
                if !self.nodes[source].reachable {
                    let missing = format!("{label}: {}", self.nodes[source].label);
                    self.nodes[id].missing_inputs.push(missing);
                }
                self.edges.push(DiscoveryPlanEdge {
                    source,
                    target: id,
                    label,
                });
            }
        }
        self.target_slice(cancellation)
    }

    fn target_slice(self, cancellation: &CancellationScope) -> eyre::Result<DiscoveryPlan> {
        let mut incoming = vec![Vec::new(); self.nodes.len()];
        for (index, edge) in self.edges.iter().enumerate() {
            incoming[edge.target].push(index);
        }
        let mut pending: Vec<_> = self
            .nodes
            .iter()
            .filter(|node| node.target)
            .map(|node| node.id)
            .collect();
        let mut retained_nodes = HashSet::new();
        let mut retained_edges = HashSet::new();
        while let Some(id) = pending.pop() {
            cancellation.bail_if_cancelled()?;
            if !retained_nodes.insert(id) || self.nodes[id].seed {
                continue;
            }
            let edges = if let Some(witness) = self.witnesses.get(&id) {
                std::slice::from_ref(witness)
            } else {
                incoming[id].as_slice()
            };
            for &index in edges {
                retained_edges.insert(index);
                pending.push(self.edges[index].source);
            }
        }
        let mut plan = DiscoveryPlan {
            nodes: Vec::new(),
            edges: Vec::new(),
            diagnostics: Vec::new(),
        };
        let mut remapped = HashMap::new();
        for mut node in self.nodes {
            if !retained_nodes.contains(&node.id) {
                continue;
            }
            if node.target && !node.reachable {
                let reason = if incoming[node.id].is_empty() {
                    "no eligible registered producer in the supplied catalog"
                } else {
                    "its candidate producers require inputs unavailable from the supplied seeds"
                };
                plan.diagnostics
                    .push(format!("Target {} is unreachable: {reason}", node.label));
            }
            remapped.insert(node.id, plan.nodes.len());
            node.id = plan.nodes.len();
            plan.nodes.push(node);
        }
        for (index, mut edge) in self.edges.into_iter().enumerate() {
            if retained_edges.contains(&index) {
                edge.source = remapped[&edge.source];
                edge.target = remapped[&edge.target];
                plan.edges.push(edge);
            }
        }
        Ok(plan)
    }
}

fn eligible(function: &Function) -> bool {
    match function.kind {
        FunctionKind::AsyncInvoke => function.is_read_only() && !function.requires_user_input(),
        FunctionKind::Conversion | FunctionKind::Projection => {
            function.effects.is_empty() && !function.is_async()
        }
        FunctionKind::Constructor => false,
    }
}

fn host_supplied_or_optional(shape: &'static Shape) -> bool {
    shape.is_shape(CancellationScope::SHAPE)
        || shape.is_shape(<OpaqueBorrow<'static, CancellationScope>>::SHAPE)
        || matches!(shape.def, Def::Option(_))
}

fn owning_input_shape(shape: &'static Shape) -> Option<&'static Shape> {
    if let Def::Pointer(pointer) = shape.def
        && matches!(
            pointer.known,
            Some(KnownPointer::Cow | KnownPointer::Box | KnownPointer::Arc | KnownPointer::Rc)
        )
        && let Some(pointee) = pointer.pointee()
        && RuntimeValue::can_own_pointee(shape, pointee)
    {
        Some(pointee)
    } else {
        None
    }
}

fn children(shape: &'static Shape) -> Vec<(&'static Shape, String)> {
    let collection_element = match shape.def {
        Def::List(list) => Some(list.t()),
        Def::Set(set) => Some(set.t()),
        Def::Array(array) => Some(array.t()),
        Def::Slice(slice) => Some(slice.t()),
        Def::NdArray(array) => Some(array.t()),
        _ => None,
    };
    if let Some(element) = collection_element {
        return vec![(element, "element (if present)".to_owned())];
    }
    match shape.def {
        Def::Option(optional) => return vec![(optional.t(), "Some (if present)".to_owned())],
        Def::Pointer(pointer) => {
            return pointer
                .pointee()
                .map(|shape| vec![(shape, "pointee".to_owned())])
                .unwrap_or_default();
        }
        Def::Map(map) => {
            return vec![
                (map.k(), "key (if present)".to_owned()),
                (map.v(), "value (if present)".to_owned()),
            ];
        }
        Def::Result(result) => {
            return vec![
                (result.t(), "Ok (if present)".to_owned()),
                (result.e(), "Err (if present)".to_owned()),
            ];
        }
        _ => {}
    }
    match shape.ty {
        Type::User(UserType::Struct(record)) => record
            .fields
            .iter()
            .map(|field| (field.shape(), field.name.to_owned()))
            .collect(),
        Type::User(UserType::Enum(enumeration)) => enumeration
            .variants
            .iter()
            .flat_map(|variant| {
                variant.data.fields.iter().map(move |field| {
                    (
                        field.shape(),
                        format!("{}::{} (if active)", variant.name, field.name),
                    )
                })
            })
            .collect(),
        _ => Vec::new(),
    }
}
