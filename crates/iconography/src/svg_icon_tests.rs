use crate::SvgIcon;
use crate::registered_svg_functions;
use arbitrary::Arbitrary;
use cloud_terrastodon_registry::FunctionKind;
use cloud_terrastodon_registry::FunctionTraitContract;
use cloud_terrastodon_registry::known_functions;
use eyre::Result;
use eyre::eyre;
use facet::Facet;
use std::borrow::Cow;

#[derive(Debug, Default, Facet, Arbitrary)]
struct DynamicIcon {
    svg: String,
    name: String,
    category: String,
}

impl SvgIcon for DynamicIcon {
    fn svg(&self) -> Cow<'static, str> {
        Cow::Owned(self.svg.clone())
    }

    fn name(&self) -> Cow<'static, str> {
        Cow::Owned(self.name.clone())
    }

    fn category(&self) -> Cow<'static, str> {
        Cow::Owned(self.category.clone())
    }
}

crate::register_svg_icon!(DynamicIcon);

#[test]
fn registered_dynamic_svg_remains_owned_after_its_input_is_dropped() -> Result<()> {
    let output = {
        let icon = DynamicIcon {
            svg: "<svg><text>generated at runtime</text></svg>".to_owned(),
            name: "Runtime icon".to_owned(),
            category: "custom".to_owned(),
        };
        let projection = registered_svg_functions()
            .find(|function| {
                function.input_shape.is_shape(DynamicIcon::SHAPE)
                    && function.trait_contract == FunctionTraitContract::Other("SvgIcon::svg")
            })
            .ok_or_else(|| eyre!("DynamicIcon has no registered SVG projection"))?;
        projection.invoke_ref_boxed(&icon)?
    };
    let svg = output
        .downcast::<Cow<'static, str>>()
        .map_err(|_| eyre!("registered SVG projection did not return Cow<'static, str>"))?;
    assert!(matches!(svg.as_ref(), Cow::Owned(_)));
    assert_eq!(
        svg.as_ref().as_ref(),
        "<svg><text>generated at runtime</text></svg>",
    );
    Ok(())
}

fn misleading_string_projection(icon: &DynamicIcon) -> Result<String> {
    Ok(icon.svg.clone())
}

fn misleading_cow_projection(icon: &DynamicIcon) -> Result<Cow<'static, str>> {
    Ok(icon.svg())
}

cloud_terrastodon_registry::register_fn_ref!(
    DynamicIcon => String,
    kind = FunctionKind::Projection,
    label = "SVG contract with a different output",
    trait_contract = FunctionTraitContract::Other("SvgIcon::svg"),
    invoke = misleading_string_projection,
    effects = []
);
cloud_terrastodon_registry::register_fn_ref!(
    DynamicIcon => Cow<'static, str>,
    kind = FunctionKind::Projection,
    label = "SVG contract with observable effects",
    trait_contract = FunctionTraitContract::Other("SvgIcon::svg"),
    invoke = misleading_cow_projection,
    effects = [Read]
);
cloud_terrastodon_registry::register_fn_ref!(
    DynamicIcon => Cow<'static, str>,
    kind = FunctionKind::Constructor,
    label = "SVG contract on a constructor",
    trait_contract = FunctionTraitContract::Other("SvgIcon::svg"),
    invoke = misleading_cow_projection,
    effects = []
);
cloud_terrastodon_registry::register_fn_ref!(
    DynamicIcon => Cow<'static, str>,
    kind = FunctionKind::Projection,
    label = "SVG label without the contract",
    trait_contract = FunctionTraitContract::Other("UnrelatedProjection"),
    invoke = misleading_cow_projection,
    effects = []
);

#[test]
fn misleading_registrations_do_not_join_pure_svg_projections() {
    for label in [
        "SVG contract with a different output",
        "SVG contract with observable effects",
        "SVG contract on a constructor",
        "SVG label without the contract",
    ] {
        assert!(
            known_functions()
                .iter()
                .any(|function| function.label == label),
            "the policy fixture must be registered: {label}",
        );
        assert!(
            !registered_svg_functions().any(|function| function.label == label),
            "incompatible operation was exposed as a pure SVG projection: {label}",
        );
    }
}
