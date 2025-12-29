use clap::Args;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use jmespath::Variable;
use jsonpath_rust::JsonPath;
use serde_json::Value;
use std::io::Read;
use std::io::Write;
use strum::Display;

#[derive(Debug, Clone, clap::ValueEnum, Display, Default)]
#[strum(serialize_all = "kebab-case")]
pub enum QueryEngine {
    /// See https://crates.io/crates/jsonpath-rust for details.
    /// Example: `$..['name', 'description']`
    JsonPath,
    /// See https://jmespath.org/ and https://crates.io/crates/jmespath for details.
    /// Example: `[name, age]`
    #[default]
    JmesPath,
    /// See https://github.com/cobalt-org/liquid-rust for details.
    /// Example: `{{ name }} {{ description }}`
    Liquid,
}
impl QueryEngine {
    pub fn query(&self, data: &Value, query: &str) -> Result<String> {
        match self {
            QueryEngine::JsonPath => {
                // TODO: pretty print since picker tui should support multi-line keys
                Ok(serde_json::to_string(&data.query(query)?)?)
            },
            QueryEngine::JmesPath => {
                let expr = jmespath::compile(query)?;
                let result = expr.search(data)?;
                match *result {
                    Variable::String(ref s) => Ok(s.to_owned()),
                    // TODO: pretty print since picker tui should support multi-line keys
                    _ => Ok(serde_json::to_string(&result)?),
                }
            }
            QueryEngine::Liquid => {
                let template = liquid::ParserBuilder::with_stdlib().build()?.parse(query)?;
                let globals = liquid::to_object(data)?;
                let rendered = template.render(&globals)?;
                Ok(rendered)
            }
        }
    }
}

#[derive(Debug, Clone, clap::ValueEnum, Display, Default)]
#[strum(serialize_all = "kebab-case")]
pub enum PickMode {
    /// Automatic detection: JSON array if stdin starts with `[`, otherwise lines
    #[default]
    Auto,
    /// Force JSON array mode
    Json,
    /// Force newline-delimited lines mode
    Lines,
}

/// Pick from options supplied on stdin
#[derive(Args, Debug, Clone, Default)]
pub struct PickArgs {
    /// Query to be passed to the query engine, determines the display value for the choices
    #[clap(long, short = 'q', default_value = "*")]
    pub query: String,
    /// Query engine to use
    #[clap(long, short = 'e', default_value_t = Default::default())]
    pub engine: QueryEngine,
    /// Input parsing mode (auto | json | lines)
    #[clap(long, value_enum, default_value_t = PickMode::Auto)]
    pub mode: PickMode,
    /// Allow multiple selections
    #[clap(long, short = 'm')]
    pub many: bool,
    /// Automatically accept if there is only one choice
    #[clap(long, short = 'a')]
    pub auto_accept: bool,
    /// Default search for the TUI
    #[clap(long, short = 'd')]
    pub default_search: Option<String>,
}

impl PickArgs {
    pub async fn invoke(self) -> Result<()> {
        // read all stdin into a buffer then resolve mode (Auto -> Json|Lines) and branch
        let mut stdin_buf = String::new();
        std::io::stdin().read_to_string(&mut stdin_buf)?;

        // Resolve Auto to a concrete mode so downstream logic can match on Json/Lines only
        let mut mode = self.mode;
        if let PickMode::Auto = mode {
            mode = if let Some(first_non_ws) = stdin_buf.chars().find(|c| !c.is_whitespace()) {
                if first_non_ws == '[' {
                    PickMode::Json
                } else {
                    PickMode::Lines
                }
            } else {
                // empty input -> prefer Lines (results in empty choices)
                PickMode::Lines
            };
        }

        match mode {
            PickMode::Json => {
                let stdin_json: Vec<Value> = serde_json::from_str(&stdin_buf)?;

                let mut choices = Vec::with_capacity(stdin_json.len());
                for value in stdin_json.iter() {
                    let key = self.engine.query(value, &self.query)?;
                    choices.push(Choice { key, value });
                }

                let rtn = PickerTui::new()
                    .set_auto_accept(self.auto_accept)
                    .set_query(self.default_search.unwrap_or_default())
                    .pick_inner(self.many, choices)?;

                serde_json::to_writer_pretty(std::io::stdout(), &rtn)?;
            }
            PickMode::Lines => {
                let lines: Vec<String> = stdin_buf
                    .lines()
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect();

                let mut choices = Vec::with_capacity(lines.len());
                for line in lines.iter() {
                    choices.push(Choice {
                        key: line.clone(),
                        value: line.clone(),
                    });
                }

                let rtn = PickerTui::new()
                    .set_auto_accept(self.auto_accept)
                    .set_query(self.default_search.unwrap_or_default())
                    .pick_inner(self.many, choices)?;

                let stdout = std::io::stdout();
                let mut out = stdout.lock();
                for line in rtn.iter() {
                    writeln!(out, "{}", line)?;
                }
            }
            PickMode::Auto => unreachable!("PickMode::Auto should be resolved before matching"),
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::cli::pick::QueryEngine;
    use clap::ValueEnum;
    use serde_json::json;

    #[test]
    fn it_works() -> eyre::Result<()> {
        let example_obj = json!({
            "name": "Alice",
            "age": 30,
            "description": "A software developer"
        });
        for engine in QueryEngine::value_variants() {
            let possible_value = engine.to_possible_value().unwrap();
            let styled_help_text = possible_value.get_help().unwrap();
            let mut found_example = false;
            for line in styled_help_text.to_string().lines() {
                if let Some(idx) = line.find("Example: `") {
                    let example = &line[idx + "Example: `".len()..line.len() - 1];
                    let result = engine.query(&example_obj, example)?;
                    println!(
                        "Engine: {}, Example: {}, Result: {}",
                        engine, example, result
                    );
                    found_example = true;
                }
            }
            if !found_example {
                eyre::bail!("No example found for engine {}", engine);
            }
        }
        Ok(())
    }
}
