use eyre::Context;
use std::io;
use std::io::Write;
use teamy_cancellation::CancellableWriter;
use teamy_cancellation::CancellationScope;

/// Tracks the emitted suffix to avoid adding a duplicate final newline.
pub(super) struct OutputWriter<'a, W: ?Sized> {
    writer: CancellableWriter<'a, W>,
    last_byte: Option<u8>,
}

impl<'a, W: Write + ?Sized> OutputWriter<'a, W> {
    pub fn new(writer: &'a mut W, cancellation: &'a CancellationScope) -> Self {
        Self {
            writer: CancellableWriter::new(writer, cancellation),
            last_byte: None,
        }
    }

    pub fn finish(&mut self) -> eyre::Result<()> {
        if self.last_byte != Some(b'\n') {
            self.write_all(b"\n")
                .wrap_err("failed to terminate command output")?;
        }
        self.flush().wrap_err("failed to flush command output")
    }
}

impl<W: Write + ?Sized> Write for OutputWriter<'_, W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let written = self.writer.write(bytes)?;
        if written != 0 {
            self.last_byte = Some(bytes[written - 1]);
        }
        Ok(written)
    }

    fn write_all(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.writer.write_all(bytes)?;
        if let Some(last_byte) = bytes.last() {
            self.last_byte = Some(*last_byte);
        }
        Ok(())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}
