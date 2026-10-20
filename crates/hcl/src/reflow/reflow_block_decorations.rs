use crate::HclProject;
use crate::reflow::HclReflower;
use hcl::edit::Decorate;
use hcl::edit::structure::Body;
use hcl::edit::structure::Structure;
use std::mem;

pub struct ReflowBlockDecorations;

#[async_trait::async_trait]
impl HclReflower for ReflowBlockDecorations {
    async fn reflow(&mut self, hcl: HclProject) -> eyre::Result<HclProject> {
        let mut reflowed = HclProject::new();
        for (path, mut body) in hcl {
            let decor = mem::take(body.decor_mut());
            let structure_count = body.len();
            let trailing_separator = decor
                .suffix()
                .is_some_and(|suffix| suffix.starts_with('\n') || suffix.starts_with("\r\n"));
            let mut updated = Body::new();
            for (index, structure) in body.into_iter().enumerate() {
                match structure {
                    Structure::Block(mut block) => {
                        // Parsed trailing comments already carry their separator.
                        // Adding another one to the last block grows blank lines
                        // every time the file is parsed and reflowed again.
                        if block
                            .decor()
                            .suffix()
                            .is_none_or(|suffix| suffix.to_string().is_empty())
                            && (index + 1 < structure_count || !trailing_separator)
                        {
                            block.decor_mut().set_suffix("\n");
                        }
                        updated.push(block);
                    }
                    structure => updated.push(structure),
                }
            }
            updated.decorate(decor);
            reflowed.insert(path, updated);
        }
        Ok(reflowed)
    }
}
