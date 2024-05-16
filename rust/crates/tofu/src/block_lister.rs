use std::path::PathBuf;

use anyhow::Context;
use hcl::edit::structure::Body;
use hcl::edit::Span;
use itertools::Itertools;
use tofu_types::prelude::LocatableBlock;
use tokio::fs;

pub async fn list_blocks(path: PathBuf) -> anyhow::Result<Vec<LocatableBlock>> {
    let content = fs::read(&path).await.context(format!("reading content from path {path:?}"))?;
    let content = String::from_utf8(content).context("parsing content as utf-8")?;
    let body: Body = content.parse().context("parsing content as body")?;
    Ok(body
        .into_blocks()
        .map(|block| LocatableBlock {
            path: path.to_owned(),
            display: if block.ident.to_string() == "import" {
                format!(
                    "{} {}",
                    block.ident,
                    block
                        .body
                        .get_attribute("to")
                        .map(|x| x.value.to_string())
                        .unwrap_or_default()
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
        })
        .collect_vec())
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
