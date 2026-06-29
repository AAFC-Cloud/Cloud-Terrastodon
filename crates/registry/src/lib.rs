use std::any::Any;
use std::any::type_name;
use std::collections::BTreeMap;
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
pub type ParseFn = fn(&str) -> eyre::Result<Box<dyn Any + Send>>;
pub type SerializeFn = fn(&(dyn Any + Send)) -> eyre::Result<String>;

#[derive(Clone)]
pub struct KnownShapeInfo {
    pub thing: &'static Thing,
    pub label: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ShapeFieldInfo {
    pub field_name: &'static str,
    pub field_shape_name: String,
    pub has_default: bool,
    pub default_value_label: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ShapeVariantInfo {
    pub variant_name: &'static str,
    pub payload_shape_name: Option<String>,
    pub payload_fields: Vec<ShapeFieldInfo>,
    pub is_default: bool,
}

#[derive(Clone, Copy)]
pub struct Thing {
    pub shape: &'static Shape,
    pub invocation: Option<Invocation>,
    pub parse: ParseFn,
    pub serialize: SerializeFn,
}

impl Thing {
    pub const fn value(shape: &'static Shape, parse: ParseFn, serialize: SerializeFn) -> Self {
        Self {
            shape,
            invocation: None,
            parse,
            serialize,
        }
    }

    pub const fn invokable(
        shape: &'static Shape,
        invocation: Invocation,
        parse: ParseFn,
        serialize: SerializeFn,
    ) -> Self {
        Self {
            shape,
            invocation: Some(invocation),
            parse,
            serialize,
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

    pub fn parse_boxed(&self, json: &str) -> eyre::Result<Box<dyn Any + Send>> {
        (self.parse)(json)
    }

    pub fn serialize_boxed(&self, value: &(dyn Any + Send)) -> eyre::Result<String> {
        (self.serialize)(value)
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
    pub output_serialize: SerializeFn,
}

impl Invocation {
    pub const fn new(
        output_shape: &'static Shape,
        invoke: InvokeFn,
        output_serialize: SerializeFn,
    ) -> Self {
        Self {
            output_shape,
            invoke,
            output_serialize,
        }
    }
}

pub const fn invocation_for_result_future<T, O>(output_shape: &'static Shape) -> Invocation
where
    T: IntoFuture<Output = eyre::Result<O>> + Any + Send + 'static,
    T::IntoFuture: Send + 'static,
    O: facet::Facet<'static> + Any + Send + 'static,
{
    Invocation::new(
        output_shape,
        invoke_result_future::<T, O>,
        serialize_boxed::<O>,
    )
}

pub fn parse_boxed<T>(json: &str) -> eyre::Result<Box<dyn Any + Send>>
where
    T: facet::Facet<'static> + Any + Send + 'static,
{
    Ok(Box::new(facet_json::from_str::<T>(json)?) as Box<dyn Any + Send>)
}

pub fn serialize_boxed<T>(value: &(dyn Any + Send)) -> eyre::Result<String>
where
    T: facet::Facet<'static> + Any + Send + 'static,
{
    let typed = value.downcast_ref::<T>().ok_or_else(|| {
        eyre::eyre!(
            "serialization expected value type {}, but received a different erased value",
            type_name::<T>()
        )
    })?;
    Ok(facet_json::to_string(typed)?)
}

pub fn invoke_result_future<T, O>(input: Box<dyn Any + Send>) -> InvocationFuture
where
    T: IntoFuture<Output = eyre::Result<O>> + Any + Send + 'static,
    T::IntoFuture: Send + 'static,
    O: facet::Facet<'static> + Any + Send + 'static,
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
            static REGISTERED_THING: $crate::Thing = $crate::Thing::value(
                <$thing_ty as facet::Facet<'static>>::SHAPE,
                $crate::parse_boxed::<$thing_ty>,
                $crate::serialize_boxed::<$thing_ty>,
            );
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
                $crate::parse_boxed::<$thing_ty>,
                $crate::serialize_boxed::<$thing_ty>,
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

pub fn known_shapes() -> Vec<KnownShapeInfo> {
    let mut by_label = BTreeMap::<String, &'static Thing>::new();
    for thing in KNOWN_THINGS {
        by_label.entry(describe_shape(thing.shape)).or_insert(thing);
    }

    by_label
        .into_iter()
        .map(|(label, thing)| KnownShapeInfo { thing, label })
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

pub fn shape_fields_for_thing(thing: &Thing) -> Vec<ShapeFieldInfo> {
    shape_fields(thing.shape)
}

pub fn shape_variants_for_thing(thing: &Thing) -> Vec<ShapeVariantInfo> {
    shape_variants(thing.shape)
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

    shape.type_name().to_string()
}

fn shape_fields(shape: &'static Shape) -> Vec<ShapeFieldInfo> {
    match shape.ty {
        Type::User(UserType::Struct(struct_type)) => struct_type
            .fields
            .iter()
            .filter(|field| !field.should_skip_deserializing())
            .map(shape_field_info)
            .collect(),
        _ => Vec::new(),
    }
}

fn shape_variants(shape: &'static Shape) -> Vec<ShapeVariantInfo> {
    match shape.ty {
        Type::User(UserType::Enum(enum_type)) => enum_type
            .variants
            .iter()
            .map(|variant| ShapeVariantInfo {
                variant_name: variant.effective_name(),
                payload_shape_name: variant_payload_shape_name(variant),
                payload_fields: variant.data.fields.iter().map(shape_field_info).collect(),
                is_default: variant.effective_name() == "Default",
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn shape_field_info(field: &facet::Field) -> ShapeFieldInfo {
    ShapeFieldInfo {
        field_name: field.effective_name(),
        field_shape_name: describe_shape(field.shape()),
        has_default: field.has_default(),
        default_value_label: default_value_label(field),
    }
}

fn variant_payload_shape_name(variant: &facet::Variant) -> Option<String> {
    match variant.data.fields {
        [] => None,
        [field] => Some(describe_shape(field.shape())),
        fields => Some(
            fields
                .iter()
                .map(|field| describe_shape(field.shape()))
                .collect::<Vec<_>>()
                .join(", "),
        ),
    }
}

fn default_value_label(field: &facet::Field) -> Option<String> {
    if !field.has_default() {
        return None;
    }

    let field_shape = field.shape();
    match field_shape.ty {
        Type::User(UserType::Enum(enum_type))
            if enum_type
                .variants
                .iter()
                .any(|variant| variant.effective_name() == "Default") =>
        {
            Some(format!("{}::Default", describe_shape(field_shape)))
        }
        _ => Some("<default>".to_string()),
    }
}

#[cfg(test)]
mod test {
    use facet::Facet;

    use super::KNOWN_THINGS;
    use super::ProductionKind;
    use super::ShapeFieldInfo;
    use super::ShapeVariantInfo;
    use super::Thing;
    use super::describe_shape;
    use super::distributed_slice;
    use super::invocation_for_result_future;
    use super::known_shapes;
    use super::shape_fields_for_thing;
    use super::shape_variants_for_thing;
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

    #[derive(Debug, Clone, Copy, Default, Facet)]
    #[repr(C)]
    enum DummyArgument {
        #[default]
        Default,
        Explicit,
    }

    #[derive(Debug, Clone, Facet)]
    #[repr(C)]
    struct DummyDefaultedRequest {
        #[facet(default)]
        tenant: DummyArgument,
    }

    impl IntoFuture for DummyListRequest {
        type Output = eyre::Result<Vec<DummyOutput>>;
        type IntoFuture = std::pin::Pin<Box<dyn Future<Output = Self::Output> + Send>>;

        fn into_future(self) -> Self::IntoFuture {
            Box::pin(async { Ok(vec![DummyOutput { value: "ok".into() }]) })
        }
    }

    #[distributed_slice(KNOWN_THINGS)]
    static THING_DUMMY_TENANT: Thing = Thing::value(
        DummyTenant::SHAPE,
        super::parse_boxed::<DummyTenant>,
        super::serialize_boxed::<DummyTenant>,
    );

    #[distributed_slice(KNOWN_THINGS)]
    static THING_DUMMY_TENANT_DUPLICATE: Thing = Thing::value(
        DummyTenant::SHAPE,
        super::parse_boxed::<DummyTenant>,
        super::serialize_boxed::<DummyTenant>,
    );

    #[distributed_slice(KNOWN_THINGS)]
    static THING_DUMMY_OUTPUT: Thing = Thing::value(
        DummyOutput::SHAPE,
        super::parse_boxed::<DummyOutput>,
        super::serialize_boxed::<DummyOutput>,
    );

    #[distributed_slice(KNOWN_THINGS)]
    static THING_DUMMY_ARGUMENT: Thing = Thing::value(
        DummyArgument::SHAPE,
        super::parse_boxed::<DummyArgument>,
        super::serialize_boxed::<DummyArgument>,
    );

    #[distributed_slice(KNOWN_THINGS)]
    static THING_DUMMY_LIST_REQUEST: Thing = Thing::invokable(
        DummyListRequest::SHAPE,
        invocation_for_result_future::<DummyListRequest, Vec<DummyOutput>>(
            <Vec<DummyOutput> as Facet<'static>>::SHAPE,
        ),
        super::parse_boxed::<DummyListRequest>,
        super::serialize_boxed::<DummyListRequest>,
    );

    #[distributed_slice(KNOWN_THINGS)]
    static THING_DUMMY_DEFAULTED_REQUEST: Thing = Thing::value(
        DummyDefaultedRequest::SHAPE,
        super::parse_boxed::<DummyDefaultedRequest>,
        super::serialize_boxed::<DummyDefaultedRequest>,
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
    pub fn known_shapes_deduplicates_shapes_by_label() {
        let shapes = known_shapes();
        let tenant_entries = shapes
            .iter()
            .filter(|entry| entry.label == describe_shape(DummyTenant::SHAPE))
            .count();

        assert_eq!(tenant_entries, 1);
    }

    #[test]
    pub fn shape_fields_report_default_labels() {
        let thing = KNOWN_THINGS
            .iter()
            .find(|thing| thing.shape.is_shape(DummyDefaultedRequest::SHAPE))
            .unwrap();
        let fields = shape_fields_for_thing(thing);

        assert_eq!(
            fields,
            vec![ShapeFieldInfo {
                field_name: "tenant",
                field_shape_name: describe_shape(DummyArgument::SHAPE),
                has_default: true,
                default_value_label: Some(format!(
                    "{}::Default",
                    describe_shape(DummyArgument::SHAPE)
                )),
            }]
        );
    }

    #[test]
    pub fn shape_variants_describe_enum_variants() {
        let thing = KNOWN_THINGS
            .iter()
            .find(|thing| thing.shape.is_shape(DummyArgument::SHAPE))
            .unwrap();
        let variants = shape_variants_for_thing(thing);

        assert_eq!(
            variants,
            vec![
                ShapeVariantInfo {
                    variant_name: "Default",
                    payload_shape_name: None,
                    payload_fields: Vec::new(),
                    is_default: true,
                },
                ShapeVariantInfo {
                    variant_name: "Explicit",
                    payload_shape_name: None,
                    payload_fields: Vec::new(),
                    is_default: false,
                },
            ]
        );
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

    #[test]
    pub fn shape_display_includes_option_inner_type() {
        assert_eq!(
            describe_shape(<Option<DummyOutput> as Facet<'static>>::SHAPE),
            format!("Option<{}>", DummyOutput::SHAPE.type_identifier)
        );
    }

    #[test]
    pub fn shape_display_uses_facet_type_name_for_other_generics() {
        assert_eq!(
            describe_shape(<Result<DummyOutput, DummyArgument> as Facet<'static>>::SHAPE),
            format!(
                "Result<{}, {}>",
                DummyOutput::SHAPE.type_identifier,
                DummyArgument::SHAPE.type_identifier,
            )
        );
    }
}
