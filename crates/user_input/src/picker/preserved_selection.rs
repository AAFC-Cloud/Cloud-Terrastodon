use compact_str::CompactString;

pub(super) fn preserved_selection(
    selected_key: Option<&CompactString>,
    result_keys: &[CompactString],
) -> Option<usize> {
    selected_key
        .and_then(|key| result_keys.iter().position(|candidate| candidate == key))
        .or_else(|| (!result_keys.is_empty()).then_some(0))
}
