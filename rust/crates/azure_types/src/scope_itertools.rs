use std::collections::HashSet;

use crate::scopes::HasScope;
use crate::scopes::Scope;

pub trait DistinctByScope: Iterator {
    fn distinct_by_scope(self) -> DistinctByScopeIterator<Self, Self::Item>
    where
        Self: Sized,
        Self::Item: HasScope;
}

impl<I> DistinctByScope for I
where
    I: Iterator,
{
    fn distinct_by_scope(self) -> DistinctByScopeIterator<Self, I::Item>
    where
        Self: Sized,
        Self::Item: HasScope,
    {
        DistinctByScopeIterator {
            iter: self,
            seen: HashSet::new(),
        }
    }
}

pub struct DistinctByScopeIterator<I, T>
where
    I: Iterator<Item = T>,
    T: HasScope,
{
    iter: I,
    seen: HashSet<String>,
}

impl<I, T> Iterator for DistinctByScopeIterator<I, T>
where
    I: Iterator<Item = T>,
    T: HasScope,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(item) = self.iter.next() {
            let scope = item.scope().expanded_form().to_string();
            if self.seen.insert(scope) {
                return Some(item);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::TestResource;

    #[test]
    fn test_distinct_by_scope() {
        let items = vec![
            TestResource::new("1", "one"),
            TestResource::new("2", "two"),
            TestResource::new("1", "one"), // duplicate
            TestResource::new("3", "three"),
        ];

        let distinct_items: Vec<_> = items.iter().distinct_by_scope().collect();

        assert_eq!(distinct_items.len(), 3);
        assert_eq!(distinct_items[0].id, items[0].id);
        assert_eq!(distinct_items[1].id, items[1].id);
        assert_eq!(distinct_items[2].id, items[3].id);
    }

    #[test]
    fn test_distinct_by_scope_empty() {
        let items: Vec<TestResource> = vec![];

        let distinct_items: Vec<_> = items.into_iter().distinct_by_scope().collect();

        assert_eq!(distinct_items.len(), 0);
    }

    #[test]
    fn test_distinct_by_scope_all_duplicates() {
        let items = vec![
            TestResource::new("1", "one"),
            TestResource::new("1", "I"),
            TestResource::new("1", "juan"),
            TestResource::new("1", "1"),
        ];

        let distinct_items: Vec<_> = items.iter().distinct_by_scope().collect();

        assert_eq!(distinct_items.len(), 1);
        assert_eq!(distinct_items[0].id, items[0].id);
    }

    #[test]
    fn test_distinct_by_scope_no_duplicates() {
        let items = vec![
            TestResource::new("1", "one"),
            TestResource::new("2", "two"),
            TestResource::new("3", "three"),
            TestResource::new("4", "four"),
        ];

        let distinct_items: Vec<_> = items.iter().distinct_by_scope().collect();

        assert_eq!(distinct_items.len(), 4);
        assert_eq!(distinct_items[0].id, items[0].id);
        assert_eq!(distinct_items[1].id, items[1].id);
        assert_eq!(distinct_items[2].id, items[2].id);
        assert_eq!(distinct_items[3].id, items[3].id);
    }
}
