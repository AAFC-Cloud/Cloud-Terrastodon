use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use facet::Def;
use facet::Facet;
use facet::Shape;
use facet::Type;
use facet::UserType;
use facet_reflect::Partial;
pub use linkme::distributed_slice;
use std::any::Any;
use std::any::type_name;
use std::collections::BTreeMap;
use std::future::Future;
use std::future::IntoFuture;
use std::pin::Pin;

pub type InvocationFuture = Pin<Box<dyn Future<Output = eyre::Result<Box<dyn Any + Send>>> + Send>>;
pub type AsyncValueFn = fn(Box<dyn Any + Send>) -> InvocationFuture;
pub type SyncValueFn = fn(Box<dyn Any + Send>) -> eyre::Result<Box<dyn Any + Send>>;
pub type SyncRefFn = fn(&(dyn Any + Send)) -> eyre::Result<Box<dyn Any + Send>>;
pub type SyncMutFn = fn(&mut (dyn Any + Send)) -> eyre::Result<Box<dyn Any + Send>>;
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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct RegistrationSite {
    pub file: &'static str,
    pub line: u32,
}

impl RegistrationSite {
    pub const fn new(file: &'static str, line: u32) -> Self {
        Self { file, line }
    }
}

impl core::fmt::Display for RegistrationSite {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}:{}", self.file, self.line)
    }
}
#[derive(Clone, Copy)]
pub struct Thing {
    pub shape: &'static Shape,
    pub parse: ParseFn,
    pub serialize: SerializeFn,
    pub registration_site: RegistrationSite,
}

impl Thing {
    pub const fn value(
        shape: &'static Shape,
        parse: ParseFn,
        serialize: SerializeFn,
        registration_site: RegistrationSite,
    ) -> Self {
        Self {
            shape,
            parse,
            serialize,
            registration_site,
        }
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
}

impl core::fmt::Display for Thing {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", describe_shape(self.shape))
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Effect {
    Read,
    Write,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ReceiverMode {
    ByValue,
    ByRef,
    ByMut,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum FunctionKind {
    AsyncInvoke,
    Conversion,
    Projection,
    Constructor,
}

#[derive(Clone, Copy)]
pub enum FunctionExecutor {
    AsyncValue(AsyncValueFn),
    SyncValue(SyncValueFn),
    SyncRef(SyncRefFn),
    SyncMut(SyncMutFn),
}

#[derive(Clone, Copy)]
pub struct Function {
    pub input_shape: &'static Shape,
    pub output_shape: &'static Shape,
    pub receiver_mode: ReceiverMode,
    pub kind: FunctionKind,
    pub label: &'static str,
    pub origin: &'static str,
    pub effects: &'static [Effect],
    pub executor: FunctionExecutor,
    pub output_serialize: SerializeFn,
    pub registration_site: RegistrationSite,
}

impl Function {
    #[expect(
        clippy::too_many_arguments,
        reason = "Registry functions are assembled from static macro metadata"
    )]
    pub const fn async_value(
        input_shape: &'static Shape,
        output_shape: &'static Shape,
        kind: FunctionKind,
        label: &'static str,
        origin: &'static str,
        effects: &'static [Effect],
        invoke: AsyncValueFn,
        output_serialize: SerializeFn,
        registration_site: RegistrationSite,
    ) -> Self {
        Self {
            input_shape,
            output_shape,
            receiver_mode: ReceiverMode::ByValue,
            kind,
            label,
            origin,
            effects,
            executor: FunctionExecutor::AsyncValue(invoke),
            output_serialize,
            registration_site,
        }
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "Registry functions are assembled from static macro metadata"
    )]
    pub const fn sync_value(
        input_shape: &'static Shape,
        output_shape: &'static Shape,
        kind: FunctionKind,
        label: &'static str,
        origin: &'static str,
        effects: &'static [Effect],
        invoke: SyncValueFn,
        output_serialize: SerializeFn,
        registration_site: RegistrationSite,
    ) -> Self {
        Self {
            input_shape,
            output_shape,
            receiver_mode: ReceiverMode::ByValue,
            kind,
            label,
            origin,
            effects,
            executor: FunctionExecutor::SyncValue(invoke),
            output_serialize,
            registration_site,
        }
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "Registry functions are assembled from static macro metadata"
    )]
    pub const fn sync_ref(
        input_shape: &'static Shape,
        output_shape: &'static Shape,
        kind: FunctionKind,
        label: &'static str,
        origin: &'static str,
        effects: &'static [Effect],
        invoke: SyncRefFn,
        output_serialize: SerializeFn,
        registration_site: RegistrationSite,
    ) -> Self {
        Self {
            input_shape,
            output_shape,
            receiver_mode: ReceiverMode::ByRef,
            kind,
            label,
            origin,
            effects,
            executor: FunctionExecutor::SyncRef(invoke),
            output_serialize,
            registration_site,
        }
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "Registry functions are assembled from static macro metadata"
    )]
    pub const fn sync_mut(
        input_shape: &'static Shape,
        output_shape: &'static Shape,
        kind: FunctionKind,
        label: &'static str,
        origin: &'static str,
        effects: &'static [Effect],
        invoke: SyncMutFn,
        output_serialize: SerializeFn,
        registration_site: RegistrationSite,
    ) -> Self {
        Self {
            input_shape,
            output_shape,
            receiver_mode: ReceiverMode::ByMut,
            kind,
            label,
            origin,
            effects,
            executor: FunctionExecutor::SyncMut(invoke),
            output_serialize,
            registration_site,
        }
    }

    pub fn production_kind(&self, requested_shape: &'static Shape) -> Option<ProductionKind> {
        if self.output_shape.is_shape(requested_shape) {
            return Some(ProductionKind::Exact);
        }
        list_element_shape(self.output_shape)
            .is_some_and(|item| item.is_shape(requested_shape))
            .then_some(ProductionKind::ListElement)
    }

    pub fn supports_slot_kind(&self, slot_is_owned: bool) -> bool {
        self.receiver_mode != ReceiverMode::ByMut || slot_is_owned
    }

    pub fn is_async(&self) -> bool {
        matches!(self.executor, FunctionExecutor::AsyncValue(_))
    }

    pub fn invoke_value_boxed(
        &self,
        input: Box<dyn Any + Send>,
    ) -> eyre::Result<FunctionInvocation> {
        match self.executor {
            FunctionExecutor::AsyncValue(invoke) => Ok(FunctionInvocation::Pending(invoke(input))),
            FunctionExecutor::SyncValue(invoke) => Ok(FunctionInvocation::Ready(invoke(input)?)),
            FunctionExecutor::SyncRef(_) => {
                eyre::bail!("{} requires ByRef", describe_function(self))
            }
            FunctionExecutor::SyncMut(_) => {
                eyre::bail!("{} requires ByMut", describe_function(self))
            }
        }
    }

    pub fn invoke_ref_boxed(&self, input: &(dyn Any + Send)) -> eyre::Result<Box<dyn Any + Send>> {
        match self.executor {
            FunctionExecutor::SyncRef(invoke) => invoke(input),
            _ => eyre::bail!("{} does not support ByRef", describe_function(self)),
        }
    }

    pub fn invoke_mut_boxed(
        &self,
        input: &mut (dyn Any + Send),
    ) -> eyre::Result<Box<dyn Any + Send>> {
        match self.executor {
            FunctionExecutor::SyncMut(invoke) => invoke(input),
            _ => eyre::bail!("{} does not support ByMut", describe_function(self)),
        }
    }
}

pub enum FunctionInvocation {
    Pending(InvocationFuture),
    Ready(Box<dyn Any + Send>),
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

#[derive(Debug, Clone, PartialEq, Eq, Default, arbitrary::Arbitrary, facet::Facet)]
#[facet(transparent)]
pub struct ArbitraryBytes(pub Vec<u8>);

impl ArbitraryBytes {
    pub fn new(bytes: impl Into<Vec<u8>>) -> Self {
        Self(bytes.into())
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    pub fn replace_remaining(&mut self, remaining: Vec<u8>) {
        self.0 = remaining;
    }
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

pub fn arbitrary_from_bytes<T>(input: &mut ArbitraryBytes) -> eyre::Result<T>
where
    T: for<'a> Arbitrary<'a> + Any + Send + 'static,
{
    let mut unstructured = Unstructured::new(input.as_slice());
    let output = T::arbitrary(&mut unstructured)?;
    input.replace_remaining(unstructured.take_rest().to_vec());
    Ok(output)
}

pub fn invoke_arbitrary_mut<T>(input: &mut (dyn Any + Send)) -> eyre::Result<Box<dyn Any + Send>>
where
    T: for<'a> Arbitrary<'a> + Any + Send + 'static,
{
    let bytes = input.downcast_mut::<ArbitraryBytes>().ok_or_else(|| {
        eyre::eyre!(
            "mutable invocation expected input type {}, but received a different erased value",
            type_name::<ArbitraryBytes>()
        )
    })?;
    Ok(Box::new(arbitrary_from_bytes::<T>(bytes)?) as Box<dyn Any + Send>)
}

#[macro_export]
macro_rules! register_thing {
    ($thing_ty:ty) => {
        const _: () = {
            #[$crate::distributed_slice($crate::KNOWN_THINGS)]
            static REGISTERED_THING: $crate::Thing = $crate::Thing::value(
                <$thing_ty as facet::Facet<'static>>::SHAPE,
                $crate::parse_boxed::<$thing_ty>,
                $crate::serialize_boxed::<$thing_ty>,
                $crate::RegistrationSite::new(file!(), line!()),
            );
        };
    };
}
#[macro_export]
macro_rules! register_arbitrary {
    ($thing_ty:ty) => {
        $crate::register_fn_mut!(
            $crate::ArbitraryBytes => $thing_ty,
            kind = $crate::FunctionKind::Constructor,
            label = "arbitrary",
            origin = "Arbitrary",
            invoke = $crate::arbitrary_from_bytes::<$thing_ty>
        );
    };
}

#[macro_export]
macro_rules! register_into_future {
    ($thing_ty:ty => $output_ty:ty) => {
        $crate::register_into_future!($thing_ty => $output_ty, effects = []);
    };
    ($thing_ty:ty => $output_ty:ty, effects = [$($effect:ident),* $(,)?]) => {
        const _: () = {
            #[$crate::distributed_slice($crate::KNOWN_FUNCTIONS)]
            static REGISTERED_FUNCTION: $crate::Function = $crate::Function::async_value(
                <$thing_ty as facet::Facet<'static>>::SHAPE,
                <$output_ty as facet::Facet<'static>>::SHAPE,
                $crate::FunctionKind::AsyncInvoke,
                "invoke",
                "IntoFuture",
                &[$($crate::Effect::$effect),*],
                $crate::invoke_result_future::<$thing_ty, $output_ty>,
                $crate::serialize_boxed::<$output_ty>,
                $crate::RegistrationSite::new(file!(), line!()),
            );
        };
    };
}

#[macro_export]
macro_rules! register_from {
    ($input_ty:ty => $output_ty:ty) => {
        $crate::register_from!($input_ty => $output_ty, effects = []);
    };
    ($input_ty:ty => $output_ty:ty, effects = [$($effect:ident),* $(,)?]) => {
        const _: () = {
            fn invoke(input: Box<dyn std::any::Any + Send>) -> eyre::Result<Box<dyn std::any::Any + Send>> {
                let input = input.downcast::<$input_ty>().map_err(|_| {
                    eyre::eyre!("conversion expected input type {}", std::any::type_name::<$input_ty>())
                })?;
                let output: $output_ty = <$output_ty as std::convert::From<$input_ty>>::from(*input);
                Ok(Box::new(output) as Box<dyn std::any::Any + Send>)
            }

            #[$crate::distributed_slice($crate::KNOWN_FUNCTIONS)]
            static REGISTERED_FUNCTION: $crate::Function = $crate::Function::sync_value(
                <$input_ty as facet::Facet<'static>>::SHAPE,
                <$output_ty as facet::Facet<'static>>::SHAPE,
                $crate::FunctionKind::Conversion,
                "from",
                "From",
                &[$($crate::Effect::$effect),*],
                invoke,
                $crate::serialize_boxed::<$output_ty>,
                $crate::RegistrationSite::new(file!(), line!()),
            );
        };
    };
}

#[macro_export]
macro_rules! register_try_from {
    ($input_ty:ty => $output_ty:ty) => {
        $crate::register_try_from!($input_ty => $output_ty, effects = []);
    };
    ($input_ty:ty => $output_ty:ty, effects = [$($effect:ident),* $(,)?]) => {
        const _: () = {
            fn invoke(input: Box<dyn std::any::Any + Send>) -> eyre::Result<Box<dyn std::any::Any + Send>> {
                let input = input.downcast::<$input_ty>().map_err(|_| {
                    eyre::eyre!("conversion expected input type {}", std::any::type_name::<$input_ty>())
                })?;
                let output: $output_ty = <$output_ty as std::convert::TryFrom<$input_ty>>::try_from(*input)?;
                Ok(Box::new(output) as Box<dyn std::any::Any + Send>)
            }

            #[$crate::distributed_slice($crate::KNOWN_FUNCTIONS)]
            static REGISTERED_FUNCTION: $crate::Function = $crate::Function::sync_value(
                <$input_ty as facet::Facet<'static>>::SHAPE,
                <$output_ty as facet::Facet<'static>>::SHAPE,
                $crate::FunctionKind::Conversion,
                "try_from",
                "TryFrom",
                &[$($crate::Effect::$effect),*],
                invoke,
                $crate::serialize_boxed::<$output_ty>,
                $crate::RegistrationSite::new(file!(), line!()),
            );
        };
    };
}
#[macro_export]
macro_rules! register_fn_ref {
    ($input_ty:ty => $output_ty:ty, kind = $kind:expr, label = $label:expr, origin = $origin:expr, invoke = $invoke:path $(,)?) => {
        $crate::register_fn_ref!($input_ty => $output_ty, kind = $kind, label = $label, origin = $origin, invoke = $invoke, effects = []);
    };
    ($input_ty:ty => $output_ty:ty, kind = $kind:expr, label = $label:expr, origin = $origin:expr, invoke = $invoke:path, effects = [$($effect:ident),* $(,)?]) => {
        const _: () = {
            fn invoke(input: &(dyn std::any::Any + Send)) -> eyre::Result<Box<dyn std::any::Any + Send>> {
                let input = input.downcast_ref::<$input_ty>().ok_or_else(|| {
                    eyre::eyre!("shared-reference invocation expected input type {}", std::any::type_name::<$input_ty>())
                })?;
                let output: $output_ty = $invoke(input)?;
                Ok(Box::new(output) as Box<dyn std::any::Any + Send>)
            }

            #[$crate::distributed_slice($crate::KNOWN_FUNCTIONS)]
            static REGISTERED_FUNCTION: $crate::Function = $crate::Function::sync_ref(
                <$input_ty as facet::Facet<'static>>::SHAPE,
                <$output_ty as facet::Facet<'static>>::SHAPE,
                $kind,
                $label,
                $origin,
                &[$($crate::Effect::$effect),*],
                invoke,
                $crate::serialize_boxed::<$output_ty>,
                $crate::RegistrationSite::new(file!(), line!()),
            );
        };
    };
}

#[macro_export]
macro_rules! register_fn_mut {
    ($input_ty:ty => $output_ty:ty, kind = $kind:expr, label = $label:expr, origin = $origin:expr, invoke = $invoke:path $(,)?) => {
        $crate::register_fn_mut!($input_ty => $output_ty, kind = $kind, label = $label, origin = $origin, invoke = $invoke, effects = []);
    };
    ($input_ty:ty => $output_ty:ty, kind = $kind:expr, label = $label:expr, origin = $origin:expr, invoke = $invoke:path, effects = [$($effect:ident),* $(,)?]) => {
        const _: () = {
            fn invoke(input: &mut (dyn std::any::Any + Send)) -> eyre::Result<Box<dyn std::any::Any + Send>> {
                let input = input.downcast_mut::<$input_ty>().ok_or_else(|| {
                    eyre::eyre!("mutable invocation expected input type {}", std::any::type_name::<$input_ty>())
                })?;
                let output: $output_ty = $invoke(input)?;
                Ok(Box::new(output) as Box<dyn std::any::Any + Send>)
            }

            #[$crate::distributed_slice($crate::KNOWN_FUNCTIONS)]
            static REGISTERED_FUNCTION: $crate::Function = $crate::Function::sync_mut(
                <$input_ty as facet::Facet<'static>>::SHAPE,
                <$output_ty as facet::Facet<'static>>::SHAPE,
                $kind,
                $label,
                $origin,
                &[$($crate::Effect::$effect),*],
                invoke,
                $crate::serialize_boxed::<$output_ty>,
                $crate::RegistrationSite::new(file!(), line!()),
            );
        };
    };
}

#[distributed_slice]
pub static KNOWN_THINGS: [Thing];
#[distributed_slice]
pub static KNOWN_FUNCTIONS: [Function];

#[distributed_slice(KNOWN_THINGS)]
static REGISTERED_ARBITRARY_BYTES: Thing = Thing::value(
    ArbitraryBytes::SHAPE,
    parse_boxed::<ArbitraryBytes>,
    serialize_boxed::<ArbitraryBytes>,
    RegistrationSite::new(file!(), line!()),
);

pub fn known_thing_for_shape(shape: &'static Shape) -> Option<&'static Thing> {
    KNOWN_THINGS
        .iter()
        .find(|thing| thing.shape.is_shape(shape))
}

pub fn known_things() -> Vec<&'static Thing> {
    KNOWN_THINGS.iter().collect()
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

pub fn known_functions() -> Vec<&'static Function> {
    KNOWN_FUNCTIONS.iter().collect()
}

pub fn functions_from(shape: &'static Shape) -> Vec<&'static Function> {
    KNOWN_FUNCTIONS
        .iter()
        .filter(|f| f.input_shape.is_shape(shape))
        .collect()
}

pub fn functions_to(shape: &'static Shape) -> Vec<&'static Function> {
    KNOWN_FUNCTIONS
        .iter()
        .filter(|f| f.production_kind(shape).is_some())
        .collect()
}

pub fn functions_from_to(
    input_shape: &'static Shape,
    output_shape: &'static Shape,
) -> Vec<&'static Function> {
    KNOWN_FUNCTIONS
        .iter()
        .filter(|f| f.input_shape.is_shape(input_shape))
        .filter(|f| f.production_kind(output_shape).is_some())
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

pub fn sequence_element_shape(shape: &'static Shape) -> Option<&'static Shape> {
    match shape.def {
        Def::List(list) => Some(list.t()),
        Def::Set(set) => Some(set.t()),
        _ => indirect_inner_shape(shape).and_then(sequence_element_shape),
    }
}

pub fn map_value_shape(shape: &'static Shape) -> Option<&'static Shape> {
    match shape.def {
        Def::Map(map) => Some(map.v()),
        _ => indirect_inner_shape(shape).and_then(map_value_shape),
    }
}

pub fn shape_field_shape(shape: &'static Shape, field_name: &str) -> Option<&'static Shape> {
    match shape.ty {
        Type::User(UserType::Struct(struct_type)) => struct_type
            .fields
            .iter()
            .find(|field| field.effective_name() == field_name)
            .map(|field| field.proxy_shape().unwrap_or_else(|| field.shape())),
        _ => indirect_inner_shape(shape).and_then(|inner| shape_field_shape(inner, field_name)),
    }
}

fn indirect_inner_shape(shape: &'static Shape) -> Option<&'static Shape> {
    transparent_inner_shape(shape).or_else(|| optional_inner_shape(shape))
}

fn transparent_inner_shape(shape: &'static Shape) -> Option<&'static Shape> {
    if !shape.is_transparent() {
        return None;
    }

    match shape.ty {
        Type::User(UserType::Struct(struct_type)) if struct_type.fields.len() == 1 => {
            Some(struct_type.fields[0].shape())
        }
        _ => None,
    }
}

fn optional_inner_shape(shape: &'static Shape) -> Option<&'static Shape> {
    match shape.def {
        Def::Option(option) => Some(option.t()),
        _ => None,
    }
}

pub fn describe_shape(shape: &'static Shape) -> String {
    if let Some(element_shape) = list_element_shape(shape) {
        return format!("List<{}>", describe_shape(element_shape));
    }
    shape.type_name().to_string()
}

pub fn describe_function(function: &Function) -> String {
    format!(
        "{} {} -> {} ({})",
        function.label,
        describe_shape(function.input_shape),
        describe_shape(function.output_shape),
        function.origin,
    )
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
        Type::User(UserType::Enum(enum_type)) => {
            let default_variant_index = default_variant_index_for_shape(shape);
            enum_type
                .variants
                .iter()
                .enumerate()
                .map(|(variant_index, variant)| ShapeVariantInfo {
                    variant_name: variant.effective_name(),
                    payload_shape_name: variant_payload_shape_name(variant),
                    payload_fields: variant.data.fields.iter().map(shape_field_info).collect(),
                    is_default: default_variant_index == Some(variant_index),
                })
                .collect()
        }
        _ => Vec::new(),
    }
}

fn default_variant_index_for_shape(shape: &'static Shape) -> Option<usize> {
    let partial = unsafe { Partial::alloc_shape_owned(shape).ok()? };
    let value = partial.set_default().ok()?.build().ok()?;
    value.peek().into_enum().ok()?.variant_index().ok()
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
    use super::*;
    use facet::Facet;
    use std::future::Future;
    use std::future::IntoFuture;

    #[derive(Debug, Clone, Copy, Facet)]
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

    crate::register_thing!(DummyTenant);
    crate::register_thing!(DummyOutput);
    crate::register_thing!(DummyListRequest);
    crate::register_thing!(DummyRenamedDefault);
    crate::register_thing!(DummyManualDefault);
    crate::register_into_future!(DummyListRequest => Vec<DummyOutput>, effects = [Read]);
    crate::register_from!(DummyOutput => DummyConverted);
    crate::register_fn_mut!(ArbitraryBytes => DummyOutput,
        kind = FunctionKind::Constructor,
        label = "arbitrary",
        origin = "Arbitrary",
        invoke = arbitrary_from_bytes::<DummyOutput>
    );

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
        let function = functions_from(DummyListRequest::SHAPE)
            .into_iter()
            .find(|f| f.kind == FunctionKind::AsyncInvoke)
            .unwrap();
        let output = match function.invoke_value_boxed(Box::new(DummyListRequest {
            tenant: DummyTenant,
        }))? {
            FunctionInvocation::Pending(future) => future.await?,
            FunctionInvocation::Ready(_) => panic!("expected pending"),
        };
        let output = output.downcast::<Vec<DummyOutput>>().unwrap();
        assert_eq!(output[0].value, "ok");
        Ok(())
    }

    #[test]
    fn by_mut_arbitrary_functions_consume_bytes() -> eyre::Result<()> {
        let function = functions_from(ArbitraryBytes::SHAPE)
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
}
