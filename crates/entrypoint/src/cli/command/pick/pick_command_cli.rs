use super::json_path_value::JsonPathValue;
use super::liquid_query_value::liquid_query_value;
use crate::cli::pick::pick_fs_command_cli::PickFsArgs;
use crate::cli::pick::pick_stdin_command_cli::PickStdinArgs;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use facet::Facet;
use facet_value::Value;
use jmespath::Variable;
use jsonpath_rust::JsonPath;
use liquid::ParserBuilder;
use liquid::model::Value as LiquidValue;
use std::io::IsTerminal;
use strum::Display;
use tracing::instrument;

/// Pick from stdin or the filesystem.
#[derive(Facet, Debug, Clone, Default)]
pub struct PickArgs {
    #[facet(flatten)]
    pub common: PickCommonArgs,
    #[facet(figue::subcommand, default)]
    pub command: Option<PickCommand>,
}

#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum PickCommand {
    /// Pick from stdin
    Stdin(PickStdinArgs),
    /// Pick from the current working directory
    Fs(PickFsArgs),
}

#[derive(Facet, Debug, Clone, Default)]
pub struct PickCommonArgs {
    /// Query to be passed to the query engine, determines the display value for the choices
    #[facet(figue::named, figue::short = 'q', default = String::new())]
    pub query: String,
    /// Query engine to use
    #[facet(figue::named, figue::short = 'e', default = QueryEngine::default())]
    pub query_engine: QueryEngine,
    /// Restrict to a single selected item (no multi-select)
    #[facet(figue::named, figue::short = 'm')]
    pub single: bool,
    /// Automatically accept if there is only one choice
    #[facet(figue::named, figue::short = 'a')]
    pub auto_accept: bool,
    /// Default query for the TUI
    #[facet(figue::named, figue::short = 'd')]
    pub default_query: Option<String>,
}

#[derive(Debug, Clone, Facet, Display, Default)]
#[strum(serialize_all = "kebab-case")]
#[repr(u8)]
pub enum QueryEngine {
    /// See https://crates.io/crates/jsonpath-rust for details.
    /// Example: `$..['name', 'description']`
    JsonPath,
    /// See https://jmespath.org/ and https://crates.io/crates/jmespath for details.
    /// Example: `[name, age]`
    JmesPath,
    /// See https://github.com/cobalt-org/liquid-rust for details.
    /// Example: `{{ name }} {{ description }}`
    #[default]
    Liquid,
}

impl QueryEngine {
    pub fn query(&self, data: &Value, query: &str) -> Result<String> {
        if query.is_empty() {
            return Ok(facet_json::to_string(data)?);
        }
        match self {
            QueryEngine::JsonPath => {
                let data = JsonPathValue::from(data);
                Ok(facet_json::to_string(&data.query(query)?)?)
            }
            QueryEngine::JmesPath => {
                let expr = jmespath::compile(query)?;
                let data = Variable::from_json(&facet_json::to_string(data)?)
                    .map_err(|error| eyre::eyre!("converting input for JMESPath: {error}"))?;
                let result = expr.search(data)?;
                match *result {
                    Variable::String(ref s) => Ok(s.to_owned()),
                    _ => Ok(result.to_string()),
                }
            }
            QueryEngine::Liquid => {
                let template = ParserBuilder::with_stdlib().build()?.parse(query)?;
                let LiquidValue::Object(globals) = liquid_query_value(data)? else {
                    eyre::bail!("Liquid query input must be a JSON object");
                };
                let rendered = template.render(&globals)?;
                Ok(rendered)
            }
        }
    }
}

pub(crate) fn write_selected_lines(lines: &[String]) -> Result<()> {
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    for line in lines {
        use std::io::Write;
        writeln!(out, "{}", line)?;
    }
    Ok(())
}

fn resolve_default_pick_command(stdin_is_terminal: bool) -> PickCommand {
    if stdin_is_terminal {
        PickCommand::Fs(PickFsArgs::default())
    } else {
        PickCommand::Stdin(PickStdinArgs::default())
    }
}

impl PickArgs {
    #[instrument(name = "pick_command", skip_all)]
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<()> {
        let command = self
            .command
            .unwrap_or_else(|| resolve_default_pick_command(std::io::stdin().is_terminal()));

        command.invoke(self.common, cancellation).await
    }
}

impl PickCommand {
    pub(crate) async fn invoke(
        self,
        common: PickCommonArgs,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        match self {
            PickCommand::Stdin(args) => args.invoke(common, cancellation).await,
            PickCommand::Fs(args) => args.invoke(common, cancellation).await,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::cli::pick::PickCommand;
    use crate::cli::pick::QueryEngine;
    use crate::cli::pick::pick_command_cli::resolve_default_pick_command;
    use facet_value::Value;

    #[test]
    fn query_engine_examples_work() -> eyre::Result<()> {
        let example_obj: Value = facet_json::from_str(
            r#"{"name":"Alice","age":30,"description":"A software developer"}"#,
        )?;
        let cases = [
            (
                QueryEngine::JsonPath,
                "$..['name', 'description']",
                r#"["Alice","A software developer"]"#,
            ),
            (QueryEngine::JmesPath, "[name, age]", r#"["Alice",30]"#),
            (QueryEngine::JmesPath, "name", "Alice"),
            (
                QueryEngine::Liquid,
                "{{ name }} {{ description }}",
                "Alice A software developer",
            ),
        ];
        for (engine, example, expected) in cases {
            let result = engine.query(&example_obj, example)?;
            assert_eq!(result, expected, "{engine}: {example}");
        }
        Ok(())
    }

    #[test]
    fn jsonpath_keeps_custom_membership_and_number_types() -> eyre::Result<()> {
        let input: Value = facet_json::from_str(r#"{"values":[1,1.0,"1",2],"allowed":[1]}"#)?;
        assert_eq!(
            QueryEngine::JsonPath.query(&input, "$.values[?in(@, $.allowed)]")?,
            "[1]"
        );
        Ok(())
    }

    #[test]
    fn resolves_default_pick_command_based_on_stdin_terminal() {
        assert!(matches!(
            resolve_default_pick_command(false),
            PickCommand::Stdin(_)
        ));
        assert!(matches!(
            resolve_default_pick_command(true),
            PickCommand::Fs(_)
        ));
    }
}
