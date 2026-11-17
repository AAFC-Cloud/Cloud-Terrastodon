use std::borrow::Cow;

/// An SVG with a display name and owning feature category.
///
/// Embedded assets return borrowed static text. Dynamic implementations return
/// owned text, so each result can outlive the icon instance.
pub trait SvgIcon {
    /// Return the SVG document.
    fn svg(&self) -> Cow<'static, str>;

    /// Return the human-readable display name within its collection.
    fn name(&self) -> Cow<'static, str>;

    /// Return the Cargo feature category that owns this icon.
    fn category(&self) -> Cow<'static, str>;
}
