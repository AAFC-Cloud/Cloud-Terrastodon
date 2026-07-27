use futures::Future;
use std::sync::Arc;
use std::time::SystemTime;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PickerLogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PickerLogSpan {
    pub name: Arc<str>,
    pub fields: Vec<(Arc<str>, Arc<str>)>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PickerLogRecord {
    pub timestamp: SystemTime,
    pub level: PickerLogLevel,
    pub message: Arc<str>,
    pub target: Arc<str>,
    pub fields: Vec<(Arc<str>, Arc<str>)>,
    pub spans: Vec<PickerLogSpan>,
    pub file: Option<Arc<str>>,
    pub line: Option<u32>,
}

pub trait PickerLogBuffer: Send + Sync {
    fn records_since(&self, cursor: &mut usize) -> Vec<PickerLogRecord>;
}

pub type PickerLogBufferHandle = Arc<dyn PickerLogBuffer>;

tokio::task_local! {
    static CURRENT_PICKER_LOG_BUFFER: PickerLogBufferHandle;
}

pub fn try_current_picker_log_buffer() -> Option<PickerLogBufferHandle> {
    CURRENT_PICKER_LOG_BUFFER.try_with(Arc::clone).ok()
}

pub trait TerminalLogBufferFutureExt: Future + Sized {
    fn with_terminal_log_buffer(
        self,
        log_buffer: PickerLogBufferHandle,
    ) -> impl Future<Output = Self::Output>;
}

impl<F> TerminalLogBufferFutureExt for F
where
    F: Future + Sized,
{
    fn with_terminal_log_buffer(
        self,
        log_buffer: PickerLogBufferHandle,
    ) -> impl Future<Output = Self::Output> {
        CURRENT_PICKER_LOG_BUFFER.scope(log_buffer, self)
    }
}

pub fn scope_picker_log_buffer<'a, F>(
    log_buffer: PickerLogBufferHandle,
    future: F,
) -> impl Future<Output = F::Output> + 'a
where
    F: Future + 'a,
{
    CURRENT_PICKER_LOG_BUFFER.scope(log_buffer, future)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct EmptyBuffer;

    impl PickerLogBuffer for EmptyBuffer {
        fn records_since(&self, _cursor: &mut usize) -> Vec<PickerLogRecord> {
            Vec::new()
        }
    }

    #[tokio::test]
    async fn nested_log_buffer_scopes_restore_after_await() {
        let outer: PickerLogBufferHandle = Arc::new(EmptyBuffer);
        let inner: PickerLogBufferHandle = Arc::new(EmptyBuffer);
        assert!(try_current_picker_log_buffer().is_none());

        let outer_for_scope = Arc::clone(&outer);
        let inner_for_scope = Arc::clone(&inner);
        scope_picker_log_buffer(outer.clone(), async move {
            tokio::task::yield_now().await;
            assert!(Arc::ptr_eq(
                &try_current_picker_log_buffer().expect("outer buffer"),
                &outer_for_scope,
            ));
            scope_picker_log_buffer(inner_for_scope.clone(), async move {
                tokio::task::yield_now().await;
                assert!(Arc::ptr_eq(
                    &try_current_picker_log_buffer().expect("inner buffer"),
                    &inner_for_scope,
                ));
            })
            .await;
            assert!(Arc::ptr_eq(
                &try_current_picker_log_buffer().expect("outer buffer restored"),
                &outer_for_scope,
            ));
        })
        .await;
        assert!(try_current_picker_log_buffer().is_none());
    }

    #[tokio::test]
    async fn raw_spawn_does_not_inherit_log_buffer_but_adapter_does() {
        let buffer: PickerLogBufferHandle = Arc::new(EmptyBuffer);
        let raw = scope_picker_log_buffer(buffer.clone(), async {
            tokio::spawn(async { try_current_picker_log_buffer().is_none() })
                .await
                .expect("raw task should complete")
        })
        .await;
        assert!(raw);

        let attached = tokio::spawn(
            async { try_current_picker_log_buffer().is_some() }.with_terminal_log_buffer(buffer),
        )
        .await
        .expect("attached task should complete");
        assert!(attached);
    }
}
