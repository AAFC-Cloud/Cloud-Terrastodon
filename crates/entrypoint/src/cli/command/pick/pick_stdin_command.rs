use crate::cli::pick::pick_command::PickCommonArgs;
use crate::cli::pick::pick_command::write_selected_lines;
use crate::serde_json_isolation::Value;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use std::io::Read;
use strum::Display;
use tracing::instrument;
use tracing::trace_span;

#[derive(Debug, Clone, facet::Facet, Display)]
#[strum(serialize_all = "kebab-case")]
#[repr(u8)]
pub enum InputFormat {
    /// Force JSON array mode
    Json,
    /// Force newline-delimited lines mode
    Lines,
}

/// Pick from stdin.
#[derive(facet::Facet, Debug, Clone, Default)]
pub struct PickStdinArgs {
    /// Stdin parsing mode (json | lines). Defaults to auto-detect when omitted.
    #[facet(figue::named)]
    pub input_format: Option<InputFormat>,
}

fn resolve_input_format(input_format: Option<InputFormat>, stdin_buf: &str) -> InputFormat {
    match input_format {
        None => {
            if let Some(first_non_ws) = stdin_buf.chars().find(|c| !c.is_whitespace()) {
                if first_non_ws == '[' {
                    InputFormat::Json
                } else {
                    InputFormat::Lines
                }
            } else {
                InputFormat::Lines
            }
        }
        Some(mode) => mode,
    }
}

impl PickStdinArgs {
    #[instrument(name = "pick_stdin_command", skip_all)]
    pub(crate) async fn invoke(self, common: PickCommonArgs) -> Result<()> {
        self.invoke_inner(common).await
    }

    async fn invoke_inner(self, common: PickCommonArgs) -> Result<()> {
        let stdin_buf = trace_span!("pick_stdin_read").in_scope(|| {
            let mut stdin_buf = String::new();
            std::io::stdin().read_to_string(&mut stdin_buf)?;
            Ok::<_, std::io::Error>(stdin_buf)
        })?;

        let input_format = trace_span!("pick_stdin_detect_format")
            .in_scope(|| resolve_input_format(self.input_format, &stdin_buf));

        match input_format {
            InputFormat::Json => {
                let stdin_json: Vec<Value> = trace_span!("pick_stdin_parse_json")
                    .in_scope(|| crate::serde_json_isolation::from_str(&stdin_buf))?;

                let choices = trace_span!("pick_stdin_build_json_choices").in_scope(|| {
                    stdin_json
                        .into_iter()
                        .map(|value| {
                            let key = common.query_engine.query(&value, &common.query)?;
                            Ok(Choice { key, value })
                        })
                        .collect::<Result<Vec<_>>>()
                })?;

                let rtn = PickerTui::<_>::new()
                    .set_auto_accept(common.auto_accept)
                    .set_query(common.default_query.unwrap_or_default())
                    .pick_inner(!common.single, choices)
                    .await?;

                crate::serde_json_isolation::to_writer_pretty(std::io::stdout(), &rtn)?;
            }
            InputFormat::Lines => {
                let choices = trace_span!("pick_stdin_build_line_choices").in_scope(|| {
                    stdin_buf
                        .lines()
                        .filter(|s| !s.is_empty())
                        .map(|line| Choice {
                            key: line.to_string(),
                            value: line.to_string(),
                        })
                        .collect::<Vec<_>>()
                });

                let rtn = PickerTui::<_>::new()
                    .set_auto_accept(common.auto_accept)
                    .set_query(common.default_query.unwrap_or_default())
                    .pick_inner(!common.single, choices)
                    .await?;

                write_selected_lines(&rtn)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::cli::pick::InputFormat;
    use crate::cli::pick::pick_stdin_command::resolve_input_format;

    #[test]
    fn resolves_pick_mode_from_stdin_content() {
        assert!(matches!(
            resolve_input_format(None, "[1,2,3]"),
            InputFormat::Json
        ));
        assert!(matches!(
            resolve_input_format(None, "hello\nworld"),
            InputFormat::Lines
        ));
        assert!(matches!(
            resolve_input_format(Some(InputFormat::Lines), "[1]"),
            InputFormat::Lines
        ));
    }
}
