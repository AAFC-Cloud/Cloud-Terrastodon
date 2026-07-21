use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use facet::Def;
use facet::Facet;
use facet::PtrConst;
use facet::PtrMut;
use facet::Shape;
use facet::Type;
use facet::UserType;
use facet_reflect::Partial;
use facet_reflect::Peek;
pub use linkme::distributed_slice;
use std::alloc::Layout;
use std::any::Any;
use std::any::type_name;
use std::collections::BTreeMap;
use std::future::Future;
use std::future::IntoFuture;
use std::pin::Pin;
use std::ptr::NonNull;

pub type InvocationFuture = Pin<Box<dyn Future<Output = eyre::Result<Box<dyn Any + Send>>> + Send>>;
pub type AsyncValueFn = fn(Box<dyn Any + Send>) -> InvocationFuture;
pub type SyncValueFn = fn(Box<dyn Any + Send>) -> eyre::Result<Box<dyn Any + Send>>;
pub type SyncRefFn = fn(&(dyn Any + Send)) -> eyre::Result<Box<dyn Any + Send>>;
pub type SyncMutFn = fn(&mut (dyn Any + Send)) -> eyre::Result<Box<dyn Any + Send>>;
pub type RuntimeFromBoxedFn = fn(Box<dyn Any + Send>) -> eyre::Result<RuntimeValue>;
pub type RuntimeToBoxedFn = fn(RuntimeValue) -> eyre::Result<Box<dyn Any + Send>>;

pub type BorrowPointerFn =
    for<'mem, 'facet> fn(&'static Shape, Peek<'mem, 'facet>) -> eyre::Result<RuntimeValue>;
pub type PromotePointerFn = fn(RuntimeValue) -> eyre::Result<RuntimeValue>;

#[derive(Clone, Copy)]
pub struct BorrowedPointerKind {
    pub known: facet::KnownPointer,
    pub borrow: BorrowPointerFn,
    pub promote_to_owned: PromotePointerFn,
}

impl BorrowedPointerKind {
    pub const fn new(
        known: facet::KnownPointer,
        borrow: BorrowPointerFn,
        promote_to_owned: PromotePointerFn,
    ) -> Self {
        Self {
            known,
            borrow,
            promote_to_owned,
        }
    }
}

/// An owning, type-erased Facet value.
pub struct RuntimeValue {
    shape: &'static Shape,
    ptr: NonNull<u8>,
    layout: Layout,
}

impl RuntimeValue {
    fn layout_for(shape: &'static Shape) -> eyre::Result<Layout> {
        shape
            .layout
            .sized_layout()
            .map_err(|_| eyre::eyre!("{} is not a sized runtime value", describe_shape(shape)))
    }

    fn allocate(shape: &'static Shape) -> eyre::Result<(facet::PtrUninit, Layout)> {
        let layout = Self::layout_for(shape)?;
        Ok((facet::alloc_for_layout(layout), layout))
    }

    unsafe fn from_initialized_ptr(shape: &'static Shape, ptr: PtrMut, layout: Layout) -> Self {
        Self {
            shape,
            ptr: unsafe { NonNull::new_unchecked(ptr.as_mut_byte_ptr()) },
            layout,
        }
    }

    fn from_partial_result(
        shape: &'static Shape,
        ptr: facet::PtrUninit,
        layout: Layout,
        result: eyre::Result<()>,
    ) -> eyre::Result<Self> {
        match result {
            Ok(()) => Ok(unsafe { Self::from_initialized_ptr(shape, ptr.assume_init(), layout) }),
            Err(error) => {
                unsafe { facet::dealloc_for_layout(ptr.assume_init(), layout) };
                Err(error)
            }
        }
    }

    /// Build an owned runtime value by mutating a reflected `Partial`.
    ///
    /// This is the bridge used by the object browser's incomplete builder: the
    /// browser chooses fields and variants, while Facet performs the actual
    /// layout, default, proxy, and invariant handling.
    pub fn build_with(
        shape: &'static Shape,
        build: impl FnOnce(Partial<'static, false>) -> eyre::Result<Partial<'static, false>>,
    ) -> eyre::Result<Self> {
        let (ptr, layout) = Self::allocate(shape)?;
        let result = unsafe { Partial::<'static, false>::from_raw_with_shape(ptr, shape) }
            .map_err(|error| eyre::eyre!("could not allocate {}: {error}", describe_shape(shape)))
            .and_then(build)
            .and_then(|partial| {
                partial.finish_in_place().map_err(|error| {
                    eyre::eyre!("could not finish {}: {error}", describe_shape(shape))
                })
            });
        Self::from_partial_result(shape, ptr, layout, result)
    }

    /// Construct a value by invoking Facet's actual `Default` operation.
    pub fn from_default(shape: &'static Shape) -> eyre::Result<Self> {
        let (ptr, layout) = Self::allocate(shape)?;
        let result = unsafe { Partial::<'static, false>::from_raw_with_shape(ptr, shape) }
            .map_err(|error| eyre::eyre!("could not allocate {}: {error}", describe_shape(shape)))
            .and_then(|partial| {
                partial.set_default().map_err(|error| {
                    eyre::eyre!("could not default {}: {error}", describe_shape(shape))
                })
            })
            .and_then(|partial| {
                partial.finish_in_place().map_err(|error| {
                    eyre::eyre!(
                        "could not finish default {}: {error}",
                        describe_shape(shape)
                    )
                })
            });
        Self::from_partial_result(shape, ptr, layout, result)
    }

    /// Parse a scalar/general value through the reflected Facet parse operation.
    pub fn from_text(shape: &'static Shape, text: &str) -> eyre::Result<Self> {
        let (ptr, layout) = Self::allocate(shape)?;
        let text = text.to_owned();
        let result = unsafe { Partial::<'static, false>::from_raw_with_shape(ptr, shape) }
            .map_err(|error| eyre::eyre!("could not allocate {}: {error}", describe_shape(shape)))
            .and_then(|partial| {
                partial.parse_from_str(&text).map_err(|error| {
                    eyre::eyre!("could not parse {}: {error}", describe_shape(shape))
                })
            })
            .and_then(|partial| {
                partial.finish_in_place().map_err(|error| {
                    eyre::eyre!("could not finish {}: {error}", describe_shape(shape))
                })
            });
        Self::from_partial_result(shape, ptr, layout, result)
    }

    /// Take ownership of a typed boxed value without serializing it.
    pub fn from_box<T>(value: Box<T>) -> eyre::Result<Self>
    where
        T: Facet<'static> + Any + Send + 'static,
    {
        let shape = T::SHAPE;
        let layout = Self::layout_for(shape)?;
        let ptr = Box::into_raw(value);
        Ok(unsafe { Self::from_initialized_ptr(shape, PtrMut::new(ptr), layout) })
    }

    /// Construct a reflected borrowable pointer from an already-owned value.
    ///
    /// The pointer-kind adapter supplies the type-specific operation through
    /// Facet's reflected pointer metadata. The caller must retain the source
    /// value for as long as the returned pointer may be used.
    pub fn from_borrowed_pointer(
        pointer_shape: &'static Shape,
        source: Peek<'_, '_>,
    ) -> eyre::Result<Self> {
        let kind = borrowed_pointer_kind_for_shape(pointer_shape).ok_or_else(|| {
            eyre::eyre!(
                "{} is not a registered borrowable pointer kind",
                describe_shape(pointer_shape)
            )
        })?;
        (kind.borrow)(pointer_shape, source)
    }

    pub const fn shape(&self) -> &'static Shape {
        self.shape
    }

    /// Promote a borrowable pointer to its owned representation using its
    /// reflected pointer operation.
    pub fn promote_to_owned(self) -> eyre::Result<Self> {
        let kind = borrowed_pointer_kind_for_shape(self.shape).ok_or_else(|| {
            eyre::eyre!(
                "{} is not a registered borrowable pointer kind",
                describe_shape(self.shape)
            )
        })?;
        (kind.promote_to_owned)(self)
    }

    pub fn peek(&self) -> Peek<'_, 'static> {
        unsafe { Peek::unchecked_new(PtrConst::new(self.ptr.as_ptr()), self.shape) }
    }

    /// Release the allocation after Facet has moved the value bytes elsewhere.
    ///
    /// `Partial::set_from_peek` transfers ownership of the initialized value,
    /// so the source must not run its drop operation. Its allocation still
    /// belongs to this wrapper and must be released separately.
    pub fn deallocate_after_move(self) {
        let this = std::mem::ManuallyDrop::new(self);
        unsafe {
            facet::dealloc_for_layout(PtrMut::new(this.ptr.as_ptr()), this.layout);
        }
    }

    /// Clone through the reflected Facet clone operation.
    pub fn try_clone(&self) -> eyre::Result<Self> {
        if !self
            .shape
            .type_ops
            .as_ref()
            .is_some_and(|ops| ops.has_clone_into())
        {
            eyre::bail!(
                "{} does not expose a Facet clone operation",
                describe_shape(self.shape)
            );
        }
        let (dst, layout) = Self::allocate(self.shape)?;
        let cloned = unsafe {
            self.shape
                .call_clone_into(PtrConst::new(self.ptr.as_ptr()), dst.assume_init())
                .is_some()
        };
        if !cloned {
            unsafe { facet::dealloc_for_layout(dst.assume_init(), layout) };
            eyre::bail!(
                "{} does not expose a usable Facet clone operation",
                describe_shape(self.shape)
            );
        }
        Ok(unsafe { Self::from_initialized_ptr(self.shape, dst.assume_init(), layout) })
    }

    /// Clone an inspected child value into an owning runtime value.
    pub fn clone_from_peek(peek: Peek<'_, '_>) -> eyre::Result<Self> {
        let shape = peek.shape();
        if !shape
            .type_ops
            .as_ref()
            .is_some_and(|ops| ops.has_clone_into())
        {
            eyre::bail!(
                "{} does not expose a Facet clone operation",
                describe_shape(shape)
            );
        }
        let (dst, layout) = Self::allocate(shape)?;
        let cloned = unsafe {
            shape
                .call_clone_into(peek.data(), dst.assume_init())
                .is_some()
        };
        if !cloned {
            unsafe { facet::dealloc_for_layout(dst.assume_init(), layout) };
            eyre::bail!(
                "{} does not expose a usable Facet clone operation",
                describe_shape(shape)
            );
        }
        Ok(unsafe { Self::from_initialized_ptr(shape, dst.assume_init(), layout) })
    }

    pub fn into_box<T>(self) -> eyre::Result<Box<dyn Any + Send>>
    where
        T: Facet<'static> + Any + Send + 'static,
    {
        if !self.shape.is_shape(T::SHAPE) {
            eyre::bail!(
                "runtime value has shape {}, expected {}",
                describe_shape(self.shape),
                describe_shape(T::SHAPE)
            );
        }
        let this = std::mem::ManuallyDrop::new(self);
        let ptr = this.ptr.as_ptr() as *mut T;
        Ok(unsafe { Box::from_raw(ptr) as Box<dyn Any + Send> })
    }

    pub fn display_string(&self) -> String {
        if let Some(proxy) = self.shape.effective_proxy(None) {
            let peek = self.peek();
            if let Ok(owned) = peek.custom_serialization_with_proxy(proxy) {
                let proxied = owned.as_peek();
                if let Some(text) = proxied.as_str() {
                    return text.to_owned();
                }
                return proxied.to_string();
            }
        }
        if let Ok(enum_value) = self.peek().into_enum()
            && let Ok(variant) = enum_value.active_variant()
        {
            return variant.effective_name().to_owned();
        }
        struct DisplayValue<'a>(&'a RuntimeValue);
        impl core::fmt::Display for DisplayValue<'_> {
            fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                unsafe {
                    self.0
                        .shape
                        .call_display(PtrConst::new(self.0.ptr.as_ptr()), formatter)
                }
                .unwrap_or_else(|| write!(formatter, "⟨{}⟩", describe_shape(self.0.shape)))
            }
        }
        DisplayValue(self).to_string()
    }
}

#[doc(hidden)]
pub fn borrow_pointer_runtime(
    pointer_shape: &'static Shape,
    source: Peek<'_, '_>,
) -> eyre::Result<RuntimeValue> {
    let Def::Pointer(pointer) = pointer_shape.def else {
        eyre::bail!("{} is not a pointer shape", describe_shape(pointer_shape));
    };
    let Some(pointee) = pointer.pointee() else {
        eyre::bail!(
            "{} has no reflected pointee shape",
            describe_shape(pointer_shape)
        );
    };
    if !pointee.is_shape(source.shape()) {
        eyre::bail!(
            "pointer pointee shape mismatch: expected {}, got {}",
            describe_shape(pointee),
            describe_shape(source.shape())
        );
    }
    let borrow = pointer.vtable.borrow_from_pointee_fn.ok_or_else(|| {
        eyre::eyre!(
            "{} does not expose a reflected borrow-from-pointee operation",
            describe_shape(pointer_shape)
        )
    })?;
    let (dst, layout) = RuntimeValue::allocate(pointer_shape)?;
    let ptr = unsafe { borrow(dst, source.data()) };
    Ok(unsafe { RuntimeValue::from_initialized_ptr(pointer_shape, ptr, layout) })
}

#[doc(hidden)]
pub fn promote_pointer_runtime(value: RuntimeValue) -> eyre::Result<RuntimeValue> {
    let shape = value.shape;
    let Def::Pointer(pointer) = shape.def else {
        eyre::bail!("{} is not a pointer shape", describe_shape(shape));
    };
    let promote = pointer.vtable.promote_to_owned_fn.ok_or_else(|| {
        eyre::eyre!(
            "{} does not expose a reflected promote-to-owned operation",
            describe_shape(shape)
        )
    })?;
    let (dst, layout) = RuntimeValue::allocate(shape)?;
    let source = std::mem::ManuallyDrop::new(value);
    let ptr = unsafe { promote(PtrConst::new(source.ptr.as_ptr()), dst.assume_init()) };
    unsafe {
        facet::dealloc_for_layout(PtrMut::new(source.ptr.as_ptr()), source.layout);
    }
    Ok(unsafe { RuntimeValue::from_initialized_ptr(shape, ptr, layout) })
}

impl Drop for RuntimeValue {
    fn drop(&mut self) {
        unsafe {
            let _ = self
                .shape
                .call_drop_in_place(PtrMut::new(self.ptr.as_ptr()));
            facet::dealloc_for_layout(PtrMut::new(self.ptr.as_ptr()), self.layout);
        }
    }
}

impl core::fmt::Debug for RuntimeValue {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("RuntimeValue")
            .field("shape", &describe_shape(self.shape))
            .field("value", &self.display_string())
            .finish()
    }
}

pub fn runtime_from_boxed<T>(value: Box<dyn Any + Send>) -> eyre::Result<RuntimeValue>
where
    T: Facet<'static> + Any + Send + 'static,
{
    let value = value.downcast::<T>().map_err(|_| {
        eyre::eyre!(
            "runtime conversion expected value type {}, but received a different erased value",
            type_name::<T>()
        )
    })?;
    RuntimeValue::from_box(value)
}

pub fn runtime_into_boxed<T>(value: RuntimeValue) -> eyre::Result<Box<dyn Any + Send>>
where
    T: Facet<'static> + Any + Send + 'static,
{
    value.into_box::<T>()
}

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
    pub from_runtime: RuntimeFromBoxedFn,
    pub to_runtime: RuntimeToBoxedFn,
    pub registration_site: RegistrationSite,
}

impl Thing {
    pub const fn value(
        shape: &'static Shape,
        from_runtime: RuntimeFromBoxedFn,
        to_runtime: RuntimeToBoxedFn,
        registration_site: RegistrationSite,
    ) -> Self {
        Self {
            shape,
            from_runtime,
            to_runtime,
            registration_site,
        }
    }

    pub fn input_dependencies(&self) -> Vec<InputDependency> {
        input_dependencies(self.shape)
    }

    pub fn runtime_from_boxed(&self, value: Box<dyn Any + Send>) -> eyre::Result<RuntimeValue> {
        (self.from_runtime)(value)
    }

    pub fn runtime_into_boxed(&self, value: RuntimeValue) -> eyre::Result<Box<dyn Any + Send>> {
        (self.to_runtime)(value)
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
    pub output_to_runtime: RuntimeFromBoxedFn,
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
        output_to_runtime: RuntimeFromBoxedFn,
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
            output_to_runtime,
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
        output_to_runtime: RuntimeFromBoxedFn,
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
            output_to_runtime,
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
        output_to_runtime: RuntimeFromBoxedFn,
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
            output_to_runtime,
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
        output_to_runtime: RuntimeFromBoxedFn,
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
            output_to_runtime,
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
                $crate::runtime_from_boxed::<$thing_ty>,
                $crate::runtime_into_boxed::<$thing_ty>,
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
                $crate::runtime_from_boxed::<$output_ty>,
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
                $crate::runtime_from_boxed::<$output_ty>,
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
                $crate::runtime_from_boxed::<$output_ty>,
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
                $crate::runtime_from_boxed::<$output_ty>,
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
                $crate::runtime_from_boxed::<$output_ty>,
                $crate::RegistrationSite::new(file!(), line!()),
            );
        };
    };
}

#[macro_export]
macro_rules! register_borrowed_pointer_kind {
    (Cow) => {
        const _: () = {
            #[$crate::distributed_slice($crate::KNOWN_BORROWED_POINTER_KINDS)]
            static REGISTERED_BORROWED_POINTER_KIND: $crate::BorrowedPointerKind =
                $crate::BorrowedPointerKind::new(
                    facet::KnownPointer::Cow,
                    $crate::borrow_pointer_runtime,
                    $crate::promote_pointer_runtime,
                );
        };
    };
    ($kind:ident) => {
        compile_error!(concat!(
            "no borrowed pointer adapter is defined for ",
            stringify!($kind)
        ));
    };
}

#[distributed_slice]
pub static KNOWN_THINGS: [Thing];
#[distributed_slice]
pub static KNOWN_FUNCTIONS: [Function];
#[distributed_slice]
pub static KNOWN_BORROWED_POINTER_KINDS: [BorrowedPointerKind];

register_borrowed_pointer_kind!(Cow);

#[distributed_slice(KNOWN_THINGS)]
static REGISTERED_ARBITRARY_BYTES: Thing = Thing::value(
    ArbitraryBytes::SHAPE,
    runtime_from_boxed::<ArbitraryBytes>,
    runtime_into_boxed::<ArbitraryBytes>,
    RegistrationSite::new(file!(), line!()),
);

pub fn known_thing_for_shape(shape: &'static Shape) -> Option<&'static Thing> {
    KNOWN_THINGS
        .iter()
        .find(|thing| thing.shape.is_shape(shape))
}

pub fn borrowed_pointer_kind_for_shape(
    shape: &'static Shape,
) -> Option<&'static BorrowedPointerKind> {
    let Def::Pointer(pointer) = shape.def else {
        return None;
    };
    KNOWN_BORROWED_POINTER_KINDS
        .iter()
        .find(|kind| pointer.known == Some(kind.known))
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
    use std::borrow::Cow;
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

    #[derive(Debug, Facet)]
    #[repr(C)]
    struct DummyNotCloneable {
        value: String,
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
    fn one_cow_pointer_kind_registration_supports_multiple_pointee_shapes() {
        assert_eq!(
            KNOWN_BORROWED_POINTER_KINDS
                .iter()
                .filter(|kind| kind.known == facet::KnownPointer::Cow)
                .count(),
            1
        );

        let pointee: &'static DummyOutput = Box::leak(Box::new(DummyOutput {
            value: "struct".to_string(),
        }));
        let source = RuntimeValue::from_box(Box::new(Cow::Borrowed(pointee)))
            .expect("Cow<DummyOutput> should be a reflected runtime value");
        let source_inner = source
            .peek()
            .into_pointer()
            .expect("Cow should reflect as a pointer")
            .borrow_inner()
            .expect("Cow should expose its pointee");
        let borrowed =
            RuntimeValue::from_borrowed_pointer(<Cow<'static, DummyOutput>>::SHAPE, source_inner)
                .expect("the registered Cow adapter should construct a borrow");
        let promoted = borrowed
            .promote_to_owned()
            .expect("the registered Cow adapter should promote to owned");
        let promoted = promoted
            .into_box::<Cow<'static, DummyOutput>>()
            .expect("promoted Cow should retain its reflected shape")
            .downcast::<Cow<'static, DummyOutput>>()
            .expect("promoted Cow should retain its concrete type");
        assert!(matches!(*promoted, Cow::Owned(_)));

        let source_text: Cow<'static, str> = Cow::Borrowed("text");
        let source_text = RuntimeValue::from_box(Box::new(source_text))
            .expect("Cow<str> should be a reflected runtime value");
        let source_text_inner = source_text
            .peek()
            .into_pointer()
            .expect("Cow<str> should reflect as a pointer")
            .borrow_inner()
            .expect("Cow<str> should expose its str pointee");
        let promoted_text =
            RuntimeValue::from_borrowed_pointer(<Cow<'static, str>>::SHAPE, source_text_inner)
                .expect("the same Cow adapter should support Cow<str>")
                .promote_to_owned()
                .expect("Cow<str> should promote through Cow::into_owned");
        let promoted_text = promoted_text
            .into_box::<Cow<'static, str>>()
            .expect("promoted Cow<str> should retain its reflected shape")
            .downcast::<Cow<'static, str>>()
            .expect("promoted Cow<str> should retain its concrete type");
        assert!(matches!(*promoted_text, Cow::Owned(_)));
        assert_eq!(promoted_text.as_ref(), "text");
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
