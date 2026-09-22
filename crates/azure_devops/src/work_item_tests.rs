//! Offline tests use generated identifiers and synthetic content. No test in
//! this module sends a mutation to a service.
use crate::work_item_copy::WorkItemCopyWriter;
use crate::*;
use cloud_terrastodon_azure_types::ArbitraryJson;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AuthContext;
use cloud_terrastodon_credentials::AuthSource;
use cloud_terrastodon_credentials::AzureDevOpsAuthContext;
use eyre::Result;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::sync::Mutex;

fn scope() -> Result<(
    AzureDevOpsOrganizationUrl,
    AzureDevOpsProjectArgument<'static>,
    AzureDevOpsAuthContext,
)> {
    let suffix = rand::random::<u32>();
    Ok((
        format!("fixture-{suffix:x}").parse()?,
        format!("fixture {suffix:x}").parse()?,
        AzureDevOpsAuthContext::None,
    ))
}

fn ids(count: usize) -> Vec<AzureDevOpsWorkItemId> {
    let start = i32::from(rand::random::<u16>()) + 1;
    (0..count)
        .map(|offset| AzureDevOpsWorkItemId::new(start + offset as i32).unwrap())
        .collect()
}

fn item(
    organization: &AzureDevOpsOrganizationUrl,
    project: &AzureDevOpsProjectArgument<'_>,
    id: AzureDevOpsWorkItemId,
) -> Result<AzureDevOpsWorkItem> {
    Ok(AzureDevOpsWorkItem {
        id,
        rev: 1,
        url: AzureDevOpsWorkItemUrl::new(Cow::Borrowed(organization), id).into_url()?,
        fields: AzureDevOpsWorkItemFields {
            title: Some("Synthetic title".to_owned()),
            work_item_type: Some("Synthetic type".parse()?),
            team_project: Some(project.to_string().parse()?),
            additional: BTreeMap::from([
                (
                    "Custom.Flag".to_owned(),
                    ArbitraryJson::try_from_facet(&false)?,
                ),
                (
                    "Custom.a~/b".to_owned(),
                    ArbitraryJson::try_from_facet(&"escaped".to_owned())?,
                ),
            ]),
            ..Default::default()
        },
        relations: Some(Vec::new()),
        links: None,
        comment_version_ref: None,
        multiline_fields_format: BTreeMap::new(),
    })
}

fn relation(
    organization: &AzureDevOpsOrganizationUrl,
    kind: &str,
    target: AzureDevOpsWorkItemId,
) -> Result<AzureDevOpsWorkItemRelation> {
    Ok(AzureDevOpsWorkItemRelation {
        rel: kind.parse()?,
        url: AzureDevOpsWorkItemUrl::new(Cow::Owned(organization.clone()), target).into(),
        attributes: Default::default(),
    })
}

fn tree(
    organization: &AzureDevOpsOrganizationUrl,
    project: &AzureDevOpsProjectArgument<'_>,
) -> Result<Vec<AzureDevOpsWorkItem>> {
    let ids = ids(3);
    let mut root = item(organization, project, ids[0])?;
    let mut child = item(organization, project, ids[1])?;
    let mut grandchild = item(organization, project, ids[2])?;
    root.relations = Some(vec![relation(
        organization,
        WORK_ITEM_CHILD_RELATION,
        child.id,
    )?]);
    child.relations = Some(vec![
        relation(organization, WORK_ITEM_PARENT_RELATION, root.id)?,
        relation(organization, WORK_ITEM_CHILD_RELATION, grandchild.id)?,
    ]);
    grandchild.relations = Some(vec![relation(
        organization,
        WORK_ITEM_PARENT_RELATION,
        child.id,
    )?]);
    // Deliberately not in dependency order.
    Ok(vec![grandchild, root, child])
}

fn plan(
    organization: &AzureDevOpsOrganizationUrl,
    project: &AzureDevOpsProjectArgument<'_>,
    root: AzureDevOpsWorkItemId,
    items: Vec<AzureDevOpsWorkItem>,
) -> Result<AzureDevOpsWorkItemCopyPlan> {
    let definitions: Vec<_> = ["System.Title", "Custom.Flag"]
        .into_iter()
        .map(|name| {
            Ok(AzureDevOpsWorkItemFieldDefinition {
                name: name.to_owned(),
                reference_name: name.parse()?,
                field_type: "string".to_owned(),
                read_only: false,
                is_identity: false,
                description: None,
                url: None,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let fields = definitions
        .iter()
        .map(|field| AzureDevOpsWorkItemTypeField {
            name: field.name.clone(),
            reference_name: field.reference_name.clone(),
            always_required: field.reference_name.as_ref() == "System.Title",
            default_value: None,
            allowed_values: None,
            dependent_fields: Vec::new(),
            help_text: None,
            url: None,
        })
        .collect();
    let relation_types = vec![AzureDevOpsWorkItemRelationType {
        name: "Related".to_owned(),
        reference_name: "System.LinkTypes.Related".parse()?,
        attributes: BTreeMap::from([(
            "directional".to_owned(),
            ArbitraryJson::try_from_facet(&false)?,
        )]),
        url: None,
    }];
    build_work_item_copy_plan(
        organization,
        project,
        root,
        &AzureDevOpsWorkItemCopyOptions {
            deep: true,
            ..Default::default()
        },
        chrono::Utc::now(),
        items,
        &definitions,
        &BTreeMap::from([("Synthetic type".parse()?, fields)]),
        &relation_types,
    )
}

#[tokio::test]
async fn batch_read_rejects_an_empty_selection_before_authentication_or_network() -> Result<()> {
    let (organization, project, _) = scope()?;
    let error = AzureDevOpsWorkItemsGetRequest {
        org_url: std::borrow::Cow::Owned(organization),
        project: Some(project),
        auth_context: std::borrow::Cow::Owned(AzureDevOpsAuthContext::None),
        ids: Vec::new(),
        expand: Default::default(),
        fields: Vec::new(),
        as_of: None,
    }
    .await
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("Specify at least one work item ID")
    );
    Ok(())
}

#[test]
fn create_patch_preserves_typed_fields_and_parent_relations() -> Result<()> {
    let (organization, _, _) = scope()?;
    let parent = ids(1)[0];
    let relation = AzureDevOpsWorkItemRelationInput::parent(&organization, parent)?;
    let patch = AzureDevOpsJsonPatch::work_item_create_patch(
        BTreeMap::from([
            (
                "System.Title".parse()?,
                ArbitraryJson::try_from_facet(&"Fixture".to_owned())?,
            ),
            (
                "Custom.Flag".parse()?,
                ArbitraryJson::try_from_facet(&false)?,
            ),
            (
                "Custom.a~/b".parse()?,
                ArbitraryJson::try_from_facet(&"escaped".to_owned())?,
            ),
        ])
        .into(),
        &[relation],
    )?;
    let encoded = facet_json::to_string(&patch)?;
    let patch: AzureDevOpsJsonPatch = facet_json::from_str(&encoded)?;
    assert!(patch.iter().any(|op| {
        op.path() == "/fields/Custom.Flag"
            && op.value().is_some_and(|value| value.as_ref() == "false")
    }));
    assert!(patch.iter().any(|op| op.path() == "/fields/Custom.a~0~1b"));
    let parent_input: AzureDevOpsWorkItemRelationInput =
        facet_json::from_str(patch.last().unwrap().value().unwrap().as_ref())?;
    assert!(parent_input.rel.as_ref() == WORK_ITEM_PARENT_RELATION);
    Ok(())
}

#[test]
fn patches_distinguish_null_from_remove() -> Result<()> {
    assert!(AzureDevOpsJsonPatch::new(Vec::new()).is_err());
    let set = AzureDevOpsJsonPatchOperation::Add {
        path: "/fields/Custom.Flag".to_owned(),
        value: ArbitraryJson::try_from_facet(&())?,
    };
    let remove = AzureDevOpsJsonPatchOperation::Remove {
        path: "/fields/Custom.Flag".to_owned(),
    };
    let encoded = facet_json::to_string(&AzureDevOpsJsonPatch::try_from(vec![set, remove])?)?;
    assert!(encoded.contains("\"value\":null"));
    let decoded: AzureDevOpsJsonPatch = facet_json::from_str(&encoded)?;
    assert!(decoded[0].value().is_some() && decoded[1].value().is_none());
    Ok(())
}

#[test]
fn relation_selection_rejects_ambiguity_and_uses_the_current_index() -> Result<()> {
    let (organization, project, _) = scope()?;
    let ids = ids(3);
    let mut source = item(&organization, &project, ids[0])?;
    source.relations = Some(vec![
        relation(&organization, "System.LinkTypes.Related", ids[1])?,
        relation(&organization, "System.LinkTypes.Related", ids[2])?,
    ]);
    let selector = AzureDevOpsWorkItemRelationSelector {
        rel: "System.LinkTypes.Related".parse()?,
        target: AzureDevOpsWorkItemUrl::new(Cow::Owned(organization.clone()), ids[2]),
    };
    let change = AzureDevOpsWorkItemRelationChange::Remove(selector.clone());
    assert!(change.patch(&organization, &source)?[0].path() == "/relations/1");
    source.relations.as_mut().unwrap().reverse();
    assert!(change.patch(&organization, &source)?[0].path() == "/relations/0");
    source.relations.as_mut().unwrap().push(relation(
        &organization,
        "System.LinkTypes.Related",
        ids[2],
    )?);
    assert!(selector.index(&organization, &source).is_err());
    let create = AzureDevOpsWorkItemRelationChange::Create(AzureDevOpsWorkItemRelationInput {
        rel: selector.rel,
        url: AzureDevOpsWorkItemUrl::new(Cow::Owned(organization.clone()), ids[2]).into(),
        attributes: Default::default(),
    });
    assert!(create.patch(&organization, &source).is_err());
    Ok(())
}

#[test]
fn browser_auth_retains_tenant_and_revision_uses_a_test_operation() -> Result<()> {
    let tenant = format!("{:032x}", rand::random::<u128>()).parse()?;
    let auth_context =
        AzureDevOpsAuthContext::for_tenant(&AuthContext::explicit(AuthSource::Browser), tenant)?;
    let request = crate::azure_devops_rest::authenticate_azure_devops_request(
        cloud_terrastodon_rest::RestRequest::new(
            reqwest::Method::PATCH,
            "https://dev.azure.com/fixture/_apis/wit/workitems/1?api-version=7.1",
        )?,
        &auth_context,
    )?;
    assert!(request.tenant == Some(tenant));
    let revision = AzureDevOpsJsonPatchOperation::Test {
        path: "/rev".to_owned(),
        value: ArbitraryJson::try_from_facet(&3)?,
    };
    assert!(
        matches!(revision, AzureDevOpsJsonPatchOperation::Test { .. }) && revision.path() == "/rev"
    );
    Ok(())
}

#[test]
fn copy_orders_parents_and_rejects_incomplete_or_cyclic_snapshots() -> Result<()> {
    let (organization, project, _) = scope()?;
    let items = tree(&organization, &project)?;
    let root = items[1].id;
    let copied = plan(&organization, &project, root, items.clone())?;
    assert!(copied.nodes.len() == 3 && copied.nodes[0].source_id == root);
    assert!(copied.nodes[2].parent == Some(copied.nodes[1].source_id));
    assert!(!copied.nodes[0].fields.contains_key("System.TeamProject"));
    assert!(copied.nodes[0].fields["Custom.Flag"].as_ref() == "false");
    let mut historical = items.clone();
    for item in &mut historical {
        let mut url = item.url.clone();
        url.path_segments_mut()
            .map_err(|_| eyre::eyre!("Invalid synthetic work item URL"))?
            .extend(["revisions", &item.rev.to_string()]);
        item.url = url;
    }
    assert!(plan(&organization, &project, root, historical.clone()).is_ok());
    historical[0].rev += 1;
    assert!(plan(&organization, &project, root, historical).is_err());
    let mut incomplete = items.clone();
    incomplete.remove(0);
    assert!(plan(&organization, &project, root, incomplete).is_err());
    let mut cycle = items;
    let leaf = cycle[0].id;
    cycle[1].relations.as_mut().unwrap().push(relation(
        &organization,
        WORK_ITEM_PARENT_RELATION,
        leaf,
    )?);
    assert!(plan(&organization, &project, root, cycle.clone()).is_err());
    cycle[1].relations.as_mut().unwrap().pop();
    cycle[0].relations.as_mut().unwrap().push(relation(
        &organization,
        WORK_ITEM_CHILD_RELATION,
        root,
    )?);
    assert!(plan(&organization, &project, root, cycle).is_err());
    Ok(())
}

#[test]
fn copy_deduplicates_reciprocal_internal_links() -> Result<()> {
    let (organization, project, _) = scope()?;
    let mut items = tree(&organization, &project)?;
    let root = items[1].id;
    let leaf = items[0].id;
    items[0].relations.as_mut().unwrap().push(relation(
        &organization,
        "System.LinkTypes.Related",
        root,
    )?);
    items[1].relations.as_mut().unwrap().push(relation(
        &organization,
        "System.LinkTypes.Related",
        leaf,
    )?);
    let copied = plan(&organization, &project, root, items)?;
    assert!(copied.deferred_relations.len() == 1);
    Ok(())
}

#[tokio::test]
async fn copy_scope_rejection_happens_before_authentication_or_network() -> Result<()> {
    let (organization, project, _) = scope()?;
    let auth_context = AzureDevOpsAuthContext::None;
    let ids = ids(2);
    let result = AzureDevOpsWorkItemCopyPlanRequest {
        org_url: Cow::Borrowed(&organization),
        project,
        auth_context: Cow::Borrowed(&auth_context),
        root: ids[0],
        options: AzureDevOpsWorkItemCopyOptions {
            deep: true,
            allowed_ids: vec![ids[1]],
            ..Default::default()
        },
    }
    .await;
    assert!(result.is_err_and(|error| error.to_string().contains("allowed-ids")));
    Ok(())
}

struct MemoryWriter {
    organization: AzureDevOpsOrganizationUrl,
    project: AzureDevOpsProjectArgument<'static>,
    calls: Mutex<Vec<AzureDevOpsJsonPatch>>,
    next_id: Mutex<i32>,
    fail_after: Option<usize>,
    items: Mutex<BTreeMap<AzureDevOpsWorkItemId, AzureDevOpsWorkItem>>,
}

#[async_trait]
impl WorkItemCopyWriter for MemoryWriter {
    async fn create(
        &self,
        _kind: AzureDevOpsWorkItemType,
        patch: AzureDevOpsJsonPatch,
    ) -> Result<AzureDevOpsWorkItem> {
        let mut calls = self.calls.lock().unwrap();
        if self.fail_after == Some(calls.len()) {
            eyre::bail!("Synthetic uncertain response");
        }
        let mut next_id = self.next_id.lock().unwrap();
        *next_id += 1;
        let mut created = item(
            &self.organization,
            &self.project,
            AzureDevOpsWorkItemId::new(*next_id)?,
        )?;
        for operation in patch.iter().filter(|op| op.path() == "/relations/-") {
            created
                .relations
                .as_mut()
                .unwrap()
                .push(facet_json::from_str(operation.value().unwrap().as_ref())?);
        }
        calls.push(patch);
        self.items
            .lock()
            .unwrap()
            .insert(created.id, created.clone());
        Ok(created)
    }
    async fn relate(
        &self,
        source: AzureDevOpsWorkItemId,
        relation: AzureDevOpsWorkItemRelationInput,
    ) -> Result<()> {
        let attributes: AzureDevOpsWorkItemRelationAttributes =
            facet_json::from_str(ArbitraryJson::try_from_facet(&relation.attributes)?.as_ref())?;
        self.items
            .lock()
            .unwrap()
            .get_mut(&source)
            .unwrap()
            .relations
            .as_mut()
            .unwrap()
            .push(AzureDevOpsWorkItemRelation {
                rel: relation.rel,
                url: relation.url,
                attributes,
            });
        Ok(())
    }
    async fn read(&self, ids: Vec<AzureDevOpsWorkItemId>) -> Result<Vec<AzureDevOpsWorkItem>> {
        let items = self.items.lock().unwrap();
        Ok(ids.into_iter().map(|id| items[&id].clone()).collect())
    }
}

#[tokio::test]
async fn copy_execution_reuses_new_parent_urls_and_completed_journals() -> Result<()> {
    let (organization, project, auth_context) = scope()?;
    let mut items = tree(&organization, &project)?;
    let leaf = items[0].id;
    items[1].relations.as_mut().unwrap().push(relation(
        &organization,
        "System.LinkTypes.Related",
        leaf,
    )?);
    let plan = plan(&organization, &project, items[1].id, items)?;
    let directory = tempfile::tempdir()?;
    let request = AzureDevOpsWorkItemCopyRequest {
        org_url: Cow::Borrowed(&organization),
        project: project.clone(),
        auth_context: Cow::Borrowed(&auth_context),
        plan,
        journal_path: directory.path().join("copy.json"),
        resume: false,
        suppress_notifications: false,
    };
    let writer = MemoryWriter {
        organization: organization.clone(),
        project: project.clone().into_owned(),
        calls: Mutex::new(Vec::new()),
        next_id: Mutex::new(i32::from(u16::MAX) + 1024),
        fail_after: None,
        items: Mutex::new(BTreeMap::new()),
    };
    let result = request.clone().execute_with(&writer).await?;
    assert!(result.complete && result.created.len() == 3);
    assert!(result.completed_relations == 1);
    {
        let calls = writer.calls.lock().unwrap();
        for index in 1..calls.len() {
            let operation = calls[index]
                .iter()
                .find(|op| op.path() == "/relations/-")
                .unwrap();
            let parent: AzureDevOpsWorkItemRelationInput =
                facet_json::from_str(operation.value().unwrap().as_ref())?;
            assert!(parent.url.work_item() == Some(&result.created[index - 1].new_url));
        }
    }
    let mut resumed = request;
    resumed.resume = true;
    resumed.execute_with(&writer).await?;
    assert!(writer.calls.lock().unwrap().len() == 3);
    // A second process holding the companion lock prevents any further writes.
    let lock = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(directory.path().join("copy.json.lock"))?;
    lock.try_lock()?;
    let loaded = AzureDevOpsWorkItemCopyJournal::load(&directory.path().join("copy.json"))?;
    let busy = AzureDevOpsWorkItemCopyRequest {
        org_url: Cow::Borrowed(&organization),
        project: project.clone(),
        auth_context: Cow::Borrowed(&auth_context),
        plan: loaded.plan,
        journal_path: directory.path().join("copy.json"),
        resume: true,
        suppress_notifications: false,
    };
    assert!(busy.execute_with(&writer).await.is_err());
    assert!(writer.calls.lock().unwrap().len() == 3);
    Ok(())
}

#[tokio::test]
async fn uncertain_create_keeps_progress_and_prevents_automatic_replay() -> Result<()> {
    let (organization, project, auth_context) = scope()?;
    let items = tree(&organization, &project)?;
    let plan = plan(&organization, &project, items[1].id, items)?;
    let directory = tempfile::tempdir()?;
    let request = AzureDevOpsWorkItemCopyRequest {
        org_url: Cow::Borrowed(&organization),
        project: project.clone(),
        auth_context: Cow::Borrowed(&auth_context),
        plan,
        journal_path: directory.path().join("copy.json"),
        resume: false,
        suppress_notifications: false,
    };
    let writer = MemoryWriter {
        organization: organization.clone(),
        project: project.clone().into_owned(),
        calls: Mutex::new(Vec::new()),
        next_id: Mutex::new(i32::from(u16::MAX) + 1024),
        fail_after: Some(1),
        items: Mutex::new(BTreeMap::new()),
    };
    assert!(request.clone().execute_with(&writer).await.is_err());
    let journal = AzureDevOpsWorkItemCopyJournal::load(&request.journal_path)?;
    assert!(journal.created.len() == 1 && journal.in_flight.is_some() && !journal.complete);
    let mut resumed = request;
    resumed.resume = true;
    assert!(resumed.execute_with(&writer).await.is_err());
    assert!(writer.calls.lock().unwrap().len() == 1);
    Ok(())
}

/// Optional smoke test. Configuration is supplied at runtime, never recorded as
/// a fixture. Only the explicit IDs are read; no queries or network traversal.
/// CT_TEST_AZDO_COPY_ROOT optionally checks a closed subtree using this snapshot.
/// Errors and assertions intentionally omit service response bodies and scope.
#[tokio::test]
#[ignore = "requires CT_TEST_AZDO_ORG, PROJECT, TENANT_ID, IDS and an existing browser session"]
async fn live_work_item_deserialization() -> Result<()> {
    fn config(name: &str) -> Result<String> {
        std::env::var(name).map_err(|_| eyre::eyre!("Live test configuration is missing"))
    }
    async fn verify(stage: &mut &'static str) -> Result<()> {
        let organization = config("CT_TEST_AZDO_ORG")?.parse()?;
        let project: AzureDevOpsProjectArgument<'static> =
            config("CT_TEST_AZDO_PROJECT")?.parse()?;
        let tenant = config("CT_TEST_AZDO_TENANT_ID")?.parse()?;
        let ids = config("CT_TEST_AZDO_IDS")?
            .split(',')
            .map(|id| id.trim().parse())
            .collect::<Result<Vec<AzureDevOpsWorkItemId>>>()?;
        let mut auth = AuthContext::resolve(AuthSource::Browser)?;
        if let AuthContext::Resolved { headless, .. } = &mut auth {
            *headless = true;
        }
        let auth_context = AzureDevOpsAuthContext::for_tenant(&auth, tenant)?;
        *stage = "selected item reads";
        let as_of = chrono::Utc::now();
        let items = AzureDevOpsWorkItemsGetRequest {
            org_url: std::borrow::Cow::Borrowed(&organization),
            project: Some(project.clone()),
            auth_context: std::borrow::Cow::Borrowed(&auth_context),
            ids: ids.clone(),
            expand: Default::default(),
            fields: Vec::new(),
            as_of: Some(as_of.clone()),
        }
        .await?;
        eyre::ensure!(
            !items.is_empty()
                && items
                    .iter()
                    .all(|item| { item.rev > 0 && item.fields.title.is_some() }),
            "Incomplete work item responses"
        );
        *stage = "field definitions";
        let fields = AzureDevOpsWorkItemFieldDefinitionListRequest {
            org_url: std::borrow::Cow::Borrowed(&organization),
            auth_context: std::borrow::Cow::Borrowed(&auth_context),
        }
        .await?;
        eyre::ensure!(!fields.is_empty(), "No field definitions returned");
        let kinds: std::collections::BTreeSet<AzureDevOpsWorkItemType> = items
            .iter()
            .map(|item| {
                item.fields
                    .work_item_type
                    .clone()
                    .ok_or_else(|| eyre::eyre!("Missing System.WorkItemType field"))
            })
            .collect::<Result<_>>()?;
        let mut type_fields = BTreeMap::new();
        for work_item_type in kinds {
            *stage = "type field definitions";
            let fields = AzureDevOpsWorkItemTypeFieldListRequest {
                org_url: std::borrow::Cow::Borrowed(&organization),
                project: project.clone(),
                auth_context: std::borrow::Cow::Borrowed(&auth_context),
                work_item_type: work_item_type.clone(),
            }
            .await?;
            eyre::ensure!(!fields.is_empty(), "No type fields returned");
            type_fields.insert(work_item_type, fields);
        }
        *stage = "relation definitions";
        let relations = AzureDevOpsWorkItemRelationTypeListRequest {
            org_url: std::borrow::Cow::Borrowed(&organization),
            auth_context: std::borrow::Cow::Borrowed(&auth_context),
        }
        .await?;
        eyre::ensure!(!relations.is_empty(), "No relation definitions returned");
        if let Ok(root) = std::env::var("CT_TEST_AZDO_COPY_ROOT") {
            *stage = "bounded copy planning";
            let plan = build_work_item_copy_plan(
                &organization,
                &project,
                root.parse()?,
                &AzureDevOpsWorkItemCopyOptions {
                    deep: true,
                    allowed_ids: ids,
                    ..Default::default()
                },
                as_of,
                items,
                &fields,
                &type_fields,
                &relations,
            )
            .map_err(|error| {
                // Classify failures using fixed labels only; never include the
                // dynamic field, identity, URL, or item values from the error.
                let reason = error.to_string();
                *stage = if reason.starts_with("A required writable field") {
                    "planning: required field"
                } else if reason.starts_with("Missing field") {
                    "planning: missing field"
                } else if reason.starts_with("Identity field") {
                    "planning: identity"
                } else if reason.starts_with("Copy currently supports") {
                    "planning: project scope"
                } else if reason.starts_with("Invalid work item relation") {
                    "planning: relation URL"
                } else if reason.starts_with("Source Parent") {
                    "planning: parent agreement"
                } else if reason.starts_with("Root Parent") {
                    "planning: root parent"
                } else if reason.starts_with("Creation requires") {
                    "planning: title"
                } else if reason.starts_with("Copy snapshot") {
                    "planning: snapshot"
                } else if reason.starts_with("Unknown internal relation") {
                    "planning: relation definition"
                } else {
                    "planning: value conversion or validation"
                };
                eyre::eyre!("Copy planning failed")
            })?;
            eyre::ensure!(!plan.nodes.is_empty(), "Copy plan is empty");
        }
        Ok(())
    }
    let mut stage = "configuration";
    match tokio::time::timeout(std::time::Duration::from_secs(60), verify(&mut stage)).await {
        Ok(Ok(())) => Ok(()),
        _ => eyre::bail!("Read-only smoke test failed during {stage}; response details suppressed"),
    }
}
