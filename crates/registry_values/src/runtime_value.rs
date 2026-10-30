use crate::borrowed_pointer_kind_for_shape;
use crate::describe_shape;
use crate::runtime_display_value::RuntimeDisplayValue;
use facet::Def;
use facet::Facet;
use facet::PtrConst;
use facet::PtrMut;
use facet::Shape;
use facet_reflect::Partial;
use facet_reflect::Peek;
use std::alloc::Layout;
use std::any::Any;
use std::any::type_name;
use std::collections::HashSet;
use std::ptr::NonNull;

pub type RuntimeFromBoxedFn = fn(Box<dyn Any + Send>) -> eyre::Result<RuntimeValue>;
pub type RuntimeToBoxedFn = fn(RuntimeValue) -> eyre::Result<Box<dyn Any + Send>>;

/// An owning, type-erased Facet allocation.
///
/// Owning the allocation does not imply owning everything reachable from its
/// contents. Values created through the unsafe borrowed-pointer bridge require
/// external lifetime/alias tracking, including for shallow clones and values
/// obtained from [`Self::into_box`].
pub struct RuntimeValue {
    shape: &'static Shape,
    ptr: NonNull<u8>,
    layout: Layout,
}

impl RuntimeValue {
    /// Shape a generic field picker should create/search when a field is a
    /// supported owning/borrowed pointer. Ordinary values return themselves.
    pub fn preferred_field_source_shape(target_shape: &'static Shape) -> &'static Shape {
        let Def::Pointer(pointer) = target_shape.def else {
            return target_shape;
        };
        let Some(pointee) = pointer.pointee() else {
            return target_shape;
        };
        if Self::can_own_pointee(target_shape, pointee)
            || Self::can_borrow_pointee(target_shape, pointee)
        {
            pointee
        } else {
            target_shape
        }
    }

    pub fn can_borrow_pointee(
        pointer_shape: &'static Shape,
        pointee_shape: &'static Shape,
    ) -> bool {
        let Def::Pointer(pointer) = pointer_shape.def else {
            return false;
        };
        pointer
            .pointee()
            .is_some_and(|expected| expected.is_shape(pointee_shape))
            && borrowed_pointer_kind_for_shape(pointer_shape).is_some()
    }

    pub fn can_own_pointee(pointer_shape: &'static Shape, pointee_shape: &'static Shape) -> bool {
        let Def::Pointer(pointer) = pointer_shape.def else {
            return false;
        };
        if !pointer
            .pointee()
            .is_some_and(|expected| expected.is_shape(pointee_shape))
        {
            return false;
        }
        if pointer.known == Some(facet::KnownPointer::Cow) {
            return pointer.vtable.borrow_from_pointee_fn.is_some()
                && pointer.vtable.promote_to_owned_fn.is_some();
        }
        pointer.vtable.new_into_fn.is_some()
    }

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
    /// Facet's reflected pointer metadata. The returned value erases the source
    /// lifetime; it does not own or retain a lease on the pointee.
    ///
    /// # Safety
    ///
    /// The source allocation and referenced data must remain live and at a stable
    /// address, with shared-reference aliasing rules respected, for every use of
    /// the returned borrow. This includes shallow clones, nested/moved values,
    /// typed boxes, and references extracted from them. The caller must prevent
    /// such values/references from escaping the lifetime it can guarantee.
    ///
    /// Dropping or promoting one pointer does not detach its borrowed clones.
    /// Promotion detaches the direct pointee borrow only: references retained by
    /// the pointee's `ToOwned` implementation still need their original owners.
    /// An Object Explorer caller must establish these requirements using active
    /// source leases and release them only after all dependent borrows die,
    /// including during task cancellation and engine shutdown.
    ///
    /// Safe code must not be able to erase a stack-owned source's lifetime:
    ///
    /// ```compile_fail,E0133
    /// use cloud_terrastodon_registry_values::RuntimeValue;
    /// use facet::Facet;
    /// use std::borrow::Cow;
    /// let source = RuntimeValue::from_box(Box::new(String::from("source"))).unwrap();
    /// let borrowed = RuntimeValue::from_borrowed_pointer(
    ///     <Cow<'static, String>>::SHAPE, source.peek());
    /// ```
    pub unsafe fn from_borrowed_pointer(
        pointer_shape: &'static Shape,
        source: Peek<'_, '_>,
    ) -> eyre::Result<Self> {
        let kind = borrowed_pointer_kind_for_shape(pointer_shape).ok_or_else(|| {
            eyre::eyre!(
                "{} is not a registered borrowable pointer kind",
                describe_shape(pointer_shape)
            )
        })?;
        // SAFETY: this function's caller supplies the lifetime/alias proof, which
        // is exactly the registered adapter's unsafe call contract.
        unsafe { (kind.borrow)(pointer_shape, source) }
    }

    /// Wrap an owned pointee in a reflected owning pointer such as `Cow::Owned`.
    ///
    /// Facet's `new_into_fn` moves the pointee bytes into ordinary owning
    /// pointers. `Cow` instead uses its explicit borrow/promote capabilities;
    /// its owned constructor is not advertised through `new_into_fn`. The
    /// temporary source stays alive until in-place promotion completes, then
    /// is consumed. This operation does not recursively own nested references.
    pub fn from_owned_pointee(
        pointer_shape: &'static Shape,
        source: RuntimeValue,
    ) -> eyre::Result<Self> {
        let Def::Pointer(pointer) = pointer_shape.def else {
            eyre::bail!("{} is not a pointer shape", describe_shape(pointer_shape));
        };
        let pointee = pointer.pointee().ok_or_else(|| {
            eyre::eyre!(
                "{} has no reflected pointee shape",
                describe_shape(pointer_shape)
            )
        })?;
        if !pointee.is_shape(source.shape) {
            eyre::bail!(
                "pointer pointee shape mismatch: expected {}, got {}",
                describe_shape(pointee),
                describe_shape(source.shape)
            );
        }
        if pointer.known == Some(facet::KnownPointer::Cow) {
            // SAFETY: source is owned by this call and remains live and unchanged
            // until the temporary borrow is promoted or dropped on an error or
            // unwind. No borrowed pointer/reference escapes this scope. Nested
            // references already satisfy source's RuntimeValue lifetime contract.
            let borrowed = unsafe { Self::from_borrowed_pointer(pointer_shape, source.peek())? };
            let owned = borrowed.promote_to_owned()?;
            drop(source);
            return Ok(owned);
        }
        let construct = pointer.vtable.new_into_fn.ok_or_else(|| {
            eyre::eyre!(
                "{} cannot be constructed from an owned pointee",
                describe_shape(pointer_shape)
            )
        })?;
        let (dst, layout) = Self::allocate(pointer_shape)?;
        let source = std::mem::ManuallyDrop::new(source);
        // SAFETY: the reflected pointee shape was checked above, source owns
        // a live initialized pointee, and dst has exactly the pointer shape's
        // layout. Facet's new_into_fn transfers the pointee bytes into the
        // returned initialized pointer value.
        let ptr = unsafe { construct(dst, PtrMut::new(source.ptr.as_ptr())) };
        // SAFETY: construct moved the pointee, so its allocation must be
        // released without running the moved value's destructor.
        unsafe {
            facet::dealloc_for_layout(PtrMut::new(source.ptr.as_ptr()), source.layout);
        }
        // SAFETY: construct returned an initialized value in dst, whose shape
        // and allocation layout are retained here.
        Ok(unsafe { Self::from_initialized_ptr(pointer_shape, ptr, layout) })
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
    ///
    /// Cloning is not recursive ownership promotion. A borrowed `Cow` remains
    /// borrowed, so callers tracking external leases must retain or duplicate
    /// those leases for the clone as well as the original.
    pub fn try_clone(&self) -> eyre::Result<Self> {
        Self::clone_from_peek(self.peek())
    }

    /// Whether clone actions can be offered without executing user clone code.
    /// Lists use the same element-wise fallback as [`Self::clone_from_peek`].
    /// This describes the type contract, not whether a particular clone can fail
    /// or panic; callers must still handle invocation errors. An empty list of a
    /// non-cloneable element type does not advertise a general Clone capability.
    pub fn can_clone_shape(mut shape: &'static Shape) -> bool {
        let mut visited = HashSet::new();
        loop {
            if shape
                .type_ops
                .as_ref()
                .is_some_and(|ops| ops.has_clone_into())
            {
                return true;
            }
            let Def::List(list) = shape.def else {
                return false;
            };
            if !visited.insert(shape) {
                return false;
            }
            shape = list.t;
        }
    }

    fn clone_list_from_peek(
        peek: Peek<'_, 'static>,
        list_def: facet::ListDef,
    ) -> eyre::Result<Self> {
        let source = unsafe { facet_reflect::PeekList::new(peek, list_def) };
        let length = source.len();

        Self::build_with(peek.shape(), |partial| {
            let mut partial = partial
                .init_list_with_capacity(length)
                .map_err(|error| eyre::eyre!("could not initialize list clone: {error}"))?;

            for item in source.iter() {
                let cloned_item = Self::clone_from_peek(item)?;
                partial = partial
                    .begin_list_item()
                    .map_err(|error| eyre::eyre!("could not begin list clone item: {error}"))?;
                partial = unsafe { partial.set_from_peek(&cloned_item.peek()) }
                    .map_err(|error| eyre::eyre!("could not set list clone item: {error}"))?;
                cloned_item.deallocate_after_move();
                partial = partial
                    .end()
                    .map_err(|error| eyre::eyre!("could not finish list clone item: {error}"))?;
            }

            Ok(partial)
        })
    }

    /// Clone an inspected child value into an owning runtime allocation.
    ///
    /// The source's internal Rust references must be static (or covered by the
    /// unsafe RuntimeValue bridge's external lifetime contract). Cloning does
    /// not turn borrowed fields into owned fields or detach external leases.
    /// The inspected allocation itself needs to stay alive only for this call.
    /// Projected lists and owned roots share the same recursive cloning path.
    ///
    /// ```compile_fail
    /// use cloud_terrastodon_registry_values::RuntimeValue;
    /// use facet_reflect::Peek;
    /// use std::borrow::Cow;
    /// let source = String::from("source");
    /// let borrowed = Cow::Borrowed(source.as_str());
    /// let cloned = RuntimeValue::clone_from_peek(Peek::new(&borrowed));
    /// ```
    pub fn clone_from_peek(peek: Peek<'_, 'static>) -> eyre::Result<Self> {
        let shape = peek.shape();
        if !shape
            .type_ops
            .as_ref()
            .is_some_and(|ops| ops.has_clone_into())
        {
            if let Def::List(list_def) = shape.def {
                return Self::clone_list_from_peek(peek, list_def);
            }
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
        RuntimeDisplayValue(self).to_string()
    }
}

#[doc(hidden)]
/// # Safety
///
/// The caller must uphold [`RuntimeValue::from_borrowed_pointer`]'s lifetime,
/// aliasing, and escaped-reference requirements for the returned pointer and
/// every value retaining its borrow. This low-level adapter does not retain a
/// source owner or a runtime lease.
pub unsafe fn borrow_pointer_runtime(
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
    // SAFETY: the pointee shape matches source, and dst is uninitialized storage
    // with exactly the pointer's layout. The constructor writes that storage
    // without consuming source; the caller guarantees the resulting borrow's
    // lifetime and shared-reference aliasing conditions.
    unsafe { borrow(dst, source.data()) };
    // SAFETY: the reviewed Facet constructor fully initialized dst in place.
    Ok(unsafe { RuntimeValue::from_initialized_ptr(pointer_shape, dst.assume_init(), layout) })
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
    // SAFETY: value owns this initialized pointer allocation exclusively. Its
    // lifetime contract keeps any borrowed pointee valid through promotion.
    // Facet mutates the pointer in place; value stays its owner even on unwind,
    // so ordinary Drop performs the one required destruction/deallocation.
    unsafe { promote(PtrMut::new(value.ptr.as_ptr())) };
    Ok(value)
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
