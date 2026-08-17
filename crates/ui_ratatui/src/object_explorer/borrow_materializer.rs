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
    drop(RuntimeValue::from_borrowed_pointer(
        pointer_shape,
        source.peek(),
    )?);
    Ok(())
}

/// Materializes a reflected borrowed pointer guarded by an active lease.
///
/// # Lifetime proof
///
/// Facet erases the Rust lifetime from RuntimeValue, so safety is enforced by
/// engine state instead: the non-clone BorrowLease names this exact source;
/// BorrowGraph proves the lease is active; every source mutation is rejected
/// while that edge exists; and callers transfer the lease with the containing
/// RuntimeValue, releasing it only after that value is dropped or promoted.
/// Arena resolution and pointer construction are synchronous under the
/// single-owner engine, and no Peek escapes this function.
pub(crate) fn materialize_borrow(
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
    RuntimeValue::from_borrowed_pointer(pointer_shape, source.peek())
}
