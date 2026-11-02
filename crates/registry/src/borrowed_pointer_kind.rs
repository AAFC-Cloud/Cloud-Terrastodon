use crate::RuntimeValue;
use facet::Def;
use facet::Shape;
use facet_reflect::Peek;
use linkme::distributed_slice;

/// Constructs a pointer whose source lifetime is enforced outside Rust's types.
///
/// # Safety
///
/// Calls must uphold [`RuntimeValue::from_borrowed_pointer`]'s source-lifetime,
/// aliasing, and escaped-reference requirements. Implementations must initialize
/// the requested pointer shape without consuming the source.
pub type BorrowPointerFn =
    for<'mem, 'facet> unsafe fn(&'static Shape, Peek<'mem, 'facet>) -> eyre::Result<RuntimeValue>;
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

#[distributed_slice]
pub static KNOWN_BORROWED_POINTER_KINDS: [BorrowedPointerKind];

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

register_borrowed_pointer_kind!(Cow);
