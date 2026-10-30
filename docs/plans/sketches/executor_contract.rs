//! Phase 1 contract proof only; this is not a production executor or cache.
//! Run check-executor-contract.ps1 to compile against the workspace's locked
//! dependency artifacts. All work below is synthetic and memory-only.

use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::CancellationToken;
use eyre::Result;
use std::any::Any;
use std::cell::Cell;
use std::future::Future;
use std::future::IntoFuture;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;

type ExecutionFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T>> + Send + 'a>>;
type InvocationFuture<'a> = ExecutionFuture<'a, Box<dyn Any + Send>>;
type AsyncExecutionFn =
    for<'a> fn(Box<dyn Any + Send>, &'a CloudTerrastodonExecutor) -> InvocationFuture<'a>;

// These fixture types represent the lower auth-types contract. They do not
// resolve real tenants, credentials, providers, configuration, or artifacts.
#[derive(Clone, Copy)]
enum AuthSelection {
    Anonymous,
    SyntheticBearer { tenant: u8 },
}

trait AuthenticationProvider: Send + Sync {
    fn admit(&self, selection: AuthSelection) -> Result<()>;

    fn authorization<'a>(
        &'a self,
        selection: AuthSelection,
        executor: &'a CloudTerrastodonExecutor,
    ) -> ExecutionFuture<'a, ()>;
}

struct DisabledAuthenticationProvider;

impl AuthenticationProvider for DisabledAuthenticationProvider {
    fn admit(&self, selection: AuthSelection) -> Result<()> {
        match selection {
            AuthSelection::Anonymous => Ok(()),
            AuthSelection::SyntheticBearer { tenant } => {
                eyre::bail!("authentication disabled for synthetic tenant {tenant}");
            }
        }
    }

    fn authorization<'a>(
        &'a self,
        selection: AuthSelection,
        executor: &'a CloudTerrastodonExecutor,
    ) -> ExecutionFuture<'a, ()> {
        Box::pin(async move {
            // Prove the capability can borrow the executing scope; no I/O.
            self.admit(selection)?;
            tokio::task::yield_now().await;
            executor.cancellation.bail_if_cancelled()?;
            Ok(())
        })
    }
}

#[derive(Clone)]
struct CloudTerrastodonExecutor {
    cancellation: CancellationScope,
    authentication: Arc<dyn AuthenticationProvider>,
    // Identity-only fixture, not a cache implementation.
    shared_resource: Arc<()>,
}

impl CloudTerrastodonExecutor {
    fn new(cancellation: CancellationScope) -> Self {
        Self {
            cancellation,
            authentication: Arc::new(DisabledAuthenticationProvider),
            shared_resource: Arc::new(()),
        }
    }

    fn execute<'a, R>(&'a self, request: R) -> RequestExecution<'a, R>
    where
        R: ExecutableRequest + 'a,
        R::Output: 'a,
    {
        RequestExecution {
            executor: self,
            request,
        }
    }

    fn with_scope(&self, cancellation: CancellationScope) -> Self {
        Self {
            cancellation,
            authentication: Arc::clone(&self.authentication),
            shared_resource: Arc::clone(&self.shared_resource),
        }
    }
}

trait ExecutableRequest: Sized + Send {
    type Output: Send;

    fn run<'a>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, Self::Output>
    where
        Self: 'a,
        Self::Output: 'a;

    fn execute<'a>(self, executor: &'a CloudTerrastodonExecutor) -> RequestExecution<'a, Self>
    where
        Self: 'a,
        Self::Output: 'a,
    {
        executor.execute(self)
    }
}

struct RequestExecution<'a, R> {
    executor: &'a CloudTerrastodonExecutor,
    request: R,
}

impl<'a, R> IntoFuture for RequestExecution<'a, R>
where
    R: ExecutableRequest + 'a,
    R::Output: 'a,
{
    type Output = Result<R::Output>;
    type IntoFuture = ExecutionFuture<'a, R::Output>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            self.executor
                .cancellation
                .cancellable_scope(move |child| self.request.run(self.executor.with_scope(child)))
                .await
        })
    }
}

fn invoke_executable<'a, R>(
    input: Box<dyn Any + Send>,
    executor: &'a CloudTerrastodonExecutor,
) -> InvocationFuture<'a>
where
    R: ExecutableRequest + 'static,
    R::Output: Any + Send + 'static,
{
    Box::pin(async move {
        let request = input
            .downcast::<R>()
            .map_err(|_| eyre::eyre!("wrong synthetic registry input"))?;
        Ok(Box::new(executor.execute(*request).await?) as Box<dyn Any + Send>)
    })
}

struct BorrowedLabel<'a> {
    label: &'a str,
}

impl<'input> ExecutableRequest for BorrowedLabel<'input> {
    type Output = &'input str;

    fn run<'a>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, Self::Output>
    where
        Self: 'a,
        Self::Output: 'a,
    {
        Box::pin(async move {
            executor
                .authentication
                .authorization(AuthSelection::Anonymous, &executor)
                .await?;
            Ok(self.label)
        })
    }
}

struct OwnedCell;

impl ExecutableRequest for OwnedCell {
    type Output = Cell<usize>;

    fn run<'a>(self, _executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, Self::Output>
    where
        Self: 'a,
        Self::Output: 'a,
    {
        Box::pin(async { Ok(Cell::new(7)) })
    }
}

struct ScopedChild;

impl ExecutableRequest for ScopedChild {
    type Output = (Arc<()>, CancellationScope);

    fn run<'a>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, Self::Output>
    where
        Self: 'a,
        Self::Output: 'a,
    {
        Box::pin(async move { Ok((executor.shared_resource, executor.cancellation)) })
    }
}

#[tokio::test]
async fn both_spellings_support_borrowed_requests_and_share_child_resources() -> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let executor = CloudTerrastodonExecutor::new(scope);
            let label = String::from("synthetic");
            assert_eq!(
                executor.execute(BorrowedLabel { label: &label }).await?,
                label.as_str()
            );
            assert_eq!(
                BorrowedLabel { label: &label }.execute(&executor).await?,
                label.as_str()
            );
            let (shared, closed_child) = executor.execute(ScopedChild).await?;
            assert!(Arc::ptr_eq(&shared, &executor.shared_resource));
            assert!(closed_child.bail_if_cancelled().is_err());
            executor.cancellation.bail_if_cancelled()?;
            Ok(())
        })
        .await
}

#[tokio::test]
async fn erased_invocation_preserves_send_but_non_sync_outputs() -> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let executor = CloudTerrastodonExecutor::new(scope);
            let invoke: AsyncExecutionFn = invoke_executable::<OwnedCell>;
            let output = invoke(Box::new(OwnedCell), &executor).await?;
            let cell = output
                .downcast::<Cell<usize>>()
                .map_err(|_| eyre::eyre!("unexpected synthetic output"))?;
            assert_eq!(cell.get(), 7);
            Ok(())
        })
        .await
}

#[tokio::test]
async fn per_caller_projection_can_borrow_non_send_state() -> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let executor = CloudTerrastodonExecutor::new(scope);
            let calls = Rc::new(Cell::new(0));
            let project = |value: Cell<usize>| {
                calls.set(calls.get() + 1);
                value.get() + calls.get()
            };
            let projected = project(executor.execute(OwnedCell).await?);
            assert_eq!(projected, 8);
            assert_eq!(calls.get(), 1);
            Ok(())
        })
        .await
}

#[tokio::test]
async fn disabled_provider_admits_local_work_and_rejects_authenticated_work() -> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let executor = CloudTerrastodonExecutor::new(scope);
            executor.authentication.admit(AuthSelection::Anonymous)?;
            assert!(
                executor
                    .authentication
                    .admit(AuthSelection::SyntheticBearer { tenant: 1 })
                    .is_err()
            );
            Ok(())
        })
        .await
}
