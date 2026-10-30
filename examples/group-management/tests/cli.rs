#![cfg(test)]

use cloud_terrastodon_group_management::ValidationReport;
use std::path::Path;
use std::process::Command;
use std::process::Output;

const DESIRED: &str = r#"[
  {
    "tenantId": "11111111-1111-4111-8111-111111111111",
    "groupId": "22222222-2222-4222-8222-222222222222",
    "displayName": "Synthetic test group",
    "membersMode": "additive",
    "members": [
      { "id": "33333333-3333-4333-8333-333333333333" }
    ],
    "ownersMode": "ignore",
    "owners": []
  }
]"#;

/// Only local validation or an explicitly disabled authentication policy is
/// exercised. Azure CLI cannot be started via PATH, and malformed synthetic
/// credential configuration must be ignored.
fn run(directory: &Path, desired: &str, command: &str, extra_args: &[&str]) -> Output {
    let path = directory.join("desired.json");
    std::fs::write(&path, desired).expect("write synthetic desired state");
    Command::new(env!("CARGO_BIN_EXE_group-management"))
        .current_dir(directory)
        .args([command, "--desired"])
        .arg(&path)
        .args(["--output-format", "json", "--log-filter", "off"])
        .args(extra_args)
        .env("PATH", "")
        .env("AZURE_CONFIG_DIR", directory.join("azure-config"))
        .env("AZURE_CLIENT_ID", "not-a-valid-synthetic-client-id")
        .env_remove("servicePrincipalId")
        .env_remove("AZURE_TENANT_ID")
        .env_remove("tenantId")
        .env_remove("AZURE_FEDERATED_TOKEN")
        .env_remove("idToken")
        .env("CLOUD_TERRASTODON_REAUTH", "DENY")
        .env("NO_COLOR", "1")
        .env_remove("RUST_BACKTRACE")
        .env_remove("RUST_LIB_BACKTRACE")
        .output()
        .expect("run local validation without directory access")
}

#[test]
fn local_validation_ignores_malformed_credentials_without_an_authentication_executable() {
    let directory = tempfile::tempdir().expect("create isolated validation directory");
    let output = run(directory.path(), DESIRED, "validate", &[]);
    assert!(
        output.status.success(),
        "local validation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("validation report is UTF-8");
    let report: ValidationReport =
        facet_json::from_str(&stdout).expect("decode machine-readable validation report");
    assert_eq!(report.group_count, 1);
    assert_eq!(report.tenant_ids.len(), 1);
    assert_eq!(
        report.tenant_ids[0].to_string(),
        "11111111-1111-4111-8111-111111111111"
    );
    assert_eq!(
        report.source_digest,
        format!("blake3:{}", blake3::hash(DESIRED.as_bytes()))
    );
    assert!(
        !directory.path().join("azure-config").exists(),
        "local validation initialized Azure CLI configuration"
    );
}

#[test]
fn invalid_local_policy_fails_before_directory_access() {
    let directory = tempfile::tempdir().expect("create isolated validation directory");
    let invalid = [
        DESIRED.replace("\"groupId\"", "\"id\""),
        DESIRED.replace(
            "\"membersMode\": \"additive\"",
            "\"membersMode\": \"ignore\"",
        ),
        DESIRED.replace("    \"membersMode\": \"additive\",\n", ""),
    ];
    for desired in invalid {
        assert_ne!(
            desired, DESIRED,
            "invalid fixture must differ from valid input"
        );
        let output = run(directory.path(), &desired, "validate", &[]);
        assert!(
            !output.status.success(),
            "invalid local policy unexpectedly succeeded: {desired}"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(!stderr.contains("Failed to spawn"), "{stderr}");
    }
    assert!(!directory.path().join("azure-config").exists());
}

#[test]
fn plan_with_authentication_disabled_fails_before_directory_access() {
    let directory = tempfile::tempdir().expect("create isolated planner directory");
    let output = run(
        directory.path(),
        DESIRED,
        "plan",
        &["--auth-source", "none"],
    );
    assert!(
        !output.status.success(),
        "disabled authentication unexpectedly allowed planning"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("auth source none"), "{stderr}");
    assert!(!stderr.contains("Failed to spawn"), "{stderr}");
    assert!(
        !stderr.contains("workload identity is incomplete"),
        "{stderr}"
    );
    assert!(!directory.path().join("azure-config").exists());
}
