# cloud_terrastodon_command

Command running helpers for the Cloud Terrastodon project.

This crate provides utilities for building, running, and managing external commands within the Cloud Terrastodon project. It includes features for:

- Specifying different command kinds (Azure CLI, Tofu, VSCode, Echo, Pwsh).
- Building command arguments and environment variables.
- Handling file arguments for commands like Azure CLI.
- Configuring output behavior (capture or display).
- Implementing retry logic for authentication failures.
- Caching command output for improved performance.
- Sending content to command stdin.
- Writing command failures and successes to files for debugging and caching.

## Usage

Here's a basic example of how to use `CommandBuilder`:

```rust
use cloud_terrastodon_command::{CommandBuilder, CommandKind};
use eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Create a new command builder for Azure CLI
    let mut command = CommandBuilder::new(CommandKind::AzureCLI);

    // Add arguments
    command.args(["account", "show"]);

    // Run the command and capture the output
    let output = command.run_raw().await?;

    // Print the output
    println!("Status: {}", output.status);
    println!("Stdout: {}", output.stdout);
    println!("Stderr: {}", output.stderr);

    Ok(())
}
```

### Caching

You can use the caching feature to avoid re-running commands with the same arguments:

```rust
use cloud_terrastodon_command::{CommandBuilder, CommandKind};
use eyre::Result;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    let cache_path = PathBuf::from_iter(["az","account","show"]); // this key is joined to the Cloud Terrastodon cache dir

    // Create a command with caching enabled
    let mut command = CommandBuilder::new(CommandKind::AzureCLI);
    command.args(["account", "show"]);
    command.use_cache_dir(cache_path);

    // The first time this runs, it will execute the command and cache the output.
    // Subsequent runs with the same command builder will use the cached output.
    let output = command.run_raw().await?;

    println!("Output from cached command: {}", output.stdout);

    Ok(())
}
```

### Handling File Arguments

Some commands, like Azure CLI, accept arguments from files. `CommandBuilder` simplifies this:

```rust
use cloud_terrastodon_command::{CommandBuilder, CommandKind};
use eyre::Result;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    let query_content = r#"
resourcecontainers
| summarize count()
"#;

    let mut command = CommandBuilder::new(CommandKind::AzureCLI);
    command.args(["graph", "query", "--graph-query"]);
    // Use file_arg to pass the query content in a temporary file
    command.file_arg("query.kql", query_content.to_string());

    let output = command.run_raw().await?;

    println!("Graph query result: {}", output.stdout);

    Ok(())
}
```

### Sending Stdin

You can send content to a command's standard input:

```rust
use cloud_terrastodon_command::{CommandBuilder, CommandKind};
use eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let mut command = CommandBuilder::new(CommandKind::Pwsh);
    command.args(["-NoProfile", "-Command", "-"]); // "-" in pwsh means read from stdin
    command.send_stdin("Write-Host 'Hello from stdin'");

    let output = command.run_raw().await?;

    println!("Pwsh output from stdin: {}", output.stdout);

    Ok(())
}
```
