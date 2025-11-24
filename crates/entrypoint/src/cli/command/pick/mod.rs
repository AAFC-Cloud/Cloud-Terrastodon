use clap::Args;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use jmespath::Variable;
use jsonpath_rust::JsonPath;
use serde_json::Value;
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

/// Pick from options supplied on stdin
#[derive(Args, Debug, Clone, Default)]
pub struct PickArgs {
    /// Query to be passed to the query engine, determines the display value for the choices
    #[clap(long, short = 'q', default_value = "$[*]")]
    pub query: String,
    /// Query engine to use
    #[clap(long, short = 'e', default_value_t = QueryEngine::JmesPath)]
    pub engine: QueryEngine,
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
        // read json from stdin
        let stdin_json = serde_json::from_reader::<_, Vec<Value>>(std::io::stdin())?;

        // transform using user query
        let mut choices = Vec::with_capacity(stdin_json.len());
        for value in stdin_json.iter() {
            let key = self.engine.query(value, &self.query)?;
            choices.push(Choice { key, value });
        }

        // launch picker tui to get results
        let rtn = PickerTui::<&Value>::new(choices)
            .set_auto_accept(self.auto_accept)
            .set_query(self.default_search.unwrap_or_default())
            .pick_inner(self.many)?;

        // write to stdout as pretty json
        serde_json::to_writer_pretty(std::io::stdout(), &rtn)?;

        Ok(())
    }
}
