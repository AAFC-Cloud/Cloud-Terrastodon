use eyre::Report;

/// Distinguishes JSON decoding failures from application mapping failures.
pub(crate) enum CommandOutputDecodeError {
    Deserialize(Report),
    Map(Report),
}
