use crate::AccountSignInPrompt;
use crate::AzureBearerToken;
use crate::azure_cli_account_identity::AzureCliAccountIdentity;
use chrono::Local;
use chrono::TimeDelta;
use cloud_terrastodon_auth_types::AzureCredentialResource;
use cloud_terrastodon_azure_types::AzureAccessToken;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::CommandKind;
use eyre::Result;
use eyre::WrapErr;
use eyre::bail;
use eyre::ensure;
use facet::Facet;
use std::collections::HashMap;
use std::fs::File;
use std::fs::OpenOptions;
use std::fs::TryLockError;
use std::path::Path;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::process::ChildStderr;
use tokio::process::Command;
use tokio::sync::Mutex;

/// One immutable login identity; credential refresh leases its directory.
///
/// Fresh login generations never import ambient CLI state. Disable OS brokers so
/// the selected config directory owns the CLI's credential cache. See
/// <https://learn.microsoft.com/cli/azure/azure-cli-configuration> and
/// <https://learn.microsoft.com/cli/azure/authenticate-azure-cli-interactively>.
pub(crate) struct AzureCliSession {
    pub(crate) directory: PathBuf,
    pub(crate) identity: AzureCliAccountIdentity,
    tokens: Mutex<HashMap<AzureCredentialResource, AzureAccessToken<AzureBearerToken>>>,
}

impl AzureCliSession {
    pub(crate) fn new(directory: PathBuf, identity: AzureCliAccountIdentity) -> Self {
        Self {
            directory,
            identity,
            tokens: Mutex::new(HashMap::new()),
        }
    }

    /// A nonblocking OS lock avoids detached blocking tasks that outlive cancellation.
    pub(crate) async fn lock(path: &Path, cancellation: &CancellationScope) -> Result<File> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)
            .wrap_err("opening the Azure CLI session lease")?;
        loop {
            match file.try_lock() {
                Ok(()) => return Ok(file),
                Err(TryLockError::WouldBlock) => {
                    cancellation
                        .cancellable_scope(|_| async {
                            tokio::time::sleep(Duration::from_millis(25)).await;
                            eyre::Ok(())
                        })
                        .await?;
                }
                Err(TryLockError::Error(error)) => {
                    return Err(error).wrap_err("leasing the Azure CLI session");
                }
            }
        }
    }

    pub(crate) async fn lease(&self, cancellation: &CancellationScope) -> Result<File> {
        let lease = Self::lock(&self.directory.join("session.lock"), cancellation).await?;
        ensure!(
            !self.directory.join("revoked").try_exists()?,
            "Azure CLI session was logged out; explicitly log in again with a new authentication context"
        );
        Ok(lease)
    }

    pub(crate) fn environment(directory: &Path) -> Result<Vec<(&'static str, String)>> {
        Ok(vec![
            (
                "AZURE_CONFIG_DIR",
                directory
                    .to_str()
                    .ok_or_else(|| eyre::eyre!("Azure CLI session path is not Unicode"))?
                    .to_owned(),
            ),
            ("AZURE_CORE_ENABLE_BROKER_ON_WINDOWS", "false".into()),
            ("AZURE_CORE_ENABLE_BROKER_ON_MAC", "false".into()),
            // Preserve CLI's documented/current platform defaults explicitly.
            // Unix encrypted stores use shared keychain identities; ordinary
            // per-directory storage is protected by the store's 0700 directory.
            // https://github.com/Azure/azure-cli/blob/dev/src/azure-cli-core/azure/cli/core/util.py
            ("AZURE_CORE_ENCRYPT_TOKEN_CACHE", cfg!(windows).to_string()),
            ("AZURE_CORE_LOGIN_EXPERIENCE_V2", "off".into()),
            ("AZURE_CORE_COLLECT_TELEMETRY", "false".into()),
            ("AZURE_LOGGING_ENABLE_LOG_FILE", "false".into()),
            ("AZURE_EXTENSION_USE_DYNAMIC_INSTALL", "no".into()),
        ])
    }

    async fn credential_process(
        directory: &Path,
        cancellation: &CancellationScope,
    ) -> Result<Command> {
        let program = cancellation
            .cancellable_scope(|_| async { CommandKind::AzureCLI.program().await })
            .await?;
        let mut command = Command::new(program);
        for (name, _) in std::env::vars_os() {
            if name
                .to_string_lossy()
                .to_ascii_uppercase()
                .starts_with("AZURE_")
            {
                command.env_remove(name);
            }
        }
        command
            .envs(Self::environment(directory)?)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        Ok(command)
    }

    /// Credential subprocess output bypasses general command artifacts entirely.
    /// Errors discard stdout/stderr and decoder context, which can contain secrets.
    pub(crate) async fn credential_command(
        directory: &Path,
        args: &[&str],
        cancellation: &CancellationScope,
    ) -> Result<Vec<u8>> {
        let mut command = Self::credential_process(directory, cancellation).await?;
        command.args(args).arg("--only-show-errors");
        // Keep the session lease until the CLI launcher has finished. In
        // particular, cancelling az.cmd must not release a lease while its
        // Python child is still updating the credential cache.
        let output = cancellation
            .deferred(
                "finishing an isolated Azure CLI credential operation",
                |_| async {
                    command
                        .output()
                        .await
                        .wrap_err("executing an isolated Azure CLI credential operation")
                },
            )
            .await?;
        ensure!(
            output.status.success(),
            "Azure CLI credential operation failed ({}); credential output was suppressed. Explicitly log in to a new session if interaction is required",
            output.status
        );
        Ok(output.stdout)
    }

    /// Azure CLI opens a browser if available, otherwise emitting its MSAL
    /// device-flow prompt directly to stderr. Forward only the validated
    /// device code to the sign-in owner. All other output stays suppressed.
    ///
    /// <https://github.com/Azure/azure-cli/blob/dev/src/azure-cli-core/azure/cli/core/auth/identity.py>
    pub(crate) async fn login_command(
        directory: &Path,
        tenant: AzureTenantId,
        prompt: &(dyn Fn(AccountSignInPrompt) + Send + Sync),
        cancellation: &CancellationScope,
    ) -> Result<()> {
        let mut command = Self::credential_process(directory, cancellation).await?;
        command
            .env("AZURE_CORE_NO_COLOR", "true")
            .args([
                "login",
                "--tenant",
                &tenant.to_string(),
                "--allow-no-subscriptions",
                "--output",
                "none",
            ])
            .stdout(Stdio::null());
        cancellation
            .deferred("finishing isolated Azure CLI sign-in", |_| async {
                // az.cmd launches a Python child on Windows. Retain the session
                // directory lease until the process and its output have drained.
                let mut child = command
                    .spawn()
                    .wrap_err("starting isolated Azure CLI sign-in")?;
                let stderr = child.stderr.take();
                let (status, notifications) = tokio::join!(child.wait(), async {
                    let stderr = stderr.ok_or_else(|| {
                        eyre::eyre!("the Azure CLI sign-in process did not expose its prompt pipe")
                    })?;
                    Self::forward_sign_in_prompts(stderr, prompt).await
                });
                let status = status.wrap_err("waiting for isolated Azure CLI sign-in")?;
                ensure!(
                    status.success(),
                    "Azure CLI sign-in failed ({status}); credential output was suppressed"
                );
                notifications
            })
            .await
    }

    async fn forward_sign_in_prompts(
        mut stderr: ChildStderr,
        prompt: &(dyn Fn(AccountSignInPrompt) + Send + Sync),
    ) -> Result<()> {
        const MAX_PROMPT_BYTES: usize = 512;
        let mut bytes = [0; MAX_PROMPT_BYTES];
        let mut line = Vec::with_capacity(MAX_PROMPT_BYTES + 1);
        loop {
            let count = stderr
                .read(&mut bytes)
                .await
                .wrap_err("reading the Azure CLI sign-in prompt pipe")?;
            for &byte in &bytes[..count] {
                if byte == b'\n' {
                    if line.len() <= MAX_PROMPT_BYTES
                        && let Some(code) = Self::device_sign_in_code(&line)
                    {
                        prompt(AccountSignInPrompt::AzureCliDeviceCode {
                            code: code.to_owned(),
                        });
                    }
                    line.clear();
                } else if line.len() <= MAX_PROMPT_BYTES {
                    line.push(byte);
                }
            }
            if count == 0 {
                // Older Azure CLI versions may emit the prompt without a final
                // newline. Check the final bounded candidate at EOF.
                if line.len() <= MAX_PROMPT_BYTES
                    && let Some(code) = Self::device_sign_in_code(&line)
                {
                    prompt(AccountSignInPrompt::AzureCliDeviceCode {
                        code: code.to_owned(),
                    });
                }
                return Ok(());
            }
        }
    }

    fn device_sign_in_code(line: &[u8]) -> Option<&str> {
        let line = std::str::from_utf8(line).ok()?.trim_end();
        let message = line.strip_prefix("WARNING: ").unwrap_or(line);
        let (page, code) = message
            .strip_prefix("To sign in, use a web browser to open the page ")?
            .strip_suffix(" to authenticate.")?
            .split_once(" and enter the code ")?;
        let valid_code = |segment: &str| {
            !segment.is_empty()
                && segment
                    .bytes()
                    .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
        };
        (matches!(
            page,
            "https://microsoft.com/devicelogin"
                | "https://www.microsoft.com/devicelogin"
                | AccountSignInPrompt::AZURE_CLI_DEVICE_SIGN_IN_URL
        ) && (6..=14).contains(&code.len())
            && match code.split_once('-') {
                Some((first, second)) => valid_code(first) && valid_code(second),
                None => valid_code(code),
            })
        .then_some(code)
    }

    pub(crate) fn decode<T: Facet<'static>>(bytes: &[u8]) -> Result<T> {
        // Credential and identity payloads must not enter diagnostic source
        // chains; JSON errors can retain excerpts of the original output.
        let text = std::str::from_utf8(bytes).map_err(|_| {
            eyre::eyre!("Azure CLI credential output is not UTF-8; output suppressed")
        })?;
        facet_json::from_str(text).map_err(|_| {
            eyre::eyre!("Azure CLI credential output could not be decoded; output suppressed")
        })
    }

    pub(crate) async fn read_identity(
        directory: &Path,
        tenant: AzureTenantId,
        cancellation: &CancellationScope,
    ) -> Result<AzureCliAccountIdentity> {
        let bytes = Self::credential_command(
            directory,
            &[
                "account",
                "show",
                "--query",
                AzureCliAccountIdentity::QUERY,
                "--output",
                "json",
            ],
            cancellation,
        )
        .await?;
        let identity: AzureCliAccountIdentity = Self::decode(&bytes)?;
        identity.validate(tenant)?;
        Ok(identity)
    }

    pub(crate) async fn verify_identity(&self, cancellation: &CancellationScope) -> Result<()> {
        let current =
            Self::read_identity(&self.directory, self.identity.tenant, cancellation).await?;
        if current != self.identity {
            std::fs::write(self.directory.join("revoked"), b"identity changed")
                .wrap_err("revoking the changed Azure CLI session")?;
            bail!(
                "Azure CLI session identity changed; create a new authentication context after explicit login"
            );
        }
        Ok(())
    }

    pub(crate) async fn fetch_token(
        &self,
        resource: AzureCredentialResource,
        cancellation: &CancellationScope,
    ) -> Result<AzureAccessToken<AzureBearerToken>> {
        let _lease = self.lease(cancellation).await?;
        self.verify_identity(cancellation).await?;
        let mut tokens = self.tokens.lock().await;
        if let Some(token) = tokens.get(&resource)
            && token.expires_on > Local::now() + TimeDelta::seconds(120)
        {
            return Ok(token.clone());
        }
        let bytes = Self::credential_command(
            &self.directory,
            &[
                "account",
                "get-access-token",
                "--tenant",
                &self.identity.tenant.to_string(),
                "--scope",
                resource.scope(),
                "--output",
                "json",
            ],
            cancellation,
        )
        .await?;
        let token: AzureAccessToken<AzureBearerToken> = Self::decode(&bytes)?;
        ensure!(
            token.tenant == self.identity.tenant,
            "Azure CLI token belongs to a different tenant; output suppressed"
        );
        self.verify_identity(cancellation).await?;
        tokens.insert(resource, token.clone());
        Ok(token)
    }
}

impl std::fmt::Debug for AzureCliSession {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("AzureCliSession([REDACTED])")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_command::CancellationToken;

    #[test]
    fn only_admits_a_device_prompt_for_the_microsoft_sign_in_page() {
        let prompt = "WARNING: To sign in, use a web browser to open the page https://microsoft.com/devicelogin and enter the code TEST-CODE to authenticate.";
        assert_eq!(
            AzureCliSession::device_sign_in_code(prompt.as_bytes()),
            Some("TEST-CODE")
        );
        let untrusted_url = prompt.replace("microsoft.com", "example.invalid");
        let additional_output = format!("{prompt} access_token=synthetic-secret");
        for rejected in [untrusted_url, additional_output] {
            assert_eq!(
                AzureCliSession::device_sign_in_code(rejected.as_bytes()),
                None
            );
        }
    }

    #[test]
    fn credential_decode_failure_does_not_retain_secret_input() {
        let secret = b"synthetic-secret is not JSON";
        let error =
            AzureCliSession::decode::<AzureAccessToken<AzureBearerToken>>(secret).unwrap_err();
        assert!(!format!("{error:#?}").contains("synthetic-secret"));
    }

    #[tokio::test]
    async fn cancelled_lease_waiter_does_not_release_another_operation() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("lease");
        CancellationToken::new()
            .with_async_scope(|scope| async move {
                let held = AzureCliSession::lock(&path, &scope).await?;
                let cancellation = CancellationToken::new();
                let wait_path = &path;
                let waiting = cancellation.with_async_scope(|scope| async move {
                    AzureCliSession::lock(wait_path, &scope).await
                });
                tokio::pin!(waiting);
                tokio::select! {
                    result = &mut waiting => panic!("lease was not exclusive: {result:?}"),
                    _ = tokio::time::sleep(Duration::from_millis(40)) => {},
                }
                cancellation.request_cancel("synthetic waiting caller cancelled");
                assert!(waiting.await.is_err());
                let contender = OpenOptions::new().read(true).write(true).open(&path)?;
                assert!(matches!(
                    contender.try_lock(),
                    Err(TryLockError::WouldBlock)
                ));
                drop(held);
                contender.try_lock()?;
                Ok(())
            })
            .await
    }
}
