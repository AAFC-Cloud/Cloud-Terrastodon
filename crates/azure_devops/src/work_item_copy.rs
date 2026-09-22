use crate::azure_devops_work_item_copy_edge::AzureDevOpsWorkItemCopyEdge;
use crate::azure_devops_work_item_copy_journal::AzureDevOpsWorkItemCopyJournal;
use crate::azure_devops_work_item_copy_node::AzureDevOpsWorkItemCopyNode;
use crate::azure_devops_work_item_copy_omission::AzureDevOpsWorkItemCopyOmission;
use crate::azure_devops_work_item_copy_options::AzureDevOpsWorkItemCopyOptions;
use crate::azure_devops_work_item_copy_plan::AzureDevOpsWorkItemCopyPlan;
use chrono::{DateTime, Utc};
use cloud_terrastodon_azure_devops_types::*;
use cloud_terrastodon_azure_types::ArbitraryJson;
use cloud_terrastodon_command::async_trait;
use eyre::Result;
use eyre::ensure;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::VecDeque;
use std::io::Write;
use std::path::Path;

#[async_trait]
pub(crate) trait WorkItemCopyWriter: Sync {
    async fn create(
        &self,
        work_item_type: AzureDevOpsWorkItemType,
        patch: AzureDevOpsJsonPatch,
    ) -> Result<AzureDevOpsWorkItem>;
    async fn relate(
        &self,
        source: AzureDevOpsWorkItemId,
        relation: AzureDevOpsWorkItemRelationInput,
    ) -> Result<()>;
    async fn read(&self, ids: Vec<AzureDevOpsWorkItemId>) -> Result<Vec<AzureDevOpsWorkItem>>;
}

fn omit(
    omissions: &mut Vec<AzureDevOpsWorkItemCopyOmission>,
    id: AzureDevOpsWorkItemId,
    category: &str,
    name: &str,
) {
    omissions.push(AzureDevOpsWorkItemCopyOmission {
        source_id: id,
        category: category.to_owned(),
        name: name.to_owned(),
    });
}

fn generated_field(field: &str) -> bool {
    field.starts_with("WEF_")
        || matches!(
            field,
            "System.Id"
                | "System.Rev"
                | "System.Parent"
                | "System.TeamProject"
                | "System.WorkItemType"
                | "System.AreaId"
                | "System.IterationId"
                | "System.NodeName"
                | "System.CreatedBy"
                | "System.CreatedDate"
                | "System.ChangedBy"
                | "System.ChangedDate"
                | "System.AuthorizedAs"
                | "System.AuthorizedDate"
                | "System.RevisedDate"
                | "System.Watermark"
                | "System.PersonId"
                | "System.History"
                | "System.CommentCount"
                | "System.BoardColumn"
                | "System.BoardColumnDone"
                | "System.BoardLane"
                | "Microsoft.VSTS.Common.StackRank"
                | "Microsoft.VSTS.Common.BacklogPriority"
                | "Microsoft.VSTS.Common.StateChangeDate"
                | "Microsoft.VSTS.Common.ActivatedBy"
                | "Microsoft.VSTS.Common.ActivatedDate"
                | "Microsoft.VSTS.Common.ResolvedBy"
                | "Microsoft.VSTS.Common.ResolvedDate"
                | "Microsoft.VSTS.Common.ClosedBy"
                | "Microsoft.VSTS.Common.ClosedDate"
        )
        || field.starts_with("System.AreaLevel")
        || field.starts_with("System.IterationLevel")
}

fn identity_value(value: &ArbitraryJson) -> Result<ArbitraryJson> {
    if facet_json::from_str::<String>(value.as_ref()).is_ok() || value.as_ref() == "null" {
        return Ok(value.clone());
    }
    let identity: BTreeMap<String, ArbitraryJson> = facet_json::from_str(value.as_ref())?;
    if let Some(name) = identity.get("uniqueName") {
        return Ok(name.clone());
    }
    let id = identity
        .get("id")
        .ok_or_else(|| eyre::eyre!("Identity field has no uniqueName or id"))?;
    ArbitraryJson::try_from_facet(&BTreeMap::from([("id".to_owned(), id.clone())]))
}

fn copied_attributes(
    relation: &AzureDevOpsWorkItemRelation,
) -> Result<AzureDevOpsWorkItemRelationInputAttributes> {
    // New links start unlocked. Retain the user-authored link comment only.
    let comment = relation.attributes.comment.clone();
    Ok(AzureDevOpsWorkItemRelationInputAttributes {
        comment,
        is_locked: None,
    })
}

fn validate_snapshot_url(
    organization: &AzureDevOpsOrganizationUrl,
    item: &AzureDevOpsWorkItem,
) -> Result<()> {
    // asOf reads return revision URLs; hierarchy links and newly created items
    // use canonical item URLs. Only normalize the source snapshot URL here.
    let mut url = item.url.clone();
    let segments: Vec<_> = url
        .path_segments()
        .ok_or_else(|| eyre::eyre!("Invalid source URL"))?
        .collect();
    if segments.len() >= 2 && segments[segments.len() - 2].eq_ignore_ascii_case("revisions") {
        ensure!(
            segments.last().and_then(|rev| rev.parse::<i32>().ok()) == Some(item.rev),
            "Source URL does not match its revision"
        );
        url.path_segments_mut()
            .map_err(|_| eyre::eyre!("Invalid source URL"))?
            .pop()
            .pop();
    }
    ensure!(
        AzureDevOpsWorkItemUrl::try_from(&url)?.work_item_id_for_organization(organization)?
            == item.id,
        "Source URL does not match its work item"
    );
    Ok(())
}

/// Pure planner: no network access or mutation. Input items must comprise the
/// complete selected subtree, fetched at the same snapshot time.
#[expect(
    clippy::too_many_arguments,
    reason = "the planner receives an explicit source snapshot and metadata"
)]
pub fn build_work_item_copy_plan(
    organization: &AzureDevOpsOrganizationUrl,
    project: &AzureDevOpsProjectArgument<'_>,
    root: AzureDevOpsWorkItemId,
    options: &AzureDevOpsWorkItemCopyOptions,
    as_of: DateTime<Utc>,
    source: Vec<AzureDevOpsWorkItem>,
    definitions: &[AzureDevOpsWorkItemFieldDefinition],
    type_fields: &BTreeMap<AzureDevOpsWorkItemType, Vec<AzureDevOpsWorkItemTypeField>>,
    relation_types: &[AzureDevOpsWorkItemRelationType],
) -> Result<AzureDevOpsWorkItemCopyPlan> {
    let count = source.len();
    let items: BTreeMap<_, _> = source.into_iter().map(|item| (item.id, item)).collect();
    ensure!(
        items.len() == count && items.contains_key(&root),
        "Copy snapshot has duplicate items or no root"
    );
    ensure!(
        options.allowed_ids.is_empty() || items.keys().all(|id| options.allowed_ids.contains(id)),
        "Copy snapshot exceeds the allowed ID set"
    );
    let root_item = &items[&root];
    let source_project = root_item
        .fields
        .team_project
        .as_ref()
        .ok_or_else(|| eyre::eyre!("Missing System.TeamProject field"))?;
    let project_name = project.to_string();
    let root_url = &root_item.url;
    ensure!(
        project_name.eq_ignore_ascii_case(source_project.as_ref())
            || root_url
                .path_segments()
                .is_some_and(|mut segments| segments
                    .any(|segment| segment.eq_ignore_ascii_case(&project_name))),
        "Copy currently supports the source project only"
    );
    let mut queue = VecDeque::from([(root, None)]);
    let mut parents = BTreeMap::new();
    let mut order = Vec::new();
    while let Some((id, parent)) = queue.pop_front() {
        ensure!(
            parents.insert(id, parent).is_none(),
            "Copy hierarchy has a cycle, duplicate edge, or multiple parents"
        );
        let item = items
            .get(&id)
            .ok_or_else(|| eyre::eyre!("Copy snapshot is missing a child"))?;
        validate_snapshot_url(organization, item)?;
        ensure!(
            item.fields
                .team_project
                .as_ref()
                .ok_or_else(|| eyre::eyre!("Missing System.TeamProject field"))?
                == source_project,
            "Copy currently supports a single source project"
        );
        let reverse: Vec<_> = item
            .relations
            .as_deref()
            .unwrap_or_default()
            .iter()
            .filter(|r| r.rel.as_ref() == WORK_ITEM_PARENT_RELATION)
            .collect();
        ensure!(
            reverse.len() <= 1,
            "Source item has multiple Parent relations"
        );
        if let Some(parent) = parent {
            ensure!(
                reverse.len() == 1
                    && reverse[0].url.work_item_id_for_organization(organization)? == parent,
                "Source Parent and Child relations disagree"
            );
        } else if let Some(parent) = reverse.first() {
            ensure!(
                !items.contains_key(&parent.url.work_item_id_for_organization(organization)?),
                "Root Parent points inside the selected subtree"
            );
        }
        order.push(id);
        if options.deep {
            let mut children = BTreeSet::new();
            for relation in item
                .relations
                .as_deref()
                .unwrap_or_default()
                .iter()
                .filter(|r| r.rel.as_ref() == WORK_ITEM_CHILD_RELATION)
            {
                ensure!(
                    children.insert(relation.url.work_item_id_for_organization(organization)?),
                    "Duplicate Child relation in source"
                );
            }
            queue.extend(children.into_iter().map(|child| (child, Some(id))));
        }
    }
    ensure!(
        parents.len() == items.len(),
        "Copy snapshot contains items outside the selected subtree"
    );
    let definitions: BTreeMap<_, _> = definitions
        .iter()
        .map(|field| (field.reference_name.as_ref(), field))
        .collect();
    let relation_types: BTreeMap<_, _> = relation_types
        .iter()
        .map(|kind| (kind.reference_name.as_ref(), kind))
        .collect();
    let mut nodes = Vec::new();
    let mut omissions = Vec::new();
    let mut deferred = BTreeMap::new();
    for id in order {
        let item = &items[&id];
        let work_item_type = item
            .fields
            .work_item_type
            .clone()
            .ok_or_else(|| eyre::eyre!("Missing System.WorkItemType field"))?;
        let type_fields = type_fields
            .get(&work_item_type)
            .ok_or_else(|| eyre::eyre!("Missing field metadata for a source type"))?;
        let supported: BTreeSet<_> = type_fields
            .iter()
            .map(|f| f.reference_name.as_ref())
            .collect();
        let mut fields = AzureDevOpsWorkItemPatchFields::default();
        let item_fields = item.fields.to_patch_fields()?;
        for (name, value) in &item_fields {
            if generated_field(name.as_ref())
                || !supported.contains(name.as_ref())
                || definitions.get(name.as_ref()).is_none_or(|f| f.read_only)
            {
                omit(&mut omissions, id, "field", name.as_ref());
                continue;
            }
            let definition = definitions[name.as_ref()];
            fields.insert(
                name.clone(),
                if definition.is_identity {
                    identity_value(value)?
                } else {
                    value.clone()
                },
            );
        }
        if id == root
            && let Some(title) = &options.title
        {
            fields.insert(
                "System.Title".parse()?,
                ArbitraryJson::try_from_facet(title)?,
            );
        }
        for required in type_fields
            .iter()
            .filter(|f| f.always_required && f.default_value.is_none())
        {
            if !generated_field(required.reference_name.as_ref())
                && definitions
                    .get(required.reference_name.as_ref())
                    .is_some_and(|f| !f.read_only)
            {
                ensure!(
                    fields.contains_key(required.reference_name.as_ref()),
                    "A required writable field is missing from the source: {}",
                    required.reference_name
                );
            }
        }
        let mut external_relations = Vec::new();
        for relation in item.relations.as_deref().unwrap_or_default() {
            if relation.rel.as_ref() == WORK_ITEM_CHILD_RELATION {
                if !options.deep {
                    omit(&mut omissions, id, "child", &relation.url.to_string());
                }
                continue;
            }
            if relation.rel.as_ref() == WORK_ITEM_PARENT_RELATION {
                if id == root {
                    if options.keep_parent {
                        external_relations.push(AzureDevOpsWorkItemRelationInput {
                            rel: relation.rel.clone(),
                            url: relation.url.clone(),
                            attributes: copied_attributes(relation)?,
                        });
                    } else {
                        omit(
                            &mut omissions,
                            id,
                            "external-parent",
                            &relation.url.to_string(),
                        );
                    }
                }
                continue;
            }
            if let Ok(target) = relation.url.work_item_id_for_organization(organization)
                && items.contains_key(&target)
            {
                let kind = relation_types.get(relation.rel.as_ref()).ok_or_else(|| {
                    eyre::eyre!("Unknown internal relation type: {}", relation.rel)
                })?;
                let opposite = kind
                    .attributes
                    .get("oppositeEndReferenceName")
                    .and_then(|value| {
                        facet_json::from_str::<String>(value.as_ref())
                            .ok()
                            .and_then(|name| {
                                name.parse::<AzureDevOpsWorkItemRelationTypeName>().ok()
                            })
                    });
                let directional = kind
                    .attributes
                    .get("directional")
                    .and_then(|value| facet_json::from_str::<bool>(value.as_ref()).ok())
                    .unwrap_or(false);
                let (from, to, rel) =
                    if let Some(opposite) = opposite.filter(|opposite| opposite < &relation.rel) {
                        (target, id, opposite)
                    } else if !directional && target < id {
                        (target, id, relation.rel.clone())
                    } else {
                        (id, target, relation.rel.clone())
                    };
                let attributes = copied_attributes(relation)?;
                deferred.entry((from, to, rel.clone())).or_insert_with(|| {
                    AzureDevOpsWorkItemCopyEdge {
                        source: from,
                        target: to,
                        rel,
                        attributes: attributes.clone(),
                    }
                });
            } else if options.keep_external_relations && relation.rel.as_ref() != "AttachedFile" {
                external_relations.push(AzureDevOpsWorkItemRelationInput {
                    rel: relation.rel.clone(),
                    url: relation.url.clone(),
                    attributes: copied_attributes(relation)?,
                });
            } else {
                omit(
                    &mut omissions,
                    id,
                    if relation.rel.as_ref() == "AttachedFile" {
                        "attachment"
                    } else {
                        "external-relation"
                    },
                    &relation.url.to_string(),
                );
            }
        }
        if item.fields.comment_count.is_some_and(|count| count > 0) {
            omit(
                &mut omissions,
                id,
                "comments",
                "Comment history is not copied",
            );
        }
        AzureDevOpsJsonPatch::work_item_create_patch(fields.clone(), &external_relations)?;
        nodes.push(AzureDevOpsWorkItemCopyNode {
            source_id: id,
            source_revision: item.rev,
            parent: parents[&id],
            work_item_type,
            fields,
            external_relations,
        });
    }
    Ok(AzureDevOpsWorkItemCopyPlan {
        root,
        organization: organization.clone(),
        project: project.clone().into_owned(),
        as_of,
        nodes,
        deferred_relations: deferred.into_values().collect(),
        omissions,
    })
}

impl AzureDevOpsWorkItemCopyJournal {
    pub fn load(path: &Path) -> Result<Self> {
        let journal: Self = facet_json::from_str(&std::fs::read_to_string(path)?)
            .map_err(|_| eyre::eyre!("Invalid copy journal"))?;
        ensure!(journal.version == 1, "Unsupported copy journal version");
        Ok(journal)
    }

    pub(crate) fn save(&self, path: &Path) -> Result<()> {
        let directory = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let mut temporary = tempfile::NamedTempFile::new_in(directory)?;
        temporary.write_all(facet_json::to_string_pretty(self)?.as_bytes())?;
        temporary.as_file().sync_all()?;
        temporary
            .persist(path)
            .map_err(|error| eyre::eyre!("Saving copy journal: {}", error.error))?;
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsWorkItemCopyPlan);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsWorkItemCopyPlan);
cloud_terrastodon_registry::register_thing!(AzureDevOpsWorkItemCopyJournal);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsWorkItemCopyJournal);
