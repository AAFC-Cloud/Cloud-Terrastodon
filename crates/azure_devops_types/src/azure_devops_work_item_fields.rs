use crate::AzureDevOpsProjectName;
use crate::AzureDevOpsWorkItemId;
use crate::AzureDevOpsWorkItemIdentity;
use crate::AzureDevOpsWorkItemPatchFields;
use crate::AzureDevOpsWorkItemType;
use chrono::{DateTime, Utc};
use cloud_terrastodon_azure_types::ArbitraryJson;
use eyre::Result;
use facet_reflect::{HasFields, Peek};
use std::collections::BTreeMap;

/// The fields returned in an Azure DevOps work item response.
///
/// Azure DevOps permits custom fields, so the known system fields are modeled
/// explicitly and the remaining field values are retained as arbitrary JSON.
#[derive(Debug, Clone, PartialEq, Eq, Default, arbitrary::Arbitrary, facet::Facet)]
pub struct AzureDevOpsWorkItemFields {
    #[facet(
        rename = "Microsoft.VSTS.Common.Priority",
        default,
        skip_serializing_if = Option::is_none
    )]
    pub common_priority: Option<i32>,
    #[facet(
        rename = "Microsoft.VSTS.Common.StateChangeDate",
        default,
        skip_serializing_if = Option::is_none
    )]
    pub common_state_change_date: Option<DateTime<Utc>>,
    #[facet(
        rename = "Microsoft.VSTS.Common.ValueArea",
        default,
        skip_serializing_if = Option::is_none
    )]
    pub common_value_area: Option<String>,
    #[facet(
        rename = "System.AreaId",
        default,
        skip_serializing_if = Option::is_none
    )]
    pub area_id: Option<i32>,
    #[facet(
        rename = "System.AreaLevel1",
        default,
        skip_serializing_if = Option::is_none
    )]
    pub area_level1: Option<String>,
    #[facet(
        rename = "System.AreaPath",
        default,
        skip_serializing_if = Option::is_none
    )]
    pub area_path: Option<String>,
    #[facet(
        rename = "System.AuthorizedAs",
        default,
        skip_serializing_if = Option::is_none
    )]
    pub authorized_as: Option<AzureDevOpsWorkItemIdentity>,
    #[facet(
        rename = "System.AuthorizedDate",
        default,
        skip_serializing_if = Option::is_none
    )]
    pub authorized_date: Option<DateTime<Utc>>,
    #[facet(
        rename = "System.ChangedBy",
        default,
        skip_serializing_if = Option::is_none
    )]
    pub changed_by: Option<AzureDevOpsWorkItemIdentity>,
    #[facet(
        rename = "System.ChangedDate",
        default,
        skip_serializing_if = Option::is_none
    )]
    pub changed_date: Option<DateTime<Utc>>,
    #[facet(
        rename = "System.CommentCount",
        default,
        skip_serializing_if = Option::is_none
    )]
    pub comment_count: Option<i32>,
    #[facet(
        rename = "System.CreatedBy",
        default,
        skip_serializing_if = Option::is_none
    )]
    pub created_by: Option<AzureDevOpsWorkItemIdentity>,
    #[facet(
        rename = "System.CreatedDate",
        default,
        skip_serializing_if = Option::is_none
    )]
    pub created_date: Option<DateTime<Utc>>,
    #[facet(
        rename = "System.Description",
        default,
        skip_serializing_if = Option::is_none
    )]
    pub description: Option<String>,
    #[facet(rename = "System.Id", default, skip_serializing_if = Option::is_none)]
    pub id: Option<AzureDevOpsWorkItemId>,
    #[facet(
        rename = "System.IterationId",
        default,
        skip_serializing_if = Option::is_none
    )]
    pub iteration_id: Option<i32>,
    #[facet(
        rename = "System.IterationLevel1",
        default,
        skip_serializing_if = Option::is_none
    )]
    pub iteration_level1: Option<String>,
    #[facet(
        rename = "System.IterationPath",
        default,
        skip_serializing_if = Option::is_none
    )]
    pub iteration_path: Option<String>,
    #[facet(
        rename = "System.NodeName",
        default,
        skip_serializing_if = Option::is_none
    )]
    pub node_name: Option<String>,
    #[facet(
        rename = "System.PersonId",
        default,
        skip_serializing_if = Option::is_none
    )]
    pub person_id: Option<i32>,
    #[facet(
        rename = "System.Reason",
        default,
        skip_serializing_if = Option::is_none
    )]
    pub reason: Option<String>,
    #[facet(rename = "System.Rev", default, skip_serializing_if = Option::is_none)]
    pub revision: Option<i32>,
    #[facet(
        rename = "System.RevisedDate",
        default,
        skip_serializing_if = Option::is_none
    )]
    pub revised_date: Option<DateTime<Utc>>,
    #[facet(rename = "System.State", default, skip_serializing_if = Option::is_none)]
    pub state: Option<String>,
    #[facet(rename = "System.Title", default, skip_serializing_if = Option::is_none)]
    pub title: Option<String>,
    #[facet(
        rename = "System.TeamProject",
        default,
        skip_serializing_if = Option::is_none
    )]
    pub team_project: Option<AzureDevOpsProjectName>,
    #[facet(
        rename = "System.Watermark",
        default,
        skip_serializing_if = Option::is_none
    )]
    pub watermark: Option<i32>,
    #[facet(
        rename = "System.WorkItemType",
        default,
        skip_serializing_if = Option::is_none
    )]
    pub work_item_type: Option<AzureDevOpsWorkItemType>,
    #[facet(flatten, default)]
    pub additional: BTreeMap<String, ArbitraryJson>,
}

impl AzureDevOpsWorkItemFields {
    /// Converts response fields to the arbitrary JSON map used by patch
    /// construction and field-oriented CLI commands. Facet reflection supplies
    /// the effective names and skips absent optional fields, so renamed and
    /// flattened fields stay in sync with the deserialization shape above.
    /// Values are serialized individually because the patch representation is
    /// raw JSON; the complete fields object is never serialized and reparsed.
    pub fn to_patch_fields(&self) -> Result<AzureDevOpsWorkItemPatchFields> {
        let mut fields = BTreeMap::new();
        for (field, value) in Peek::new(self).into_struct()?.fields_for_serialize() {
            fields.insert(field.effective_name().parse()?, arbitrary_json(value)?);
        }
        Ok(fields.into())
    }

    /// Returns a field value in the raw JSON form used by patch values.
    pub fn field(&self, name: &str) -> Result<Option<ArbitraryJson>> {
        for (field, value) in Peek::new(self).into_struct()?.fields_for_serialize() {
            if field.effective_name() == name {
                return Ok(Some(arbitrary_json(value)?));
            }
        }
        Ok(None)
    }
}

fn arbitrary_json(value: Peek<'_, '_>) -> Result<ArbitraryJson> {
    Ok(facet_json::RawJson::from_owned(facet_json::peek_to_string(value)?).into())
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsWorkItemFields);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsWorkItemFields);
