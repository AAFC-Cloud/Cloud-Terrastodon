use crate::CancellationScope;
use eyre::Result;
use eyre::WrapErr;
use facet::Facet;
use facet::OpaqueBorrow;
use facet::Type;
use facet::UserType;
use facet_reflect::Poke;
use std::any::Any;
use std::any::type_name;

/// Bind an executable request's single direct scope field to its caller's token.
///
/// Both ordinary and `#[facet(opaque)]` scope fields are recognized by shape,
/// regardless of their name or serialization attributes. Tokenless inputs are
/// unchanged. Multiple matching fields are rejected before modifying the input;
/// nested records may contain independent scopes and are not traversed.
pub fn bind_request_cancellation<T>(
    input: &mut (dyn Any + Send),
    cancellation: &CancellationScope,
) -> Result<()>
where
    T: Facet<'static> + Any + Send,
{
    let request = input
        .downcast_mut::<T>()
        .ok_or_else(|| eyre::eyre!("cancellation binding expected {}", type_name::<T>()))?;
    let Type::User(UserType::Struct(struct_type)) = T::SHAPE.ty else {
        return Ok(());
    };
    let mut token_fields = struct_type.fields.iter().enumerate().filter(|(_, field)| {
        field.shape().is_shape(CancellationScope::SHAPE)
            || field
                .shape()
                .is_shape(<OpaqueBorrow<'static, CancellationScope>>::SHAPE)
    });
    let Some((index, field)) = token_fields.next() else {
        return Ok(());
    };
    if let Some((_, other)) = token_fields.next() {
        eyre::bail!(
            "cannot bind cancellation for {}: multiple CancellationScope fields ('{}' and '{}')",
            type_name::<T>(),
            field.name,
            other.name,
        );
    }

    let mut reflected_request = Poke::new(request).into_struct()?;
    let mut token_field = reflected_request.field(index)?;
    let result = if token_field.shape().is_shape(CancellationScope::SHAPE) {
        token_field.set(cancellation.clone())
    } else {
        // Facet represents #[facet(opaque)] fields with this transparent wrapper.
        token_field.set(OpaqueBorrow::<'static, _>::new(cancellation.clone()))
    };
    result.wrap_err_with(|| {
        format!(
            "binding cancellation field '{}' on {}",
            field.name,
            type_name::<T>()
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use teamy_cancellation::CancellationToken;

    #[derive(Facet)]
    struct Request {
        cancellation: String,
        #[facet(rename = "wire_signal")]
        stop_signal: CancellationScope,
    }

    #[test]
    fn binding_uses_the_field_type_and_shares_the_callers_cancellation() -> Result<()> {
        CancellationToken::new().with_scope(|caller| {
            let stale = CancellationScope::unbound();
            stale.request_cancel("old request");
            let mut request = Request {
                cancellation: "ordinary request data".to_owned(),
                stop_signal: stale,
            };

            bind_request_cancellation::<Request>(&mut request, &caller)?;

            assert!(!request.stop_signal.is_cancelled());
            assert_eq!(request.cancellation, "ordinary request data");
            caller.request_cancel("current invocation");
            assert_eq!(
                request.stop_signal.cancellation_reason().as_deref(),
                Some("current invocation")
            );
            Ok(())
        })
    }

    #[derive(Facet)]
    struct AmbiguousRequest {
        first: CancellationScope,
        // Exercise ambiguity between ordinary and field-opaque token shapes.
        #[facet(opaque, skip_serializing)]
        second: CancellationScope,
    }

    #[test]
    fn ambiguous_token_fields_are_rejected_without_replacing_either() {
        CancellationToken::new()
            .with_scope(|caller| {
                let mut request = AmbiguousRequest {
                    first: CancellationScope::unbound(),
                    second: CancellationScope::unbound(),
                };
                request.second.request_cancel("existing second token");

                caller.request_cancel("current invocation");

                let error = bind_request_cancellation::<AmbiguousRequest>(&mut request, &caller)
                    .expect_err("two fields cannot identify a single invocation token");

                assert!(
                    error
                        .to_string()
                        .contains("multiple CancellationScope fields")
                );
                assert!(!request.first.is_cancelled());
                assert_eq!(
                    request.second.cancellation_reason().as_deref(),
                    Some("existing second token")
                );

                Ok::<_, eyre::Report>(())
            })
            .unwrap();
    }

    #[derive(Facet)]
    struct RequestWithNestedInput {
        input: Request,
    }

    #[test]
    fn tokenless_inputs_keep_independent_scopes_in_nested_records() -> Result<()> {
        CancellationToken::new().with_scope(|caller| {
            let mut request = RequestWithNestedInput {
                input: Request {
                    cancellation: "nested data".to_owned(),
                    stop_signal: CancellationScope::unbound(),
                },
            };

            caller.request_cancel("current invocation");

            bind_request_cancellation::<RequestWithNestedInput>(&mut request, &caller)?;

            assert!(!request.input.stop_signal.is_cancelled());
            assert_eq!(request.input.cancellation, "nested data");
            Ok(())
        })
    }
}
