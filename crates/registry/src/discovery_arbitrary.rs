use crate::ArbitraryBytes;
use crate::CancellationScope;
use crate::Function;
use crate::FunctionExecutor;
use crate::FunctionKind;
use crate::FunctionTraitContract;
use crate::RuntimeValue;
use crate::describe_shape;
use facet::Def;
use facet::Facet;
use facet::Shape;
use facet::Type;
use facet::UserType;
use rand::Rng;
use rand::rngs::StdRng;

const GENERATION_ATTEMPTS: usize = 16;
const GENERATION_BYTES: usize = 64 * 1024;

/// Fabricate only the declared response, never a missing request or domain
/// input. Only an explicit Arbitrary constructor in the supplied catalog may
/// generate the response; linked operations outside that catalog are not used.
pub(crate) fn fabricate_response(
    shape: &'static Shape,
    mut catalog: impl Iterator<Item = &'static Function> + Clone,
    random: &mut StdRng,
    cancellation: &CancellationScope,
    diagnostics: &mut Vec<String>,
) -> eyre::Result<RuntimeValue> {
    if contains_request_shape(shape, catalog.clone(), cancellation)? {
        eyre::bail!(
            "response {} contains an executable request; refusing its Arbitrary constructor",
            describe_shape(shape)
        );
    }
    let generator = catalog
        .find(|function| {
            function.kind == FunctionKind::Constructor
                && function.trait_contract == FunctionTraitContract::Arbitrary
                && function.effects.is_empty()
                && !function.is_async()
                && function.input_shape.is_shape(ArbitraryBytes::SHAPE)
                && function.output_shape.is_shape(shape)
        })
        .ok_or_else(|| {
            eyre::eyre!(
                "no exact Arbitrary response generator for {} in the supplied catalog",
                describe_shape(shape)
            )
        })?;
    let mut last_error = None;
    let mut empty_response = None;
    for _ in 0..GENERATION_ATTEMPTS {
        cancellation.bail_if_cancelled()?;
        let mut bytes = vec![0_u8; GENERATION_BYTES];
        random.fill_bytes(&mut bytes);
        let mut input = Box::new(ArbitraryBytes::new(bytes));
        // Match the synchronous executor directly. Even dishonest constructor
        // metadata cannot make an AsyncValue function execute here.
        let response = match generator.executor {
            FunctionExecutor::SyncMut(invoke) => invoke(input.as_mut()),
            FunctionExecutor::SyncRef(invoke) => invoke(input.as_ref()),
            FunctionExecutor::SyncValue(invoke) => invoke(input),
            FunctionExecutor::AsyncValue(_) => unreachable!("filtered async constructor"),
        }
        .and_then(generator.output_to_runtime);
        cancellation.bail_if_cancelled()?;
        match response {
            Ok(value) if is_empty_collection(&value) => empty_response = Some(value),
            Ok(value) => return Ok(value),
            Err(error) => last_error = Some(error),
        }
    }
    if let Some(empty_response) = empty_response {
        diagnostics.push(format!(
            "Arbitrary {} remained empty after {GENERATION_ATTEMPTS} bounded attempts; element coverage is unknown",
            describe_shape(shape)
        ));
        return Ok(empty_response);
    }
    Err(last_error.unwrap_or_else(|| eyre::eyre!("Arbitrary generator produced no response")))
}

fn contains_request_shape(
    shape: &'static Shape,
    functions: impl Iterator<Item = &'static Function> + Clone,
    cancellation: &CancellationScope,
) -> eyre::Result<bool> {
    let mut pending = vec![shape];
    let mut visited = Vec::<&'static Shape>::new();
    while let Some(shape) = pending.pop() {
        cancellation.bail_if_cancelled()?;
        if visited.iter().any(|seen| seen.is_shape(shape)) {
            continue;
        }
        visited.push(shape);
        if functions.clone().any(|function| {
            function.kind == FunctionKind::AsyncInvoke && function.input_shape.is_shape(shape)
        }) {
            return Ok(true);
        }
        match shape.def {
            Def::List(list) => pending.push(list.t()),
            Def::Set(set) => pending.push(set.t()),
            Def::Array(array) => pending.push(array.t()),
            Def::Slice(slice) => pending.push(slice.t()),
            Def::NdArray(array) => pending.push(array.t()),
            Def::Option(optional) => pending.push(optional.t()),
            Def::Map(map) => pending.extend([map.k(), map.v()]),
            Def::Result(result) => pending.extend([result.t(), result.e()]),
            Def::Pointer(pointer) => pending.extend(pointer.pointee()),
            _ => {}
        }
        match shape.ty {
            Type::User(UserType::Struct(record)) => {
                pending.extend(record.fields.iter().map(|field| field.shape()));
            }
            Type::User(UserType::Enum(enumeration)) => {
                pending.extend(
                    enumeration
                        .variants
                        .iter()
                        .flat_map(|variant| variant.data.fields.iter().map(|field| field.shape())),
                );
            }
            _ => {}
        }
    }
    Ok(false)
}

fn is_empty_collection(value: &RuntimeValue) -> bool {
    let peek = value.peek();
    if let Ok(list) = peek.into_list_like() {
        return list.is_empty();
    }
    if let Ok(set) = peek.into_set() {
        return set.is_empty();
    }
    if let Ok(map) = peek.into_map() {
        return map.is_empty();
    }
    false
}
