use crate::sorting::HclBlockSortable;
use cloud_terrastodon_hcl_types::prelude::AsHclString;
use eyre::Result;
use eyre::eyre;
use hcl::edit::Decorate;
use hcl::edit::structure::Attribute;
use hcl::edit::structure::Block;
use hcl::edit::structure::Body;
use hcl::edit::structure::Structure;
use indoc::formatdoc;
use itertools::Itertools;
use std::collections::HashMap;
use tracing::error;

#[derive(Default, Clone)]
pub struct BodyFormatter {
    resource_blocks: HashMap<String, Block>,
    resource_keys_by_id: HashMap<String, String>,
    terraform_blocks: Vec<Block>,
    import_blocks: Vec<Block>,
    provider_blocks: Vec<Block>,
    other_blocks: Vec<Block>,
    attrs: Vec<Attribute>,
}
impl TryFrom<Body> for BodyFormatter {
    type Error = eyre::Error;

    fn try_from(src: Body) -> Result<Self> {
        let mut rtn = BodyFormatter::default();
        // Populate holders
        for structure in src.into_iter() {
            match structure {
                Structure::Block(block) => {
                    match block.ident.as_str() {
                        "resource" => {
                            // Get the lookup key
                            let key = block.labels.iter().map(|l| l.to_string()).join(".");

                            // Add it to the lookup table
                            rtn.resource_blocks.insert(key, block);
                        }
                        "provider" => rtn.provider_blocks.push(block),
                        "import" => {
                            // Add it to the alias table
                            let id = block
                                .body
                                .get_attribute("id")
                                .ok_or(
                                    eyre!("import block missing attribute \"id\" ")
                                        .wrap_err(block.as_hcl_string()),
                                )?
                                .value
                                .as_str()
                                .ok_or(
                                    eyre!("import block attribute \"id\" not a string")
                                        .wrap_err(block.as_hcl_string()),
                                )?
                                .to_string();
                            let key = block
                                .body
                                .get_attribute("to")
                                .ok_or(
                                    eyre!("import block missing attribute \"to\"")
                                        .wrap_err(block.as_hcl_string()),
                                )?
                                .value
                                .to_string();
                            rtn.resource_keys_by_id.insert(id, key);

                            // Add it to the import block list
                            rtn.import_blocks.push(block);
                        }
                        "terraform" => {
                            rtn.terraform_blocks.push(block);
                        }
                        _ => rtn.other_blocks.push(block),
                    };
                }
                Structure::Attribute(attr) => {
                    rtn.attrs.push(attr);
                }
            }
        }
        Ok(rtn)
    }
}
impl From<BodyFormatter> for Body {
    fn from(mut value: BodyFormatter) -> Self {
        // Create output
        let mut output = Body::new();

        // Add terraform block to the top of the output
        value
            .terraform_blocks
            .into_iter()
            .sort_blocks()
            .for_each(|block| output.push(block));

        // Add providers to the top of the output
        value
            .provider_blocks
            .into_iter()
            .sort_blocks()
            .for_each(|block| output.push(block));

        // Add unknowns
        value
            .other_blocks
            .into_iter()
            .sort_blocks()
            .for_each(|block| output.push(block));

        // Sort import blocks by destination
        value
            .import_blocks
            .sort_by_key(|b| b.body.get_attribute("to").map(|a| a.value.to_string()));

        // Add import blocks followed by their resource blocks
        for mut import_block in value.import_blocks.into_iter() {
            // Import block must contain `to` key
            let Some(key) = import_block.body.get_attribute("to") else {
                output.push(import_block);
                continue;
            };

            // Get and trim key
            let to = key.value.to_string();
            let key = to.trim();

            // Find resource block
            let found = value.resource_blocks.remove(key);
            let Some(mut resource_block) = found else {
                error!(
                    "Couldn't find resource block for import to {key}, this import will be omitted!"
                );
                // output.push(import_block);
                continue;
            };

            // Determine label
            let section_heading = format!(
                "{} {}",
                resource_block
                    .labels
                    .first()
                    .map(|x| x.to_string())
                    .unwrap_or_default(),
                resource_block
                    .body
                    .get_attribute("display_name")
                    .or(resource_block.body.get_attribute("name"))
                    .map(|x| x.value.to_string())
                    .or(resource_block.labels.get(1).map(|x| x.to_string()))
                    .unwrap_or_default()
                    .trim()
            );

            // Apply label
            import_block.decor_mut().set_prefix(formatdoc! {"
                #############
                ## {}
                #############
            ", section_heading});

            // Clear block decorations
            resource_block.decor_mut().set_prefix("");

            // Push blocks
            output.push(import_block);
            output.push(resource_block);
        }

        // Add remaining resource blocks
        value
            .resource_blocks
            .into_iter()
            .for_each(|(_k, v)| output.push(v));

        // Return
        output
    }
}

#[expect(dead_code, reason = "Old type being superceded by `ct tf reflow` command")]
pub trait PrettyBody {
    fn to_string_pretty(self) -> Result<String>;
}

impl PrettyBody for Body {
    fn to_string_pretty(self) -> Result<String> {
        let body: Body = self;
        let body: BodyFormatter = body.try_into()?;
        let body: Body = body.into();
        Ok(body.to_string())
    }
}
