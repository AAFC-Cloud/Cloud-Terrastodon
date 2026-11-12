use cloud_terrastodon_azure_devops_types::AzureDevOpsDescriptor;
use cloud_terrastodon_azure_devops_types::AzureDevOpsGroupMember;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectMember;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectPermissionObject;
use std::collections::HashMap;
use std::sync::Arc;

pub(crate) struct AzureDevOpsProjectMemberAccumulator {
    pub member: Arc<AzureDevOpsGroupMember>,
    pub permission_objects: HashMap<AzureDevOpsDescriptor, AzureDevOpsProjectPermissionObject>,
}
impl From<AzureDevOpsProjectMemberAccumulator> for AzureDevOpsProjectMember {
    fn from(value: AzureDevOpsProjectMemberAccumulator) -> Self {
        let mut permission_objects = value
            .permission_objects
            .into_values()
            .collect::<Vec<AzureDevOpsProjectPermissionObject>>();
        permission_objects.sort_by(|a, b| {
            a.display_name
                .to_lowercase()
                .cmp(&b.display_name.to_lowercase())
                .then_with(|| a.descriptor.to_string().cmp(&b.descriptor.to_string()))
        });

        AzureDevOpsProjectMember {
            descriptor: value.member.descriptor.clone(),
            display_name: value.member.display_name.clone(),
            principal_name: value.member.principal_name.clone(),
            mail_address: value.member.mail_address.clone(),
            origin: value.member.origin.clone(),
            origin_id: value.member.origin_id.clone(),
            subject_kind: value.member.subject_kind.clone(),
            permission_objects,
        }
    }
}
