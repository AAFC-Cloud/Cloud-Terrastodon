use cloud_terrastodon_registry::Function;
use cloud_terrastodon_registry::FunctionExecutor;
use cloud_terrastodon_registry::FunctionKind;
use cloud_terrastodon_registry::FunctionTraitContract;
use cloud_terrastodon_registry::ReceiverMode;
use cloud_terrastodon_registry::known_functions;
use facet::Facet;
use std::borrow::Cow;

/// Iterate linked pure SVG, display-name, and category projections.
///
/// Explicit `SvgIcon::svg`, `SvgIcon::name`, and `SvgIcon::category` contracts
/// identify these functions. Results are synchronous shared-reference
/// projections with no effects and a `Cow<'static, str>` output. Match their
/// input shapes to group a registered icon's projections.
pub fn registered_svg_functions() -> impl Iterator<Item = &'static Function> {
    known_functions().iter().filter(|function| {
        function.kind == FunctionKind::Projection
            && function.receiver_mode == ReceiverMode::ByRef
            && matches!(function.executor, FunctionExecutor::SyncRef(_))
            && function.effects.is_empty()
            && function.output_shape.is_shape(Cow::<'static, str>::SHAPE)
            && matches!(
                function.trait_contract,
                FunctionTraitContract::Other(
                    "SvgIcon::svg" | "SvgIcon::name" | "SvgIcon::category"
                )
            )
    })
}
