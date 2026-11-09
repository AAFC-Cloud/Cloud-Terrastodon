#![cfg(test)]

use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use cloud_terrastodon_registry::ArbitraryBytes;
use cloud_terrastodon_registry::known_arbitrary_values;
use cloud_terrastodon_registry::known_function_descriptors;
use cloud_terrastodon_registry::known_thing_for_shape;
use cloud_terrastodon_registry::known_things;
use facet::Facet;
use std::cell::Cell;
use std::sync::Arc;

// Deliberately not Clone: shared collection adapters must retain the allocation,
// and automatic registration must not strengthen the value's trait contract.
#[derive(Facet, Arbitrary)]
struct SharedValue {
    value: u8,
}

cloud_terrastodon_registry::register_thing!(SharedValue);
cloud_terrastodon_registry::register_arbitrary!(SharedValue, associated = GENERATOR);

#[derive(Facet)]
struct OwnedValue {
    #[facet(opaque)]
    value: Cell<u8>,
}

impl<'a> Arbitrary<'a> for OwnedValue {
    fn arbitrary(input: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            value: Cell::new(input.arbitrary()?),
        })
    }
}

cloud_terrastodon_registry::register_thing!(OwnedValue);
cloud_terrastodon_registry::register_arbitrary!(OwnedValue);

#[derive(Facet, Arbitrary)]
struct OverlappingValue(u8);

type OverlappingAlias = OverlappingValue;

cloud_terrastodon_registry::register_thing!(OverlappingValue);
cloud_terrastodon_registry::register_thing!(OverlappingAlias);
cloud_terrastodon_registry::register_thing!(Vec<OverlappingValue>);
cloud_terrastodon_registry::register_arbitrary!(OverlappingValue);
cloud_terrastodon_registry::register_arbitrary!(static EXPLICIT_LIST: Vec<OverlappingValue>);

fn round_trip<T: Facet<'static> + Send + 'static>(value: T) -> eyre::Result<T> {
    let registration = known_thing_for_shape(T::SHAPE).expect("registered type adapter");
    let reflected = registration.runtime_from_boxed(Box::new(value))?;
    let returned = registration.runtime_into_boxed(reflected)?;
    Ok(*returned.downcast::<T>().expect("same concrete value type"))
}

#[test]
fn automatic_collection_adapters_preserve_shared_non_clone_values() -> eyre::Result<()> {
    let item = Arc::new(SharedValue { value: 23 });
    let list = Arc::new(vec![Arc::clone(&item)]);
    let returned = round_trip(Arc::clone(&list))?;
    assert!(Arc::ptr_eq(&returned, &list));
    assert!(Arc::ptr_eq(&returned[0], &item));
    assert!(Arc::ptr_eq(&round_trip(Arc::clone(&item))?, &item));
    assert!(Arc::ptr_eq(&round_trip(vec![Arc::clone(&item)])?[0], &item));
    assert_eq!(round_trip(vec![SharedValue { value: 5 }])?[0].value, 5);
    assert_eq!(
        round_trip(Arc::new(vec![SharedValue { value: 7 }]))?[0].value,
        7
    );

    let generator = known_arbitrary_values()
        .iter()
        .find(|registration| {
            registration
                .output_shape
                .is_shape(Arc::<Vec<Arc<SharedValue>>>::SHAPE)
        })
        .expect("the shared inventory constructor accompanies the item registration");
    let generated = generator.generate(&mut ArbitraryBytes::new(vec![1; 32]))?;
    let reflected = (generator.output_to_runtime)(generated)?;
    let generated = reflected
        .into_box::<Arc<Vec<Arc<SharedValue>>>>()?
        .downcast::<Arc<Vec<Arc<SharedValue>>>>()
        .expect("constructor retains the concrete shared inventory type");
    assert!(!generated.is_empty());
    assert_eq!(
        generator.registration_site,
        SharedValue::GENERATOR.registration_site
    );
    assert!(known_thing_for_shape(Vec::<Vec<SharedValue>>::SHAPE).is_none());
    assert!(known_arbitrary_values().iter().all(|registration| {
        !registration
            .output_shape
            .is_shape(Vec::<Vec<SharedValue>>::SHAPE)
    }));
    Ok(())
}

#[test]
fn non_sync_values_keep_their_owned_adapters_and_constructors() -> eyre::Result<()> {
    let values = round_trip(vec![OwnedValue {
        value: Cell::new(4),
    }])?;
    values[0].value.set(9);
    assert_eq!(values[0].value.get(), 9);
    assert!(known_thing_for_shape(Arc::<OwnedValue>::SHAPE).is_none());
    assert!(
        known_arbitrary_values()
            .iter()
            .all(|registration| { !registration.output_shape.is_shape(Arc::<OwnedValue>::SHAPE) })
    );
    let generator = known_arbitrary_values()
        .iter()
        .find(|registration| registration.output_shape.is_shape(Vec::<OwnedValue>::SHAPE))
        .expect("owned lists still have an offline constructor");
    let generated = generator.generate(&mut ArbitraryBytes::new(vec![1; 16]))?;
    let generated = generated.downcast::<Vec<OwnedValue>>().expect("owned list");
    assert!(!generated.is_empty());
    generated[0].value.set(12);
    assert_eq!(generated[0].value.get(), 12);
    Ok(())
}

#[test]
fn explicit_collection_handles_win_over_automatic_forms_without_duplicates() -> eyre::Result<()> {
    for shape in [OverlappingValue::SHAPE, Vec::<OverlappingValue>::SHAPE] {
        assert_eq!(
            known_things()
                .iter()
                .filter(|thing| thing.shape.is_shape(shape))
                .count(),
            1
        );
        assert_eq!(
            known_arbitrary_values()
                .iter()
                .filter(|value| value.output_shape.is_shape(shape))
                .count(),
            1
        );
    }
    let registered = known_arbitrary_values()
        .iter()
        .find(|registration| {
            registration
                .output_shape
                .is_shape(Vec::<OverlappingValue>::SHAPE)
        })
        .expect("the explicit constructor remains registered");
    assert!(std::ptr::eq(*registered, &EXPLICIT_LIST));
    let descriptors = known_function_descriptors()
        .filter(|descriptor| {
            descriptor
                .output_shape
                .is_shape(Vec::<OverlappingValue>::SHAPE)
        })
        .collect::<Vec<_>>();
    assert_eq!(descriptors.len(), 1);
    assert!(std::ptr::eq(descriptors[0], EXPLICIT_LIST.descriptor()));
    let generated = EXPLICIT_LIST.generate(&mut ArbitraryBytes::new(vec![1; 8]))?;
    let runtime = (EXPLICIT_LIST.output_to_runtime)(generated)?;
    assert!(
        runtime
            .peek()
            .get::<Vec<OverlappingValue>>()?
            .iter()
            .all(|value| value.0 == 1)
    );
    Ok(())
}
