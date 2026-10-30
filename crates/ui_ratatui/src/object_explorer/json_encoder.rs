use eyre::WrapErr;
use facet_reflect::Peek;

/// The only boundary in object_explorer allowed to turn reflected values into
/// JSON. Keeping the counter instance-owned makes eager serialization visible
/// in deterministic tests without a process-global hook.
#[derive(Debug, Default)]
pub(crate) struct JsonEncoder {
    encoded_values: usize,
}

impl JsonEncoder {
    // Capture the caller while the typed serialization error is still available.
    // Engine message boundaries preserve this report before converting it to text.
    #[track_caller]
    pub(crate) fn encode(&mut self, value: Peek<'_, 'static>) -> eyre::Result<String> {
        self.record_encoded()?;
        facet_json::peek_to_string(value).wrap_err("could not serialize reflected value")
    }

    #[track_caller]
    pub(crate) fn encode_pretty(&mut self, value: Peek<'_, 'static>) -> eyre::Result<String> {
        self.record_encoded()?;
        facet_json::peek_to_string_pretty(value).wrap_err("could not serialize reflected value")
    }

    fn record_encoded(&mut self) -> eyre::Result<()> {
        self.encoded_values = self
            .encoded_values
            .checked_add(1)
            .ok_or_else(|| eyre::eyre!("JSON serialization counter overflowed"))?;
        Ok(())
    }

    pub(crate) const fn encoded_values(&self) -> usize {
        self.encoded_values
    }
}
