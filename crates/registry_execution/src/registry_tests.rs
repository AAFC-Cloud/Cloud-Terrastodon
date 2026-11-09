use super::*;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::Authorization;
use cloud_terrastodon_auth_types::AzureCredentialResource;
use cloud_terrastodon_cache::ArtifactObservation;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::CacheScope;
use cloud_terrastodon_cache::ExecutorKind;
use cloud_terrastodon_cache::NonEmptyString;
use cloud_terrastodon_cache::WorkCache;
use cloud_terrastodon_cache::WorkOutput;
use cloud_terrastodon_cache::WorkReuse;
use cloud_terrastodon_execution::AnonymousAuthenticationProvider;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::TestAuthenticationProvider;
use cloud_terrastodon_registry::ArbitraryBytes;
use cloud_terrastodon_registry::FunctionKind;
use cloud_terrastodon_registry::KNOWN_BORROWED_POINTER_KINDS;
use cloud_terrastodon_registry::ProductionKind;
use cloud_terrastodon_registry::ReceiverMode;
use cloud_terrastodon_registry::RuntimeValue;
use cloud_terrastodon_registry::describe_shape;
use cloud_terrastodon_registry::known_shapes;
use cloud_terrastodon_registry::shape_variants_for_thing;
use facet::Facet;
use std::any::Any;
use std::borrow::Cow;
use std::cell::Cell;
use std::future::Future;
use std::future::IntoFuture;
use std::future::pending;
use std::future::poll_fn;
use std::io;
use std::io::ErrorKind;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::task::Poll;
use std::time::Duration;
use teamy_cancellation::CancellationError;
use teamy_cancellation::CancellationToken;

#[derive(Debug, Clone, Copy, Default, Facet)]
#[repr(C)]
struct DummyTenant;

#[derive(Debug, Clone, Arbitrary, Facet, PartialEq, Eq)]
#[repr(C)]
struct DummyOutput {
    value: String,
}

#[derive(Debug, Clone, Facet)]
#[repr(C)]
struct DummyListRequest {
    tenant: DummyTenant,
}

impl IntoFuture for DummyListRequest {
    type Output = eyre::Result<Vec<DummyOutput>>;
    type IntoFuture = std::pin::Pin<Box<dyn Future<Output = Self::Output> + Send>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async { Ok(vec![DummyOutput { value: "ok".into() }]) })
    }
}

#[derive(Debug, Clone, Facet, PartialEq, Eq)]
#[repr(C)]
struct DummyConverted {
    value: String,
}

#[derive(Debug, Clone, Default, Facet, PartialEq, Eq)]
#[repr(C)]
enum DummyRenamedDefault {
    #[default]
    Unspecified,
    Explicit,
}

#[derive(Debug, Clone, Facet, PartialEq, Eq)]
#[repr(C)]
enum DummyManualDefault {
    Unspecified,
    Explicit,
}

#[derive(Debug, Facet)]
#[repr(C)]
struct DummyNotCloneable {
    value: String,
}

#[allow(clippy::derivable_impls)]
impl Default for DummyManualDefault {
    fn default() -> Self {
        Self::Explicit
    }
}

impl From<DummyOutput> for DummyConverted {
    fn from(value: DummyOutput) -> Self {
        Self { value: value.value }
    }
}

cloud_terrastodon_registry::register_thing!(DummyTenant);
cloud_terrastodon_registry::register_thing!(DummyOutput);
cloud_terrastodon_registry::register_thing!(DummyListRequest);
cloud_terrastodon_registry::register_thing!(DummyRenamedDefault);
cloud_terrastodon_registry::register_thing!(DummyManualDefault);
crate::register_into_future!(DummyListRequest => Vec<DummyOutput>, effects = [Read]);
cloud_terrastodon_registry::register_from!(DummyOutput => DummyConverted, associated = REGISTERED_INTO_CONVERTED, effects = []);
cloud_terrastodon_registry::register_from!(static U8_TO_U16: u8 => u16);
cloud_terrastodon_registry::register_arbitrary!(DummyOutput, associated = REGISTERED_ARBITRARY);
cloud_terrastodon_registry::register_arbitrary!(static DUMMY_OUTPUTS_ARBITRARY: Vec<DummyOutput>);

#[test]
fn known_shapes_deduplicates() {
    assert!(
        known_shapes()
            .iter()
            .any(|shape| shape.label == describe_shape(ArbitraryBytes::SHAPE))
    );
}

#[test]
fn enum_default_variant_comes_from_facet_default_construction() {
    let thing = known_shapes()
        .into_iter()
        .find(|shape| shape.label == "DummyRenamedDefault")
        .expect("DummyRenamedDefault should be registered");
    let variants = shape_variants_for_thing(thing.thing);

    assert!(
        variants
            .iter()
            .any(|variant| { variant.variant_name == "Unspecified" && variant.is_default })
    );
    assert!(
        variants
            .iter()
            .all(|variant| { variant.variant_name != "Explicit" || !variant.is_default })
    );
}

#[test]
fn one_cow_pointer_kind_registration_supports_multiple_pointee_shapes() {
    assert_eq!(
        KNOWN_BORROWED_POINTER_KINDS
            .iter()
            .filter(|kind| kind.known == facet::KnownPointer::Cow)
            .count(),
        1
    );

    let pointee: Cow<'static, DummyOutput> = Cow::Owned(DummyOutput {
        value: "struct".to_string(),
    });
    let source = RuntimeValue::from_box(Box::new(pointee))
        .expect("Cow<DummyOutput> should be a reflected runtime value");
    let source_inner = source
        .peek()
        .into_pointer()
        .expect("Cow should reflect as a pointer")
        .borrow_inner()
        .expect("Cow should expose its pointee");
    // SAFETY: source remains alive and unchanged until the borrow is promoted;
    // no borrowed value or reference escapes that interval.
    let borrowed = unsafe {
        RuntimeValue::from_borrowed_pointer(<Cow<'static, DummyOutput>>::SHAPE, source_inner)
    }
    .expect("the registered Cow adapter should construct a borrow");
    let promoted = borrowed
        .promote_to_owned()
        .expect("the registered Cow adapter should promote to owned");
    drop(source);
    let promoted = promoted
        .into_box::<Cow<'static, DummyOutput>>()
        .expect("promoted Cow should retain its reflected shape")
        .downcast::<Cow<'static, DummyOutput>>()
        .expect("promoted Cow should retain its concrete type");
    assert!(matches!(*promoted, Cow::Owned(_)));
    assert_eq!(promoted.value, "struct");

    let source_text: Cow<'static, str> = Cow::Owned(String::from("text"));
    let source_text = RuntimeValue::from_box(Box::new(source_text))
        .expect("Cow<str> should be a reflected runtime value");
    let source_text_inner = source_text
        .peek()
        .into_pointer()
        .expect("Cow<str> should reflect as a pointer")
        .borrow_inner()
        .expect("Cow<str> should expose its str pointee");
    // SAFETY: source_text stays alive and unchanged through promotion, and
    // the resulting owned String retains no reference to source_text.
    let promoted_text = unsafe {
        RuntimeValue::from_borrowed_pointer(<Cow<'static, str>>::SHAPE, source_text_inner)
    }
    .expect("the same Cow adapter should support Cow<str>")
    .promote_to_owned()
    .expect("Cow<str> should promote in place");
    drop(source_text);
    let promoted_text = promoted_text
        .into_box::<Cow<'static, str>>()
        .expect("promoted Cow<str> should retain its reflected shape")
        .downcast::<Cow<'static, str>>()
        .expect("promoted Cow<str> should retain its concrete type");
    assert!(matches!(*promoted_text, Cow::Owned(_)));
    assert_eq!(promoted_text.as_ref(), "text");
}

#[test]
fn cow_promotion_preserves_allocation_and_borrowed_clone_lifetimes() {
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;

    static CLONES: AtomicUsize = AtomicUsize::new(0);
    static DROPS: AtomicUsize = AtomicUsize::new(0);

    #[derive(Debug, Facet)]
    struct Tracked {
        value: String,
    }

    impl Clone for Tracked {
        fn clone(&self) -> Self {
            CLONES.fetch_add(1, Ordering::SeqCst);
            Self {
                value: self.value.clone(),
            }
        }
    }

    impl Drop for Tracked {
        fn drop(&mut self) {
            DROPS.fetch_add(1, Ordering::SeqCst);
        }
    }

    let source = RuntimeValue::from_box(Box::new(Tracked {
        value: "tracked".to_owned(),
    }))
    .unwrap();
    // SAFETY: source stays alive and unchanged until both borrowed values
    // are dropped or promoted. Tracked owns its String, so promotion leaves
    // no nested reference to the source, and no reference escapes this test.
    let borrowed = unsafe {
        RuntimeValue::from_borrowed_pointer(<Cow<'static, Tracked>>::SHAPE, source.peek())
    }
    .unwrap();
    let allocation = borrowed.peek().data().as_byte_ptr();
    let shallow_clone = borrowed.try_clone().unwrap();
    assert_eq!(CLONES.load(Ordering::SeqCst), 0);

    let promoted = borrowed.promote_to_owned().unwrap();
    assert_eq!(promoted.peek().data().as_byte_ptr(), allocation);
    assert_eq!(CLONES.load(Ordering::SeqCst), 1);
    assert!(matches!(
        shallow_clone.peek().get::<Cow<'static, Tracked>>().unwrap(),
        Cow::Borrowed(_)
    ));
    drop(shallow_clone);
    assert_eq!(DROPS.load(Ordering::SeqCst), 0);
    drop(source);
    assert_eq!(DROPS.load(Ordering::SeqCst), 1);

    let promoted = promoted.promote_to_owned().unwrap();
    assert_eq!(promoted.peek().data().as_byte_ptr(), allocation);
    assert_eq!(CLONES.load(Ordering::SeqCst), 1);
    assert_eq!(
        promoted
            .peek()
            .get::<Cow<'static, Tracked>>()
            .unwrap()
            .value,
        "tracked"
    );
    drop(promoted);
    assert_eq!(DROPS.load(Ordering::SeqCst), 2);
}

#[test]
fn reflected_pointer_can_take_ownership_of_a_pointee() {
    let source = RuntimeValue::from_box(Box::new(DummyOutput {
        value: "moved".to_owned(),
    }))
    .expect("pointee is representable");

    let pointer = RuntimeValue::from_owned_pointee(<Cow<'static, DummyOutput>>::SHAPE, source)
        .expect("Cow exposes Facet's owned-pointee constructor");
    let pointer = pointer
        .into_box::<Cow<'static, DummyOutput>>()
        .expect("pointer retains its reflected shape")
        .downcast::<Cow<'static, DummyOutput>>()
        .expect("pointer retains its concrete type");

    assert!(matches!(&*pointer, Cow::Owned(_)));
    assert_eq!(pointer.value, "moved");
}

#[test]
fn enum_manual_default_comes_from_facet_default_construction() {
    let thing = known_shapes()
        .into_iter()
        .find(|shape| shape.label == "DummyManualDefault")
        .expect("DummyManualDefault should be registered");
    let variants = shape_variants_for_thing(thing.thing);

    assert!(
        variants
            .iter()
            .any(|variant| { variant.variant_name == "Explicit" && variant.is_default })
    );
    assert!(
        variants
            .iter()
            .all(|variant| { variant.variant_name != "Unspecified" || !variant.is_default })
    );
}

#[test]
fn runtime_value_uses_reflected_default_parse_and_clone_operations() -> eyre::Result<()> {
    let default = RuntimeValue::from_default(DummyRenamedDefault::SHAPE)?;
    assert_eq!(default.display_string(), "Unspecified");

    let parsed = RuntimeValue::from_text(bool::SHAPE, "true")?;
    assert_eq!(parsed.display_string(), "true");
    assert!(RuntimeValue::from_text(bool::SHAPE, "not a bool").is_err());

    let cloned = default.try_clone()?;
    let boxed = cloned.into_box::<DummyRenamedDefault>()?;
    assert!(matches!(
        *boxed.downcast::<DummyRenamedDefault>().unwrap(),
        DummyRenamedDefault::Unspecified
    ));

    let not_cloneable = RuntimeValue::from_box(Box::new(DummyNotCloneable {
        value: "value".to_string(),
    }))?;
    assert!(not_cloneable.try_clone().is_err());
    Ok(())
}

#[test]
fn runtime_value_deep_clones_reflected_lists() -> eyre::Result<()> {
    let source = RuntimeValue::from_box(Box::new(vec![
        DummyOutput {
            value: "first".to_string(),
        },
        DummyOutput {
            value: "second".to_string(),
        },
    ]))?;

    let cloned = source.try_clone()?.into_box::<Vec<DummyOutput>>()?;
    assert_eq!(
        *cloned.downcast::<Vec<DummyOutput>>().unwrap(),
        vec![
            DummyOutput {
                value: "first".to_string(),
            },
            DummyOutput {
                value: "second".to_string(),
            },
        ]
    );
    Ok(())
}

#[test]
fn runtime_value_clones_nested_lists_from_roots_and_projections() -> eyre::Result<()> {
    let expected = vec![
        vec![DummyOutput {
            value: "nested".into(),
        }],
        vec![],
        vec![DummyOutput {
            value: "independent".into(),
        }],
    ];
    let source = RuntimeValue::from_box(Box::new(expected.clone()))?;
    assert!(RuntimeValue::can_clone_shape(source.shape()));
    let owned = source.try_clone()?;
    let projected = RuntimeValue::clone_from_peek(source.peek())?;
    drop(source);
    assert_eq!(owned.peek().get::<Vec<Vec<DummyOutput>>>()?, &expected);
    assert_eq!(projected.peek().get::<Vec<Vec<DummyOutput>>>()?, &expected);
    Ok(())
}

#[test]
fn runtime_value_rejects_lists_without_element_clone_support() -> eyre::Result<()> {
    let source = RuntimeValue::from_box(Box::new(vec![vec![DummyNotCloneable {
        value: "source remains intact".into(),
    }]]))?;
    assert!(!RuntimeValue::can_clone_shape(source.shape()));
    assert!(source.try_clone().is_err());
    assert!(RuntimeValue::clone_from_peek(source.peek()).is_err());
    assert_eq!(
        source.peek().get::<Vec<Vec<DummyNotCloneable>>>()?[0][0].value,
        "source remains intact"
    );
    Ok(())
}

#[test]
fn direct_registration_handles_alias_the_linked_entries() {
    for handle in [
        DummyListRequest::REGISTERED_INTO_FUTURE,
        Function::Pure(DummyOutput::REGISTERED_INTO_CONVERTED),
        Function::Pure(&U8_TO_U16),
        Function::Arbitrary(DummyOutput::REGISTERED_ARBITRARY),
        Function::Arbitrary(&DUMMY_OUTPUTS_ARBITRARY),
    ] {
        let registrations = known_functions()
            .iter()
            .filter(|function| {
                function.input_shape.is_shape(handle.input_shape)
                    && function.output_shape.is_shape(handle.output_shape)
                    && function.kind == handle.kind
            })
            .collect::<Vec<_>>();
        assert_eq!(
            registrations.len(),
            1,
            "a handle must not add a second entry"
        );
        assert!(
            registrations[0] == handle,
            "a handle must reference the linked static, not copied metadata",
        );
    }
}

#[test]
fn function_lookup_tracks_receivers() {
    let async_functions = functions_from_to(DummyListRequest::SHAPE, DummyOutput::SHAPE);
    assert_eq!(async_functions[0].receiver_mode, ReceiverMode::ByValue);
    assert_eq!(
        async_functions[0].production_kind(DummyOutput::SHAPE),
        Some(ProductionKind::ListElement)
    );

    let mut_functions = functions_from_to(ArbitraryBytes::SHAPE, DummyOutput::SHAPE);
    assert!(
        mut_functions
            .iter()
            .any(|f| f.receiver_mode == ReceiverMode::ByMut)
    );
}

#[tokio::test]
async fn async_functions_invoke_erased_requests() -> eyre::Result<()> {
    CancellationToken::new()
        .cancellable_scope(|cancellation| async move {
            let executor = test_executor(cancellation)?;
            let function = functions_from(DummyListRequest::SHAPE)
                .into_iter()
                .find(|f| f.kind == FunctionKind::AsyncInvoke)
                .unwrap();
            let output = match function.invoke_value_boxed(
                Box::new(DummyListRequest {
                    tenant: DummyTenant,
                }),
                &executor,
            )? {
                FunctionInvocation::Pending(future) => future.await?,
                FunctionInvocation::Ready(_) => panic!("expected pending"),
            };
            let output = output.downcast::<Vec<DummyOutput>>().unwrap();
            assert_eq!(output[0].value, "ok");
            Ok(())
        })
        .await
}

#[tokio::test]
async fn default_production_composes_registered_field_producers() -> eyre::Result<()> {
    CancellationToken::new()
        .cancellable_scope(|cancellation| async move {
            let executor = test_executor(cancellation)?;
            assert!(shape_can_be_produced_from_defaults(DummyListRequest::SHAPE));

            let function = functions_from(DummyListRequest::SHAPE)
                .into_iter()
                .find(|function| function.kind == FunctionKind::AsyncInvoke)
                .unwrap();
            let output = invoke_with_default_input(function, &executor).await?;
            let output = output.downcast::<Vec<DummyOutput>>().unwrap();
            assert_eq!(output[0].value, "ok");
            Ok(())
        })
        .await
}

#[test]
fn by_mut_arbitrary_functions_consume_bytes() -> eyre::Result<()> {
    let function = functions_from_to(ArbitraryBytes::SHAPE, DummyOutput::SHAPE)
        .into_iter()
        .find(|f| f.receiver_mode == ReceiverMode::ByMut)
        .unwrap();
    let mut input: Box<dyn Any + Send> = Box::new(ArbitraryBytes::new(vec![1; 256]));
    let starting = input.downcast_ref::<ArbitraryBytes>().unwrap().0.len();
    let _ = function.invoke_mut_boxed(input.as_mut())?;
    let ending = input.downcast_ref::<ArbitraryBytes>().unwrap().0.len();
    assert!(ending < starting);
    Ok(())
}
#[derive(Facet)]
struct CancellationProbeRequest {
    #[facet(skip_serializing)]
    stop_signal: CancellationScope,
}

impl std::future::IntoFuture for CancellationProbeRequest {
    type Output = eyre::Result<bool>;
    type IntoFuture = std::pin::Pin<Box<dyn std::future::Future<Output = Self::Output> + Send>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move { Ok(!self.stop_signal.is_cancelled()) })
    }
}
cloud_terrastodon_registry::register_thing!(CancellationProbeRequest);
register_into_future!(CancellationProbeRequest => bool, effects = [Read]);

#[tokio::test]
async fn invocation_binds_the_callers_token_instead_of_reflected_input_state() -> eyre::Result<()> {
    CancellationToken::new()
        .cancellable_scope(|invocation_token| async move {
            let executor = test_executor(invocation_token)?;
            let input_token = CancellationScope::unbound();
            input_token.request_cancel("stale input cancellation");

            let function = functions_from(CancellationProbeRequest::SHAPE)
                .into_iter()
                .find(|function| function.kind == FunctionKind::AsyncInvoke)
                .unwrap();
            let invocation = function.invoke_value_boxed(
                Box::new(CancellationProbeRequest {
                    stop_signal: input_token,
                }),
                &executor,
            )?;
            let FunctionInvocation::Pending(future) = invocation else {
                panic!("expected async probe")
            };
            assert!(*future.await?.downcast::<bool>().unwrap());
            Ok(())
        })
        .await
}

fn test_executor(scope: CancellationScope) -> eyre::Result<CloudTerrastodonExecutor> {
    Ok(CloudTerrastodonExecutor::new(
        WorkCache::temporary()?,
        Arc::new(AnonymousAuthenticationProvider),
        scope,
    ))
}

#[derive(Facet)]
struct CachedRequest {
    value: String,
}

impl Default for CachedRequest {
    fn default() -> Self {
        Self {
            value: "synthetic-default-value".to_owned(),
        }
    }
}

impl ExecutableRequest for CachedRequest {
    type Error = eyre::Report;
    type Output = Arc<DummyOutput>;

    fn run<'a>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, Self::Output>
    where
        Self: 'a,
        Self::Output: 'a,
    {
        Box::pin(async move {
            executor
                .cached_work(
                    ExecutorKind::InProcess,
                    CacheKey {
                        scope: CacheScope::try_from_components(["synthetic", "registry-value"])?,
                        valid_for: Duration::from_secs(60),
                    },
                    CacheOperationOwner::of::<Self>(),
                    &AuthenticationSelection::Anonymous,
                    NonEmptyString::new(self.value.clone())?,
                    WorkReuse::ReusableRead,
                    move |child| async move {
                        child
                            .authorization(
                                &AuthenticationSelection::Anonymous,
                                AzureCredentialResource::AzureResourceManager,
                            )
                            .await?;
                        Ok(WorkOutput::new(
                            DummyOutput { value: self.value },
                            ArtifactObservation::now(),
                        ))
                    },
                )?
                .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(CachedRequest);
register_executable_request!(CachedRequest => Arc<DummyOutput>, effects = [Read]);

#[tokio::test]
async fn executable_registration_uses_bound_authentication_and_shares_cached_output()
-> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let provider = Arc::new(TestAuthenticationProvider {
                allow_authenticated: false.into(),
                ..TestAuthenticationProvider::new(
                    "synthetic-test-session",
                    Authorization::Anonymous,
                )?
            });
            let executor =
                CloudTerrastodonExecutor::new(WorkCache::temporary()?, provider.clone(), scope);
            let direct = executor
                .execute(CachedRequest {
                    value: "shared".into(),
                })
                .await?;
            let function = CachedRequest::REGISTERED_EXECUTABLE_REQUEST;
            assert_eq!(
                known_functions()
                    .iter()
                    .filter(|entry| *entry == function)
                    .count(),
                1
            );
            let FunctionInvocation::Pending(invocation) = function.invoke_value_boxed(
                Box::new(CachedRequest {
                    value: "shared".into(),
                }),
                &executor,
            )?
            else {
                panic!("executable requests must produce pending invocations");
            };
            let erased = invocation.await?.downcast::<Arc<DummyOutput>>().unwrap();
            assert!(Arc::ptr_eq(&direct, &erased));
            assert_eq!(provider.authorizations.load(Ordering::SeqCst), 1);

            let FunctionInvocation::Pending(wrong_input) =
                function.invoke_value_boxed(Box::new(DummyTenant), &executor)?
            else {
                panic!("executable requests must produce pending invocations");
            };
            let error = wrong_input.await.unwrap_err();
            assert!(
                error
                    .to_string()
                    .contains(std::any::type_name::<CachedRequest>())
            );
            Ok(())
        })
        .await
}

#[tokio::test]
async fn default_production_executes_new_requests_with_the_callers_shared_resources()
-> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let provider = Arc::new(TestAuthenticationProvider {
                allow_authenticated: false.into(),
                ..TestAuthenticationProvider::new(
                    "synthetic-test-session",
                    Authorization::Anonymous,
                )?
            });
            let executor =
                CloudTerrastodonExecutor::new(WorkCache::temporary()?, provider.clone(), scope);
            assert!(default_production_plan(CachedRequest::SHAPE).is_some());
            assert_eq!(provider.authorizations.load(Ordering::SeqCst), 0);

            let produced =
                invoke_with_default_input(CachedRequest::REGISTERED_EXECUTABLE_REQUEST, &executor)
                    .await?
                    .downcast::<Arc<DummyOutput>>()
                    .unwrap();
            assert_eq!(produced.value, CachedRequest::default().value);
            assert_eq!(provider.authorizations.load(Ordering::SeqCst), 1);

            let direct = executor.execute(CachedRequest::default()).await?;
            assert!(Arc::ptr_eq(&produced, &direct));
            assert_eq!(provider.authorizations.load(Ordering::SeqCst), 1);
            Ok(())
        })
        .await
}

#[derive(Facet)]
struct NonSyncOutput {
    #[facet(opaque)]
    value: Cell<u32>,
}

#[derive(Facet)]
struct NonSyncRequest;

impl ExecutableRequest for NonSyncRequest {
    type Error = eyre::Report;
    type Output = NonSyncOutput;

    fn run<'a>(self, _executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, Self::Output>
    where
        Self: 'a,
        Self::Output: 'a,
    {
        Box::pin(async {
            Ok(NonSyncOutput {
                value: Cell::new(7),
            })
        })
    }
}

cloud_terrastodon_registry::register_thing!(NonSyncRequest);
cloud_terrastodon_registry::register_thing!(NonSyncOutput);
register_executable_request!(NonSyncRequest => NonSyncOutput, effects = [Read]);

#[tokio::test]
async fn executable_registration_returns_owned_non_sync_values() -> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let executor = test_executor(scope)?;
            let FunctionInvocation::Pending(invocation) =
                NonSyncRequest::REGISTERED_EXECUTABLE_REQUEST
                    .invoke_value_boxed(Box::new(NonSyncRequest), &executor)?
            else {
                panic!("executable requests must produce pending invocations");
            };
            let output = invocation.await?.downcast::<NonSyncOutput>().unwrap();
            output.value.set(output.value.get() + 1);
            assert_eq!(output.value.get(), 8);
            Ok(())
        })
        .await
}

#[derive(Debug)]
enum CustomRequestFailure {
    Partial {
        completed: Cell<u32>,
        source: io::Error,
    },
    Cancelled(CancellationError),
}

impl From<CancellationError> for CustomRequestFailure {
    fn from(error: CancellationError) -> Self {
        Self::Cancelled(error)
    }
}

// A request's error need not implement Error or Sync. Its explicit conversion
// decides which structured source and domain context reach erased consumers.
impl From<CustomRequestFailure> for eyre::Report {
    fn from(error: CustomRequestFailure) -> Self {
        match error {
            CustomRequestFailure::Partial { completed, source } => eyre::Report::new(source)
                .wrap_err(format!(
                    "completed {} synthetic items",
                    completed.into_inner()
                )),
            CustomRequestFailure::Cancelled(error) => eyre::Report::new(error),
        }
    }
}

#[derive(Facet)]
struct CustomErrorRequest;

impl ExecutableRequest for CustomErrorRequest {
    type Output = ();
    type Error = CustomRequestFailure;

    fn run<'a>(
        self,
        _executor: CloudTerrastodonExecutor,
    ) -> ExecutionFuture<'a, Self::Output, Self::Error>
    where
        Self: 'a,
        Self::Output: 'a,
        Self::Error: 'a,
    {
        Box::pin(async {
            Err(CustomRequestFailure::Partial {
                completed: Cell::new(3),
                source: io::Error::new(ErrorKind::ConnectionReset, "synthetic interrupted read"),
            })
        })
    }
}

register_executable_request!(CustomErrorRequest => (), effects = [Read]);

#[tokio::test]
async fn registry_erases_custom_errors_only_at_invocation_and_preserves_their_source()
-> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let executor = test_executor(scope)?;
            let direct = executor.execute(CustomErrorRequest).await.unwrap_err();
            let CustomRequestFailure::Partial { completed, source } = direct else {
                panic!("direct execution must retain the custom partial-result error");
            };
            assert_eq!(completed.get(), 3);
            assert_eq!(source.kind(), ErrorKind::ConnectionReset);

            let FunctionInvocation::Pending(invocation) =
                CustomErrorRequest::REGISTERED_EXECUTABLE_REQUEST
                    .invoke_value_boxed(Box::new(CustomErrorRequest), &executor)?
            else {
                panic!("executable requests must produce pending invocations");
            };
            let report = invocation.await.unwrap_err();
            assert_eq!(report.to_string(), "completed 3 synthetic items");
            assert_eq!(
                report.downcast_ref::<io::Error>().unwrap().kind(),
                ErrorKind::ConnectionReset,
            );
            Ok(())
        })
        .await
}

#[derive(Facet)]
struct PendingRequest;

impl ExecutableRequest for PendingRequest {
    type Error = eyre::Report;
    type Output = ();

    fn run<'a>(self, _executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, Self::Output>
    where
        Self: 'a,
        Self::Output: 'a,
    {
        Box::pin(pending())
    }
}

register_executable_request!(PendingRequest => (), effects = [Read]);

#[derive(Facet)]
struct PendingLegacyRequest;

impl IntoFuture for PendingLegacyRequest {
    type Output = eyre::Result<()>;
    type IntoFuture = std::future::Pending<Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        pending()
    }
}

register_into_future!(PendingLegacyRequest => (), effects = [Read]);

#[tokio::test]
async fn pending_new_and_legacy_invocations_obey_the_supplied_executor_scope() -> eyre::Result<()> {
    for (function, input) in [
        (
            PendingRequest::REGISTERED_EXECUTABLE_REQUEST,
            Box::new(PendingRequest) as Box<dyn Any + Send>,
        ),
        (
            PendingLegacyRequest::REGISTERED_INTO_FUTURE,
            Box::new(PendingLegacyRequest) as Box<dyn Any + Send>,
        ),
    ] {
        CancellationToken::new()
            .with_async_scope(|scope| async move {
                let executor = test_executor(scope)?;
                let FunctionInvocation::Pending(mut invocation) =
                    function.invoke_value_boxed(input, &executor)?
                else {
                    panic!("asynchronous registrations must produce pending invocations");
                };
                poll_fn(|context| {
                    assert!(invocation.as_mut().poll(context).is_pending());
                    Poll::Ready(())
                })
                .await;
                executor
                    .cancellation()
                    .request_cancel("synthetic registry owner cancelled");
                let error = tokio::time::timeout(Duration::from_secs(5), invocation)
                    .await?
                    .unwrap_err();
                assert_eq!(
                    error.downcast_ref::<CancellationError>().unwrap().reason,
                    "synthetic registry owner cancelled"
                );
                Ok::<_, eyre::Report>(())
            })
            .await?;
    }
    Ok(())
}
