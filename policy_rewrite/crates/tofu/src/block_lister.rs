use hcl::edit::{structure::Body, Span};
use itertools::Itertools;
use tofu_types::prelude::LocatableBlock;

pub fn list_blocks(content: &str) -> anyhow::Result<Vec<LocatableBlock>> {
    let body: Body = content.parse()?;
    Ok(body
        .into_blocks()
        .map(|block| LocatableBlock {
            display: format!(
                "{} {}",
                block.ident,
                block.labels.iter().map(|x| x.to_string()).join(".")
            ),
            line_number: block
                .span()
                .and_then(|span| find_line_column(content, span.start))
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