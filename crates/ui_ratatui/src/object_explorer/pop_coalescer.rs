use super::value_address::ValueAddress;

/// Incremental state for one Pop operator.
///
/// QueryCursor uses this directly so a no-match scan can yield between work
/// budgets without hiding an unbounded loop inside Iterator::next.
#[derive(Default)]
pub(crate) struct AdjacentPop {
    last_emitted: Option<ValueAddress>,
}

impl AdjacentPop {
    pub(crate) fn apply(&mut self, address: ValueAddress) -> Option<ValueAddress> {
        let parent = address.parent()?;
        if self.last_emitted.as_ref() == Some(&parent) {
            return None;
        }
        self.last_emitted = Some(parent.clone());
        Some(parent)
    }

    #[cfg(test)]
    pub(crate) fn retained_address_count(&self) -> usize {
        usize::from(self.last_emitted.is_some())
    }
}

/// Pop one path segment and coalesce adjacent equal parents.
///
/// Pre-order query operators preserve subtree contiguity, so all matches that
/// map to the same parent arrive together. Retaining only the last parent is
/// therefore sufficient and keeps memory independent of match count.
pub(crate) struct PopCoalescer<I>
where
    I: Iterator<Item = ValueAddress>,
{
    input: I,
    state: AdjacentPop,
}

impl<I> PopCoalescer<I>
where
    I: Iterator<Item = ValueAddress>,
{
    pub(crate) fn new(input: I) -> Self {
        Self {
            input,
            state: AdjacentPop::default(),
        }
    }

    #[cfg(test)]
    pub(crate) fn retained_address_count(&self) -> usize {
        self.state.retained_address_count()
    }
}

impl<I> Iterator for PopCoalescer<I>
where
    I: Iterator<Item = ValueAddress>,
{
    type Item = ValueAddress;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let address = self.input.next()?;
            if let Some(parent) = self.state.apply(address) {
                return Some(parent);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object_explorer::slot_id::SlotId;
    use crate::object_explorer::value_path::ValuePathSegment;

    #[test]
    fn pop_coalesces_contiguous_parents_with_bounded_state() {
        let members = ValueAddress::root(SlotId::new(5));
        let member_zero = members.child(ValuePathSegment::Index(0));
        let zero_permissions =
            member_zero.child(ValuePathSegment::Field("permission_objects".to_owned()));
        let member_three = members.child(ValuePathSegment::Index(3));
        let three_permissions =
            member_three.child(ValuePathSegment::Field("permission_objects".to_owned()));
        let matches = vec![
            zero_permissions.child(ValuePathSegment::Index(1)),
            zero_permissions.child(ValuePathSegment::Index(4)),
            zero_permissions.child(ValuePathSegment::Index(8)),
            three_permissions.child(ValuePathSegment::Index(2)),
            three_permissions.child(ValuePathSegment::Index(3)),
        ];
        let mut popped = PopCoalescer::new(matches.into_iter());

        assert_eq!(popped.next(), Some(zero_permissions));
        assert_eq!(popped.retained_address_count(), 1);
        assert_eq!(popped.next(), Some(three_permissions));
        assert_eq!(popped.retained_address_count(), 1);
        assert_eq!(popped.next(), None);
        assert_eq!(popped.retained_address_count(), 1);
    }

    #[test]
    fn pop_drops_root_addresses_that_have_no_parent() {
        let roots = vec![
            ValueAddress::root(SlotId::new(1)),
            ValueAddress::root(SlotId::new(2)),
        ];

        assert_eq!(PopCoalescer::new(roots.into_iter()).next(), None);
    }
}
