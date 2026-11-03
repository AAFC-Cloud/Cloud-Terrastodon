use crate::AzureCliSessions;
use crate::auth_logout_error::AuthLogoutError;
#[cfg(windows)]
use crate::browser_access_token::BROWSER_REFRESH_TOKEN_TARGET_PREFIX;
use crate::browser_access_token::BROWSER_SESSION_FILE;
#[cfg(windows)]
use crate::pim_graph_access_token::PIM_REFRESH_TOKEN_TARGET_PREFIX;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::CommandOutput;
use cloud_terrastodon_command::RetryBehaviour;
use cloud_terrastodon_pathing::AppDir;
use eyre::Context;
use eyre::Result;
use std::path::Path;

/// Clear local browser/PIM refresh tokens and log out of Azure CLI and Azure DevOps.
///
/// All sources are attempted, independently of the selected authentication source.
/// Once admitted, cleanup finishes even after cancellation. This does not revoke
/// tokens remotely, end browser cookies, or alter credentials supplied by a parent
/// shell or pipeline (including PATs and workload identity).
///
/// Uses [`az logout`](https://learn.microsoft.com/en-us/cli/azure/reference-index#az-logout)
/// for the active account, [`az account clear`](https://learn.microsoft.com/en-us/cli/azure/account#az-account-clear)
/// for remaining CLI accounts, and [`az devops logout`](https://learn.microsoft.com/en-us/cli/azure/devops#az-devops-logout)
/// without an organization for all organization credentials.
pub async fn logout_all_auth_sources(cancellation: &CancellationScope) -> Result<()> {
    cancellation
        .deferred(
            "logging out of all authentication sources",
            |cancellation| async move {
                let mut errors = Vec::new();
                #[cfg(windows)]
                for prefix in [
                    BROWSER_REFRESH_TOKEN_TARGET_PREFIX,
                    PIM_REFRESH_TOKEN_TARGET_PREFIX,
                ] {
                    if let Err(error) =
                        crate::windows_credential_manager::delete_credentials_with_prefix(&format!(
                            "{prefix}:"
                        ))
                    {
                        errors.push(error.wrap_err("clearing stored browser/PIM refresh tokens"));
                    }
                }
                if let Err(error) =
                    remove_browser_session_file(&AppDir::Config.join(BROWSER_SESSION_FILE))
                {
                    errors.push(error);
                }
                if let Err(error) = AzureCliSessions::logout_all(&cancellation).await {
                    errors.push(error.wrap_err("clearing application-owned Azure CLI sessions"));
                }

                // Never cache logout or enter CommandBuilder's compatibility login retry.
                // Keep attempting the other sources after an individual failure.
                // az logout only removes the active account. account clear calls
                // Profile.logout_all(), removing all cached users and service principals:
                // https://github.com/Azure/azure-cli/blob/dev/src/azure-cli/azure/cli/command_modules/profile/custom.py
                for args in [
                    &["logout"][..],
                    &["account", "clear"][..],
                    &["devops", "logout"][..],
                ] {
                    let mut command = CommandBuilder::new(CommandKind::AzureCLI);
                    command.args(args.iter().copied());
                    command.use_retry_behaviour(RetryBehaviour::Fail);
                    if let Err(error) = command.run_raw(&cancellation).await {
                        // Both logout commands can fail when already logged out.
                        // Match only their known diagnostics; other failures remain errors.
                        let already_logged_out_message = match args {
                            ["logout"] => Some("Please run 'az login' to setup account."),
                            ["devops", "logout"] => Some("No credentials were found."),
                            _ => None,
                        };
                        if already_logged_out_message
                            .is_some_and(|message| command_reports_only_error(&error, message))
                        {
                            continue;
                        }
                        errors.push(error.wrap_err(format!("running az {}", args.join(" "))));
                    }
                }
                if errors.is_empty() {
                    Ok(())
                } else {
                    Err(AuthLogoutError { errors }.into())
                }
            },
        )
        .await
}

fn remove_browser_session_file(path: &Path) -> Result<()> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error).wrap_err("removing the local browser authentication session"),
    }
}

fn command_reports_only_error(error: &eyre::Report, expected: &str) -> bool {
    // Official no-credentials diagnostics:
    // https://github.com/Azure/azure-devops-cli-extension/blob/master/azure-devops/azext_devops/dev/common/_credentials.py
    // https://github.com/Azure/azure-cli/blob/dev/src/azure-cli-core/azure/cli/core/_profile.py
    let Some(output) = error.downcast_ref::<CommandOutput>() else {
        return false;
    };
    // CommandKind::AzureCLI adds --debug, which includes module-qualified error
    // lines and diagnostic context. Accept the known message only when every
    // ERROR line reports that same condition.
    let stderr = String::from_utf8_lossy(&output.stderr);
    let mut found = false;
    for line in stderr.lines().filter(|line| line.starts_with("ERROR:")) {
        let message = line
            .strip_prefix("ERROR: cli.azure.cli.core.azclierror: ")
            .or_else(|| line.strip_prefix("ERROR: "));
        if message != Some(expected) {
            return false;
        }
        found = true;
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_command::CancellationToken;

    #[test]
    fn removing_session_is_idempotent_and_does_not_decode_credentials() -> Result<()> {
        let directory = std::env::temp_dir().join(format!("ct-logout-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&directory)?;
        let session = directory.join(BROWSER_SESSION_FILE);
        let unrelated = directory.join("configuration.json");
        std::fs::write(&session, "broken session metadata")?;
        std::fs::write(&unrelated, "keep")?;
        remove_browser_session_file(&session)?;
        remove_browser_session_file(&session)?;
        assert!(!session.exists());
        assert_eq!(std::fs::read_to_string(&unrelated)?, "keep");
        // A directory at the session path must surface a real removal failure.
        std::fs::create_dir(&session)?;
        assert!(remove_browser_session_file(&session).is_err());
        std::fs::remove_dir(&session)?;
        std::fs::remove_file(&unrelated)?;
        std::fs::remove_dir(&directory)?;
        Ok(())
    }

    #[test]
    fn known_no_credentials_diagnostic_tolerates_debug_context_but_not_other_errors() {
        let error = |stderr: &str| {
            eyre::Report::from(CommandOutput {
                stdout: "".into(),
                stderr: stderr.into(),
                status: 1,
            })
            .wrap_err("command execution failed")
        };
        let expected = "No credentials were found.";
        assert!(command_reports_only_error(
            &error("ERROR: No credentials were found.\r\n"),
            expected
        ));
        assert!(!command_reports_only_error(
            &error("ERROR: Credential store access denied."),
            expected
        ));
        assert!(!command_reports_only_error(
            &error("ERROR: No credentials were found.\nERROR: Another failure"),
            expected
        ));
        assert!(command_reports_only_error(
            &error(
                "DEBUG: cli.knack.cli: Command arguments: ['devops', 'logout', '--debug']\nERROR: cli.azure.cli.core.azclierror: No credentials were found.\nERROR: No credentials were found.\nDEBUG: cli.__main__: Command ran in 1 second"
            ),
            expected
        ));
        assert!(command_reports_only_error(
            &error("ERROR: cli.azure.cli.core.azclierror: Please run 'az login' to setup account."),
            "Please run 'az login' to setup account."
        ));
        assert!(!command_reports_only_error(
            &eyre::eyre!("executable missing"),
            expected
        ));
    }

    #[tokio::test]
    async fn cancellation_before_admission_prevents_all_logout_work() -> Result<()> {
        CancellationToken::new()
            .with_async_scope(|cancellation| async move {
                cancellation.request_cancel("do not access credentials or execute commands");
                let error = logout_all_auth_sources(&cancellation)
                    .await
                    .expect_err("logout must not begin");
                use cloud_terrastodon_command::CancellationReportExt;
                assert!(error.is_cancellation());
                Ok(())
            })
            .await
    }
}
