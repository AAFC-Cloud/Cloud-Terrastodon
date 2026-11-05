#![allow(
    clippy::disallowed_types,
    reason = "The batch wire entry and its generated Facet code retain the body until its response shape is selected."
)]

use crate::MicrosoftGraphBatchResponseEntry;
use crate::MicrosoftGraphBatchResponseEntryBodyProxy;
use cloud_terrastodon_command::FromCommandOutput;
use facet_json::RawJson;
use std::collections::HashMap;

#[derive(Debug, facet::Facet)]
pub(crate) struct MicrosoftGraphBatchWireResponseEntry {
    pub id: String,
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: RawJson<'static>,
}

impl MicrosoftGraphBatchWireResponseEntry {
    pub(crate) fn into_typed<T: FromCommandOutput>(
        self,
    ) -> eyre::Result<MicrosoftGraphBatchResponseEntry<T>> {
        let body = MicrosoftGraphBatchResponseEntryBodyProxy(self.body).try_into()?;
        Ok(MicrosoftGraphBatchResponseEntry {
            id: self.id,
            status: self.status,
            headers: self.headers,
            body,
        })
    }
}
