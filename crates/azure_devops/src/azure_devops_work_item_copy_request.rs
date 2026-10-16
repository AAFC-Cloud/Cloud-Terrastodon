use crate::azure_devops_work_item_copy_journal::AzureDevOpsWorkItemCopyJournal;
use crate::azure_devops_work_item_copy_mapping::AzureDevOpsWorkItemCopyMapping;
use crate::azure_devops_work_item_copy_plan::AzureDevOpsWorkItemCopyPlan;
use crate::azure_devops_work_item_relation_selector::AzureDevOpsWorkItemRelationSelector;
use crate::rest_work_item_copy_writer::RestWorkItemCopyWriter;
use crate::work_item_copy_writer::WorkItemCopyWriter;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_devops_types::AzureDevOpsJsonPatch;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemRelationInput;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemRelationTypeName;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemRelationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemUrl;
use cloud_terrastodon_azure_devops_types::WORK_ITEM_CHILD_RELATION;
use cloud_terrastodon_azure_devops_types::WORK_ITEM_PARENT_RELATION;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_credentials::AzureDevOpsAuthContext;
use eyre::Result;
use eyre::WrapErr;
use eyre::ensure;
use facet::Facet;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::future::Future;
use std::future::IntoFuture;
use std::path::PathBuf;
use std::pin::Pin;
#[derive(Debug, Clone, Facet)]
pub struct AzureDevOpsWorkItemCopyRequest<'a> {
    /// Caller-owned cancellation scope, excluded from serialized request input.
    #[facet(skip_serializing)]
    pub cancellation: CancellationScope,
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub project: AzureDevOpsProjectArgument<'a>,
    pub auth_context: Cow<'a, AzureDevOpsAuthContext>,
    pub plan: AzureDevOpsWorkItemCopyPlan,
    pub journal_path: PathBuf,
    pub resume: bool,
    /// Do not fire notifications for newly created work items and links.
    pub suppress_notifications: bool,
}

impl<'a> Arbitrary<'a> for AzureDevOpsWorkItemCopyRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            cancellation: CancellationScope::arbitrary(u)?,
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            project: AzureDevOpsProjectArgument::arbitrary(u)?.into_owned(),
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
            plan: AzureDevOpsWorkItemCopyPlan::arbitrary(u)?,
            journal_path: PathBuf::arbitrary(u)?,
            resume: bool::arbitrary(u)?,
            suppress_notifications: bool::arbitrary(u)?,
        })
    }
}

impl<'a> AzureDevOpsWorkItemCopyRequest<'a> {
    pub(crate) async fn execute_with(
        self,
        writer: &impl WorkItemCopyWriter,
    ) -> Result<AzureDevOpsWorkItemCopyJournal> {
        ensure!(
            self.plan.organization == *self.org_url.as_ref()
                && self.plan.project.to_string() == self.project.to_string(),
            "Copy plan belongs to a different organization or project"
        );
        // A separate stable inode remains locked while the journal is atomically
        // replaced. The OS releases the lock even if the process crashes.
        let mut lock_path = self.journal_path.as_os_str().to_os_string();
        lock_path.push(".lock");
        let _lock = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(PathBuf::from(lock_path))?;
        _lock.try_lock().map_err(|_| {
            eyre::eyre!("The copy journal is already in use or could not be locked")
        })?;
        let mut journal = if self.resume {
            let journal = AzureDevOpsWorkItemCopyJournal::load(&self.journal_path)?;
            ensure!(
                facet_json::to_string(&journal.plan)? == facet_json::to_string(&self.plan)?,
                "Copy journal and plan differ"
            );
            journal
        } else {
            // Reserve the destination without overwriting another copy run.
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&self.journal_path)?;
            AzureDevOpsWorkItemCopyJournal {
                version: 1,
                plan: self.plan,
                created: Vec::new(),
                completed_relations: 0,
                in_flight: None,
                complete: false,
            }
        };
        ensure!(
            journal.in_flight.is_none(),
            "Copy journal contains an uncertain write; reconcile that operation before resuming"
        );
        ensure!(
            journal.created.len() <= journal.plan.nodes.len()
                && journal.completed_relations <= journal.plan.deferred_relations.len(),
            "Invalid copy journal progress"
        );
        ensure!(
            journal.completed_relations == 0 || journal.created.len() == journal.plan.nodes.len(),
            "Links cannot precede item creation in a copy journal"
        );
        ensure!(
            !journal.complete
                || (journal.created.len() == journal.plan.nodes.len()
                    && journal.completed_relations == journal.plan.deferred_relations.len()),
            "Completed journal has unfinished operations"
        );
        let mut mapping = BTreeMap::new();
        let mut new_ids = BTreeSet::new();
        for (node, created) in journal.plan.nodes.iter().zip(&journal.created) {
            self.cancellation.bail_if_cancelled()?;
            ensure!(
                node.source_id == created.source_id && new_ids.insert(created.new_id),
                "Copy journal mappings are inconsistent"
            );
            ensure!(
                created
                    .new_url
                    .work_item_id_for_organization(self.org_url.as_ref())?
                    == created.new_id,
                "Invalid destination URL in copy journal"
            );
            mapping.insert(created.source_id, created.clone());
        }
        let mut visited = BTreeSet::new();
        for node in &journal.plan.nodes {
            self.cancellation.bail_if_cancelled()?;
            ensure!(
                (node.source_id == journal.plan.root) == node.parent.is_none(),
                "Copy plan must have exactly one root"
            );
            ensure!(
                node.parent.is_none_or(|parent| visited.contains(&parent))
                    && visited.insert(node.source_id),
                "Copy plan must order each parent before its children"
            );
            let mut relations = node.external_relations.clone();
            if let Some(parent) = node.parent {
                relations.push(AzureDevOpsWorkItemRelationInput::parent(
                    self.org_url.as_ref(),
                    parent,
                )?);
            }
            AzureDevOpsJsonPatch::work_item_create_patch(node.fields.clone(), &relations)?;
        }
        ensure!(
            journal
                .plan
                .nodes
                .first()
                .is_some_and(|n| n.source_id == journal.plan.root),
            "Invalid copy root"
        );
        ensure!(
            new_ids.is_disjoint(&visited),
            "Copy journal reuses source IDs as destinations"
        );
        for edge in &journal.plan.deferred_relations {
            self.cancellation.bail_if_cancelled()?;
            ensure!(
                visited.contains(&edge.source)
                    && visited.contains(&edge.target)
                    && edge.source != edge.target,
                "Copy relation must join two different selected items"
            );
            ensure!(
                edge.rel.as_ref() != WORK_ITEM_PARENT_RELATION
                    && edge.rel.as_ref() != WORK_ITEM_CHILD_RELATION,
                "Hierarchy links cannot be deferred"
            );
            AzureDevOpsWorkItemRelationInput {
                rel: edge.rel.clone(),
                url: AzureDevOpsWorkItemRelationUrl::from(AzureDevOpsWorkItemUrl::new(
                    Cow::Owned(self.org_url.as_ref().clone()),
                    edge.target,
                )),
                attributes: edge.attributes.clone(),
            }
            .add_patch()?;
        }
        journal.save(&self.journal_path)?;
        for index in journal.created.len()..journal.plan.nodes.len() {
            self.cancellation.bail_if_cancelled()?;
            let node = &journal.plan.nodes[index];
            let mut relations = node.external_relations.clone();
            if let Some(parent) = node.parent {
                let parent = mapping
                    .get(&parent)
                    .ok_or_else(|| eyre::eyre!("Parent has not been created"))?;
                relations.push(AzureDevOpsWorkItemRelationInput {
                    rel: WORK_ITEM_PARENT_RELATION
                        .parse::<AzureDevOpsWorkItemRelationTypeName>()?,
                    url: AzureDevOpsWorkItemRelationUrl::from(parent.new_url.clone()),
                    attributes: Default::default(),
                });
            }
            let patch =
                AzureDevOpsJsonPatch::work_item_create_patch(node.fields.clone(), &relations)?;
            let work_item_type = node.work_item_type.clone();
            let source_id = node.source_id;
            // Observe the mutation, SDK cache invalidation, and durable journal
            // update as one completion phase before accepting the next stop.
            let journal = &mut journal;
            let mapping = &mut mapping;
            let new_ids = &mut new_ids;
            let visited = &visited;
            let journal_path = &self.journal_path;
            let org_url = self.org_url.as_ref();
            self.cancellation
                .deferred(
                    "creating a copied work item and recording its result",
                    |cancellation| async move {
                        journal.in_flight = Some(format!("create:{source_id}"));
                        journal.save(journal_path)?;
                        let created = writer
                            .create(work_item_type, patch, &cancellation)
                            .await
                            .wrap_err(
                                "Copy stopped during creation; inspect the journal before retrying",
                            )?;
                        ensure!(
                            !visited.contains(&created.id) && new_ids.insert(created.id),
                            "Create returned an existing source or destination ID"
                        );
                        ensure!(
                            AzureDevOpsWorkItemUrl::try_from(&created.url)?
                                .work_item_id_for_organization(org_url)?
                                == created.id,
                            "Create returned an invalid destination URL"
                        );
                        let entry = AzureDevOpsWorkItemCopyMapping {
                            source_id,
                            new_id: created.id,
                            new_url: AzureDevOpsWorkItemUrl::new(
                                Cow::Owned(org_url.clone()),
                                created.id,
                            ),
                        };
                        mapping.insert(source_id, entry.clone());
                        journal.created.push(entry);
                        journal.in_flight = None;
                        journal.save(journal_path)
                    },
                )
                .await?;
        }
        for index in journal.completed_relations..journal.plan.deferred_relations.len() {
            self.cancellation.bail_if_cancelled()?;
            let edge = &journal.plan.deferred_relations[index];
            let source = mapping
                .get(&edge.source)
                .ok_or_else(|| eyre::eyre!("Missing copied relation source"))?;
            let target = mapping
                .get(&edge.target)
                .ok_or_else(|| eyre::eyre!("Missing copied relation target"))?;
            let relation = AzureDevOpsWorkItemRelationInput {
                rel: edge.rel.clone(),
                url: AzureDevOpsWorkItemRelationUrl::from(target.new_url.clone()),
                attributes: edge.attributes.clone(),
            };
            let source_id = source.new_id;
            let journal = &mut journal;
            let journal_path = &self.journal_path;
            self.cancellation
                .deferred(
                    "linking copied work items and recording the completed link",
                    |cancellation| async move {
                        journal.in_flight = Some(format!("relation:{index}"));
                        journal.save(journal_path)?;
                        writer.relate(source_id, relation, &cancellation).await.wrap_err(
                            "Copy stopped while linking items; inspect the journal before retrying",
                        )?;
                        journal.completed_relations += 1;
                        journal.in_flight = None;
                        journal.save(journal_path)
                    },
                )
                .await?;
        }
        self.cancellation.bail_if_cancelled()?;
        // Verification is read-only and may safely be retried on resume.
        let verified = writer
            .read(
                journal.created.iter().map(|entry| entry.new_id).collect(),
                &self.cancellation,
            )
            .await?;
        ensure!(
            verified.len() == journal.created.len(),
            "Copy verification returned an incomplete item set"
        );
        let verified: BTreeMap<_, _> = verified.into_iter().map(|item| (item.id, item)).collect();
        for node in &journal.plan.nodes {
            self.cancellation.bail_if_cancelled()?;
            let item = verified
                .get(&mapping[&node.source_id].new_id)
                .ok_or_else(|| eyre::eyre!("A copied item could not be verified"))?;
            let expected_parent = if let Some(parent) = node.parent {
                Some(mapping[&parent].new_id)
            } else {
                node.external_relations
                    .iter()
                    .find(|relation| relation.rel.as_ref() == WORK_ITEM_PARENT_RELATION)
                    .map(|relation| {
                        relation
                            .url
                            .work_item_id_for_organization(self.org_url.as_ref())
                    })
                    .transpose()?
            };
            let actual_parents: Vec<_> = item
                .relations
                .as_deref()
                .unwrap_or_default()
                .iter()
                .filter(|relation| relation.rel.as_ref() == WORK_ITEM_PARENT_RELATION)
                .collect();
            ensure!(
                actual_parents.len() == usize::from(expected_parent.is_some()),
                "Copied item has an unexpected parent count"
            );
            if let Some(expected) = expected_parent {
                ensure!(
                    actual_parents[0]
                        .url
                        .work_item_id_for_organization(self.org_url.as_ref())?
                        == expected,
                    "Copied item has an unexpected parent"
                );
            }
        }
        for edge in &journal.plan.deferred_relations {
            self.cancellation.bail_if_cancelled()?;
            let item = &verified[&mapping[&edge.source].new_id];
            AzureDevOpsWorkItemRelationSelector {
                rel: edge.rel.clone(),
                target: mapping[&edge.target].new_url.clone(),
            }
            .index(self.org_url.as_ref(), item)
            .wrap_err("Copied link verification failed")?;
        }
        self.cancellation
            .with_deferral("saving verified copy progress", |_completion| {
                journal.complete = true;
                journal.save(&self.journal_path)?;
                Ok(journal)
            })
    }
}

impl<'a> IntoFuture for AzureDevOpsWorkItemCopyRequest<'a> {
    type Output = Result<AzureDevOpsWorkItemCopyJournal>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let writer = RestWorkItemCopyWriter {
                org_url: self.org_url.as_ref().clone(),
                project: self.project.clone().into_owned(),
                auth_context: self.auth_context.as_ref().clone(),
                suppress_notifications: self.suppress_notifications,
            };
            self.execute_with(&writer).await
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsWorkItemCopyRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsWorkItemCopyRequest<'static>);
cloud_terrastodon_registry::register_into_future!(
    AzureDevOpsWorkItemCopyRequest<'static> => AzureDevOpsWorkItemCopyJournal,
    effects = [Write]
);
