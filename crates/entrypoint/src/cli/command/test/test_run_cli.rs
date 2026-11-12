use cloud_terrastodon_app::CancellationReportExt;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_types::AuthSource;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_cli_types::CloudTerrastodonAccountArgument;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_gitea::GiteaInstanceUrl;
use cloud_terrastodon_registry::Effect;
use cloud_terrastodon_registry_execution::RuntimeTestCaseReport;
use cloud_terrastodon_registry_execution::RuntimeTestRegistration;
use cloud_terrastodon_registry_execution::RuntimeTestReport;
use cloud_terrastodon_registry_execution::RuntimeTestResult;
use cloud_terrastodon_registry_execution::runtime_tests_matching;
use eyre::Context;
use eyre::Result;
use eyre::ensure;
use facet::Facet;
use facet::Type;
use facet::UserType;
use serde_json::Map;
use serde_json::Value;
use std::path::PathBuf;

/// Execute registered verification requests against explicit operator-selected scopes.
#[derive(Debug, Facet)]
pub struct TestRunArgs {
    /// Full-name registry glob, not a per-test subcommand.
    #[facet(figue::positional)]
    pub pattern: String,
    /// Resolve an explicitly tracked tenant ID or alias for Entra authentication.
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,
    /// Account ID or alias in the selected tenant or service instance.
    /// Omission uses that scope's configured account default.
    #[facet(figue::named)]
    pub account: Option<CloudTerrastodonAccountArgument>,
    /// Override the selected account's preferred authentication mechanism.
    #[facet(figue::named, default)]
    pub auth_source: AuthSource,
    #[facet(figue::named)]
    pub org: Option<AzureDevOpsOrganizationUrl>,
    #[facet(figue::named)]
    pub project: Option<AzureDevOpsProjectArgument<'static>>,
    #[facet(figue::named)]
    pub instance: Option<GiteaInstanceUrl>,
    /// JSON object mapping test names to additional explicit request fields.
    #[facet(figue::named)]
    pub input: Option<PathBuf>,
    /// Allowed effects; defaults to Read. Every case effect must be allowed.
    #[facet(figue::named, default)]
    pub allow_effect: Vec<Effect>,
    /// Run each request's existing invalidation contract before normal execution.
    #[facet(figue::named, default)]
    pub clean: bool,
}

impl TestRunArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let selected = runtime_tests_matching(&self.pattern)?;
        ensure!(
            !selected.is_empty(),
            "no runtime tests match {:?}",
            self.pattern
        );
        let input: Map<String, Value> = match &self.input {
            Some(path) => serde_json::from_slice(
                &tokio::fs::read(path)
                    .await
                    .wrap_err("reading runtime test inputs")?,
            )
            .wrap_err("runtime test inputs must be an object keyed by registered case name")?,
            None => Map::new(),
        };
        // A manifest may be shared across several pattern-selected runs, but
        // misspelled case names must not silently discard approved inputs.
        let registered = runtime_tests_matching("*")?;
        for name in input.keys() {
            ensure!(
                registered
                    .iter()
                    .any(|registration| registration.name == name),
                "runtime test input names an unknown case: {name}"
            );
        }
        if !self.clean {
            eprintln!(
                "Warning: --clean was not supplied; eligible cached responses may be reused. Cache misses can still contact services."
            );
        }
        let allowed = if self.allow_effect.is_empty() {
            &[Effect::Read][..]
        } else {
            &self.allow_effect
        };
        let mut report = RuntimeTestReport {
            clean: self.clean,
            debug_decoding: cfg!(debug_assertions),
            cancelled: false,
            cases: Vec::with_capacity(selected.len()),
        };
        // Cases are deliberately serial. Every admitted future is awaited to
        // completion, including protected response/artifact/invalidation work.
        // Ordinary case failures are data; cancellation closes further admission.
        for registration in selected {
            let result = if report.cancelled || executor.cancellation().is_cancelled() {
                RuntimeTestResult::Cancelled {
                    reason: "cancellation stopped case admission".to_owned(),
                }
            } else if !registration.effects_allowed(allowed) {
                RuntimeTestResult::Skipped {
                    reason: format!(
                        "requires effects {:?}; allowed {:?}",
                        registration.request.effects, allowed
                    ),
                }
            } else {
                match self
                    .prepare_input(registration, input.get(registration.name), executor)
                    .await
                {
                    Ok(input) => match (registration.execute)(&input, executor, self.clean).await {
                        Ok(observation) => RuntimeTestResult::Passed(observation),
                        Err(error) if error.is_cancellation() => RuntimeTestResult::Cancelled {
                            reason: error.to_string(),
                        },
                        Err(error) => RuntimeTestResult::Failed {
                            message: error.to_string(),
                        },
                    },
                    Err(error) if error.is_cancellation() => RuntimeTestResult::Cancelled {
                        reason: error.to_string(),
                    },
                    Err(error) => RuntimeTestResult::Failed {
                        message: error.to_string(),
                    },
                }
            };
            report.cancelled |= matches!(result, RuntimeTestResult::Cancelled { .. });
            report.cases.push(RuntimeTestCaseReport {
                name: registration.name.to_owned(),
                result,
            });
        }
        report.cancelled |= executor.cancellation().is_cancelled();
        let outcome = report.outcome();
        Ok(CliOutput::facet(report)
            .with_required_emission("emit runtime verification report")
            .with_result(outcome))
    }

    /// Common flags supply only fields the registered input actually declares.
    /// No defaults, generators, discovery paths, or arbitrary identifiers are
    /// synthesized for other fields: the request's own decoder validates them.
    async fn prepare_input(
        &self,
        registration: &RuntimeTestRegistration,
        input: Option<&Value>,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<String> {
        let mut fields = match input {
            None => Map::new(),
            Some(Value::Object(fields)) => fields.clone(),
            Some(_) => {
                eyre::bail!("case input must be a JSON object");
            }
        };
        let Type::User(UserType::Struct(record)) = registration.request.input_shape.ty else {
            eyre::bail!("runtime test request must declare named input fields");
        };
        let has_field = |name: &str| record.fields.iter().any(|field| field.name == name);
        for name in fields.keys() {
            ensure!(has_field(name), "unknown explicit request field {name}");
        }
        let account = self.account.as_ref().map(ToString::to_string);
        if has_field("authentication") || has_field("auth_selection") {
            ensure!(
                self.auth_source != AuthSource::None,
                "auth source none cannot execute an authenticated runtime test"
            );
            ensure!(
                self.auth_source != AuthSource::PersonalAccessToken
                    || (has_field("auth_selection") && self.tenant.is_none()),
                "PAT authentication requires an Azure DevOps test without --tenant"
            );
            if has_field("auth_selection") && self.tenant.is_none() {
                ensure!(
                    matches!(
                        self.auth_source,
                        AuthSource::Auto | AuthSource::PersonalAccessToken
                    ),
                    "bearer authentication requires an explicit tenant; use --tenant default to select the configured tenant"
                );
            }
            let tenant = match &self.tenant {
                Some(tenant) => Some(tenant.resolve(executor.cancellation()).await?),
                None => None,
            };
            if has_field("auth_selection") {
                let org = self
                    .org
                    .as_ref()
                    .ok_or_else(|| eyre::eyre!("--org is required for this test"))?;
                let selection = executor.resolve_azure_devops_authentication(
                    org,
                    tenant,
                    account.as_deref(),
                    self.auth_source,
                )?;
                let selection = serde_json::from_str(&facet_json::to_string(&selection)?)?;
                Self::insert_explicit(&mut fields, "auth_selection", selection)?;
            } else {
                let tenant =
                    tenant.ok_or_else(|| eyre::eyre!("--tenant is required for this test"))?;
                let selection = executor.resolve_azure_authentication(
                    tenant,
                    account.as_deref(),
                    self.auth_source,
                )?;
                let selection = serde_json::from_str(&facet_json::to_string(&selection)?)?;
                Self::insert_explicit(&mut fields, "authentication", selection)?;
            }
        }
        if has_field("account_id") && has_field("tenant") {
            ensure!(
                self.auth_source != AuthSource::None,
                "auth source none cannot execute an authenticated runtime test"
            );
            ensure!(
                self.auth_source == AuthSource::Auto,
                "Gitea runtime tests use the selected account's Tea credential source; an Azure authentication-source override is not applicable"
            );
            let instance = self
                .instance
                .as_ref()
                .ok_or_else(|| eyre::eyre!("--instance is required for this test"))?;
            let account_id =
                executor.resolve_gitea_authentication(instance.as_ref(), account.as_deref())?;
            Self::insert_explicit(
                &mut fields,
                "account_id",
                serde_json::from_str(&facet_json::to_string(&account_id)?)?,
            )?;
            Self::insert_explicit(
                &mut fields,
                "tenant",
                serde_json::from_str(&facet_json::to_string(instance)?)?,
            )?;
        }
        if has_field("org_url")
            && let Some(org) = &self.org
        {
            Self::insert_explicit(&mut fields, "org_url", Value::String(org.to_string()))?;
        }
        if has_field("project")
            && let Some(project) = &self.project
        {
            Self::insert_explicit(&mut fields, "project", Value::String(project.to_string()))?;
        }
        Ok(serde_json::to_string(&fields)?)
    }

    fn insert_explicit(fields: &mut Map<String, Value>, name: &str, value: Value) -> Result<()> {
        if let Some(existing) = fields.get(name) {
            ensure!(
                existing == &value,
                "explicit --input field {name} conflicts with the selected command-line scope or account"
            );
        } else {
            fields.insert(name.to_owned(), value);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn selected_scope_cannot_silently_replace_an_explicit_request_identity() -> Result<()> {
        let mut fields = Map::new();
        TestRunArgs::insert_explicit(&mut fields, "org_url", Value::String("approved".into()))?;
        TestRunArgs::insert_explicit(&mut fields, "org_url", Value::String("approved".into()))?;
        assert!(
            TestRunArgs::insert_explicit(&mut fields, "org_url", Value::String("different".into()))
                .is_err()
        );
        assert_eq!(fields["org_url"], Value::String("approved".into()));
        Ok(())
    }
    use cloud_terrastodon_app::OutputFormat;
    use cloud_terrastodon_cache::WorkCache;
    use cloud_terrastodon_execution::ExecutableRequest;
    use cloud_terrastodon_execution::ExecutionFuture;
    use cloud_terrastodon_execution::UnreachableAuthenticationProvider;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;
    use teamy_cancellation::CancellationError;
    use teamy_cancellation::CancellationToken;

    static LATER_CASE_STARTS: AtomicUsize = AtomicUsize::new(0);

    #[derive(Facet)]
    struct CancelledCase {}
    impl ExecutableRequest for CancelledCase {
        type Output = ();
        type Error = eyre::Report;
        fn run<'a>(self, _executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, ()>
        where
            Self: 'a,
        {
            Box::pin(async {
                Err(CancellationError::new("synthetic account session retired").into())
            })
        }
    }
    cloud_terrastodon_registry_execution::register_runtime_test!(CancelledCase => (), name = "synthetic-run-cancellation-1", effects = [Read], uncached);

    #[derive(Facet)]
    struct LaterCase {}
    impl ExecutableRequest for LaterCase {
        type Output = ();
        type Error = eyre::Report;
        fn run<'a>(self, _executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, ()>
        where
            Self: 'a,
        {
            Box::pin(async {
                LATER_CASE_STARTS.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
        }
    }
    cloud_terrastodon_registry_execution::register_runtime_test!(LaterCase => (), name = "synthetic-run-cancellation-2", effects = [Read], uncached);

    #[tokio::test]
    async fn a_cancelled_case_stops_admission_and_preserves_its_final_report() -> Result<()> {
        CancellationToken::new()
            .with_async_scope(async |scope| {
                let executor = CloudTerrastodonExecutor::new(
                    WorkCache::temporary()?,
                    Arc::new(UnreachableAuthenticationProvider),
                    scope,
                );
                let output = TestRunArgs {
                    pattern: "synthetic-run-cancellation-*".into(),
                    tenant: None,
                    account: None,
                    auth_source: AuthSource::None,
                    org: None,
                    project: None,
                    instance: None,
                    input: None,
                    allow_effect: vec![],
                    clean: true,
                }
                .invoke(&executor)
                .await?;
                assert!(!executor.cancellation().is_cancelled());
                assert_eq!(LATER_CASE_STARTS.load(Ordering::SeqCst), 0);
                // Cancellation can also arrive after the report is assembled.
                // The leaf's output must retain emission without root dispatch.
                executor
                    .cancellation()
                    .request_cancel("synthetic cancellation before report emission");
                let mut bytes = Vec::new();
                let error = output
                    .write_to(
                        Some(OutputFormat::Json),
                        false,
                        &mut bytes,
                        executor.cancellation(),
                    )
                    .expect_err("emit the report before returning its failed outcome");
                assert_eq!(error.to_string(), "runtime verification was cancelled");
                assert!(executor.cancellation().bail_if_cancelled().is_err());
                let report: RuntimeTestReport = facet_json::from_slice(&bytes)?;
                assert!(report.cancelled);
                assert_eq!(report.cases.len(), 2);
                assert!(
                    report
                        .cases
                        .iter()
                        .all(|case| matches!(case.result, RuntimeTestResult::Cancelled { .. }))
                );
                eyre::Ok(())
            })
            .await
    }
    #[tokio::test]
    async fn incompatible_azure_authentication_fails_before_default_tenant_lookup() -> Result<()> {
        CancellationToken::new()
            .with_async_scope(async |scope| {
                let executor = CloudTerrastodonExecutor::new(
                    WorkCache::temporary()?,
                    Arc::new(UnreachableAuthenticationProvider),
                    scope,
                );
                for (name, tenant, auth_source, expected) in [
                    (
                        "resource-group-list",
                        Some(AzureTenantArgument::default()),
                        AuthSource::None,
                        "auth source none",
                    ),
                    (
                        "resource-group-list",
                        Some(AzureTenantArgument::default()),
                        AuthSource::PersonalAccessToken,
                        "PAT authentication",
                    ),
                    (
                        "azure-devops-projects-list",
                        Some(AzureTenantArgument::default()),
                        AuthSource::PersonalAccessToken,
                        "PAT authentication",
                    ),
                    (
                        "azure-devops-projects-list",
                        None,
                        AuthSource::Browser,
                        "explicit tenant",
                    ),
                ] {
                    let args = TestRunArgs {
                        pattern: name.into(),
                        tenant,
                        account: None,
                        auth_source,
                        org: Some("synthetic".parse()?),
                        project: None,
                        instance: None,
                        input: None,
                        allow_effect: vec![],
                        clean: true,
                    };
                    let registration = runtime_tests_matching(name)?[0];
                    let error = args
                        .prepare_input(registration, None, &executor)
                        .await
                        .unwrap_err();
                    assert!(error.to_string().contains(expected), "{error:#}");
                }
                eyre::Ok(())
            })
            .await
    }

    #[tokio::test]
    async fn gitea_auth_rejection_precedes_scope_and_account_resolution() -> Result<()> {
        CancellationToken::new()
            .with_async_scope(async |scope| {
                let executor = CloudTerrastodonExecutor::new(
                    WorkCache::temporary()?,
                    Arc::new(UnreachableAuthenticationProvider),
                    scope,
                );
                let registration = runtime_tests_matching("gitea-repo-list")?[0];
                let mut args = TestRunArgs {
                    pattern: "gitea-repo-list".into(),
                    tenant: None,
                    account: None,
                    auth_source: AuthSource::None,
                    org: None,
                    project: None,
                    instance: None,
                    input: None,
                    allow_effect: vec![],
                    clean: true,
                };
                let disabled = args
                    .prepare_input(registration, None, &executor)
                    .await
                    .expect_err("disabled authentication must fail locally");
                assert!(disabled.to_string().contains("auth source none"));
                args.auth_source = AuthSource::Browser;
                let mismatch = args
                    .prepare_input(registration, None, &executor)
                    .await
                    .expect_err("foreign credential source must fail locally");
                assert!(mismatch.to_string().contains("override is not applicable"));
                eyre::Ok(())
            })
            .await
    }
}
