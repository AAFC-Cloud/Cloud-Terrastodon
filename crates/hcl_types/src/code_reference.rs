use crate::prelude::HclBlock;
use eyre::Result;
use hcl::edit::prelude::Span;
use hcl::edit::structure::Block;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct LocationWithinFile {
    pub path: PathBuf,
    pub line: usize,
    pub column: usize,
}
impl std::fmt::Display for LocationWithinFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}:{}:{}",
            self.path.display(),
            self.line,
            self.column
        ))
    }
}

#[derive(Debug, Clone)]
pub struct CodeReference {
    pub hcl_block: HclBlock,
    pub location: LocationWithinFile,
}
impl std::fmt::Display for CodeReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{} | {}",
            self.location,
            self.hcl_block
        ))
    }
}
impl CodeReference {
    pub fn try_from_block(content: &str, block: Block, path: &PathBuf) -> Result<Self> {
        let span = block
            .span()
            .and_then(|span| find_line_column(content, span.start))
            .unwrap_or((0, 0));
        let hcl_block: HclBlock = block.try_into()?;
        Ok(CodeReference {
            location: LocationWithinFile {
                path: path.to_owned(),
                line: span.0,
                column: span.1,
            },
            hcl_block,
        })
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
