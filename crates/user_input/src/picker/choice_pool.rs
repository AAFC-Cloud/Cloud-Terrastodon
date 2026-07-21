use crate::Choice;
use compact_str::CompactString;
use rustc_hash::FxBuildHasher;
use rustc_hash::FxHashMap;
use std::ops::Deref;
use std::ops::DerefMut;

/// The choice store used by the picker UI.
///
/// Keeping this separate from the terminal code makes the event/candidate
/// semantics testable without a terminal and gives duplicate keys one clear
/// definition: the most recently received value wins.
#[derive(Debug)]
pub(super) struct ChoicePool<T>(FxHashMap<CompactString, T>);

impl<T> Default for ChoicePool<T> {
    fn default() -> Self {
        Self(FxHashMap::with_hasher(FxBuildHasher))
    }
}

impl<T> Deref for ChoicePool<T> {
    type Target = FxHashMap<CompactString, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for ChoicePool<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> ChoicePool<T> {
    pub(crate) fn inject(
        &mut self,
        choices: impl IntoIterator<Item = Choice<T>>,
        mut visit_new_key: impl FnMut(&CompactString),
    ) -> bool {
        let mut changed = false;
        for choice in choices {
            let key = CompactString::from(choice.key);
            if !self.0.contains_key(&key) {
                visit_new_key(&key);
                changed = true;
            }
            self.0.insert(key, choice.value);
        }
        changed
    }
}
