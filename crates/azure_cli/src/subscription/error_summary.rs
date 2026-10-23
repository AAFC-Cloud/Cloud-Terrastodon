use cloud_terrastodon_command::CommandOutput;

const MAX_SUMMARY_CHARS: usize = 240;

/// Keep the underlying failure and recovery hint without command debug output.
pub(super) fn summarize(error: &eyre::Report) -> String {
    if let Some(output) = error.downcast_ref::<CommandOutput>() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let parts = command_error_parts(&stderr);
        return if parts.is_empty() {
            format!("External command failed (exit code {}).", output.status)
        } else {
            concise_summary(&parts)
        };
    }

    let cause = compact_whitespace(&error.root_cause().to_string());
    let message = if cause.is_empty() {
        compact_whitespace(&error.to_string())
    } else {
        cause
    };
    if message.is_empty() {
        "Request failed.".to_owned()
    } else {
        truncate(&message, MAX_SUMMARY_CHARS)
    }
}

fn command_error_parts(stderr: &str) -> Vec<String> {
    let mut errors = Vec::new();
    let mut fallback = Vec::new();
    let mut in_error = false;
    let mut in_traceback = false;

    for line in stderr
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        if let Some(message) = line
            .strip_prefix("ERROR:")
            .or_else(|| line.strip_prefix("Error:"))
        {
            let message = strip_logger_prefix(message.trim());
            in_traceback = starts_traceback(message);
            in_error = !in_traceback;
            if in_error {
                push_unique(&mut errors, message);
            }
            continue;
        }
        if is_log_line(line) {
            in_error = false;
            in_traceback = false;
            continue;
        }
        if starts_traceback(line) {
            in_error = false;
            in_traceback = true;
            continue;
        }
        if in_traceback {
            continue;
        }
        if in_error {
            push_unique(&mut errors, line);
        } else {
            push_unique(&mut fallback, line);
        }
    }

    if !errors.is_empty() {
        // Some CLI versions log between the error and its unprefixed login hint.
        for hint in fallback.iter().filter(|line| is_recovery_hint(line)) {
            push_unique(&mut errors, hint);
        }
        return errors;
    }
    // Unstructured stderr can contain many unrelated lines. Keep its first
    // useful message and any explicit login guidance that follows it.
    let mut parts = Vec::new();
    for line in fallback {
        if parts.is_empty() || is_recovery_hint(&line) {
            push_unique(&mut parts, &line);
        }
    }
    parts
}

fn strip_logger_prefix(message: &str) -> &str {
    for prefix in ["cli.azure.cli.core.azclierror:", "az_command_data_logger:"] {
        if let Some(message) = message.strip_prefix(prefix) {
            return message.trim();
        }
    }
    message
}

fn is_log_line(line: &str) -> bool {
    ["DEBUG:", "INFO:", "WARNING:", "WARN:", "TRACE:"]
        .into_iter()
        .any(|prefix| line.starts_with(prefix))
}

fn starts_traceback(line: &str) -> bool {
    line.starts_with("Traceback (")
        || line.starts_with("File \"")
        || line.starts_with("During handling of the above exception")
        || line.starts_with("The above exception was the direct cause")
}

fn strip_diagnostic_suffix(message: &str) -> &str {
    ["Trace ID:", "Correlation ID:", "Timestamp:"]
        .into_iter()
        .filter_map(|marker| message.find(marker))
        .min()
        .map_or(message, |index| message[..index].trim_end())
}

fn push_unique(parts: &mut Vec<String>, message: &str) {
    let message = compact_whitespace(strip_diagnostic_suffix(message));
    if !message.is_empty() && !parts.contains(&message) {
        parts.push(message);
    }
}

fn compact_whitespace(message: &str) -> String {
    message
        .chars()
        .map(|ch| if ch.is_control() { ' ' } else { ch })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_recovery_hint(message: &str) -> bool {
    let message = message.to_ascii_lowercase();
    message.contains("az login")
        || message.contains("az tenant login")
        || (message.starts_with("please")
            && (message.contains("log in")
                || message.contains("login")
                || message.contains("sign in")
                || message.contains("reauthenticate")))
}

fn concise_summary(parts: &[String]) -> String {
    let summary = parts.join(" ");
    if summary.chars().count() <= MAX_SUMMARY_CHARS {
        return summary;
    }

    let (recovery, failure): (Vec<_>, Vec<_>) = parts
        .iter()
        .flat_map(|part| match split_command_hint(part) {
            Some((failure, hint)) => vec![failure, hint],
            None => vec![part.as_str()],
        })
        .partition(|part| is_recovery_hint(part));
    if recovery.is_empty() || failure.is_empty() {
        return truncate(&summary, MAX_SUMMARY_CHARS);
    }

    // Reserve room for an actionable command when a long Entra error includes
    // dates, policy details, and other diagnostic prose before its login hint.
    let recovery = recovery.join(" ");
    let recovery = truncate(&recovery, MAX_SUMMARY_CHARS - 48);
    let failure = failure.join(" ");
    let failure = truncate(&failure, MAX_SUMMARY_CHARS - recovery.chars().count() - 2);
    format!("{failure}; {recovery}")
}

fn split_command_hint(message: &str) -> Option<(&str, &str)> {
    let lower = message.to_ascii_lowercase();
    let command = ["az login", "az tenant login"]
        .into_iter()
        .filter_map(|command| lower.find(command))
        .min()?;
    let hint = lower[..command]
        .rfind("please ")
        .or_else(|| lower[..command].rfind("run "))
        .unwrap_or(command);
    (hint > 0).then(|| (message[..hint].trim_end(), &message[hint..]))
}

fn truncate(message: &str, max_chars: usize) -> String {
    if message.chars().count() <= max_chars {
        return message.to_owned();
    }
    let shortened: String = message.chars().take(max_chars.saturating_sub(1)).collect();
    format!("{}…", shortened.trim_end())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn command_error(stderr: &str) -> eyre::Report {
        eyre::Report::from(CommandOutput {
            stdout: Vec::new().into(),
            stderr: stderr.as_bytes().to_vec().into(),
            status: 1,
        })
        .wrap_err("Synthetic token lookup wrapper")
        .wrap_err("Synthetic Resource Graph wrapper")
    }

    #[test]
    fn plain_cli_error_omits_wrappers_and_debug_logs() {
        let error = command_error(
            "DEBUG: cli.knack.cli: command arguments\nERROR: Please run 'az login' to setup account.\nINFO: command finished",
        );
        assert_eq!(summarize(&error), "Please run 'az login' to setup account.");
    }

    #[test]
    fn multiline_auth_error_keeps_colons_and_recovery_command() {
        let error = command_error(
            "ERROR: cli.azure.cli.core.azclierror: AADSTS70043: Session expired. Issued at 2025-01-01T10:20:30Z.\nTrace ID: synthetic-trace\nCorrelation ID: synthetic-correlation\nTimestamp: 2025-01-02 10:20:30Z\nPlease explicitly log in with:\naz login --tenant 11111111-1111-1111-1111-111111111111\nDEBUG: cli.knack.cli: finished\nTraceback (most recent call last):\n  File \"synthetic.py\", line 1\n    raise ValueError()",
        );
        let summary = summarize(&error);
        assert!(summary.contains("AADSTS70043: Session expired."));
        assert!(summary.contains("2025-01-01T10:20:30Z"));
        assert!(summary.contains("Please explicitly log in with:"));
        assert!(summary.contains("az login --tenant 11111111-1111-1111-1111-111111111111"));
        assert!(!summary.contains("DEBUG"));
        assert!(!summary.contains("Traceback"));
        assert!(!summary.contains("synthetic-trace"));
    }

    #[test]
    fn duplicate_logger_errors_and_continuations_are_deduplicated() {
        let error = command_error(
            "ERROR: cli.azure.cli.core.azclierror: Authentication required\naz login\nERROR: az_command_data_logger: Authentication required\naz login\nWARNING: unrelated warning",
        );
        assert_eq!(summarize(&error), "Authentication required az login");
    }

    #[test]
    fn logging_between_the_error_and_recovery_hint_does_not_hide_login_guidance() {
        let error = command_error(
            "ERROR: AADSTS70043: Session expired.\nDEBUG: finished\nPlease explicitly log in with:\naz login --scope https://example.com/.default\nINFO: synthetic telemetry",
        );
        assert_eq!(
            summarize(&error),
            "AADSTS70043: Session expired. Please explicitly log in with: az login --scope https://example.com/.default"
        );
    }

    #[test]
    fn long_single_line_errors_keep_the_trailing_login_command() {
        let stderr = format!(
            "ERROR: AADSTS70043: {} Please run 'az login --tenant 11111111-1111-1111-1111-111111111111'.",
            "Synthetic policy detail. ".repeat(40)
        );
        let summary = summarize(&command_error(&stderr));
        assert!(summary.starts_with("AADSTS70043:"));
        assert!(
            summary
                .contains("Please run 'az login --tenant 11111111-1111-1111-1111-111111111111'.")
        );
        assert!(summary.chars().count() <= MAX_SUMMARY_CHARS);
    }

    #[test]
    fn differing_inline_diagnostics_do_not_duplicate_auth_errors() {
        let error = command_error(
            "ERROR: cli.azure.cli.core.azclierror: AADSTS70043: Session expired. Trace ID: synthetic-first Correlation ID: synthetic-first Timestamp: 2025-01-01 10:20:30Z\naz login\nERROR: az_command_data_logger: AADSTS70043: Session expired. Trace ID: synthetic-second Correlation ID: synthetic-second Timestamp: 2025-01-02 10:20:30Z\naz login",
        );
        assert_eq!(summarize(&error), "AADSTS70043: Session expired. az login");
    }

    #[test]
    fn unstructured_stderr_uses_a_useful_message() {
        let error = command_error(
            "DEBUG: noisy log\nConnection refused\nUnrelated detail\naz login\nINFO: finished",
        );
        assert_eq!(summarize(&error), "Connection refused az login");
    }

    #[test]
    fn empty_or_debug_only_stderr_uses_exit_status() {
        for stderr in [
            "",
            "DEBUG: diagnostic\nINFO: finished\nWARNING: unrelated",
            "Traceback (most recent call last):\n  File \"synthetic.py\", line 1\n    raise ValueError()\nValueError: synthetic debug exception",
        ] {
            assert_eq!(
                summarize(&command_error(stderr)),
                "External command failed (exit code 1)."
            );
        }
    }

    #[test]
    fn ordinary_error_uses_root_cause_and_empty_cause_uses_context() {
        let error = eyre::eyre!("  Synthetic service\n unavailable  ").wrap_err("Request wrapper");
        assert_eq!(summarize(&error), "Synthetic service unavailable");
        assert_eq!(
            summarize(&eyre::eyre!("").wrap_err("Binding tenant failed")),
            "Binding tenant failed"
        );
        assert_eq!(summarize(&eyre::eyre!("")), "Request failed.");
    }

    #[test]
    fn truncation_is_unicode_safe_and_retains_login_guidance() {
        let error = eyre::eyre!("界".repeat(400));
        let summary = summarize(&error);
        assert_eq!(summary.chars().count(), MAX_SUMMARY_CHARS);
        assert!(summary.ends_with('…'));

        let stderr = format!(
            "ERROR: AADSTS70043: {}\nPlease explicitly log in with:\naz login --tenant 11111111-1111-1111-1111-111111111111",
            "Synthetic policy detail. ".repeat(40)
        );
        let summary = summarize(&command_error(&stderr));
        assert!(summary.chars().count() <= MAX_SUMMARY_CHARS);
        assert!(summary.starts_with("AADSTS70043:"));
        assert!(summary.contains("az login --tenant 11111111-1111-1111-1111-111111111111"));
    }
}
