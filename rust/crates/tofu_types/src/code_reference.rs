use std::path::PathBuf;

use hcl::edit::prelude::Span;
use hcl::edit::structure::Block;
use itertools::Itertools;

pub struct CodeReference {
    pub display: String,
    pub line_number: usize,
    pub path: PathBuf,
}
impl std::fmt::Display for CodeReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{} | {}", self.path.display(), self.display))
    }
}
impl CodeReference {
    pub fn from_block(content: &str, block: &Block, path: &PathBuf) -> Self {
        CodeReference {
            path: path.to_owned(),
            display: if block.ident.to_string() == "import" {
                format!(
                    "{} - to = {}",
                    block.ident,
                    block
                        .body
                        .get_attribute("to")
                        .map(|x| x.value.to_string())
                        .unwrap_or_default()
                        .trim()
                )
            } else if block.ident.to_string() == "provider" {
                match block
                    .body
                    .get_attribute("alias")
                    .map(|x| x.value.to_string())
                {
                    Some(alias) => format!(
                        "provider {} - alias={}",
                        block.labels.iter().map(|x| x.to_string()).join(" "),
                        alias
                    ),
                    None => format!(
                        "provider {}",
                        block.labels.iter().map(|x| x.to_string()).join(" ")
                    ),
                }
            } else if (block.ident.to_string() == "data" || block.ident.to_string() == "resource")
                && let Some(name) = block
                    .body
                    .get_attribute("display_name")
                    .or_else(|| block.body.get_attribute("name"))
                && block
                    .labels
                    .get(1)
                    .map(|label| label.to_string())
                    .filter(|label| Some(label.as_str()) != name.value.as_str())
                    .is_some()
            {
                format!(
                    "{} {} - {} = {}",
                    block.ident.to_string(),
                    block.labels.iter().map(|x| x.to_string()).join(" "),
                    name.key.to_string(),
                    name.value.to_string()
                )
            } else {
                format!(
                    "{} {}",
                    block.ident,
                    block.labels.iter().map(|x| x.to_string()).join(".")
                )
            },
            line_number: block
                .span()
                .and_then(|span| find_line_column(&content, span.start))
                .map(|pos| pos.0)
                .unwrap_or_default(),
        }
    }
}

fn find_line_column(s: &str, char_index: usize) -> Option<(usize, usize)> {
    let mut line = 1;
    let mut col = 0;
    let mut current_index = 0;

    for c in s.chars() {
        if current_index == char_index {
            return Some((line, col));
        }

        if c == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }

        current_index += c.len_utf8();
    }

    None // Return None if char_index is out of bounds
}
