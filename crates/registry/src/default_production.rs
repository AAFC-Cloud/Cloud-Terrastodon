use crate::Function;
use crate::FunctionInvocation;
use crate::InvocationFuture;
use crate::ProductionKind;
use crate::RuntimeValue;
use crate::Type;
use crate::UserType;
use crate::describe_shape;
use crate::functions_to;
use crate::known_thing_for_shape;
use facet::Shape;
use std::future::Future;
use std::pin::Pin;

/// A registry-discovered recipe for producing a value without user-provided
/// input. The recipe is deliberately type-erased: callers can inspect its
/// target shape, while the registry remains responsible for deciding which
/// defaults and registered producers form a valid chain.
pub struct DefaultProductionPlan {
    shape: &'static Shape,
    kind: DefaultProductionKind,
}

enum DefaultProductionKind {
    Default,
    Struct {
        fields: Vec<(usize, Box<DefaultProductionPlan>)>,
    },
    Invoke {
        function: &'static Function,
        input: Box<DefaultProductionPlan>,
    },
}

impl DefaultProductionPlan {
    pub fn shape(&self) -> &'static Shape {
        self.shape
    }
}

/// Find a recipe for constructing `shape` from reflected defaults and
/// registered producers.
///
/// Direct `Default` implementations are preferred. Otherwise a struct is
/// assembled from its fields, and finally an exact-output registered function
/// is considered. Each branch gets its own cycle guard, so a recursive type or
/// producer cannot prevent another valid producer path from being discovered.
pub fn default_production_plan(shape: &'static Shape) -> Option<DefaultProductionPlan> {
    let mut visiting = Vec::new();
    plan_for_shape(shape, &mut visiting)
}

pub fn shape_can_be_produced_from_defaults(shape: &'static Shape) -> bool {
    default_production_plan(shape).is_some()
}

/// Invoke a registered function after recursively producing its input from
/// the registry's default-production plan.
pub fn invoke_with_default_input(function: &'static Function) -> InvocationFuture {
    Box::pin(async move {
        let plan = default_production_plan(function.input_shape).ok_or_else(|| {
            eyre::eyre!(
                "no default production path is registered for {}",
                describe_shape(function.input_shape)
            )
        })?;
        let input = realize_plan_as_boxed(&plan).await?;

        let output = match function.receiver_mode {
            crate::ReceiverMode::ByValue => function.invoke_value_boxed(input)?,
            crate::ReceiverMode::ByRef => {
                FunctionInvocation::Ready(function.invoke_ref_boxed(input.as_ref())?)
            }
            crate::ReceiverMode::ByMut => {
                let mut input = input;
                FunctionInvocation::Ready(function.invoke_mut_boxed(input.as_mut())?)
            }
        };

        match output {
            FunctionInvocation::Pending(future) => future.await,
            FunctionInvocation::Ready(output) => Ok(output),
        }
    })
}

fn plan_for_shape(
    shape: &'static Shape,
    visiting: &mut Vec<&'static Shape>,
) -> Option<DefaultProductionPlan> {
    if shape
        .type_ops
        .as_ref()
        .is_some_and(|type_ops| type_ops.has_default_in_place())
    {
        return Some(DefaultProductionPlan {
            shape,
            kind: DefaultProductionKind::Default,
        });
    }

    if visiting.iter().any(|candidate| candidate.is_shape(shape)) {
        return None;
    }
    visiting.push(shape);

    let struct_plan = match shape.ty {
        Type::User(UserType::Struct(struct_type)) => {
            let mut fields = Vec::new();
            let mut possible = true;
            for (field_index, field) in struct_type.fields.iter().enumerate() {
                if field.should_skip_deserializing() || field.has_default() {
                    continue;
                }
                let field_shape = field.proxy_shape().unwrap_or_else(|| field.shape());
                let Some(plan) = plan_for_shape(field_shape, visiting) else {
                    possible = false;
                    break;
                };
                fields.push((field_index, Box::new(plan)));
            }
            possible.then_some(DefaultProductionPlan {
                shape,
                kind: DefaultProductionKind::Struct { fields },
            })
        }
        _ => None,
    };
    if let Some(plan) = struct_plan {
        visiting.pop();
        return Some(plan);
    }

    let plan = functions_to(shape).into_iter().find_map(|function| {
        (function.production_kind(shape) == Some(ProductionKind::Exact))
            .then(|| plan_for_shape(function.input_shape, visiting))
            .flatten()
            .map(|input| DefaultProductionPlan {
                shape,
                kind: DefaultProductionKind::Invoke {
                    function,
                    input: Box::new(input),
                },
            })
    });

    visiting.pop();
    plan
}

fn realize_plan(
    plan: &DefaultProductionPlan,
) -> Pin<Box<dyn Future<Output = eyre::Result<RuntimeValue>> + Send + '_>> {
    Box::pin(async move {
        match &plan.kind {
            DefaultProductionKind::Default => RuntimeValue::from_default(plan.shape),
            DefaultProductionKind::Struct { fields } => {
                let mut values = Vec::with_capacity(fields.len());
                for (field_index, field_plan) in fields {
                    let thing = known_thing_for_shape(field_plan.shape()).ok_or_else(|| {
                        eyre::eyre!(
                            "{} is not a registered field thing",
                            describe_shape(field_plan.shape())
                        )
                    })?;
                    values.push((
                        *field_index,
                        thing,
                        realize_plan_as_boxed(field_plan).await?,
                    ));
                }
                RuntimeValue::build_with(plan.shape, |mut partial| {
                    for (field_index, thing, value) in values {
                        let value = thing.runtime_from_boxed(value)?;
                        partial = partial.begin_nth_field(field_index)?;
                        partial = unsafe { partial.set_from_peek(&value.peek()) }?;
                        value.deallocate_after_move();
                        partial = partial.end()?;
                    }
                    Ok(partial)
                })
            }
            DefaultProductionKind::Invoke { function, input } => {
                let input = realize_plan_as_boxed(input).await?;
                let output = match function.receiver_mode {
                    crate::ReceiverMode::ByValue => function.invoke_value_boxed(input)?,
                    crate::ReceiverMode::ByRef => {
                        FunctionInvocation::Ready(function.invoke_ref_boxed(input.as_ref())?)
                    }
                    crate::ReceiverMode::ByMut => {
                        let mut input = input;
                        FunctionInvocation::Ready(function.invoke_mut_boxed(input.as_mut())?)
                    }
                };
                let output = match output {
                    FunctionInvocation::Pending(future) => future.await?,
                    FunctionInvocation::Ready(output) => output,
                };
                (function.output_to_runtime)(output)
            }
        }
    })
}

fn realize_plan_as_boxed(
    plan: &DefaultProductionPlan,
) -> Pin<Box<dyn Future<Output = eyre::Result<Box<dyn std::any::Any + Send>>> + Send + '_>> {
    Box::pin(async move {
        let value = realize_plan(plan).await?;
        let thing = known_thing_for_shape(plan.shape).ok_or_else(|| {
            eyre::eyre!(
                "{} is not a registered input thing",
                describe_shape(plan.shape)
            )
        })?;
        thing.runtime_into_boxed(value)
    })
}
