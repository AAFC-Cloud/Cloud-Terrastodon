use crate::RuntimeTestCaseReport;
use crate::RuntimeTestObservation;
use crate::RuntimeTestReport;
use crate::RuntimeTestResult;
use crate::runtime_tests_matching;
use cloud_terrastodon_cache::WorkCache;
use cloud_terrastodon_execution::CacheInvalidatableRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::UnreachableAuthenticationProvider;
use cloud_terrastodon_registry::Effect;
use facet::Facet;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use teamy_cancellation::CancellationToken;

static STARTS: AtomicUsize = AtomicUsize::new(0);
static INVALIDATIONS: AtomicUsize = AtomicUsize::new(0);

#[derive(Facet)]
struct VerificationFixture {
    reject_refresh: bool,
}

impl ExecutableRequest for VerificationFixture {
    type Output = ();
    type Error = eyre::Report;
    fn run<'a>(self, _executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, ()>
    where
        Self: 'a,
    {
        Box::pin(async {
            STARTS.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
    }
}
impl CacheInvalidatableRequest for VerificationFixture {
    fn invalidate<'a>(
        &'a self,
        _executor: &'a CloudTerrastodonExecutor,
    ) -> ExecutionFuture<'a, ()> {
        Box::pin(async {
            INVALIDATIONS.fetch_add(1, Ordering::SeqCst);
            eyre::ensure!(!self.reject_refresh, "synthetic refresh failure");
            Ok(())
        })
    }
}
crate::register_executable_request!(VerificationFixture => (), effects = [Read, Write]);
crate::register_runtime_test!(
    VerificationFixture,
    name = "synthetic-verification-policy",
    cached
);

#[tokio::test]
async fn registry_selection_and_refresh_preserve_explicit_execution_policy() -> eyre::Result<()> {
    let selected = runtime_tests_matching("synthetic-verification-*")?;
    assert_eq!(selected.len(), 1);
    let registration = selected[0];
    assert_eq!(
        STARTS.load(Ordering::SeqCst),
        0,
        "listing must not invoke cases"
    );
    assert!(!registration.effects_allowed(&[Effect::Read]));
    assert!(registration.effects_allowed(&[Effect::Read, Effect::Write]));
    assert!(runtime_tests_matching("synthetic-verification-polic?")?.len() == 1);
    assert!(runtime_tests_matching("verification-policy")?.is_empty());

    CancellationToken::new()
        .with_async_scope(async |scope| {
            let executor = CloudTerrastodonExecutor::new(
                WorkCache::temporary()?,
                Arc::new(UnreachableAuthenticationProvider),
                scope,
            );
            (registration.execute)(r#"{"reject_refresh":true}"#, &executor, false).await?;
            assert_eq!(STARTS.load(Ordering::SeqCst), 1);
            assert_eq!(INVALIDATIONS.load(Ordering::SeqCst), 0);
            let failure = (registration.execute)(r#"{"reject_refresh":true}"#, &executor, true)
                .await
                .unwrap_err();
            assert!(failure.to_string().contains("synthetic refresh failure"));
            assert_eq!(
                STARTS.load(Ordering::SeqCst),
                1,
                "failed refresh must prevent execution"
            );
            assert_eq!(INVALIDATIONS.load(Ordering::SeqCst), 1);
            (registration.execute)(r#"{"reject_refresh":false}"#, &executor, true).await?;
            assert_eq!(STARTS.load(Ordering::SeqCst), 2);
            assert_eq!(INVALIDATIONS.load(Ordering::SeqCst), 2);
            eyre::Ok(())
        })
        .await
}

#[test]
fn report_outcome_does_not_turn_missing_failed_or_cancelled_cases_into_success() {
    let mut report = RuntimeTestReport {
        clean: false,
        debug_decoding: true,
        cancelled: false,
        cases: Vec::new(),
    };
    assert!(report.outcome().is_err());
    report.cases.push(RuntimeTestCaseReport {
        name: "not-admitted".into(),
        result: RuntimeTestResult::Skipped {
            reason: "write not permitted".into(),
        },
    });
    assert!(report.outcome().is_err());
    report.cases.push(RuntimeTestCaseReport {
        name: "completed".into(),
        result: RuntimeTestResult::Passed(RuntimeTestObservation::default()),
    });
    assert!(report.outcome().is_ok());
    report.cancelled = true;
    assert!(
        report.outcome().is_err(),
        "even a successfully drained final case retains run cancellation"
    );
    report.cancelled = false;
    report.cases.push(RuntimeTestCaseReport {
        name: "cancelled".into(),
        result: RuntimeTestResult::Cancelled {
            reason: "operator cancelled".into(),
        },
    });
    assert!(report.outcome().is_err());
    report.cases.pop();
    report.cases.push(RuntimeTestCaseReport {
        name: "failed".into(),
        result: RuntimeTestResult::Failed {
            message: "decoder rejected response".into(),
        },
    });
    assert!(report.outcome().is_err());
}

#[derive(Facet)]
struct VerificationOnlyFixture;

impl ExecutableRequest for VerificationOnlyFixture {
    type Output = ();
    type Error = eyre::Report;
    fn run<'a>(self, _executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, ()>
    where
        Self: 'a,
    {
        Box::pin(async { panic!("metadata inspection must not execute verification") })
    }
}

crate::register_runtime_test!(VerificationOnlyFixture => (), name = "synthetic-purpose-isolated", effects = [Read], uncached);

#[test]
fn verification_only_operations_do_not_enter_ordinary_discovery() -> eyre::Result<()> {
    assert_eq!(
        runtime_tests_matching("synthetic-purpose-isolated")?.len(),
        1
    );
    assert!(!crate::known_functions().iter().any(|function| {
        function
            .input_shape
            .is_shape(VerificationOnlyFixture::SHAPE)
    }));
    Ok(())
}
