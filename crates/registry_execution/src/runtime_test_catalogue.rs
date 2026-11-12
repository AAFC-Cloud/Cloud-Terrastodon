use crate::RUNTIME_TEST_REGISTRATIONS;
use crate::RuntimeTestRegistration;
use eyre::ensure;
use regex::Regex;
use std::collections::HashSet;

/// Select case names using a full-name glob (`*` and `?`), without executing them.
pub fn runtime_tests_matching(
    pattern: &str,
) -> eyre::Result<Vec<&'static RuntimeTestRegistration>> {
    ensure!(
        !pattern.is_empty(),
        "runtime test pattern must not be empty"
    );
    let expression = format!(
        "^{}$",
        regex::escape(pattern)
            .replace("\\*", ".*")
            .replace("\\?", ".")
    );
    let matcher = Regex::new(&expression)?;
    let mut names = HashSet::new();
    let mut selected = Vec::new();
    for registration in RUNTIME_TEST_REGISTRATIONS {
        ensure!(
            names.insert(registration.name),
            "duplicate runtime test registration: {}",
            registration.name
        );
        if matcher.is_match(registration.name) {
            selected.push(registration);
        }
    }
    selected.sort_by_key(|case| case.name);
    Ok(selected)
}
