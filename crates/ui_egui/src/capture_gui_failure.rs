use cloud_terrastodon_command::CancellationError;
use eyre::Report;
use teamy_tracing::DiagnosticReport;

/// Record a failure retained by the GUI and keep the same report for its details view.
/// Call when handling the failure, outside the render loop.
pub(crate) fn capture_gui_failure(error: Report, message: &str) -> DiagnosticReport {
    let report = DiagnosticReport::new(error);
    // A plain cancellation is an expected dismissal. A failure encountered
    // while cleaning up may wrap it and is still useful in the logs.
    if !report
        .report
        .chain()
        .next()
        .is_some_and(|cause| cause.is::<CancellationError>())
    {
        tracing::error!(error = report.as_error(), "{message}");
    }
    report
}
