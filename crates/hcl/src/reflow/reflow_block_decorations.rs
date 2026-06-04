use crate::reflow::HclReflower;
use hcl::edit::Decorate;
use hcl::edit::structure::Body;
use hcl::edit::structure::Structure;
use std::collections::HashMap;
use std::mem;
use std::path::PathBuf;

pub struct ReflowBlockDecorations;

#[async_trait::async_trait]
impl HclReflower for ReflowBlockDecorations {
    async fn reflow(
        &mut self,
        hcl: HashMap<PathBuf, Body>,
    ) -> eyre::Result<HashMap<PathBuf, Body>> {
        let mut reflowed = HashMap::new();
        for (path, mut body) in hcl {
            let decor = mem::take(body.decor_mut());
            let mut updated = Body::new();
            for structure in body.into_iter() {
                match structure {
                    Structure::Block(mut block) => {
                        if block
                            .decor()
                            .suffix()
                            .is_none_or(|suffix| suffix.to_string().is_empty())
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
