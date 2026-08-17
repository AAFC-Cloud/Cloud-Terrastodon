use cloud_terrastodon_registry::{
    ArbitraryBytes, DefaultProductionPlan, DefaultProductionPlanKind, Function, RuntimeValue,
    describe_function, describe_shape, known_thing_for_shape,
};
use facet::Shape;

use super::arena::Arena;
use super::arena_slot_state::ArenaSlotState;
use super::borrow_graph::BorrowGraph;
use super::field_binding::FieldBinding;
use super::invocation_controller::InvocationController;
use super::invocation_host::InvocationHost;
use super::invocation_mode::InvocationMode;
use super::slot_id::SlotId;
use super::value_builder::BuilderStore;

pub(super) enum ProductionNodeAdvance {
    Progress(SlotId),
    Waiting,
    Ready(SlotId),
    Failed(String),
}

pub(super) enum ProductionNode {
    Existing {
        output: SlotId,
    },
    Default {
        shape: &'static Shape,
        output: Option<SlotId>,
    },
    ArbitraryBytes {
        bytes: Option<Vec<u8>>,
        output: Option<SlotId>,
    },
    Struct(StructProductionNode),
    Invoke(InvokeProductionNode),
}

pub(super) struct StructProductionNode {
    shape: &'static Shape,
    output: Option<SlotId>,
    fields: Vec<ProductionFieldNode>,
    next_field: usize,
}

struct ProductionFieldNode {
    index: usize,
    name: &'static str,
    value: ProductionNode,
    linked: bool,
}

pub(super) struct InvokeProductionNode {
    function: &'static Function,
    input: Box<ProductionNode>,
    output: Option<SlotId>,
}

impl ProductionNode {
    pub(super) fn from_default_plan(plan: &DefaultProductionPlan) -> Self {
        match plan.kind() {
            DefaultProductionPlanKind::Default => Self::Default {
                shape: plan.shape(),
                output: None,
            },
            DefaultProductionPlanKind::Struct(fields) => Self::Struct(StructProductionNode {
                shape: plan.shape(),
                output: None,
                fields: fields
                    .iter()
                    .map(|field| ProductionFieldNode {
                        index: field.field_index,
                        name: field.field_name,
                        value: Self::from_default_plan(&field.plan),
                        linked: false,
                    })
                    .collect(),
                next_field: 0,
            }),
            DefaultProductionPlanKind::Invoke { function, input } => {
                Self::invoke(function, Self::from_default_plan(input))
            }
        }
    }

    pub(super) fn existing(output: SlotId) -> Self {
        Self::Existing { output }
    }

    pub(super) fn arbitrary_bytes(bytes: Vec<u8>) -> Self {
        Self::ArbitraryBytes {
            bytes: Some(bytes),
            output: None,
        }
    }

    pub(super) fn invoke(function: &'static Function, input: Self) -> Self {
        Self::Invoke(InvokeProductionNode {
            function,
            input: Box::new(input),
            output: None,
        })
    }

    pub(super) fn output_if_allocated(&self) -> Option<SlotId> {
        match self {
            Self::Existing { output } => Some(*output),
            Self::Default { output, .. } | Self::ArbitraryBytes { output, .. } => *output,
            Self::Struct(node) => node.output,
            Self::Invoke(node) => node.output,
        }
    }

    /// The request root consumed by the outermost producer invocation.
    pub(super) fn outer_input(&self) -> Option<SlotId> {
        match self {
            Self::Invoke(node) => node.input.output_if_allocated(),
            _ => None,
        }
    }

    pub(super) fn advance_one(
        &mut self,
        arena: &mut Arena,
        builders: &mut BuilderStore,
        borrow_graph: &mut BorrowGraph,
        invocations: &mut InvocationController,
        invocation_host: &mut dyn InvocationHost,
    ) -> ProductionNodeAdvance {
        match self {
            Self::Existing { output } => root_state(arena, *output),
            Self::Default { shape, output } => {
                if let Some(output) = *output {
                    return root_state(arena, output);
                }
                match RuntimeValue::from_default(shape)
                    .map_err(|error| error.to_string())
                    .and_then(|value| arena.insert_ready(value).map_err(|error| error.to_string()))
                {
                    Ok(slot) => {
                        *output = Some(slot);
                        ProductionNodeAdvance::Progress(slot)
                    }
                    Err(error) => ProductionNodeAdvance::Failed(format!(
                        "could not default {}: {error}",
                        describe_shape(shape)
                    )),
                }
            }
            Self::ArbitraryBytes { bytes, output } => {
                if let Some(output) = *output {
                    return root_state(arena, output);
                }
                let Some(bytes) = bytes.take() else {
                    return ProductionNodeAdvance::Failed(
                        "arbitrary source bytes were already consumed without an arena root"
                            .to_owned(),
                    );
                };
                match RuntimeValue::from_box(Box::new(ArbitraryBytes::new(bytes)))
                    .map_err(|error| error.to_string())
                    .and_then(|value| arena.insert_ready(value).map_err(|error| error.to_string()))
                {
                    Ok(slot) => {
                        *output = Some(slot);
                        ProductionNodeAdvance::Progress(slot)
                    }
                    Err(error) => ProductionNodeAdvance::Failed(format!(
                        "could not create ArbitraryBytes source: {error}"
                    )),
                }
            }
            Self::Struct(node) => {
                node.advance_one(arena, builders, borrow_graph, invocations, invocation_host)
            }
            Self::Invoke(node) => {
                node.advance_one(arena, builders, borrow_graph, invocations, invocation_host)
            }
        }
    }
}

impl StructProductionNode {
    fn advance_one(
        &mut self,
        arena: &mut Arena,
        builders: &mut BuilderStore,
        borrow_graph: &mut BorrowGraph,
        invocations: &mut InvocationController,
        invocation_host: &mut dyn InvocationHost,
    ) -> ProductionNodeAdvance {
        let output = match self.output {
            Some(output) => output,
            None => match builders.create_and_finalize(arena, borrow_graph, self.shape) {
                Ok((output, _)) => {
                    self.output = Some(output);
                    return ProductionNodeAdvance::Progress(output);
                }
                Err(error) => {
                    return ProductionNodeAdvance::Failed(format!(
                        "could not create {} producer input: {error}",
                        describe_shape(self.shape)
                    ));
                }
            },
        };

        match root_state(arena, output) {
            ProductionNodeAdvance::Ready(output) => {
                return ProductionNodeAdvance::Ready(output);
            }
            ProductionNodeAdvance::Failed(message) => {
                return ProductionNodeAdvance::Failed(message);
            }
            ProductionNodeAdvance::Progress(_) => unreachable!("root inspection does not mutate"),
            ProductionNodeAdvance::Waiting => {}
        }

        let Some(field) = self.fields.get_mut(self.next_field) else {
            return ProductionNodeAdvance::Failed(format!(
                "default plan populated every declared field but {} remained Building",
                describe_shape(self.shape)
            ));
        };
        if field.linked {
            self.next_field += 1;
            return ProductionNodeAdvance::Progress(output);
        }

        match field
            .value
            .advance_one(arena, builders, borrow_graph, invocations, invocation_host)
        {
            ProductionNodeAdvance::Progress(root) => ProductionNodeAdvance::Progress(root),
            ProductionNodeAdvance::Waiting => ProductionNodeAdvance::Waiting,
            ProductionNodeAdvance::Failed(message) => ProductionNodeAdvance::Failed(format!(
                "could not produce field {} of {}: {message}",
                field.name,
                describe_shape(self.shape)
            )),
            ProductionNodeAdvance::Ready(source) => {
                match builders.set_field_and_finalize(
                    arena,
                    borrow_graph,
                    output,
                    field.index,
                    FieldBinding::MoveFrom(source),
                ) {
                    Ok(_) => {
                        field.linked = true;
                        self.next_field += 1;
                        ProductionNodeAdvance::Progress(output)
                    }
                    Err(error) => ProductionNodeAdvance::Failed(format!(
                        "could not move produced field {} into {}: {error}",
                        field.name,
                        describe_shape(self.shape)
                    )),
                }
            }
        }
    }
}

impl InvokeProductionNode {
    fn advance_one(
        &mut self,
        arena: &mut Arena,
        builders: &mut BuilderStore,
        borrow_graph: &mut BorrowGraph,
        invocations: &mut InvocationController,
        invocation_host: &mut dyn InvocationHost,
    ) -> ProductionNodeAdvance {
        if let Some(output) = self.output {
            return root_state(arena, output);
        }

        let input = match self.input.advance_one(
            arena,
            builders,
            borrow_graph,
            invocations,
            invocation_host,
        ) {
            ProductionNodeAdvance::Progress(root) => {
                return ProductionNodeAdvance::Progress(root);
            }
            ProductionNodeAdvance::Waiting => return ProductionNodeAdvance::Waiting,
            ProductionNodeAdvance::Failed(message) => {
                return ProductionNodeAdvance::Failed(message);
            }
            ProductionNodeAdvance::Ready(input) => input,
        };
        let Some(input_thing) = known_thing_for_shape(self.function.input_shape) else {
            return ProductionNodeAdvance::Failed(format!(
                "{} has no registered runtime Thing",
                describe_shape(self.function.input_shape)
            ));
        };
        match invocations.invoke(
            arena,
            builders,
            borrow_graph,
            invocation_host,
            input,
            input_thing,
            self.function,
            InvocationMode::Consume,
        ) {
            Ok(start) => {
                let output = start.output();
                self.output = Some(output);
                ProductionNodeAdvance::Progress(output)
            }
            Err(error) => ProductionNodeAdvance::Failed(format!(
                "could not invoke {}: {error}",
                describe_function(self.function)
            )),
        }
    }
}

fn root_state(arena: &Arena, slot: SlotId) -> ProductionNodeAdvance {
    match arena.slot(slot).map(|slot| slot.state()) {
        Some(ArenaSlotState::Ready(_)) => ProductionNodeAdvance::Ready(slot),
        Some(ArenaSlotState::Building | ArenaSlotState::Pending) => ProductionNodeAdvance::Waiting,
        Some(ArenaSlotState::Failed(message)) => {
            ProductionNodeAdvance::Failed(format!("slot {slot} failed: {message}"))
        }
        Some(ArenaSlotState::Cancelled) => {
            ProductionNodeAdvance::Failed(format!("slot {slot} was cancelled"))
        }
        Some(ArenaSlotState::Consumed) => {
            ProductionNodeAdvance::Failed(format!("slot {slot} was consumed before production"))
        }
        Some(ArenaSlotState::Tombstone { .. }) => {
            ProductionNodeAdvance::Failed(format!("slot {slot} was deleted before production"))
        }
        None => ProductionNodeAdvance::Failed(format!("slot {slot} no longer exists")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arbitrary_node_has_no_root_until_the_engine_advances_it() {
        let node = ProductionNode::arbitrary_bytes(vec![1, 2, 3]);
        assert_eq!(node.output_if_allocated(), None);
    }

    #[test]
    fn existing_node_preserves_the_supplied_arena_identity() {
        let slot = SlotId::new(7);
        let node = ProductionNode::existing(slot);
        assert_eq!(node.output_if_allocated(), Some(slot));
    }
}
