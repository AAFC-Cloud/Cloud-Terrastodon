use cloud_terrastodon_iconography::registered_svg_functions;
use cloud_terrastodon_iconography_azure as _;
use cloud_terrastodon_iconography_azure_devops as _;
use cloud_terrastodon_iconography_cli::validation::validate_svg;
use cloud_terrastodon_registry::Function;
use cloud_terrastodon_registry::FunctionKind;
use cloud_terrastodon_registry::FunctionTraitContract;
use cloud_terrastodon_registry::ReceiverMode;
use cloud_terrastodon_registry::RuntimeValue;
use cloud_terrastodon_registry::describe_shape;
use cloud_terrastodon_registry::known_thing_for_shape;
use eyre::Result;
use eyre::WrapErr;
use std::any::Any;
use std::borrow::Cow;
use std::collections::BTreeSet;

#[test]
fn linked_category_icons_support_complete_offline_registry_invocation() -> Result<()> {
    let functions = registered_svg_functions().collect::<Vec<_>>();
    let mut names = BTreeSet::new();
    for svg_function in functions
        .iter()
        .copied()
        .filter(|function| function.trait_contract == FunctionTraitContract::Other("SvgIcon::svg"))
    {
        let input_name = describe_shape(svg_function.input_shape);
        let projections = functions
            .iter()
            .copied()
            .filter(|function| function.input_shape.is_shape(svg_function.input_shape))
            .collect::<Vec<_>>();
        let contracts = projections
            .iter()
            .map(|function| function.trait_contract.as_str())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            contracts,
            BTreeSet::from(["SvgIcon::svg", "SvgIcon::name", "SvgIcon::category"]),
            "{input_name}"
        );
        assert_eq!(projections.len(), 3, "{input_name}");

        let thing = known_thing_for_shape(svg_function.input_shape)
            .ok_or_else(|| eyre::eyre!("Icon input {input_name} is not registered"))?;
        let input = thing
            .runtime_into_boxed(RuntimeValue::from_default(svg_function.input_shape)?)
            .wrap_err_with(|| format!("Constructing registered icon {input_name}"))?;
        for function in &projections {
            assert_eq!(function.kind, FunctionKind::Projection, "{input_name}");
            assert_eq!(function.receiver_mode, ReceiverMode::ByRef, "{input_name}");
            assert!(function.effects.is_empty(), "{input_name}");
            let output = invoke_text(function, input.as_ref())?;
            match function.trait_contract {
                FunctionTraitContract::Other("SvgIcon::svg") => {
                    validate_svg(output.as_bytes(), &input_name)?;
                    assert!(matches!(output, Cow::Borrowed(_)), "{input_name}");
                }
                FunctionTraitContract::Other("SvgIcon::name") => {
                    assert!(!output.trim().is_empty(), "{input_name}");
                    names.insert(output.into_owned());
                }
                FunctionTraitContract::Other("SvgIcon::category") => {
                    assert!(!output.trim().is_empty(), "{input_name}");
                    assert_ne!(output.as_ref(), "repos", "The repos category is disabled");
                }
                _ => unreachable!("only icon contracts were selected"),
            }
        }
    }

    for expected in ["Resource group", "Subscription", "Boards", "Plan"] {
        assert!(
            names.contains(expected),
            "Missing enabled icon {expected}: {names:?}"
        );
    }
    assert!(!names.contains("Repos"), "The repos category is disabled");
    Ok(())
}

fn invoke_text(function: &Function, input: &(dyn Any + Send)) -> Result<Cow<'static, str>> {
    let output = function.invoke_ref_boxed(input).wrap_err_with(|| {
        format!(
            "Invoking {} for {}",
            function.trait_contract,
            describe_shape(function.input_shape),
        )
    })?;
    output
        .downcast::<Cow<'static, str>>()
        .map(|text| *text)
        .map_err(|_| eyre::eyre!("Icon projection did not return a Cow string"))
}
