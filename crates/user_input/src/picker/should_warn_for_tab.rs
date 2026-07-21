use compact_str::CompactString;
use rustc_hash::FxHashSet;

pub(super) fn should_warn_for_tab(
    warned_keys: &mut FxHashSet<CompactString>,
    key: &CompactString,
) -> bool {
    key.contains('\t') && warned_keys.insert(key.clone())
}
