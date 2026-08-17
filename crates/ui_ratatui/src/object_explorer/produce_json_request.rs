use super::arena_query_context::ArenaQueryContext;
use super::arena_query_context::ArenaQueryContextError;
use super::arena_query_session::JsonBatchBudget;
use super::breadcrumbs::Breadcrumbs;
use std::error::Error;
use std::fmt;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use tokio::io::AsyncWrite;
use tokio::io::AsyncWriteExt;

const EXPORT_WORK_PER_BATCH: usize = 256;
const EXPORT_BYTES_PER_BATCH: usize = 64 * 1024;

#[derive(Debug)]
pub(crate) enum ProduceJsonError {
    Query(ArenaQueryContextError),
    Io {
        path: PathBuf,
        operation: &'static str,
        source: std::io::Error,
    },
}

impl fmt::Display for ProduceJsonError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Query(error) => error.fmt(formatter),
            Self::Io {
                path,
                operation,
                source,
            } => write!(
                formatter,
                "could not {operation} {}: {source}",
                path.display()
            ),
        }
    }
}

impl Error for ProduceJsonError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Query(error) => Some(error),
            Self::Io { source, .. } => Some(source),
        }
    }
}

impl From<ArenaQueryContextError> for ProduceJsonError {
    fn from(value: ArenaQueryContextError) -> Self {
        Self::Query(value)
    }
}

/// Registry-facing request retained as an ordinary context-free IntoFuture.
///
/// Breadcrumbs is an ordinary reflected field. The generic value picker may
/// clone a Tab's projected `.breadcrumbs` value (or move a genuinely owned
/// Breadcrumbs root) into this request; ProduceJson has no Tab-specific source
/// lookup or borrow mechanism.
#[derive(Clone, Debug, facet::Facet)]
#[repr(C)]
pub(crate) struct ProduceJsonRequest {
    breadcrumbs: Breadcrumbs,
    filename: String,
}

impl ProduceJsonRequest {
    pub(crate) fn new(breadcrumbs: Breadcrumbs, filename: impl Into<String>) -> Self {
        Self {
            breadcrumbs,
            filename: filename.into(),
        }
    }

    pub(crate) const fn breadcrumbs(&self) -> &Breadcrumbs {
        &self.breadcrumbs
    }

    pub(crate) fn filename(&self) -> &str {
        &self.filename
    }

    pub(crate) fn run(
        self,
        context: ArenaQueryContext,
    ) -> Pin<Box<dyn Future<Output = Result<String, ProduceJsonError>> + Send>> {
        Box::pin(async move {
            let filename = PathBuf::from(&self.filename);
            let mut file = tokio::fs::File::create(&filename).await.map_err(|source| {
                ProduceJsonError::Io {
                    path: filename.clone(),
                    operation: "create",
                    source,
                }
            })?;

            self.write_to_sink(context, &mut file).await
        })
    }

    /// Stream this request to any bounded asynchronous sink.
    ///
    /// `AsyncWrite` backpressure suspends only this writer future. The engine
    /// retains the coherent read barrier but remains free to service bounded
    /// reads and to enqueue mutations until this session completes or drops.
    pub(crate) async fn write_to_sink<W>(
        self,
        context: ArenaQueryContext,
        sink: &mut W,
    ) -> Result<String, ProduceJsonError>
    where
        W: AsyncWrite + Unpin + Send,
    {
        let Self {
            breadcrumbs,
            filename,
        } = self;
        let path = PathBuf::from(&filename);
        let session = context.open_export(breadcrumbs).await?;

        sink.write_all(b"[")
            .await
            .map_err(|source| ProduceJsonError::Io {
                path: path.clone(),
                operation: "write",
                source,
            })?;

        let mut emitted_any = false;
        loop {
            let batch = session
                .next_json_batch(JsonBatchBudget::new(
                    EXPORT_WORK_PER_BATCH,
                    EXPORT_BYTES_PER_BATCH,
                ))
                .await?;
            sink.write_all(batch.fragment.as_bytes())
                .await
                .map_err(|source| ProduceJsonError::Io {
                    path: path.clone(),
                    operation: "write",
                    source,
                })?;
            emitted_any |= batch.emitted != 0;
            if batch.complete {
                break;
            }
        }

        sink.write_all(if emitted_any { b"\n]\n" } else { b"]\n" })
            .await
            .map_err(|source| ProduceJsonError::Io {
                path: path.clone(),
                operation: "finish writing",
                source,
            })?;
        sink.flush().await.map_err(|source| ProduceJsonError::Io {
            path: path.clone(),
            operation: "flush",
            source,
        })?;

        session.complete().await?;
        Ok(path.display().to_string())
    }
}

impl IntoFuture for ProduceJsonRequest {
    type Output = eyre::Result<String>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            // Lookup occurs when the async body is polled, after the host's
            // spawn adapter has installed the task-local capability.
            let context = ArenaQueryContext::current().map_err(ProduceJsonError::from)?;
            Ok(self.run(context).await?)
        })
    }
}

cloud_terrastodon_registry::register_thing!(ProduceJsonRequest);
cloud_terrastodon_registry::register_into_future!(
    ProduceJsonRequest => String,
    effects = [Write]
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object_explorer::arena::Arena;
    use crate::object_explorer::arena_query_context::ArenaQueryContextFutureExt;
    use crate::object_explorer::breadcrumb::Breadcrumb;
    use crate::object_explorer::breadcrumbs::Breadcrumbs;
    use crate::object_explorer::explorer_command::OwnedValuePacket;
    use crate::object_explorer::explorer_engine::ExplorerEngine;
    use crate::object_explorer::tab::Tab;
    use crate::object_explorer::value_address::ValueAddress;
    use cloud_terrastodon_registry::RuntimeValue;
    use facet::Facet;
    use facet::Type;
    use facet::UserType;
    use std::sync::atomic::AtomicU64;
    use std::sync::atomic::Ordering;
    use tokio::io::AsyncReadExt;
    use tokio::io::duplex;
    use tokio::time::Duration;
    use tokio::time::timeout;

    static NEXT_TEST_FILE: AtomicU64 = AtomicU64::new(1);

    #[derive(Clone, Debug, Facet)]
    #[repr(C)]
    struct PrettyThing {
        name: String,
        enabled: bool,
    }

    fn runtime<T>(value: T) -> RuntimeValue
    where
        T: Facet<'static> + Send + 'static,
    {
        RuntimeValue::from_box(Box::new(value)).expect("test value is representable")
    }

    fn test_path(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "cloud-terrastodon-{label}-{}-{}.json",
            std::process::id(),
            NEXT_TEST_FILE.fetch_add(1, Ordering::Relaxed)
        ))
    }

    #[test]
    fn produce_json_shape_accepts_breadcrumbs_without_a_tab_field() {
        let Type::User(UserType::Struct(request)) = ProduceJsonRequest::SHAPE.ty else {
            panic!("ProduceJsonRequest must remain a reflected struct");
        };
        let fields = request
            .fields
            .iter()
            .map(|field| (field.effective_name(), field.shape()))
            .collect::<Vec<_>>();

        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].0, "breadcrumbs");
        assert!(fields[0].1.is_shape(Breadcrumbs::SHAPE));
        assert_eq!(fields[1].0, "filename");
        assert!(!fields.iter().any(|(name, _)| *name == "tab"));
    }

    #[tokio::test]
    async fn produce_json_into_future_reports_typed_missing_context() {
        let request = ProduceJsonRequest::new(
            Breadcrumbs::default(),
            test_path("missing").display().to_string(),
        );

        let error = request
            .into_future()
            .await
            .expect_err("unscoped request must fail");

        assert!(matches!(
            error.downcast_ref::<ProduceJsonError>(),
            Some(ProduceJsonError::Query(ArenaQueryContextError::Missing))
        ));
    }

    #[tokio::test]
    async fn produce_json_context_free_future_streams_engine_query() {
        let mut arena = Arena::default();
        arena
            .insert_ready(runtime(Tab::new("export", Breadcrumbs::default())))
            .unwrap();
        arena
            .insert_ready(runtime(String::from("streamed-value")))
            .unwrap();
        let engine = ExplorerEngine::new(arena);
        let (context, inbox) = ArenaQueryContext::channel(4);
        let filename = test_path("stream");
        let request =
            ProduceJsonRequest::new(Breadcrumbs::default(), filename.display().to_string());

        let client = async move {
            let producer = tokio::spawn(
                request
                    .into_future()
                    .with_arena_query_context(context.clone()),
            );
            let written = producer
                .await
                .expect("producer task completes")
                .expect("export succeeds");
            assert_eq!(written, filename.display().to_string());
            let document = tokio::fs::read_to_string(&filename)
                .await
                .expect("exported file is readable");
            tokio::fs::remove_file(&filename)
                .await
                .expect("test export is removed");
            drop(context);
            document
        };

        let (_engine, document) = tokio::join!(engine.run(inbox), client);
        assert!(document.starts_with('['));
        assert!(document.trim_end().ends_with(']'));
        assert!(document.contains("streamed-value"));
    }

    #[tokio::test]
    async fn produce_json_explicit_run_needs_no_task_local_context() {
        let mut arena = Arena::default();
        arena
            .insert_ready(runtime(String::from("explicit-run")))
            .unwrap();
        let engine = ExplorerEngine::new(arena);
        let (context, inbox) = ArenaQueryContext::channel(2);
        let filename = test_path("explicit");
        let request =
            ProduceJsonRequest::new(Breadcrumbs::default(), filename.display().to_string());

        let client = async move {
            assert!(ArenaQueryContext::try_current().is_none());
            request
                .run(context.clone())
                .await
                .expect("explicit context export succeeds");
            let document = tokio::fs::read_to_string(&filename)
                .await
                .expect("exported file is readable");
            tokio::fs::remove_file(&filename)
                .await
                .expect("test export is removed");
            drop(context);
            document
        };

        let (_engine, document) = tokio::join!(engine.run(inbox), client);
        assert!(document.contains("explicit-run"));
    }

    #[tokio::test]
    async fn produce_json_streams_tab_query_without_materializing_results() {
        let mut arena = Arena::default();
        arena
            .insert_ready(runtime(String::from("selected")))
            .unwrap();
        arena.insert_ready(runtime(42_u64)).unwrap();
        let tab = Tab::new(
            "selected strings",
            Breadcrumbs::new(vec![
                Breadcrumb::ShapeFilter {
                    included_shapes: vec![cloud_terrastodon_registry::describe_shape(
                        String::SHAPE,
                    )],
                },
                Breadcrumb::AddressKindFilter {
                    include_roots: true,
                    include_descendants: false,
                },
            ]),
        );
        let request =
            ProduceJsonRequest::new(tab.breadcrumbs().clone(), "in-memory-tab-query.json");
        arena.insert_ready(runtime(tab)).unwrap();
        let engine = ExplorerEngine::new(arena);
        let (context, inbox) = ArenaQueryContext::channel(2);

        let client = async move {
            let mut sink = Vec::new();
            request
                .write_to_sink(context.clone(), &mut sink)
                .await
                .expect("filtered export succeeds");
            drop(context);
            String::from_utf8(sink).expect("JSON is UTF-8")
        };

        let (engine, document) = tokio::join!(engine.run(inbox), client);
        assert_eq!(document, "[\n  \"selected\"\n]\n");
        assert_eq!(
            engine.json_serialization_count(),
            1,
            "only matched addresses cross the explicit JSON boundary"
        );
    }

    #[tokio::test]
    async fn produce_json_uses_facet_pretty_json_without_materializing_the_array() {
        let mut arena = Arena::default();
        arena
            .insert_ready(runtime(PrettyThing {
                name: "Ada".to_owned(),
                enabled: true,
            }))
            .unwrap();
        let request = ProduceJsonRequest::new(
            Breadcrumbs::new(vec![
                Breadcrumb::ShapeFilter {
                    included_shapes: vec![cloud_terrastodon_registry::describe_shape(
                        PrettyThing::SHAPE,
                    )],
                },
                Breadcrumb::AddressKindFilter {
                    include_roots: true,
                    include_descendants: false,
                },
            ]),
            "pretty.json",
        );
        let engine = ExplorerEngine::new(arena);
        let (context, inbox) = ArenaQueryContext::channel(2);
        let client = async move {
            let mut sink = Vec::new();
            request
                .write_to_sink(context.clone(), &mut sink)
                .await
                .unwrap();
            drop(context);
            String::from_utf8(sink).unwrap()
        };

        let (engine, document) = tokio::join!(engine.run(inbox), client);
        assert_eq!(
            document,
            "[\n  {\n    \"name\": \"Ada\",\n    \"enabled\": true\n  }\n]\n"
        );
        assert_eq!(engine.json_serialization_count(), 1);
    }

    #[tokio::test]
    async fn produce_json_releases_barrier_on_failure_and_cancel() {
        let wait = Duration::from_secs(5);

        // A bounded in-memory pipe models a slow sink. Once its one-byte
        // buffer fills, reads remain serviceable and mutations are retained
        // by the barrier. Closing the reader turns that backpressure into a
        // write failure; dropping the session must then ingest the mutation.
        {
            let mut arena = Arena::default();
            let stable = arena
                .insert_ready(runtime(String::from("stable-before-failure")))
                .unwrap();
            let pending = arena.insert_pending().unwrap();
            let engine = ExplorerEngine::new(arena);
            let (context, inbox) = ArenaQueryContext::channel(8);
            let handle = context.engine_handle();

            let client = async move {
                let producer_context = context.clone();
                let request =
                    ProduceJsonRequest::new(Breadcrumbs::default(), "blocked-failure.json");
                let (writer, mut reader) = duplex(1);
                let producer = tokio::spawn(async move {
                    let mut writer = writer;
                    request.write_to_sink(producer_context, &mut writer).await
                });

                let mut opening = [0_u8; 1];
                timeout(wait, reader.read_exact(&mut opening))
                    .await
                    .expect("slow sink receives opening bracket promptly")
                    .expect("in-memory sink remains readable");
                assert_eq!(opening, *b"[");

                let mut receipt = handle
                    .submit_set_ready(
                        pending,
                        OwnedValuePacket::new(String::from("ingested-after-failure")),
                    )
                    .await
                    .expect("mutation enters the engine inbox");
                assert_eq!(
                    timeout(wait, handle.resolve_json(ValueAddress::root(stable)))
                        .await
                        .expect("sink backpressure does not block engine reads")
                        .expect("stable value resolves"),
                    "\"stable-before-failure\""
                );
                assert_eq!(receipt.try_result().expect("receipt remains valid"), None);

                drop(reader);
                let error = timeout(wait, producer)
                    .await
                    .expect("broken sink wakes the producer")
                    .expect("producer task itself remains healthy")
                    .expect_err("closed sink must fail the export");
                assert!(matches!(error, ProduceJsonError::Io { .. }));
                timeout(wait, receipt.wait())
                    .await
                    .expect("failure releases the export barrier")
                    .expect("deferred mutation succeeds");

                drop(handle);
                drop(context);
                pending
            };

            let (engine, pending) = tokio::join!(engine.run(inbox), client);
            assert_eq!(
                engine
                    .arena()
                    .ready_value(pending)
                    .and_then(|value| value.peek().as_str()),
                Some("ingested-after-failure")
            );
        }

        // Task abortion exercises the cancellation lease rather than an
        // async cleanup path. The queued mutation resumes in the same way.
        {
            let mut arena = Arena::default();
            let stable = arena
                .insert_ready(runtime(String::from("stable-before-cancel")))
                .unwrap();
            let pending = arena.insert_pending().unwrap();
            let engine = ExplorerEngine::new(arena);
            let (context, inbox) = ArenaQueryContext::channel(8);
            let handle = context.engine_handle();

            let client = async move {
                let producer_context = context.clone();
                let request =
                    ProduceJsonRequest::new(Breadcrumbs::default(), "blocked-cancel.json");
                let (writer, mut reader) = duplex(1);
                let producer = tokio::spawn(async move {
                    let mut writer = writer;
                    request.write_to_sink(producer_context, &mut writer).await
                });

                let mut opening = [0_u8; 1];
                timeout(wait, reader.read_exact(&mut opening))
                    .await
                    .expect("slow sink receives opening bracket promptly")
                    .expect("in-memory sink remains readable");
                assert_eq!(opening, *b"[");

                let mut receipt = handle
                    .submit_set_ready(
                        pending,
                        OwnedValuePacket::new(String::from("ingested-after-cancel")),
                    )
                    .await
                    .expect("mutation enters the engine inbox");
                assert_eq!(
                    timeout(wait, handle.resolve_json(ValueAddress::root(stable)))
                        .await
                        .expect("sink backpressure does not block engine reads")
                        .expect("stable value resolves"),
                    "\"stable-before-cancel\""
                );
                assert_eq!(receipt.try_result().expect("receipt remains valid"), None);

                producer.abort();
                assert!(
                    timeout(wait, producer)
                        .await
                        .expect("aborted producer joins")
                        .expect_err("producer is cancelled")
                        .is_cancelled()
                );
                drop(reader);
                timeout(wait, receipt.wait())
                    .await
                    .expect("cancellation releases the export barrier")
                    .expect("deferred mutation succeeds");

                drop(handle);
                drop(context);
                pending
            };

            let (engine, pending) = tokio::join!(engine.run(inbox), client);
            assert_eq!(
                engine
                    .arena()
                    .ready_value(pending)
                    .and_then(|value| value.peek().as_str()),
                Some("ingested-after-cancel")
            );
        }
    }
}
