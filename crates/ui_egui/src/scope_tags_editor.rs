use crate::scope_tag_draft_row::ScopeTagDraftRow;
use cloud_terrastodon_azure::AzureResourceTags;
use cloud_terrastodon_command::app_work::Loadable;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;

/// A stable Azure snapshot and its editable draft for one scope.
#[derive(Default)]
pub(crate) struct ScopeTagsEditor {
    pub source: Loadable<Arc<AzureResourceTags>>,
    pub write: Loadable<()>,
    pub rows: Vec<ScopeTagDraftRow>,
    next_row_id: usize,
}

impl ScopeTagsEditor {
    pub const MAX_TAGS: usize = 50;

    pub fn is_loading(&self) -> bool {
        matches!(self.source, Loadable::Loading { .. })
            || matches!(self.write, Loadable::Loading { .. })
    }

    pub fn can_refresh(&self) -> bool {
        !self.is_loading() && !self.has_changes()
    }

    pub fn has_changes(&self) -> bool {
        self.source.as_loaded().is_some_and(|source| {
            let saved = &source.properties.tags;
            self.rows.len() != saved.len()
                || self
                    .rows
                    .iter()
                    .any(|row| saved.get(&row.name) != Some(&row.value))
                || saved
                    .keys()
                    .any(|name| !self.rows.iter().any(|row| row.name == *name))
        })
    }

    pub fn add_row(&mut self) {
        self.rows.push(ScopeTagDraftRow {
            id: self.next_row_id,
            name: String::new(),
            value: String::new(),
        });
        self.next_row_id += 1;
    }

    pub fn reset_draft(&mut self) {
        self.rows.clear();
        if let Some(source) = self.source.as_loaded() {
            let mut tags: Vec<_> = source.properties.tags.iter().collect();
            tags.sort_unstable_by_key(|(name, _)| *name);
            for (name, value) in tags {
                self.rows.push(ScopeTagDraftRow {
                    id: self.next_row_id,
                    name: name.clone(),
                    value: value.clone(),
                });
                self.next_row_id += 1;
            }
        }
        self.write = Loadable::NotLoaded;
    }

    /// Check the ARM limits common to tagged scopes. Providers can impose tighter rules.
    /// See [tag limitations](https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/tag-resources#limitations).
    pub fn draft_error(&self) -> Option<String> {
        if self.rows.len() > Self::MAX_TAGS {
            return Some(format!("Azure accepts at most {} tags.", Self::MAX_TAGS));
        }
        let mut names = HashSet::with_capacity(self.rows.len());
        for row in &self.rows {
            if row.name.is_empty() {
                return Some("Enter a name for each tag.".into());
            }
            if !names.insert(row.name.to_lowercase()) {
                return Some("Tag names must be unique, ignoring case.".into());
            }
        }
        None
    }

    pub fn draft_tags(&self) -> HashMap<String, String> {
        self.rows
            .iter()
            .map(|row| (row.name.clone(), row.value.clone()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_azure::AzureResourceTagsProperties;
    use std::time::Instant;

    #[test]
    fn draft_detects_case_sensitive_value_edits_and_rejects_duplicate_names() {
        let original = HashMap::from([("Environment".to_owned(), "Production".to_owned())]);
        let now = Instant::now();
        let mut editor = ScopeTagsEditor {
            source: Loadable::Loaded {
                value: Arc::new(AzureResourceTags {
                    id: None,
                    kind: None,
                    name: None,
                    properties: AzureResourceTagsProperties {
                        tags: original.clone(),
                    },
                }),
                started_at: now,
                finished_at: now,
            },
            ..Default::default()
        };
        editor.reset_draft();
        assert!(!editor.has_changes());
        editor.rows[0].value = "production".into();
        assert!(editor.has_changes());
        assert_eq!(editor.draft_tags()["Environment"], "production");
        assert!(!editor.can_refresh());

        editor.add_row();
        editor.rows[1].name = "environment".into();
        assert!(editor.draft_error().is_some());
        editor.reset_draft();
        assert_eq!(editor.draft_tags(), original);
        assert!(editor.can_refresh());
    }
}
