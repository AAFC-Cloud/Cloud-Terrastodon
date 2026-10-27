/// Prepare untrusted text for a single terminal line before applying styling.
///
/// Control characters become spaces so source text cannot move the cursor,
/// start terminal escape sequences, or insert additional lines. This is not an
/// ANSI parser: apply it to source values, before adding intentional styling.
pub trait TerminalTextExt {
    fn sanitized_for_terminal(&self) -> String;
}

impl<T: AsRef<str> + ?Sized> TerminalTextExt for T {
    fn sanitized_for_terminal(&self) -> String {
        self.as_ref()
            .chars()
            .map(|character| {
                if character.is_control() {
                    ' '
                } else {
                    character
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::TerminalTextExt;

    #[test]
    fn terminal_controls_cannot_change_the_display_and_unicode_survives() {
        let text = "waffles\n\r\t\u{1b}[2J\u{7}\u{85} café 界 e\u{301}";
        assert_eq!(
            text.sanitized_for_terminal(),
            "waffles    [2J   café 界 e\u{301}"
        );
    }
}
