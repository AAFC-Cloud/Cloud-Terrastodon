use super::*;
use facet::Facet;
use std::cell::Cell;
use std::collections::BTreeMap;
use std::io;
use std::io::Write;
use std::rc::Rc;
use teamy_cancellation::CancellationError;
use teamy_cancellation::CancellationReportExt;
use teamy_cancellation::CancellationScope;
use teamy_cancellation::CancellationToken;

struct SelectedFormat;

impl CliOutputValue for SelectedFormat {
    fn write_to(
        &self,
        format: OutputFormat,
        writer: &mut dyn Write,
        _cancellation: &CancellationScope,
    ) -> eyre::Result<()> {
        write!(writer, "{format:?}")?;
        Ok(())
    }
}

#[derive(Facet, Clone, Debug, PartialEq)]
struct Report {
    name: String,
    count: u32,
}

#[derive(Default)]
struct RecordingWriter {
    bytes: Vec<u8>,
    flushed: bool,
}

impl Write for RecordingWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.flushed = true;
        Ok(())
    }
}

#[test]
fn pipelines_default_to_json_and_explicit_formats_override_detection() -> eyre::Result<()> {
    CancellationToken::new().with_scope(|cancellation| {
        let output = CliOutput::new(SelectedFormat);
        for (requested, terminal, expected) in [
            (None, true, "Text"),
            (None, false, "Json"),
            (Some(OutputFormat::Json), true, "Json"),
            (Some(OutputFormat::Text), false, "Text"),
            (Some(OutputFormat::FacetPretty), true, "FacetPretty"),
            (Some(OutputFormat::FacetPretty), false, "FacetPretty"),
            (Some(OutputFormat::Auto), true, "Text"),
            (Some(OutputFormat::Auto), false, "Json"),
        ] {
            assert_eq!(
                output
                    .render(requested, terminal, &cancellation)?
                    .as_deref(),
                Some(expected)
            );
        }
        Ok(())
    })
}

#[test]
fn custom_text_retains_generic_json_and_facet_pretty_without_calling_the_callback()
-> eyre::Result<()> {
    CancellationToken::new().with_scope(|cancellation| {
        let report = Report {
            name: "sample".into(),
            count: 3,
        };
        let calls = Rc::new(Cell::new(0));
        let callback_calls = Rc::clone(&calls);
        let output =
            CliOutput::facet_with_text(report.clone(), move |report, writer, _cancellation| {
                callback_calls.set(callback_calls.get() + 1);
                write!(writer, "{}: {} items", report.name, report.count)?;
                Ok(())
            });
        let generic = CliOutput::facet(report.clone());

        for format in [OutputFormat::Json, OutputFormat::FacetPretty] {
            for terminal in [true, false] {
                assert_eq!(
                    output.render(Some(format), terminal, &cancellation)?,
                    generic.render(Some(format), terminal, &cancellation)?
                );
            }
        }
        let pipeline_json = output.render(None, false, &cancellation)?.unwrap();
        assert_eq!(facet_json::from_str::<Report>(&pipeline_json)?, report);
        assert_eq!(calls.get(), 0);

        for terminal in [true, false] {
            assert_eq!(
                output.render(Some(OutputFormat::Text), terminal, &cancellation)?,
                Some("sample: 3 items".into())
            );
        }
        assert_eq!(calls.get(), 2);
        assert_eq!(
            output.render(Some(OutputFormat::Auto), true, &cancellation)?,
            Some("sample: 3 items".into())
        );
        assert_eq!(calls.get(), 3);
        Ok(())
    })
}

#[test]
fn custom_text_callback_errors_preserve_emitted_prefix_without_flushing() -> eyre::Result<()> {
    CancellationToken::new().with_scope(|cancellation| {
        let output = CliOutput::facet_with_text(
            Report {
                name: "sample".into(),
                count: 3,
            },
            |_, writer, _cancellation| {
                writer.write_all(b"partial result")?;
                Err(eyre::eyre!("custom text rendering failed"))
            },
        );
        let mut writer = RecordingWriter::default();
        let error = output
            .write_to(Some(OutputFormat::Text), false, &mut writer, &cancellation)
            .unwrap_err();
        assert_eq!(error.to_string(), "custom text rendering failed");
        assert_eq!(writer.bytes, b"partial result");
        assert!(!writer.flushed);
        Ok(())
    })
}

#[test]
fn deferred_command_errors_follow_output_and_flush_without_changing_json_shape() -> eyre::Result<()>
{
    CancellationToken::new().with_scope(|cancellation| {
        let report = Report {
            name: "unsuccessful response".into(),
            count: 3,
        };
        let output = CliOutput::facet(report.clone())
            .with_result(Err(eyre::eyre!("request was unsuccessful")));
        // Previewing output does not consume its deferred command result.
        let preview = output
            .render(Some(OutputFormat::Json), false, &cancellation)?
            .unwrap();
        assert_eq!(
            Some(preview.clone()),
            CliOutput::facet(report.clone()).render(
                Some(OutputFormat::Json),
                false,
                &cancellation,
            )?
        );
        assert_eq!(facet_json::from_str::<Report>(&preview)?, report);

        let mut writer = RecordingWriter::default();
        let error = output
            .write_to(None, false, &mut writer, &cancellation)
            .unwrap_err();
        assert_eq!(error.to_string(), "request was unsuccessful");
        assert!(writer.flushed);
        assert!(writer.bytes.ends_with(b"\n"));
        let json = String::from_utf8(writer.bytes)?;
        assert_eq!(json, format!("{preview}\n"));
        assert_eq!(facet_json::from_str::<Report>(&json)?, report);
        Ok(())
    })
}

#[test]
fn deferred_command_errors_are_returned_when_there_is_no_output() -> eyre::Result<()> {
    CancellationToken::new().with_scope(|cancellation| {
        let mut bytes = Vec::new();
        let error = CliOutput::none()
            .with_result(Err(eyre::eyre!("command failed without output")))
            .write_to(None, false, &mut bytes, &cancellation)
            .unwrap_err();
        assert_eq!(error.to_string(), "command failed without output");
        assert!(bytes.is_empty());
        Ok(())
    })
}

#[test]
fn json_is_valid_and_has_no_terminal_escapes_even_on_a_terminal() -> eyre::Result<()> {
    CancellationToken::new().with_scope(|cancellation| {
        for terminal in [true, false] {
            let report = Report {
                name: "a \"quoted\" name\nwith another line".into(),
                count: 3,
            };
            let output = CliOutput::facet(report.clone());
            let json = output
                .render(Some(OutputFormat::Json), terminal, &cancellation)?
                .unwrap();
            assert!(!json.contains('\u{1b}'), "{json}");
            assert_eq!(facet_json::from_str::<Report>(&json)?, report);
        }
        Ok(())
    })
}

#[test]
fn explicit_text_shows_the_underlying_value() -> eyre::Result<()> {
    CancellationToken::new().with_scope(|cancellation| {
        let text = CliOutput::facet(Report {
            name: "sample".into(),
            count: 3,
        })
        .render(Some(OutputFormat::Text), false, &cancellation)?
        .unwrap();
        assert!(text.contains("sample"), "{text}");
        Ok(())
    })
}

#[test]
fn facet_pretty_shows_fields_regardless_of_format_detection() -> eyre::Result<()> {
    CancellationToken::new().with_scope(|cancellation| {
        for terminal in [true, false] {
            let text = CliOutput::facet(Report {
                name: "sample".into(),
                count: 3,
            })
            .render(Some(OutputFormat::FacetPretty), terminal, &cancellation)?
            .unwrap();
            assert!(text.contains("name"), "{text}");
            assert!(text.contains("sample"), "{text}");
            assert!(text.contains("count"), "{text}");
        }
        Ok(())
    })
}

#[test]
fn none_has_no_output_in_any_format() -> eyre::Result<()> {
    CancellationToken::new().with_scope(|cancellation| {
        for format in [
            None,
            Some(OutputFormat::Text),
            Some(OutputFormat::Json),
            Some(OutputFormat::FacetPretty),
        ] {
            assert_eq!(CliOutput::none().render(format, true, &cancellation)?, None);
        }
        Ok(())
    })
}

struct CancelAfterPrefixWriter {
    token: CancellationToken,
    bytes: Vec<u8>,
    limit_per_write: usize,
    cancel_after: usize,
    write_calls: usize,
    flushed: bool,
}

impl Write for CancelAfterPrefixWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.write_calls += 1;
        let written = bytes.len().min(self.limit_per_write);
        self.bytes.extend_from_slice(&bytes[..written]);
        if self.bytes.len() >= self.cancel_after && !self.token.is_cancelled() {
            self.token.request_cancel("output prefix was sufficient");
        }
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.flushed = true;
        Ok(())
    }
}

struct IncrementalOutput {
    attempts: Rc<Cell<usize>>,
}

impl CliOutputValue for IncrementalOutput {
    fn write_to(
        &self,
        _format: OutputFormat,
        writer: &mut dyn Write,
        _cancellation: &CancellationScope,
    ) -> eyre::Result<()> {
        // Model an unbounded producer, with a fuse to fail rather than hang if
        // the output boundary ever stops observing cancellation on writes.
        for _ in 0..10_000 {
            self.attempts.set(self.attempts.get() + 1);
            writer.write_all(b"entry\n")?;
        }
        eyre::bail!("incremental output failed to observe cancellation")
    }
}

struct LargeBuffer(Vec<u8>);

impl CliOutputValue for LargeBuffer {
    fn write_to(
        &self,
        _format: OutputFormat,
        writer: &mut dyn Write,
        _cancellation: &CancellationScope,
    ) -> eyre::Result<()> {
        writer.write_all(&self.0)?;
        Ok(())
    }
}

struct EmptyWriteOutput {
    token: CancellationToken,
    attempts: Rc<Cell<usize>>,
}

impl CliOutputValue for EmptyWriteOutput {
    fn write_to(
        &self,
        _format: OutputFormat,
        writer: &mut dyn Write,
        _cancellation: &CancellationScope,
    ) -> eyre::Result<()> {
        for _ in 0..10_000 {
            self.attempts.set(self.attempts.get() + 1);
            if self.attempts.get() == 1 {
                self.token.request_cancel("empty output cancelled");
            }
            writer.write_all(b"")?;
        }
        eyre::bail!("empty writes failed to observe cancellation")
    }
}

#[derive(Debug)]
struct RendererWriteError(io::Error);

impl std::fmt::Display for RendererWriteError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("custom renderer failed while writing")
    }
}

impl std::error::Error for RendererWriteError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

struct WrappedWriteOutput;

impl CliOutputValue for WrappedWriteOutput {
    fn write_to(
        &self,
        _format: OutputFormat,
        writer: &mut dyn Write,
        _cancellation: &CancellationScope,
    ) -> eyre::Result<()> {
        writer.write_all(b"entry\n").map_err(RendererWriteError)?;
        Ok(())
    }
}

#[test]
fn empty_stream_writes_observe_cancellation_without_emitting_bytes() -> eyre::Result<()> {
    let token = CancellationToken::new();
    token.with_scope(|cancellation| {
        let attempts = Rc::new(Cell::new(0));
        let mut bytes = Vec::new();
        let error = CliOutput::new(EmptyWriteOutput {
            token: token.clone(),
            attempts: Rc::clone(&attempts),
        })
        .write_to(None, false, &mut bytes, &cancellation)
        .unwrap_err();
        assert_eq!(attempts.get(), 1);
        assert!(bytes.is_empty());
        assert_eq!(
            error.as_cancellation().unwrap().reason,
            "empty output cancelled"
        );
        Ok(())
    })
}

#[test]
fn cancellation_is_recovered_through_a_renderers_custom_error_source() -> eyre::Result<()> {
    let token = CancellationToken::new();
    token.with_scope(|cancellation| {
        let mut writer = CancelAfterPrefixWriter {
            token: token.clone(),
            bytes: Vec::new(),
            limit_per_write: 3,
            cancel_after: 3,
            write_calls: 0,
            flushed: false,
        };
        let error = CliOutput::new(WrappedWriteOutput)
            .write_to(None, false, &mut writer, &cancellation)
            .unwrap_err();
        assert_eq!(writer.bytes, b"ent");
        assert_eq!(
            error.downcast_ref::<CancellationError>().unwrap().reason,
            "output prefix was sufficient"
        );
        assert!(error.downcast_ref::<RendererWriteError>().is_some());
        assert!(error.chain().any(|cause| cause.is::<io::Error>()));
        Ok(())
    })
}

#[test]
fn incremental_output_stops_mid_record_and_retains_the_typed_cancellation() -> eyre::Result<()> {
    let token = CancellationToken::new();
    token.with_scope(|cancellation| {
        let attempts = Rc::new(Cell::new(0));
        let output = CliOutput::new(IncrementalOutput {
            attempts: Rc::clone(&attempts),
        })
        .with_result(Err(eyre::eyre!("deferred command failure")));
        let mut writer = CancelAfterPrefixWriter {
            token: token.clone(),
            bytes: Vec::new(),
            limit_per_write: 3,
            cancel_after: 9,
            write_calls: 0,
            flushed: false,
        };
        let error = output
            .write_to(None, false, &mut writer, &cancellation)
            .unwrap_err();
        assert_eq!(writer.bytes, b"entry\nent");
        assert_eq!(attempts.get(), 2);
        assert_eq!(writer.write_calls, 3);
        assert_eq!(
            error.as_cancellation().unwrap().reason,
            "output prefix was sufficient"
        );
        Ok(())
    })
}

#[test]
fn one_large_renderer_write_is_interruptible_before_the_whole_buffer_is_written() -> eyre::Result<()>
{
    let token = CancellationToken::new();
    token.with_scope(|cancellation| {
        let buffer = vec![b'x'; 1024 * 1024];
        let expected_length = buffer.len();
        let mut writer = CancelAfterPrefixWriter {
            token: token.clone(),
            bytes: Vec::new(),
            limit_per_write: usize::MAX,
            cancel_after: 1,
            write_calls: 0,
            flushed: false,
        };
        let error = CliOutput::new(LargeBuffer(buffer))
            .write_to(None, false, &mut writer, &cancellation)
            .unwrap_err();
        assert!(!writer.bytes.is_empty());
        assert!(writer.bytes.len() < expected_length);
        assert_eq!(writer.write_calls, 1);
        assert!(error.is_cancellation());
        Ok(())
    })
}

#[test]
fn a_supplied_deferral_allows_output_and_flush_to_finish() -> eyre::Result<()> {
    let token = CancellationToken::new();
    token.with_scope(|cancellation| {
        let mut writer = CancelAfterPrefixWriter {
            token: token.clone(),
            bytes: Vec::new(),
            limit_per_write: 3,
            cancel_after: 3,
            write_calls: 0,
            flushed: false,
        };
        cancellation.with_deferral("finish an already-required report", |protected| {
            CliOutput::facet_with_text(b"required report".to_vec(), |bytes, writer, scope| {
                for chunk in bytes.chunks(4) {
                    scope.bail_if_cancelled()?;
                    writer.write_all(chunk)?;
                }
                Ok(())
            })
            .write_to(Some(OutputFormat::Text), false, &mut writer, &protected)
        })?;
        assert!(token.is_cancelled());
        assert_eq!(writer.bytes, b"required report\n");
        assert!(writer.flushed);
        assert!(cancellation.bail_if_cancelled().is_err());
        Ok(())
    })
}

#[test]
fn custom_text_callbacks_stream_until_the_supplied_scope_accepts_cancellation() -> eyre::Result<()>
{
    let token = CancellationToken::new();
    token.with_scope(|cancellation| {
        let signal = token.clone();
        let iterations = Rc::new(Cell::new(0));
        let callback_iterations = Rc::clone(&iterations);
        let output =
            CliOutput::facet_with_text(vec![1, 2, 3, 4, 5], move |items, writer, scope| {
                for item in items {
                    scope.bail_if_cancelled()?;
                    write!(writer, "{item}")?;
                    callback_iterations.set(callback_iterations.get() + 1);
                    if callback_iterations.get() == 2 {
                        signal.request_cancel("text production cancelled");
                    }
                }
                Ok(())
            });
        let mut bytes = Vec::new();
        let error = output
            .write_to(Some(OutputFormat::Text), false, &mut bytes, &cancellation)
            .unwrap_err();
        assert_eq!(iterations.get(), 2);
        assert_eq!(bytes, b"12");
        assert_eq!(
            error.as_cancellation().unwrap().reason,
            "text production cancelled"
        );
        Ok(())
    })
}

#[test]
fn custom_text_callbacks_stop_producing_when_the_destination_requests_cancellation()
-> eyre::Result<()> {
    let token = CancellationToken::new();
    token.with_scope(|cancellation| {
        let attempts = Rc::new(Cell::new(0));
        let callback_attempts = Rc::clone(&attempts);
        let output = CliOutput::facet_with_text((), move |_, writer, _scope| {
            // A finite fuse prevents a hanging test if writer checkpoints regress.
            for _ in 0..10_000 {
                callback_attempts.set(callback_attempts.get() + 1);
                writer.write_all(b"entry\n")?;
            }
            eyre::bail!("custom text output failed to observe cancellation")
        });
        let mut writer = CancelAfterPrefixWriter {
            token: token.clone(),
            bytes: Vec::new(),
            limit_per_write: usize::MAX,
            cancel_after: 1,
            write_calls: 0,
            flushed: false,
        };
        let error = output
            .write_to(Some(OutputFormat::Text), false, &mut writer, &cancellation)
            .unwrap_err();

        assert!(
            attempts.get() <= 2,
            "the producer continued after cancellation"
        );
        assert_eq!(writer.write_calls, 1);
        assert_eq!(writer.bytes, b"entry\n");
        assert!(!writer.flushed);
        assert_eq!(
            error.as_cancellation().unwrap().reason,
            "output prefix was sufficient"
        );
        Ok(())
    })
}

enum FailureStage {
    Write,
    Flush,
    CancellationDuringFlush,
}

struct FailingWriter(FailureStage);

impl Write for FailingWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if matches!(self.0, FailureStage::Write) {
            return Err(io::Error::new(io::ErrorKind::BrokenPipe, "write failed"));
        }
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        match self.0 {
            FailureStage::Write => Ok(()),
            FailureStage::Flush => Err(io::Error::other("flush failed")),
            FailureStage::CancellationDuringFlush => Err(io::Error::other(CancellationError::new(
                "flush was cancelled",
            ))),
        }
    }
}

#[test]
fn write_and_flush_failures_take_precedence_over_the_deferred_command_result() -> eyre::Result<()> {
    CancellationToken::new().with_scope(|cancellation| {
        for (failure, expected) in [
            (FailureStage::Write, "write failed"),
            (FailureStage::Flush, "flush failed"),
        ] {
            let error = CliOutput::facet("synthetic output")
                .with_result(Err(eyre::eyre!("deferred command failure")))
                .write_to(None, false, FailingWriter(failure), &cancellation)
                .unwrap_err();
            assert!(format!("{error:#}").contains(expected), "{error:#}");
            assert!(error.downcast_ref::<io::Error>().is_some());
        }
        Ok(())
    })
}

#[test]
fn cancellation_inside_a_flush_io_error_remains_a_typed_cancellation() -> eyre::Result<()> {
    CancellationToken::new().with_scope(|cancellation| {
        let error = CliOutput::new(SelectedFormat)
            .with_result(Err(eyre::eyre!("deferred command failure")))
            .write_to(
                None,
                false,
                FailingWriter(FailureStage::CancellationDuringFlush),
                &cancellation,
            )
            .unwrap_err();
        assert_eq!(
            error.downcast_ref::<CancellationError>().unwrap().reason,
            "flush was cancelled"
        );
        assert_eq!(
            error.downcast_ref::<io::Error>().unwrap().kind(),
            io::ErrorKind::Other
        );
        assert!(format!("{error:#}").contains("failed to flush command output"));
        Ok(())
    })
}

#[test]
fn json_output_stops_when_its_destination_requests_cancellation() -> eyre::Result<()> {
    #[derive(Facet)]
    struct FlattenedReport {
        name: String,
        #[facet(flatten)]
        entries: BTreeMap<String, u32>,
    }

    const CANCELLATION_KEY: &[u8] = b"\"entry_05\"";

    let token = CancellationToken::new();
    token.with_scope(|cancellation| {
        let report = FlattenedReport {
            name: String::from("flattened output"),
            entries: (0..32)
                .map(|index| (format!("entry_{index:02}"), index))
                .collect(),
        };
        let expected = facet_json::to_vec_pretty(&report)?;
        let cancel_after = expected
            .windows(CANCELLATION_KEY.len())
            .position(|window| window == CANCELLATION_KEY)
            .expect("the fixture must contain the selected flattened key")
            + CANCELLATION_KEY.len();
        let mut writer = CancelAfterPrefixWriter {
            token: token.clone(),
            bytes: Vec::new(),
            limit_per_write: usize::MAX,
            cancel_after,
            write_calls: 0,
            flushed: false,
        };
        let error = CliOutput::facet(report)
            .write_to(Some(OutputFormat::Json), false, &mut writer, &cancellation)
            .unwrap_err();

        assert!(!writer.bytes.is_empty());
        assert!(writer.bytes.len() < expected.len());
        assert!(expected.starts_with(&writer.bytes));
        assert!(
            writer
                .bytes
                .windows(CANCELLATION_KEY.len())
                .any(|window| window == CANCELLATION_KEY)
        );
        assert!(!writer.flushed);
        assert_eq!(
            error.downcast_ref::<CancellationError>().unwrap().reason,
            "output prefix was sufficient"
        );
        assert!(error.downcast_ref::<io::Error>().is_some());
        assert!(
            format!("{error:#}").contains("failed to serialize or write command output as JSON")
        );
        Ok(())
    })
}

#[test]
fn a_protected_json_writer_finishes_after_the_raw_signal_is_requested() -> eyre::Result<()> {
    let token = CancellationToken::new();
    token.with_scope(|cancellation| {
        let mut expected = facet_json::to_vec_pretty(&[1_u32, 2, 3])?;
        expected.push(b'\n');
        let mut writer = CancelAfterPrefixWriter {
            token: token.clone(),
            bytes: Vec::new(),
            limit_per_write: usize::MAX,
            cancel_after: 1,
            write_calls: 0,
            flushed: false,
        };

        cancellation.with_deferral("finish a required JSON artifact", |protected| {
            CliOutput::facet([1_u32, 2, 3]).write_to(
                Some(OutputFormat::Json),
                false,
                &mut writer,
                &protected,
            )
        })?;

        assert!(token.is_cancelled());
        assert_eq!(writer.bytes, expected);
        assert!(writer.flushed);
        assert!(cancellation.bail_if_cancelled().is_err());
        Ok(())
    })
}
