use super::slot_id::SlotId;
use super::value_address::ValueAddress;

/// On-demand reflected topology used by the address traversal algorithm.
///
/// Implementations decide child order from Facet/container metadata. The
/// cursor asks only for the next relationship it needs and never requests a
/// descendant count.
pub(crate) trait AddressSource {
    fn roots(&self) -> Box<dyn Iterator<Item = SlotId> + '_>;
    fn children(
        &self,
        parent: &ValueAddress,
    ) -> Option<Box<dyn Iterator<Item = ValueAddress> + '_>>;
}

/// Lazy pre-order cursor over root-plus-path identities.
pub(crate) struct PreorderCursor<'source, S>
where
    S: AddressSource,
{
    source: &'source S,
    frames: Vec<Box<dyn Iterator<Item = ValueAddress> + 'source>>,
}

impl<'source, S> PreorderCursor<'source, S>
where
    S: AddressSource,
{
    pub(crate) fn new(source: &'source S) -> Self {
        let roots = source.roots().map(ValueAddress::root);
        Self {
            source,
            frames: vec![Box::new(roots)],
        }
    }

    pub(crate) fn from_address(source: &'source S, address: ValueAddress) -> Self {
        Self {
            source,
            frames: vec![Box::new(std::iter::once(address))],
        }
    }

    pub(crate) fn empty(source: &'source S) -> Self {
        Self {
            source,
            frames: Vec::new(),
        }
    }

    /// Advance once while allowing a metadata predicate to prune this
    /// address's complete descendant subtree before any child iterator is
    /// created.
    pub(crate) fn next_with_descend(
        &mut self,
        mut should_descend: impl FnMut(&ValueAddress) -> bool,
    ) -> Option<ValueAddress> {
        loop {
            let frame = self.frames.last_mut()?;
            match frame.next() {
                Some(address) => {
                    if should_descend(&address)
                        && let Some(children) = self.source.children(&address)
                    {
                        self.frames.push(children);
                    }
                    return Some(address);
                }
                None => {
                    self.frames.pop();
                }
            }
        }
    }
}

impl<S> Iterator for PreorderCursor<'_, S>
where
    S: AddressSource,
{
    type Item = ValueAddress;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_with_descend(|_| true)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::object_explorer::value_path::ValuePathSegment;

    #[derive(Default)]
    struct FixtureAddressSource {
        roots: Vec<SlotId>,
        children: BTreeMap<ValueAddress, Vec<ValueAddress>>,
    }

    impl FixtureAddressSource {
        fn with_roots(mut self, mut roots: Vec<SlotId>) -> Self {
            roots.sort();
            self.roots = roots;
            self
        }

        fn add_children(&mut self, parent: ValueAddress, children: Vec<ValueAddress>) {
            self.children.insert(parent, children);
        }
    }

    impl AddressSource for FixtureAddressSource {
        fn roots(&self) -> Box<dyn Iterator<Item = SlotId> + '_> {
            Box::new(self.roots.iter().copied())
        }

        fn children(
            &self,
            parent: &ValueAddress,
        ) -> Option<Box<dyn Iterator<Item = ValueAddress> + '_>> {
            let children = self.children.get(parent)?.clone();
            Some(Box::new(children.into_iter()))
        }
    }

    #[test]
    fn query_order_uses_slot_id_and_reflected_preorder() {
        let root_two = ValueAddress::root(SlotId::new(2));
        let name = root_two.child(ValuePathSegment::Field("name".to_owned()));
        let entries = root_two.child(ValuePathSegment::Field("operations".to_owned()));
        let entry_zero = entries.child(ValuePathSegment::Index(0));
        let entry_one = entries.child(ValuePathSegment::Index(1));
        let root_nine = ValueAddress::root(SlotId::new(9));
        let mut source =
            FixtureAddressSource::default().with_roots(vec![SlotId::new(9), SlotId::new(2)]);
        source.add_children(root_two.clone(), vec![name.clone(), entries.clone()]);
        source.add_children(entries.clone(), vec![entry_zero.clone(), entry_one.clone()]);

        assert_eq!(
            PreorderCursor::new(&source).collect::<Vec<_>>(),
            vec![root_two, name, entries, entry_zero, entry_one, root_nine,]
        );
    }

    #[test]
    fn map_order_matches_reflected_iteration_for_root_revision() {
        let root = ValueAddress::root(SlotId::new(4));
        let map = root.child(ValuePathSegment::Field("by_name".to_owned()));
        // This deliberately differs from lexical key order. The source adapter
        // is responsible for exposing Facet's native iteration order.
        let reflected_order = vec![
            map.child(ValuePathSegment::Key("zeta".to_owned())),
            map.child(ValuePathSegment::Key("alpha".to_owned())),
        ];
        let mut source = FixtureAddressSource::default().with_roots(vec![SlotId::new(4)]);
        source.add_children(root.clone(), vec![map.clone()]);
        source.add_children(map.clone(), reflected_order.clone());

        let traversed = PreorderCursor::new(&source).collect::<Vec<_>>();

        assert_eq!(&traversed[2..], reflected_order);
        assert_eq!(
            traversed[2].path().segments().last(),
            Some(&ValuePathSegment::Key("zeta".to_owned()))
        );
    }
}
