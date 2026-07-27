use compact_str::CompactString;
use std::borrow::Borrow;

pub(super) fn preserved_selection<K>(
    selected_key: Option<&CompactString>,
    result_keys: &[K],
) -> Option<usize>
where
    K: Borrow<CompactString>,
{
    selected_key
        .and_then(|key| {
            result_keys
                .iter()
                .position(|candidate| candidate.borrow() == key)
        })
        .or_else(|| (!result_keys.is_empty()).then_some(0))
}
