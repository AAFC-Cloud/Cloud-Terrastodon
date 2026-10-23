use crate::CancellationScope;
use crate::DiscoveryValue;
use crate::RuntimeValue;
use crate::describe_shape;
use facet::Def;
use facet::Facet;
use facet::KnownPointer;
use facet::OpaqueBorrow;
use facet::Shape;
use facet::Type;
use facet::UserType;
use std::collections::HashMap;

/// Assemble only an executable request's own struct. Domain values and selectors
/// must already exist in the pool or come from a registered pure conversion.
pub(crate) fn assemble_request(
    shape: &'static Shape,
    values: &HashMap<&'static Shape, Vec<DiscoveryValue>>,
    cancellation: &CancellationScope,
) -> eyre::Result<(RuntimeValue, usize)> {
    let Type::User(UserType::Struct(struct_type)) = shape.ty else {
        eyre::bail!("{} is not a request struct", describe_shape(shape));
    };
    let mut fields = Vec::with_capacity(struct_type.fields.len());
    let mut depth = 0;
    for (index, field) in struct_type.fields.iter().enumerate() {
        cancellation.bail_if_cancelled()?;
        // Actual shapes matter here: a String proxy is not a discovered domain
        // selector, even when the selector's wire representation is a string.
        let (value, field_depth) =
            input_field(field.shape(), values, cancellation).map_err(|error| {
                error.wrap_err(format!(
                    "missing field '{}' of {}",
                    field.name,
                    describe_shape(shape)
                ))
            })?;
        fields.push((index, value));
        depth = depth.max(field_depth);
    }
    let value = RuntimeValue::build_with(shape, |mut partial| {
        for (index, value) in fields {
            partial = partial.begin_nth_field(index)?;
            // SAFETY: the field was built against its actual reflected shape.
            // set_from_peek transfers ownership into the partial allocation;
            // deallocate_after_move releases only the now-empty source storage.
            partial = unsafe { partial.set_from_peek(&value.peek()) }?;
            value.deallocate_after_move();
            partial = partial.end()?;
        }
        Ok(partial)
    })?;
    Ok((value, depth.saturating_add(1)))
}

fn input_field(
    shape: &'static Shape,
    values: &HashMap<&'static Shape, Vec<DiscoveryValue>>,
    cancellation: &CancellationScope,
) -> eyre::Result<(RuntimeValue, usize)> {
    if shape.is_shape(CancellationScope::SHAPE) {
        return Ok((RuntimeValue::from_box(Box::new(cancellation.clone()))?, 0));
    }
    if shape.is_shape(<OpaqueBorrow<'static, CancellationScope>>::SHAPE) {
        return Ok((
            RuntimeValue::from_box(Box::new(OpaqueBorrow::<'static, _>::new(
                cancellation.clone(),
            )))?,
            0,
        ));
    }
    if let Some(existing) = values.get(shape).and_then(|values| values.first()) {
        return Ok((existing.value.try_clone()?, existing.depth));
    }
    if let Def::Pointer(pointer) = shape.def
        && matches!(
            pointer.known,
            Some(KnownPointer::Cow | KnownPointer::Box | KnownPointer::Arc | KnownPointer::Rc)
        )
        && let Some(pointee) = pointer.pointee()
        && RuntimeValue::can_own_pointee(shape, pointee)
        && let Some(existing) = values.get(pointee).and_then(|values| values.first())
    {
        return Ok((
            RuntimeValue::from_owned_pointee(shape, existing.value.try_clone()?)?,
            existing.depth,
        ));
    }
    if matches!(shape.def, Def::Option(_)) {
        return Ok((RuntimeValue::from_default(shape)?, 0));
    }
    eyre::bail!("no discovered value of {}", describe_shape(shape));
}
