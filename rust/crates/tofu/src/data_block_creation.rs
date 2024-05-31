use crate::data_lookup_holder::DataLookupHolder;
use crate::import_lookup_holder::ResourceId;
use anyhow::Result;
use azure::prelude::PolicyDefinition;
use azure::prelude::PolicyDefinitionId;
use azure::prelude::PolicySetDefinition;
use azure::prelude::PolicySetDefinitionId;
use azure::prelude::Scope;
use azure::prelude::ScopeImpl;
use command::prelude::CommandBuilder;
use hcl::edit::structure::Body;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use std::collections::HashSet;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use tofu_types::prelude::Sanitizable;
use tofu_types::prelude::TofuAzureRMDataKind;
use tofu_types::prelude::TofuDataBlock;
use tofu_types::prelude::TofuDataReference;
use tofu_types::prelude::TryAsTofuBlocks;
use tokio::task::JoinSet;
use tokio::time::interval;
use tracing::error;

pub async fn create_data_blocks_for_ids(
    ids: &HashSet<ResourceId>,
) -> Result<(Body, DataLookupHolder)> {
    let mut body = Body::new();

    // Prepare progress indicators
    let pb = ProgressBar::new(ids.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:30.cyan/blue} {pos:>7}/{len:7} {msg}")?,
    );
    // Prepare work pool
    enum WorkResult {
        Definition {
            id: PolicyDefinitionId,
            definition: PolicyDefinition,
        },
        SetDefinition {
            id: PolicySetDefinitionId,
            definition: PolicySetDefinition,
        },
    }
    let mut work_pool = JoinSet::new();

    // Spawn work
    for missing in ids {
        match ScopeImpl::from_str(missing) {
            Ok(ScopeImpl::PolicyDefinition(policy_definition_id)) => {
                pb.set_message(format!(
                    "Spawning worker for fetching policy_definition {}",
                    policy_definition_id.short_form()
                ));
                work_pool.spawn(async move {
                    CommandBuilder::new(command::prelude::CommandKind::AzureCLI)
                        .args([
                            "policy",
                            "definition",
                            "show",
                            "--name",
                            policy_definition_id.short_form(),
                        ])
                        .use_cache_dir(Some(PathBuf::from_iter([
                            "az policy definition show --name",
                            policy_definition_id.short_form(),
                        ])))
                        .run::<PolicyDefinition>()
                        .await
                        .map(|definition| WorkResult::Definition {
                            id: policy_definition_id,
                            definition,
                        })
                });
            }
            Ok(ScopeImpl::PolicySetDefinition(policy_set_definition_id)) => {
                pb.set_message(format!(
                    "Spawning worker for fetching policy_set_definition {}",
                    policy_set_definition_id.short_form()
                ));
                work_pool.spawn(async move {
                    CommandBuilder::new(command::prelude::CommandKind::AzureCLI)
                        .args([
                            "policy",
                            "set-definition",
                            "show",
                            "--name",
                            policy_set_definition_id.short_form(),
                        ])
                        .use_cache_dir(Some(PathBuf::from_iter([
                            "az policy set-definition show --name",
                            policy_set_definition_id.short_form(),
                        ])))
                        .run::<PolicySetDefinition>()
                        .await
                        .map(|definition| WorkResult::SetDefinition {
                            id: policy_set_definition_id,
                            definition,
                        })
                });
            }
            Ok(x) => {
                error!("Kind {x} doesn't have a patcher data missing impl");
                continue;
            }
            Err(e) => {
                error!("Couldn't determine kind for {missing}: {e:?}");
                continue;
            }
        };
    }

    pb.set_message("Waiting for results");
    pb.tick();

    let pb = Arc::new(Mutex::new(pb));
    let pb_thread = pb.clone();
    let ticker = tokio::spawn(async move {
        let mut period = interval(Duration::from_secs(1));
        // period.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            period.tick().await;
            match pb_thread.lock() {
                Ok(pb) => pb.tick(),
                Err(e) => {
                    error!("Failed to tick progress bar! {e:?}");
                }
            }
        }
    });

    // Collect results
    let mut lookup_holder = DataLookupHolder::default();
    while let Some(res) = work_pool.join_next().await {
        let result = res??;

        // Extract the info
        let (id, reference) = match result {
            WorkResult::Definition { id, definition } => (
                ScopeImpl::PolicyDefinition(id.clone()),
                TofuDataReference::AzureRM {
                    kind: TofuAzureRMDataKind::PolicyDefinition,
                    name: definition.name.sanitize(),
                },
            ),
            WorkResult::SetDefinition { id, definition } => (
                ScopeImpl::PolicySetDefinition(id.clone()),
                TofuDataReference::AzureRM {
                    kind: TofuAzureRMDataKind::PolicySetDefinition,
                    name: definition.name.sanitize(),
                },
            ),
        };
        match pb.lock() {
            Ok(pb) => {
                pb.inc(1);
                pb.set_message(format!("Received info for {reference}"));
            }
            Err(e) => {
                error!("Failed to update progress bar! {e:?}");
            }
        }

        // Add the data block to the document
        TofuDataBlock::LookupByName {
            reference: reference.clone(),
            name: id.short_form().to_owned(),
        }
        .try_as_tofu_blocks()?
        .for_each(|b| body.push(b));

        // Add the reference to the lookup
        lookup_holder.data_references_by_id.insert(id, reference);
    }

    match pb.lock() {
        Ok(pb) => {
            pb.finish();
        }
        Err(e) => {
            error!("Failed to update progress bar! {e:?}");
        }
    }

    ticker.abort();
    Ok((body, lookup_holder))
}
