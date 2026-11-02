use crate::operation_paths_proxy::OperationPathsProxy;
use facet::Facet;
use std::path::PathBuf;

/// Files examined and changed by a completed maintenance operation.
#[derive(Clone, Debug, Facet)]
pub struct OperationReport {
    pub operation: String,
    pub icon_count: usize,
    #[facet(proxy = OperationPathsProxy)]
    pub files: Vec<PathBuf>,
    #[facet(proxy = OperationPathsProxy)]
    pub changed_files: Vec<PathBuf>,
}
#[cfg(test)]
mod tests {
    use super::OperationReport;
    use cloud_terrastodon_app::CliOutput;
    use cloud_terrastodon_app::OutputFormat;
    use eyre::Result;
    use std::path::PathBuf;
    use teamy_cancellation::CancellationToken;

    #[test]
    fn redirected_cli_reports_preserve_artifact_paths_in_json() -> Result<()> {
        let paths = vec![PathBuf::from("workspace with spaces/étoile/azure_icon.rs")];
        let report = OperationReport {
            operation: "generate".into(),
            icon_count: 1,
            files: paths.clone(),
            changed_files: paths.clone(),
        };
        CancellationToken::new().with_scope(|scope| -> Result<()> {
            let rendered = CliOutput::facet(report)
                .render(Some(OutputFormat::Json), false, &scope)?
                .expect("JSON reports produce output");
            let decoded: OperationReport = facet_json::from_str(&rendered)?;
            assert_eq!(decoded.files, paths);
            assert_eq!(decoded.changed_files, paths);
            Ok(())
        })
    }
}
