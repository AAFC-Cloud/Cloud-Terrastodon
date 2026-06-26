use std::any::Any;
use std::any::type_name;
use std::future::Future;
use std::future::IntoFuture;
use std::pin::Pin;

use facet::Def;
use facet::Shape;
use facet::Type;
use facet::UserType;
pub use linkme::distributed_slice;

pub type InvocationFuture = Pin<Box<dyn Future<Output = eyre::Result<Box<dyn Any + Send>>> + Send>>;
pub type InvokeFn = fn(Box<dyn Any + Send>) -> InvocationFuture;

#[derive(Clone, Copy)]
pub struct Thing {
    pub shape: &'static Shape,
    pub invocation: Option<Invocation>,
}

impl Thing {
    pub const fn value(shape: &'static Shape) -> Self {
        Self {
            shape,
            invocation: None,
        }
    }

    pub const fn invokable(shape: &'static Shape, invocation: Invocation) -> Self {
        Self {
            shape,
            invocation: Some(invocation),
        }
    }

    pub fn is_invokable(&self) -> bool {
        self.invocation.is_some()
    }

    pub fn output_shape(&self) -> Option<&'static Shape> {
        self.invocation.map(|invocation| invocation.output_shape)
    }

    pub fn production_kind(&self, requested_shape: &'static Shape) -> Option<ProductionKind> {
        let output_shape = self.output_shape()?;
        if output_shape.is_shape(requested_shape) {
            return Some(ProductionKind::Exact);
        }

        if list_element_shape(output_shape).is_some_and(|item| item.is_shape(requested_shape)) {
            return Some(ProductionKind::ListElement);
        }

        None
    }

    pub fn input_dependencies(&self) -> Vec<InputDependency> {
        input_dependencies(self.shape)
    }

    pub fn invoke_boxed(&self, input: Box<dyn Any + Send>) -> eyre::Result<InvocationFuture> {
        let Some(invocation) = self.invocation else {
            eyre::bail!("{} is not invokable", describe_shape(self.shape));
        };
        Ok((invocation.invoke)(input))
    }
}

impl core::fmt::Display for Thing {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", describe_shape(self.shape))
    }
}

#[derive(Clone, Copy)]
pub struct Invocation {
    pub output_shape: &'static Shape,
    pub invoke: InvokeFn,
}

impl Invocation {
    pub const fn new(output_shape: &'static Shape, invoke: InvokeFn) -> Self {
        Self {
            output_shape,
            invoke,
        }
    }
}

pub const fn invocation_for_result_future<T, O>(output_shape: &'static Shape) -> Invocation
where
    T: IntoFuture<Output = eyre::Result<O>> + Any + Send + 'static,
    T::IntoFuture: Send + 'static,
    O: Any + Send + 'static,
{
    Invocation::new(output_shape, invoke_result_future::<T, O>)
}

pub fn invoke_result_future<T, O>(input: Box<dyn Any + Send>) -> InvocationFuture
where
    T: IntoFuture<Output = eyre::Result<O>> + Any + Send + 'static,
    T::IntoFuture: Send + 'static,
    O: Any + Send + 'static,
{
    Box::pin(async move {
        let request = input.downcast::<T>().map_err(|_| {
            eyre::eyre!(
                "invocation expected input type {}, but received a different boxed value",
                type_name::<T>()
            )
        })?;
        let output = (*request).into_future().await?;
        Ok(Box::new(output) as Box<dyn Any + Send>)
    })
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ProductionKind {
    Exact,
    ListElement,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct InputDependency {
    pub field_name: &'static str,
    pub shape: &'static Shape,
}

#[macro_export]
macro_rules! register_thing {
    ($thing_ty:ty) => {
        const _: () = {
            #[linkme::distributed_slice($crate::KNOWN_THINGS)]
            static REGISTERED_THING: $crate::Thing =
                $crate::Thing::value(<$thing_ty as facet::Facet<'static>>::SHAPE);
        };
    };
    ($thing_ty:ty => $output_ty:ty) => {
        const _: () = {
            #[linkme::distributed_slice($crate::KNOWN_THINGS)]
            static REGISTERED_THING: $crate::Thing = $crate::Thing::invokable(
                <$thing_ty as facet::Facet<'static>>::SHAPE,
                $crate::invocation_for_result_future::<$thing_ty, $output_ty>(
                    <$output_ty as facet::Facet<'static>>::SHAPE,
                ),
            );
        };
    };
}
#[distributed_slice]
pub static KNOWN_THINGS: [Thing];

pub fn known_thing_for_shape(shape: &'static Shape) -> Option<&'static Thing> {
    KNOWN_THINGS
        .iter()
        .find(|thing| thing.shape.is_shape(shape))
}

pub fn things_producing(shape: &'static Shape) -> Vec<&'static Thing> {
    KNOWN_THINGS
        .iter()
        .filter(|thing| thing.production_kind(shape).is_some())
        .collect()
}

pub fn input_dependencies(shape: &'static Shape) -> Vec<InputDependency> {
    match shape.ty {
        Type::User(UserType::Struct(struct_type)) => struct_type
            .fields
            .iter()
            .filter(|field| !field.should_skip_deserializing())
            .map(|field| InputDependency {
                field_name: field.name,
                shape: field.shape(),
            })
            .collect(),
        _ => Vec::new(),
    }
}

pub fn list_element_shape(shape: &'static Shape) -> Option<&'static Shape> {
    match shape.def {
        Def::List(list) => Some(list.t()),
        _ => None,
    }
}

pub fn describe_shape(shape: &'static Shape) -> String {
    if let Some(element_shape) = list_element_shape(shape) {
        return format!("List<{}>", describe_shape(element_shape));
    }

    shape.type_identifier.to_string()
}

#[cfg(test)]
mod test {
    use facet::Facet;

    use super::KNOWN_THINGS;
    use super::ProductionKind;
    use super::Thing;
    use super::describe_shape;
    use super::distributed_slice;
    use super::invocation_for_result_future;
    use super::things_producing;

    #[derive(Debug, Clone, Copy, Facet)]
    #[repr(C)]
    struct DummyTenant;

    #[derive(Debug, Clone, Facet)]
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

    #[distributed_slice(KNOWN_THINGS)]
    static THING_DUMMY_TENANT: Thing = Thing::value(DummyTenant::SHAPE);

    #[distributed_slice(KNOWN_THINGS)]
    static THING_DUMMY_OUTPUT: Thing = Thing::value(DummyOutput::SHAPE);

    #[distributed_slice(KNOWN_THINGS)]
    static THING_DUMMY_LIST_REQUEST: Thing = Thing::invokable(
        DummyListRequest::SHAPE,
        invocation_for_result_future::<DummyListRequest, Vec<DummyOutput>>(
            <Vec<DummyOutput> as Facet<'static>>::SHAPE,
        ),
    );

    #[test]
    pub fn invokable_thing_describes_dependencies_and_output() {
        let thing = KNOWN_THINGS
            .iter()
            .find(|thing| thing.shape.is_shape(DummyListRequest::SHAPE))
            .unwrap();
        let dependencies = thing.input_dependencies();

        assert!(thing.is_invokable());
        assert_eq!(
            thing.production_kind(DummyOutput::SHAPE),
            Some(ProductionKind::ListElement)
        );
        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].field_name, "tenant");
        assert!(dependencies[0].shape.is_type::<DummyTenant>());
    }

    #[test]
    pub fn output_shape_can_be_backpropagated_to_invokable_thing() {
        let output_producers = things_producing(DummyOutput::SHAPE);
        assert_eq!(output_producers.len(), 1);
        assert!(output_producers[0].shape.is_shape(DummyListRequest::SHAPE));

        let output_dependencies = output_producers[0].input_dependencies();
        assert_eq!(output_dependencies.len(), 1);
        assert!(output_dependencies[0].shape.is_type::<DummyTenant>());
    }

    #[tokio::test]
    pub async fn invokable_thing_can_invoke_erased_request() -> eyre::Result<()> {
        let thing = KNOWN_THINGS
            .iter()
            .find(|thing| thing.shape.is_shape(DummyListRequest::SHAPE))
            .unwrap();
        let future = thing.invoke_boxed(Box::new(DummyListRequest {
            tenant: DummyTenant,
        }))?;
        let output = future.await?;
        let output = output.downcast::<Vec<DummyOutput>>().unwrap();

        assert_eq!(output.len(), 1);
        assert_eq!(output[0].value, "ok");
        Ok(())
    }

    #[test]
    pub fn shape_display_uses_shape_type_identifiers() {
        assert_eq!(
            describe_shape(DummyListRequest::SHAPE),
            DummyListRequest::SHAPE.type_identifier.to_string()
        );
        assert_eq!(
            describe_shape(<Vec<DummyOutput> as Facet<'static>>::SHAPE),
            format!("List<{}>", DummyOutput::SHAPE.type_identifier)
        );
    }
}
