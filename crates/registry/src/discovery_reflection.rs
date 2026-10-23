use crate::CancellationScope;
use crate::DiscoveryValue;
use crate::Function;
use crate::RuntimeValue;
use crate::describe_shape;
use facet::Shape;
use facet_reflect::HasFields;
use facet_reflect::Peek;
use std::collections::HashMap;
use std::collections::HashSet;

/// Copy descendants while their containing allocation is alive. Cloning Facet
/// values preserves their existing ownership; no erased synthetic borrow is
/// introduced by this traversal.
///
/// `remaining` caps new representatives cloned into the result. Traversing known
/// shapes spends no retention slots; inspection work has its own bounded budget.
pub(crate) fn reflected_descendants(
    value: &RuntimeValue,
    depth: usize,
    producer: Option<&'static Function>,
    known_values: &HashMap<&'static Shape, Vec<DiscoveryValue>>,
    remaining: usize,
    cancellation: &CancellationScope,
    diagnostics: &mut Vec<String>,
) -> eyre::Result<(Vec<DiscoveryValue>, bool)> {
    let mut pending = vec![(children(value.peek()), depth.saturating_add(1))];
    let mut discovered: Vec<DiscoveryValue> = Vec::new();
    // Facet's Shape equality and hashing use its type ID, matching is_shape.
    // Keep this separate from the pool: known parents can have new children.
    let mut visited_shapes = HashSet::<&'static Shape>::new();
    let inspection_limit = remaining.max(1).saturating_mul(64);
    let mut inspected = 0_usize;
    while let Some((iterator, child_depth)) = pending.last_mut() {
        cancellation.bail_if_cancelled()?;
        let Some(child) = iterator.next() else {
            pending.pop();
            continue;
        };
        let child_depth = *child_depth;
        if inspected >= inspection_limit {
            diagnostics.push(format!(
                "Stopped at the {inspection_limit}-inspection reflection budget before convergence"
            ));
            return Ok((discovered, false));
        }
        inspected += 1;
        if !visited_shapes.insert(child.shape()) {
            continue;
        }
        // The parent is reserved for insertion after reflection finishes. Exclude
        // its shape as well as shapes already present in the indexed pool.
        if !known_values.contains_key(child.shape()) && !value.shape().is_shape(child.shape()) {
            if discovered.len() >= remaining {
                diagnostics
                    .push("Stopped at the reflected-value budget before convergence".to_owned());
                return Ok((discovered, false));
            }
            match RuntimeValue::clone_from_peek(child) {
                Ok(value) => discovered.push(DiscoveryValue {
                    value,
                    depth: child_depth,
                    producer,
                }),
                Err(error) => diagnostics.push(format!(
                    "Cannot retain reflected {}: {error}",
                    describe_shape(child.shape())
                )),
            }
        }
        pending.push((children(child), child_depth.saturating_add(1)));
    }
    Ok((discovered, true))
}

fn children<'mem>(
    peek: Peek<'mem, 'static>,
) -> Box<dyn Iterator<Item = Peek<'mem, 'static>> + 'mem> {
    if let Ok(pointer) = peek.into_pointer() {
        return Box::new(pointer.borrow_inner().into_iter());
    }
    if let Ok(optional) = peek.into_option() {
        return Box::new(optional.value().into_iter());
    }
    // A homogeneous collection contributes its first representative. This
    // matches the pool's one-value-per-shape contract without allocating or
    // scanning an unbounded list of duplicate element shapes.
    if let Ok(list) = peek.into_list_like() {
        return Box::new(list.iter().take(1));
    }
    if let Ok(set) = peek.into_set() {
        return Box::new(set.iter().take(1));
    }
    if let Ok(object) = peek.into_struct() {
        return Box::new(object.fields().map(|(_, child)| child));
    }
    if let Ok(object) = peek.into_enum()
        && let Ok(variant) = object.active_variant()
    {
        return Box::new(
            (0..variant.data.fields.len())
                .filter_map(move |index| object.field(index).ok().flatten()),
        );
    }
    if let Ok(tuple) = peek.into_tuple() {
        return Box::new(tuple.fields().map(|(_, child)| child));
    }
    if let Ok(map) = peek.into_map() {
        return Box::new(map.iter().take(1).flat_map(|(key, value)| [key, value]));
    }
    Box::new(std::iter::empty())
}
