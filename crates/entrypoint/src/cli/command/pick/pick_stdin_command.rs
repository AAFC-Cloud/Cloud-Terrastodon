use crate::cli::pick::pick_command::PickCommonArgs;
use crate::cli::pick::pick_command::write_selected_lines;
use clap::Args;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use serde_json::Value;
use std::io::Read;
use strum::Display;

#[derive(Debug, Clone, clap::ValueEnum, Display)]
#[strum(serialize_all = "kebab-case")]
pub enum InputFormat {
    /// Force JSON array mode
    Json,
    /// Force newline-delimited lines mode
    Lines,
}

/// Pick from stdin.
#[derive(Args, Debug, Clone, Default)]
pub struct PickStdinArgs {
    /// Stdin parsing mode (json | lines). Defaults to auto-detect when omitted.
    #[clap(long, value_enum)]
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
    pub(crate) async fn invoke(self, common: PickCommonArgs) -> Result<()> {
        let mut stdin_buf = String::new();
        std::io::stdin().read_to_string(&mut stdin_buf)?;

        match resolve_input_format(self.input_format, &stdin_buf) {
            InputFormat::Json => {
                let stdin_json: Vec<Value> = serde_json::from_str(&stdin_buf)?;

                let mut choices = Vec::with_capacity(stdin_json.len());
                for value in &stdin_json {
                    let key = common.query_engine.query(value, &common.query)?;
                    choices.push(Choice { key, value });
                }

                let rtn = PickerTui::new()
                    .set_auto_accept(common.auto_accept)
                    .set_query(common.default_query.unwrap_or_default())
                    .pick_inner(!common.single, choices)?;

                serde_json::to_writer_pretty(std::io::stdout(), &rtn)?;
            }
            InputFormat::Lines => {
                let lines: Vec<String> = stdin_buf
                    .lines()
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect();

                let mut choices = Vec::with_capacity(lines.len());
                for line in &lines {
                    choices.push(Choice {
                        key: line.clone(),
                        value: line.clone(),
                    });
                }

                let rtn = PickerTui::new()
                    .set_auto_accept(common.auto_accept)
                    .set_query(common.default_query.unwrap_or_default())
                    .pick_inner(!common.single, choices)?;

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
        assert!(matches!(resolve_input_format(None, "[1,2,3]"), InputFormat::Json));
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
