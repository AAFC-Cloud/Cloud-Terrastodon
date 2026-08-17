use facet_reflect::Peek;

/// The only boundary in object_explorer allowed to turn reflected values into
/// JSON. Keeping the counter instance-owned makes eager serialization visible
/// in deterministic tests without a process-global hook.
#[derive(Debug, Default)]
pub(crate) struct JsonEncoder {
    encoded_values: usize,
}

impl JsonEncoder {
    pub(crate) fn encode(&mut self, value: Peek<'_, 'static>) -> Result<String, String> {
        self.record_encoded();
        facet_json::peek_to_string(value)
            .map_err(|error| format!("could not serialize reflected value: {error:?}"))
    }

    pub(crate) fn encode_pretty(&mut self, value: Peek<'_, 'static>) -> Result<String, String> {
        self.record_encoded();
        facet_json::peek_to_string_pretty(value)
            .map_err(|error| format!("could not serialize reflected value: {error:?}"))
    }

    fn record_encoded(&mut self) {
        self.encoded_values = self
            .encoded_values
            .checked_add(1)
            .expect("JSON serialization counter overflowed");
    }

    pub(crate) const fn encoded_values(&self) -> usize {
        self.encoded_values
    }
}
