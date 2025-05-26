use crate::prelude::get_resource_group_choices;
use crate::prelude::get_role_assignment_choices;
use crate::prelude::get_security_group_choices;
use cloud_terrastodon_hcl_types::prelude::HCLImportBlock;
use cloud_terrastodon_hcl_types::prelude::HCLProviderBlock;
use cloud_terrastodon_hcl_types::prelude::HCLProviderReference;
use cloud_terrastodon_hcl_types::prelude::ProviderKind;
use cloud_terrastodon_hcl_types::prelude::edit::structure::Block;
use cloud_terrastodon_hcl_types::prelude::edit::structure::Body;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::FzfArgs;
use cloud_terrastodon_user_input::pick;
use cloud_terrastodon_user_input::pick_many;
use std::collections::HashSet;
use strum::VariantArray;

#[derive(strum::VariantArray, Debug, Clone, Copy)]
pub enum HCLImportable {
    ResourceGroup,
    SecurityGroup,
    RoleAssignment,
}
impl std::fmt::Display for HCLImportable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl HCLImportable {
    pub async fn try_into_import_blocks(
        &self,
    ) -> eyre::Result<Vec<Choice<(HCLImportBlock, Option<HCLProviderBlock>)>>> {
        let rtn: Vec<Choice<(HCLImportBlock, Option<HCLProviderBlock>)>>;
        match self {
            HCLImportable::ResourceGroup => {
                rtn = get_resource_group_choices()
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
                                    let mut import_block: HCLImportBlock = rg.into();
                                    import_block.provider = HCLProviderReference::Alias {
                                        kind: ProviderKind::AzureRM,
                                        name: sub.name.clone(),
                                    };
                                    import_block
                                },
                                Some(sub.into_provider_block()),
                            ),
                        },
                    )
                    .collect();
            }
            HCLImportable::SecurityGroup => {
                rtn = get_security_group_choices()
                    .await?
                    .into_iter()
                    .map(|choice| Choice {
                        key: choice.key,
                        value: (choice.value.into(), None),
                    })
                    .collect();
            }
            HCLImportable::RoleAssignment => {
                rtn = get_role_assignment_choices()
                    .await?
                    .into_iter()
                    .map(|choice| Choice {
                        key: choice.key,
                        value: (choice.value.into(), None),
                    })
                    .collect();
            }
        }
        Ok(rtn)
    }
    pub fn pick() -> eyre::Result<HCLImportable> {
        Ok(*pick(FzfArgs {
            choices: HCLImportable::VARIANTS
                .iter()
                .map(|x| Choice {
                    key: x.to_string(),
                    value: x,
                })
                .collect(),
            header: Some("Pick the kind of thing to import".to_string()),
            ..Default::default()
        })?
        .value)
    }
    pub async fn pick_into_body(self) -> eyre::Result<Body> {
        let import_blocks = self.try_into_import_blocks().await?;
        let import_blocks = pick_many(FzfArgs {
            choices: import_blocks,
            header: Some("Pick the resources to import".to_string()),
            ..Default::default()
        })?;
        let providers = import_blocks
            .iter()
            .filter_map(
                |Choice {
                     value: (_, provider),
                     ..
                 }| provider.clone(),
            )
            .collect::<HashSet<_>>();
        let body = Body::builder().blocks(providers).blocks(
            import_blocks
                .into_iter()
                .map(
                    |Choice {
                         value: (import_block, ..),
                         ..
                     }| {
                        let block: Result<Block, _> = import_block.try_into();
                        block
                    },
                )
                .collect::<Result<Vec<_>, _>>()?,
        );

        let body = body.build();
        Ok(body)
    }
}
