use eyre::Report;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;

/// A failed table-row conversion retaining its index, JSON, and decoder error.
///
/// The SDK can inspect this error to retain the failing row alongside the
/// original [Resource Graph response](https://learn.microsoft.com/en-us/rest/api/azureresourcegraph/resourcegraph/resources/resources?view=rest-azureresourcegraph-resourcegraph-2022-10-01#queryresponse).
#[derive(Debug)]
pub struct ResourceGraphEntryDeserializeError {
    // Keep the row, requested target, and cause together as one decode failure;
    // the row decoder constructs this diagnostic relationship.
    index: usize,
    type_name: &'static str,
    entry_json: String,
    source: Report,
}

impl ResourceGraphEntryDeserializeError {
    pub(crate) fn new(
        index: usize,
        type_name: &'static str,
        entry_json: String,
        source: Report,
    ) -> Self {
        Self {
            index,
            type_name,
            entry_json,
            source,
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn entry_json(&self) -> &str {
        &self.entry_json
    }
}

impl Display for ResourceGraphEntryDeserializeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "failed to deserialize entry {} as {}",
            self.index, self.type_name
        )
    }
}

impl Error for ResourceGraphEntryDeserializeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.source.as_ref())
    }
}
