use super::arena::Arena;
use super::arena_address_source::ArenaAddressSource;
use super::borrow_graph::BorrowGraph;
use super::borrow_lease::BorrowLease;
use super::value_address::ValueAddress;
use cloud_terrastodon_registry::RuntimeValue;
use facet::Shape;

/// Checks that Facet can represent this address as the requested pointer type.
///
/// The temporary pointer never escapes this function. ExplorerEngine is
/// single-owner and applies no other command while this synchronous check is
/// running, so the resolved source cannot be replaced before the pointer is
/// dropped.
pub(crate) fn validate_borrow(
    arena: &Arena,
    address: &ValueAddress,
    pointer_shape: &'static Shape,
) -> eyre::Result<()> {
    let source = ArenaAddressSource::new(arena).resolve(address)?;
    // SAFETY: the shared arena borrow keeps the source stable during this
    // synchronous validation. The temporary is dropped here without cloning,
    // extracting a reference, or otherwise letting the borrow escape.
    drop(unsafe { RuntimeValue::from_borrowed_pointer(pointer_shape, source.peek())? });
    Ok(())
}

/// Materializes a reflected borrowed pointer guarded by an active lease.
///
/// # Entry-time checks and lifetime obligations
///
/// RuntimeValue's unsafe bridge erases the Rust lifetime, so callers must
/// enforce it through engine state: the non-clone BorrowLease names this exact
/// source and BorrowGraph checks the lease is active. Callers must transfer
/// the lease with the containing RuntimeValue and retain source protection
/// until all dependent values are dropped or promoted. This entry-time check
/// is not a proof of cancellation or shutdown ordering.
/// Arena resolution and pointer construction are synchronous under the
/// single-owner engine, and no Peek escapes this function.
///
/// # Safety
///
/// The caller must retain the checked lease with the returned value and every
/// shallow clone or moved/nested representation of it. It must prevent source
/// mutation/deletion and release the lease only after dependent values and
/// extracted references are gone (or their direct borrows are promoted while
/// any remaining nested references are still protected). The active-lease
/// check here cannot enforce these obligations after this function returns.
pub(crate) unsafe fn materialize_borrow(
    arena: &Arena,
    borrow_graph: &BorrowGraph,
    lease: &BorrowLease,
    pointer_shape: &'static Shape,
) -> eyre::Result<RuntimeValue> {
    if !borrow_graph.contains(lease) {
        eyre::bail!(
            "borrow lease for {} is not active in the engine graph",
            lease.source()
        );
    }
    let source = ArenaAddressSource::new(arena).resolve(lease.source())?;
    // SAFETY: resolution and the active-lease check establish the initial
    // source; the caller guarantees continued protection for escaped borrows.
    unsafe { RuntimeValue::from_borrowed_pointer(pointer_shape, source.peek()) }
}
