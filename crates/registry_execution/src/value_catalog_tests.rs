use crate::DiscoveryPlanBuildRequest;
use crate::Function;
use crate::RegisteredFunction;
use crate::functions_from_to;
use crate::functions_to;
use crate::known_functions;
use arbitrary::Arbitrary;
use cloud_terrastodon_registry::ArbitraryBytes;
use cloud_terrastodon_registry::RegisteredShape;
use cloud_terrastodon_registry::known_function_descriptors;
use cloud_terrastodon_registry::known_thing_for_shape;
use cloud_terrastodon_registry::known_things;
use facet::Facet;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

static GENERATED: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, PartialEq, Eq, Facet)]
struct PureValue {
    marker: u16,
}

impl<'a> Arbitrary<'a> for PureValue {
    fn arbitrary(input: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        GENERATED.fetch_add(1, Ordering::SeqCst);
        Ok(Self {
            marker: input.arbitrary()?,
        })
    }
}

cloud_terrastodon_registry::register_thing!(PureValue);
cloud_terrastodon_registry::register_arbitrary!(PureValue, associated = REGISTERED_GENERATOR);

#[test]
fn pure_registrations_join_the_catalog_without_running_or_duplicating_them() -> eyre::Result<()> {
    let values = known_things()
        .into_iter()
        .filter(|thing| thing.shape.is_shape(PureValue::SHAPE))
        .collect::<Vec<_>>();
    assert_eq!(values.len(), 1);
    assert!(std::ptr::eq(
        values[0],
        known_thing_for_shape(PureValue::SHAPE).unwrap()
    ));

    let constructors = functions_from_to(ArbitraryBytes::SHAPE, PureValue::SHAPE);
    let [constructor] = constructors.as_slice() else {
        panic!("the low descriptor must have one catalog handle");
    };
    assert!(*constructor == Function::Arbitrary(PureValue::REGISTERED_GENERATOR));
    assert!(std::ptr::eq(
        constructor.descriptor(),
        PureValue::REGISTERED_GENERATOR.descriptor(),
    ));
    assert_eq!(
        known_function_descriptors()
            .filter(|descriptor| std::ptr::eq(*descriptor, constructor.descriptor()))
            .count(),
        1,
        "the executable and pure catalogues must share one authoritative descriptor",
    );
    assert!(*constructor == functions_to(PureValue::SHAPE)[0]);

    let handle = RegisteredFunction::from_function(*constructor);
    assert_eq!(handle.to_string().parse::<RegisteredFunction>()?, handle);
    assert!(DiscoveryPlanBuildRequest::default_functions().contains(&handle));
    assert!(
        RegisteredShape::all(
            known_functions()
                .iter()
                .map(|function| function.descriptor()),
            known_things()
        )
        .contains(&RegisteredShape::from_shape(PureValue::SHAPE))
    );
    assert_eq!(GENERATED.load(Ordering::SeqCst), 0);

    let mut input = ArbitraryBytes::new(vec![1, 2, 3, 4]);
    let output = constructor.invoke_mut_boxed(&mut input)?;
    let runtime = (constructor.output_to_runtime)(output)?;
    assert!(runtime.shape().is_shape(PureValue::SHAPE));
    assert!(input.as_slice().len() < 4);
    assert_eq!(GENERATED.load(Ordering::SeqCst), 1);
    assert!(*constructor == functions_to(PureValue::SHAPE)[0]);
    Ok(())
}
