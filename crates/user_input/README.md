# cloud_terrastodon_user_input

## English

Helper functions for interacting with users in the terminal.

This crate provides convenient functions for prompting the user for input, including:

*   Getting a single line of input.
*   Asking a yes/no question.
*   Allowing the user to pick one or many items from a list using `fzf`.

###  Prerequisites

This crate relies on `fzf` being installed on the system. Please refer to the `fzf` installation instructions for your operating system:

*   **Windows**: [https://github.com/junegunn/fzf?tab=readme-ov-file#windows-packages](https://github.com/junegunn/fzf?tab=readme-ov-file#windows-packages)
*   **Linux/macOS**: [https://github.com/junegunn/fzf?tab=readme-ov-file#linux-packages](https://github.com/junegunn/fzf?tab=readme-ov-file#linux-packages)

## Française

Fonctions d'aide pour interagir avec les utilisateurs dans le terminal.

Ce crate fournit des fonctions pratiques pour demander l'avis de l'utilisateur, notamment :

*   Obtenir une seule ligne de saisie.
*   Poser une question par oui ou par non.
*   Permettre à l'utilisateur de choisir un ou plusieurs éléments d'une liste à l'aide de `fzf`.

### Prérequis

Cette caisse dépend de l'installation de `fzf` sur le système. Veuillez vous référer aux instructions d'installation de `fzf` pour votre système d'exploitation :

*   **Windows** : [https://github.com/junegunn/fzf?tab=readme-ov-file#windows-packages](https://github.com/junegunn/fzf?tab=readme-ov-file#windows-packages)
*   **Linux/macOS** : [https://github.com/junegunn/fzf?tab=readme-ov-file#linux-packages](https://github.com/junegunn/fzf?tab=readme-ov-file#linux-packages)

## Installation

```bash
cargo add cloud_terrastodon_user_input
```

## Examples / Exemples

### are_you_sure.rs

```rust
use cloud_terrastodon_user_input::prelude::are_you_sure;

fn main() -> eyre::Result<()> {
    if are_you_sure("This will delete the universe. Are you sure you want to proceed?".to_string())? {
        println!("Proceeding!");
    } else {
        println!("Action cancelled.");
    }

    Ok(())
}
```



### pick_many_nouns.rs

```rust
use cloud_terrastodon_user_input::prelude::FzfArgs;
use cloud_terrastodon_user_input::prelude::pick_many;

pub fn main() -> eyre::Result<()> {
    let nouns = vec!["dog", "cat", "house", "pickle", "mouse"];
    let chosen = pick_many(FzfArgs {
        choices: nouns,
        header: Some("Press tab to select entries".to_string()),
        prompt: Some("Pick some nouns >".to_string()),
        ..Default::default()
    })?;

    println!("You chose {:?}", chosen);

    Ok(())
}

```



### starting_search.rs

```rust
use cloud_terrastodon_user_input::prelude::FzfArgs;
use cloud_terrastodon_user_input::prelude::pick;

pub fn main() -> eyre::Result<()> {
    let nouns = vec!["dog", "cat", "house", "pickle", "mouse"];
    let chosen = pick(FzfArgs {
        choices: nouns,
        header: Some("Pick a noun".to_string()),
        query: Some("ouse".to_string()),
        ..Default::default()
    })?;
    println!("You chose {}", chosen);

    Ok(())
}

```



### read_line.rs

```rust
use cloud_terrastodon_user_input::prelude::read_line;
use std::io::Write;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    print!("Enter your name: ");
    std::io::stdout().flush()?;
    let user_input = read_line().await?;
    println!("Hello, {}!", user_input);

    Ok(())
}

```



### pick_a_noun.rs

```rust
use cloud_terrastodon_user_input::prelude::FzfArgs;
use cloud_terrastodon_user_input::prelude::pick;

pub fn main() -> eyre::Result<()> {
    let nouns = vec!["dog", "cat", "house", "pickle", "mouse"];
    let chosen = pick(FzfArgs {
        choices: nouns,
        header: Some("Pick a noun".to_string()),
        ..Default::default()
    })?;
    println!("You chose {}", chosen);

    Ok(())
}

```



### pick_a_path.rs

```rust
use cloud_terrastodon_user_input::prelude::Choice;
use cloud_terrastodon_user_input::prelude::FzfArgs;
use cloud_terrastodon_user_input::prelude::pick;

pub fn main() -> eyre::Result<()> {
    let mut choices = Vec::new();
    let mut dir = std::fs::read_dir(".")?;
    while let Some(entry) = dir.next() {
        let entry = entry?;
        choices.push(entry);
    }

    let chosen = pick(FzfArgs {
        choices: choices
            .into_iter()
            .map(|entry| Choice {
                key: entry.path().display().to_string(), // the value shown to the user
                value: entry, // the inner value we want to have after the user picks
            })
            .collect(),
        header: Some("Pick a path".to_string()),
        ..Default::default()
    })?;

    println!("You chose {}", chosen.file_name().to_string_lossy());

    Ok(())
}

```



### prompt_line.rs

```rust
use cloud_terrastodon_user_input::prelude::prompt_line;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let user_input = prompt_line("Enter your name: ").await?;
    println!("Hello, {}!", user_input);

    Ok(())
}
```
