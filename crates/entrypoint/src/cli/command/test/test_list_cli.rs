use super::runtime_test_entry::RuntimeTestEntry;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_registry_execution::runtime_tests_matching;
use facet::Facet;

/// Metadata inspection does not construct requests or resolve credentials.
#[derive(Debug, Facet)]
pub struct TestListArgs {
    /// Full-name glob; '*' matches any sequence and '?' one character.
    #[facet(figue::positional, default = "*")]
    pub pattern: String,
}

impl TestListArgs {
    pub fn invoke(self) -> eyre::Result<CliOutput> {
        Ok(CliOutput::facet(
            runtime_tests_matching(&self.pattern)?
                .into_iter()
                .map(RuntimeTestEntry::from)
                .collect::<Vec<_>>(),
        ))
    }
}
