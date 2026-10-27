use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_registry::DiscoveryPlan;
use cloud_terrastodon_registry::DiscoveryPlanNode;
use cloud_terrastodon_registry::DiscoveryPlanNodeKind;
use cloud_terrastodon_registry::Function;
use cloud_terrastodon_registry::FunctionKind;
use cloud_terrastodon_registry::Thing;
use cloud_terrastodon_registry::describe_shape;
use eyre::Result;
use eyre::ensure;
use facet::Def;
use facet::Shape;
use facet::Type;
use facet::UserType;
use std::collections::BTreeMap;
use std::collections::HashSet;

/// Resolves named targets and builds offline plans from an explicitly supplied
/// registry catalog. The same catalog supplies the interactive target palette.
///
/// No registered operation, arbitrary generator, or credential lookup is run.
#[derive(Clone)]
pub(super) struct DumpPlanner {
    // Keep the name index and palette consistent with the catalog and seeds
    // captured when this planning session starts.
    functions: Vec<&'static Function>,
    seeds: Vec<&'static Shape>,
    shapes_by_name: BTreeMap<String, HashSet<&'static Shape>>,
    target_names: Vec<String>,
}

impl DumpPlanner {
    pub(super) fn new(
        functions: impl IntoIterator<Item = &'static Function>,
        things: impl IntoIterator<Item = &'static Thing>,
        seeds: Vec<&'static Shape>,
    ) -> Self {
        let functions: Vec<_> = functions.into_iter().collect();
        let requests: HashSet<_> = functions
            .iter()
            .filter(|function| function.kind == FunctionKind::AsyncInvoke)
            .map(|function| function.input_shape)
            .collect();
        let mut available: HashSet<_> = things.into_iter().map(|thing| thing.shape).collect();
        for function in &functions {
            available.insert(function.input_shape);
            available.insert(function.output_shape);
        }

        // Start with every unattended read, including reads blocked by missing
        // scope. A selectable target should explain why it is blocked instead
        // of disappearing from the palette when its inputs are unavailable.
        let mut outputs = HashSet::new();
        let mut pending: Vec<_> = functions
            .iter()
            .filter(|function| {
                function.kind == FunctionKind::AsyncInvoke
                    && function.is_read_only()
                    && !function.requires_user_input()
            })
            .map(|function| function.output_shape)
            .collect();
        loop {
            while let Some(shape) = pending.pop() {
                if outputs.insert(shape) {
                    pending.extend(children(shape));
                }
            }
            // Only conversions/projections fed by a read's output contribute
            // palette targets. Constructors and unrelated conversions do not
            // make arbitrary registry types into useful inventory targets.
            for function in &functions {
                if matches!(
                    function.kind,
                    FunctionKind::Conversion | FunctionKind::Projection
                ) && function.effects.is_empty()
                    && !function.is_async()
                    && outputs.contains(function.input_shape)
                    && !outputs.contains(function.output_shape)
                {
                    pending.push(function.output_shape);
                }
            }
            if pending.is_empty() {
                break;
            }
        }
        available.extend(outputs.iter().copied());
        let mut shapes_by_name = BTreeMap::<String, HashSet<&'static Shape>>::new();
        for shape in available {
            shapes_by_name
                .entry(describe_shape(shape))
                .or_default()
                .insert(shape);
        }
        let target_names = shapes_by_name
            .iter()
            .filter(|(_, shapes)| {
                shapes.len() == 1
                    && shapes.iter().any(|shape| {
                        outputs.contains(shape)
                            && !requests.contains(shape)
                            && is_named_target(shape)
                    })
            })
            .map(|(name, _)| name.clone())
            .collect();
        Self {
            functions,
            seeds,
            shapes_by_name,
            target_names,
        }
    }

    /// Sorted, unambiguous names of domain outputs offered by the supplied reads.
    pub(super) fn target_names(&self) -> &[String] {
        &self.target_names
    }

    /// Empty selection intentionally produces an empty graph so removing the
    /// last canvas target is a valid edit. Unknown names remain blocked targets.
    pub(super) fn plan(
        &self,
        target_names: &[String],
        cancellation: &CancellationScope,
    ) -> Result<DiscoveryPlan> {
        let mut names = target_names.to_vec();
        names.sort();
        names.dedup();
        let mut targets = Vec::new();
        let mut unknown = Vec::new();
        for name in names {
            cancellation.bail_if_cancelled()?;
            ensure!(!name.trim().is_empty(), "target type names cannot be empty");
            if let Some(shapes) = self.shapes_by_name.get(&name) {
                ensure!(
                    shapes.len() == 1,
                    "target type name {name:?} is ambiguous in the linked registry"
                );
                targets.extend(shapes.iter().copied());
            } else {
                unknown.push(name);
            }
        }
        let mut plan = DiscoveryPlan::build(
            self.functions.iter().copied(),
            self.seeds.iter().copied(),
            targets,
            cancellation,
        )?;
        for name in unknown {
            plan.diagnostics.push(format!("Target {name} has no linked type registration or request signature. It cannot be planned until registered."));
            plan.nodes.push(DiscoveryPlanNode {
                id: plan.nodes.len(),
                label: name,
                kind: DiscoveryPlanNodeKind::Shape,
                reachable: false,
                seed: false,
                target: true,
                level: 0,
                missing_inputs: Vec::new(),
                effects: Vec::new(),
                fields: Vec::new(),
            });
        }
        Ok(plan)
    }
}

fn is_named_target(shape: &'static Shape) -> bool {
    // Container names and primitive fields are useful planner dependencies,
    // but not target cards. Nominal domain newtypes remain selectable.
    // Figue's attribute grammar declares the metadata namespace "args",
    // independent of the crate alias used in #[facet(figue::named)]. Marker
    // attributes carry unit payloads, so a typed Attr downcast cannot find them.
    if shape.module_path.is_none()
        || !matches!(shape.def, Def::Undefined | Def::Scalar)
        || shape
            .attributes
            .iter()
            .any(|attribute| attribute.ns() == Some("args"))
    {
        return false;
    }
    match shape.ty {
        Type::User(UserType::Struct(record)) => !record.fields.iter().any(|field| {
            field
                .attributes
                .iter()
                .any(|attribute| attribute.ns() == Some("args"))
        }),
        Type::User(UserType::Enum(enumeration)) => !enumeration.variants.iter().any(|variant| {
            variant
                .attributes
                .iter()
                .any(|attribute| attribute.ns() == Some("args"))
                || variant.data.fields.iter().any(|field| {
                    field
                        .attributes
                        .iter()
                        .any(|attribute| attribute.ns() == Some("args"))
                })
        }),
        _ => false,
    }
}

/// Traverse the same output relationships the structural planner can expose.
/// Values are never fabricated or inspected; recursion is deduplicated by Shape.
fn children(shape: &'static Shape) -> Vec<&'static Shape> {
    match shape.def {
        Def::List(list) => return vec![list.t()],
        Def::Set(set) => return vec![set.t()],
        Def::Array(array) => return vec![array.t()],
        Def::Slice(slice) => return vec![slice.t()],
        Def::NdArray(array) => return vec![array.t()],
        Def::Option(optional) => return vec![optional.t()],
        Def::Pointer(pointer) => return pointer.pointee().into_iter().collect(),
        Def::Map(map) => return vec![map.k(), map.v()],
        Def::Result(result) => return vec![result.t(), result.e()],
        _ => {}
    }
    match shape.ty {
        Type::User(UserType::Struct(record)) => {
            record.fields.iter().map(|field| field.shape()).collect()
        }
        Type::User(UserType::Enum(enumeration)) => enumeration
            .variants
            .iter()
            .flat_map(|variant| variant.data.fields.iter().map(|field| field.shape()))
            .collect(),
        _ => Vec::new(),
    }
}

#[cfg(test)]
#[path = "dump_planner_tests.rs"]
mod tests;
