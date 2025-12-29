use crate::prelude::get_resource_group_choices;
use crate::prelude::get_role_assignment_choices;
use crate::prelude::get_security_group_choices;
use cloud_terrastodon_hcl_types::prelude::HclImportBlock;
use cloud_terrastodon_hcl_types::prelude::HclProviderBlock;
use cloud_terrastodon_hcl_types::prelude::HclProviderReference;
use cloud_terrastodon_hcl_types::prelude::ProviderKind;
use cloud_terrastodon_hcl_types::prelude::edit::structure::Block;
use cloud_terrastodon_hcl_types::prelude::edit::structure::Body;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use std::collections::HashSet;
use std::convert::TryFrom;
use strum::VariantArray;

#[derive(strum::VariantArray, Debug, Clone, Copy)]
pub enum HclImportable {
    ResourceGroup,
    SecurityGroup,
    RoleAssignment,
}
impl std::fmt::Display for HclImportable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl HclImportable {
    pub async fn try_into_import_blocks(
        &self,
    ) -> eyre::Result<Vec<Choice<(HclImportBlock, Option<HclProviderBlock>)>>> {
        let rtn: Vec<Choice<(HclImportBlock, Option<HclProviderBlock>)>> = match self {
            HclImportable::ResourceGroup => get_resource_group_choices()
                .await?
                .into_iter()
                .map(
                    |Choice {
                         key,
                         value: (rg, sub),
                     }| Choice {
                        key,
                        value: (
                            {
                                let mut import_block: HclImportBlock = rg.into();
                                import_block.provider = HclProviderReference::Alias {
                                    kind: ProviderKind::AzureRM,
                                    name: sub.name.to_string(),
                                };
                                import_block
                            },
                            Some(sub.into_provider_block()),
                        ),
                    },
                )
                .collect(),
            HclImportable::SecurityGroup => get_security_group_choices()
                .await?
                .into_iter()
                .map(|choice| Choice {
                    key: choice.key,
                    value: (choice.value.into(), None),
                })
                .collect(),
            HclImportable::RoleAssignment => get_role_assignment_choices()
                .await?
                .into_iter()
                .map(|choice| Choice {
                    key: choice.key,
                    value: (choice.value.into(), None),
                })
                .collect(),
        };
        Ok(rtn)
    }
    pub fn pick() -> eyre::Result<HclImportable> {
        Ok(PickerTui::new()
            .set_header("Pick the kind of thing to import")
            .pick_one(HclImportable::VARIANTS.iter().copied().map(|x| Choice {
                key: x.to_string(),
                value: x,
            }))?)
    }
    pub async fn pick_into_body(self) -> eyre::Result<Body> {
        let import_blocks = self.try_into_import_blocks().await?;
        let import_blocks = PickerTui::new()
            .set_header("Pick the resources to import")
            .pick_many(import_blocks)?;
        let providers = import_blocks
            .iter()
            .filter_map(|(_, provider)| provider.clone())
            .collect::<HashSet<_>>();
        let body = Body::builder().blocks(providers).blocks(
            import_blocks
                .into_iter()
                .map(|(import_block, _)| Block::try_from(import_block))
                .collect::<Result<Vec<Block>, _>>()?,
        );

        let body = body.build();
        Ok(body)
    }
}
