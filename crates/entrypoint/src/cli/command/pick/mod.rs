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
    /// See https://crates.io/crates/jsonpath-rust for syntax details.
    /// For example, `$[*].name` to select all `name` fields from an array of objects.
    JsonPath,
    /// See https://jmespath.org/ and https://crates.io/crates/jmespath for syntax details.
    #[default]
    JmesPath,
}
impl QueryEngine {
    pub fn query(&self, data: &Value, query: &str) -> Result<String> {
        match self {
            QueryEngine::JsonPath => Ok(serde_json::to_string(&data.query(query)?)?),
            QueryEngine::JmesPath => {
                let expr = jmespath::compile(query)?;
                let result = expr.search(data)?;
                match *result {
                    Variable::String(ref s) => Ok(s.to_owned()),
                    _ => Ok(serde_json::to_string(&result)?),
                }
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
    #[clap(long, short = 'q', default_value = "$[*]")]
    pub query: String,
    /// Query engine to use
    #[clap(long, short = 'e', default_value_t = QueryEngine::JmesPath)]
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

                let rtn = PickerTui::<&Value>::new(choices)
                    .set_auto_accept(self.auto_accept)
                    .set_query(self.default_search.unwrap_or_default())
                    .pick_inner(self.many)?;

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

                let rtn = PickerTui::<String>::new(choices)
                    .set_auto_accept(self.auto_accept)
                    .set_query(self.default_search.unwrap_or_default())
                    .pick_inner(self.many)?;

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
