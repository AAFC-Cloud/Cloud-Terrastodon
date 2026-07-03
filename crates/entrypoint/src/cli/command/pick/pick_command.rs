use crate::cli::pick::pick_fs_command::PickFsArgs;
use crate::cli::pick::pick_stdin_command::PickStdinArgs;
use crate::serde_json_isolation::Value;
use eyre::Result;
use jmespath::Variable;
use jsonpath_rust::JsonPath;
use std::io::IsTerminal;
use strum::Display;

/// Pick from stdin or the filesystem.
#[derive(facet::Facet, Debug, Clone, Default)]
pub struct PickArgs {
    #[facet(flatten)]
    pub common: PickCommonArgs,
    #[facet(figue::subcommand, default)]
    pub command: Option<PickCommand>,
}

#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum PickCommand {
    /// Pick from stdin
    Stdin(PickStdinArgs),
    /// Pick from the current working directory
    Fs(PickFsArgs),
}

#[derive(facet::Facet, Debug, Clone, Default)]
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

#[derive(Debug, Clone, facet::Facet, Display, Default)]
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
            return crate::serde_json_isolation::to_string(data);
        }
        match self {
            QueryEngine::JsonPath => crate::serde_json_isolation::to_string(&data.query(query)?),
            QueryEngine::JmesPath => {
                let expr = jmespath::compile(query)?;
                let result = expr.search(data)?;
                match *result {
                    Variable::String(ref s) => Ok(s.to_owned()),
                    _ => crate::serde_json_isolation::to_string(&result),
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
    pub async fn invoke(self) -> Result<()> {
        let command = self
            .command
            .unwrap_or_else(|| resolve_default_pick_command(std::io::stdin().is_terminal()));

        command.invoke(self.common).await
    }
}

impl PickCommand {
    pub(crate) async fn invoke(self, common: PickCommonArgs) -> Result<()> {
        match self {
            PickCommand::Stdin(args) => args.invoke(common).await,
            PickCommand::Fs(args) => args.invoke(common).await,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::cli::pick::PickCommand;
    use crate::cli::pick::QueryEngine;
    use crate::cli::pick::pick_command::resolve_default_pick_command;
    use crate::serde_json_isolation::json;

    #[test]
    fn query_engine_examples_work() -> eyre::Result<()> {
        let example_obj = json!({
            "name": "Alice",
            "age": 30,
            "description": "A software developer"
        });
        let cases = [
            (QueryEngine::JsonPath, "$..['name', 'description']"),
            (QueryEngine::JmesPath, "[name, age]"),
            (QueryEngine::Liquid, "{{ name }} {{ description }}"),
        ];
        for (engine, example) in cases {
            let result = engine.query(&example_obj, example)?;
            println!(
                "Engine: {}, Example: {}, Result: {}",
                engine, example, result
            );
        }
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
