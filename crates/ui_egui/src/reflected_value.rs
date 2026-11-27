use crate::portal_azure_devops_organization_view::PortalAzureDevOpsOrganizationView;
use crate::portal_tab_open::PortalTabOpen;
use crate::portal_tab_target::PortalTabTarget;
use crate::scope_role_assignment_principal_link::ScopeRoleAssignmentPrincipalLink;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_azure::AzureActivityLogEventLevel;
use cloud_terrastodon_azure::AzureLocationName;
use cloud_terrastodon_azure::AzureResourceGroupId;
use cloud_terrastodon_azure::AzureStorageAccountBlobContainerId;
use cloud_terrastodon_azure::AzureStorageAccountId;
use cloud_terrastodon_azure::AzureTenantId;
use cloud_terrastodon_azure::EntraApplicationClientId;
use cloud_terrastodon_azure::EntraApplicationObjectId;
use cloud_terrastodon_azure::EntraDirectoryAuditResult;
use cloud_terrastodon_azure::EntraGroupId;
use cloud_terrastodon_azure::EntraUserId;
use cloud_terrastodon_azure::ManagementGroupId;
use cloud_terrastodon_azure::ResourceType;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::ScopeImpl;
use cloud_terrastodon_azure::SubscriptionId;
use cloud_terrastodon_azure_devops::AzureDevOpsBuildDefinitionQueueStatus;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsServiceEndpointKind;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemId;
use cloud_terrastodon_bmc_helix_types::BmcHelixServerName;
use cloud_terrastodon_bmc_helix_types::BmcHelixServerUrl;
use cloud_terrastodon_bmc_helix_types::BmcHelixTicketRecordId;
use cloud_terrastodon_bmc_helix_types::BmcHelixWorkOrderId;
use eframe::egui::Ui;
use facet::Def;
use facet::Facet;
use facet_reflect::Peek;
use facet_reflect::PeekStruct;
use jiff::Timestamp;

/// A read-only view of a Facet value for inventory columns and object details.
#[derive(Clone, Copy)]
pub(crate) struct ReflectedValue<'a>(Peek<'a, 'static>);

impl<'a> ReflectedValue<'a> {
    pub fn new(value: &'a impl Facet<'static>) -> Self {
        Self(Peek::new(value))
    }

    pub fn field(self, name: &str) -> Option<Self> {
        let record = self.0.into_struct().ok()?;
        let field = record.ty().fields.iter().find(|field| field.name == name)?;
        if field.is_sensitive() {
            return None;
        }
        record.field_by_name(name).ok().map(Self)
    }

    pub fn get<T: Facet<'static>>(self) -> Option<&'a T> {
        self.0.get().ok()
    }

    /// Resolve nested column paths through the same sensitive-field boundary as details.
    pub fn column(self, path: &str) -> Option<Self> {
        path.split('.')
            .try_fold(self, |value, part| value.table_value()?.field(part))
    }

    pub fn table_sort_key(self) -> (Option<i128>, String) {
        let Some(value) = self.table_value() else {
            return (None, String::new());
        };
        let number = value
            .get::<usize>()
            .map(|&number| number as i128)
            .or_else(|| value.get::<u64>().map(|&number| i128::from(number)))
            .or_else(|| value.get::<u32>().map(|&number| i128::from(number)))
            .or_else(|| value.get::<i64>().map(|&number| i128::from(number)))
            .or_else(|| value.get::<i32>().map(|&number| i128::from(number)))
            .or_else(|| {
                value
                    .get::<Timestamp>()
                    .map(|timestamp| timestamp.as_nanosecond())
            })
            .or_else(|| {
                value
                    .get::<AzureDevOpsWorkItemId>()
                    .map(|id| i128::from(id.get()))
            });
        if number.is_some() {
            (number, String::new())
        } else {
            (
                None,
                value
                    .table_text()
                    .map_or_else(String::new, |text| text.to_lowercase()),
            )
        }
    }

    /// Produce a compact cell label without exposing the contents of nested records.
    pub fn table_text(self) -> Option<String> {
        let mut value = self.table_value()?;
        for _ in 0..8 {
            if let Some(level) = value.get::<AzureActivityLogEventLevel>() {
                return Some(level.as_str().to_owned());
            }
            if let Some(result) = value.get::<EntraDirectoryAuditResult>() {
                return Some(result.as_str().to_owned());
            }
            if let Some(location) = value.get::<AzureLocationName>() {
                return Some(location.to_string());
            }
            if let Some(id) = value.get::<ScopeImpl>() {
                return Some(id.expanded_form());
            }
            if let Some(id) = value.get::<AzureResourceGroupId>() {
                return Some(id.expanded_form());
            }
            if let Some(id) = value.get::<AzureStorageAccountBlobContainerId>() {
                return Some(id.to_string());
            }
            if let Some(id) = value.get::<SubscriptionId>() {
                return Some(id.expanded_form());
            }
            if let Some(id) = value.get::<AzureTenantId>() {
                return Some(id.to_string());
            }
            if let Some(id) = value.get::<CloudTerrastodonAccountId>() {
                return Some(id.to_string());
            }
            if let Some(id) = value.get::<EntraUserId>() {
                return Some(id.to_string());
            }
            if let Some(id) = value.get::<EntraGroupId>() {
                return Some(id.to_string());
            }
            if let Some(id) = value.get::<EntraApplicationObjectId>() {
                return Some(id.to_string());
            }
            if let Some(id) = value.get::<EntraApplicationClientId>() {
                return Some(id.to_string());
            }
            if let Some(link) = value.get::<ScopeRoleAssignmentPrincipalLink>() {
                return Some(link.identifier());
            }
            if let Some(kind) = value.get::<ResourceType>() {
                return Some(kind.to_string());
            }
            if let Some(kind) = value.get::<AzureDevOpsServiceEndpointKind>() {
                return Some(String::from(kind));
            }
            if let Some(status) = value.get::<AzureDevOpsBuildDefinitionQueueStatus>() {
                return Some(status.to_string());
            }
            if let Some(name) = value.get::<BmcHelixServerName>() {
                return Some(name.to_string());
            }
            if let Some(url) = value.get::<BmcHelixServerUrl>() {
                return Some(url.to_string());
            }
            if let Some(id) = value.get::<BmcHelixWorkOrderId>() {
                return Some(id.to_string());
            }
            if let Some(id) = value.get::<BmcHelixTicketRecordId>() {
                return Some(id.to_string());
            }
            if let Some(timestamp) = value.get::<Timestamp>() {
                return Some(timestamp.to_string());
            }
            if matches!(value.0.shape().def, Def::Scalar) && value.0.shape().vtable.has_display() {
                return Some(value.0.to_string());
            }
            value = if let Ok(record) = value.0.into_struct() {
                let [field] = record.ty().fields else {
                    return None;
                };
                value.field(field.name)?.table_value()?
            } else if let Ok(enumeration) = value.0.into_enum() {
                let variant = enumeration.active_variant().ok()?;
                return variant
                    .data
                    .fields
                    .is_empty()
                    .then(|| variant.name.to_owned());
            } else {
                return None;
            };
        }
        None
    }

    fn table_value(self) -> Option<Self> {
        let mut value = self;
        for _ in 0..8 {
            value = if let Ok(option) = value.0.into_option() {
                Self(option.value()?)
            } else if let Ok(pointer) = value.0.into_pointer() {
                Self(pointer.borrow_inner()?)
            } else {
                return Some(value);
            };
        }
        None
    }

    /// Resolve linkable identities from their Facet types, including in optional
    /// and shared fields. Scalar strings and display names have no route.
    pub fn navigation_target(self) -> Option<PortalTabTarget> {
        let mut value = self;
        for _ in 0..8 {
            if let Some(id) = value.get::<AzureTenantId>() {
                return Some(PortalTabTarget::AzureTenant(*id));
            }
            if let Some(target) = value.get::<PortalTabTarget>() {
                return Some(target.clone());
            }
            if let Some(organization) = value.get::<AzureDevOpsOrganizationUrl>() {
                return Some(PortalAzureDevOpsOrganizationView::Projects.target(organization));
            }
            if let Some(id) = value.get::<AzureResourceGroupId>() {
                return Some(PortalTabTarget::ResourceGroup(id.clone()));
            }
            if let Some(id) = value.get::<SubscriptionId>() {
                return Some(PortalTabTarget::Subscription(*id));
            }
            if let Some(id) = value.get::<EntraUserId>() {
                return Some(PortalTabTarget::EntraUser(*id));
            }
            if let Some(id) = value.get::<EntraGroupId>() {
                return Some(PortalTabTarget::EntraGroup(*id));
            }
            if let Some(id) = value.get::<EntraApplicationObjectId>() {
                return Some(PortalTabTarget::EntraAppRegistration(*id));
            }
            if let Some(link) = value.get::<ScopeRoleAssignmentPrincipalLink>() {
                return Some(link.navigation_target());
            }
            if let Some(id) = value.get::<ManagementGroupId>() {
                return Some(PortalTabTarget::ManagementGroup(id.clone()));
            }
            if let Some(id) = value.get::<AzureStorageAccountId>() {
                return Some(PortalTabTarget::StorageAccount(id.clone()));
            }
            if let Some(id) = value.get::<AzureStorageAccountBlobContainerId>() {
                return Some(PortalTabTarget::StorageContainer(id.clone()));
            }
            if let Some(kind) = value.get::<ResourceType>() {
                return Some(PortalTabTarget::ResourcesOfType(kind.clone()));
            }
            if let Some(scope) = value.get::<ScopeImpl>() {
                if matches!(scope, ScopeImpl::Unknown(id) if !id.starts_with('/') || id.as_str() == "/")
                {
                    return None;
                }
                return Some(PortalTabTarget::from_arm_scope(scope));
            }
            value = if let Ok(option) = value.0.into_option() {
                Self(option.value()?)
            } else if let Ok(pointer) = value.0.into_pointer() {
                Self(pointer.borrow_inner()?)
            } else {
                return None;
            };
        }
        None
    }

    /// Show the object's fields even when the object also has a short Display label.
    pub fn show(self, ui: &mut Ui) {
        self.show_value(ui, None, &mut None);
    }

    /// Present supported identities as links to their other portal views.
    pub fn show_links(self, ui: &mut Ui, current: &PortalTabTarget) -> Option<PortalTabOpen> {
        let mut navigation = None;
        self.show_value(ui, Some(current), &mut navigation);
        navigation
    }

    fn show_value(
        self,
        ui: &mut Ui,
        current: Option<&PortalTabTarget>,
        navigation: &mut Option<PortalTabOpen>,
    ) {
        if let Ok(record) = self.0.into_struct() {
            Self::show_record(ui, record, 0, current, navigation);
        } else {
            self.show_field(ui, self.0.shape().effective_name(), 0, current, navigation);
        }
    }

    fn show_record(
        ui: &mut Ui,
        record: PeekStruct<'a, 'static>,
        depth: usize,
        current: Option<&PortalTabTarget>,
        navigation: &mut Option<PortalTabOpen>,
    ) {
        for (index, field) in record.ty().fields.iter().enumerate() {
            let label = field.effective_name().replace('_', " ");
            if field.is_sensitive() {
                ui.label(format!("{label} · Redacted"));
            } else if let Ok(value) = record.field(index) {
                Self(value).show_field(ui, &label, depth, current, navigation);
            }
        }
    }

    fn show_field(
        self,
        ui: &mut Ui,
        label: &str,
        depth: usize,
        current: Option<&PortalTabTarget>,
        navigation: &mut Option<PortalTabOpen>,
    ) {
        if depth >= 8 {
            ui.label(format!("{label} · {}", self.0.shape().effective_name()));
            return;
        }
        if let Some(current) = current
            && let Some(target) = self.navigation_target()
            && target != *current
        {
            ui.horizontal_wrapped(|ui| {
                ui.strong(label);
                let response = ui.link(target.title().as_ref());
                if let Some(open) = PortalTabOpen::from_response(&response, || target) {
                    *navigation = Some(open);
                }
            });
        } else if let Ok(option) = self.0.into_option() {
            if let Some(value) = option.value() {
                Self(value).show_field(ui, label, depth + 1, current, navigation);
            } else {
                ui.label(format!("{label} · —"));
            }
        } else if let Ok(map) = self.0.into_map() {
            ui.collapsing(format!("{label} ({})", map.len()), |ui| {
                let mut entries: Vec<_> = map
                    .iter()
                    .enumerate()
                    .map(|(index, (key, value))| {
                        // A structured key can itself contain sensitive fields.
                        let label = if matches!(key.shape().def, Def::Scalar) {
                            key.to_string()
                        } else {
                            format!("Entry {}", index + 1)
                        };
                        (label, value)
                    })
                    .collect();
                entries.sort_unstable_by(|(left, _), (right, _)| left.cmp(right));
                for (index, (key, value)) in entries.into_iter().enumerate() {
                    ui.push_id(index, |ui| {
                        Self(value).show_field(ui, &key, depth + 1, current, navigation)
                    });
                }
            });
        } else if let Ok(list) = self.0.into_list_like() {
            ui.collapsing(format!("{label} ({})", list.len()), |ui| {
                for (index, value) in list.iter().enumerate() {
                    ui.push_id(index, |ui| {
                        Self(value).show_field(
                            ui,
                            &(index + 1).to_string(),
                            depth + 1,
                            current,
                            navigation,
                        );
                    });
                }
            });
        } else if let Ok(pointer) = self.0.into_pointer()
            && let Some(inner) = pointer.borrow_inner()
        {
            Self(inner).show_field(ui, label, depth + 1, current, navigation);
        } else if let Ok(record) = self.0.into_struct() {
            if let [field] = record.ty().fields
                && !field.is_sensitive()
                && let Ok(value) = record.field(0)
            {
                Self(value).show_field(ui, label, depth + 1, current, navigation);
            } else {
                ui.collapsing(label, |ui| {
                    Self::show_record(ui, record, depth + 1, current, navigation)
                });
            }
        } else if let Ok(value) = self.0.into_enum()
            && let Ok(variant) = value.active_variant()
        {
            if variant.data.fields.is_empty() {
                ui.label(format!("{label} · {}", variant.name));
            } else {
                ui.collapsing(format!("{label} · {}", variant.name), |ui| {
                    for (index, field) in variant.data.fields.iter().enumerate() {
                        let label = field.effective_name().replace('_', " ");
                        if field.is_sensitive() {
                            ui.label(format!("{label} · Redacted"));
                        } else if let Ok(Some(child)) = value.field(index) {
                            Self(child).show_field(ui, &label, depth + 1, current, navigation);
                        }
                    }
                });
            }
        } else if self.0.shape().vtable.has_display() {
            ui.horizontal_wrapped(|ui| {
                ui.strong(label);
                ui.label(self.0.to_string());
            });
        } else {
            ui.label(format!("{label} · {}", self.0.shape().effective_name()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Facet)]
    struct ProtectedRecord {
        label: String,
        #[facet(sensitive)]
        credential: String,
    }

    #[test]
    fn inventory_projection_cannot_read_a_sensitive_field() {
        let record = ProtectedRecord {
            label: "Synthetic record".to_owned(),
            credential: "synthetic-secret".to_owned(),
        };
        let value = ReflectedValue::new(&record);
        assert_eq!(
            value
                .field("label")
                .and_then(ReflectedValue::get::<String>)
                .map(String::as_str),
            Some("Synthetic record")
        );
        assert!(value.field("credential").is_none());
        let optional = Some(record);
        let value = ReflectedValue::new(&optional);
        assert!(value.column("credential").is_none());
        assert_eq!(
            value
                .column("label")
                .and_then(ReflectedValue::table_text)
                .as_deref(),
            Some("Synthetic record")
        );
    }

    #[test]
    fn reflected_inventory_counts_sort_numerically() {
        let mut counts = [10usize, 2, 100, 21, 0];
        counts.sort_by_cached_key(|value| ReflectedValue::new(value).table_sort_key());
        assert_eq!(counts, [0, 2, 10, 21, 100]);
    }

    #[test]
    fn reflected_work_item_ids_sort_numerically() -> eyre::Result<()> {
        let mut ids = [
            AzureDevOpsWorkItemId::new(42)?,
            AzureDevOpsWorkItemId::new(7)?,
        ];
        ids.sort_by_cached_key(|id| ReflectedValue::new(id).table_sort_key());
        assert_eq!(ids.map(AzureDevOpsWorkItemId::get), [7, 42]);
        Ok(())
    }

    #[test]
    fn reflected_service_connection_type_retains_its_service_value() -> eyre::Result<()> {
        let kind: AzureDevOpsServiceEndpointKind = "github".parse()?;
        assert_eq!(
            ReflectedValue::new(&kind).table_text().as_deref(),
            Some("github")
        );
        Ok(())
    }

    #[test]
    fn navigation_uses_the_reflected_identity_instead_of_display_text() -> eyre::Result<()> {
        let id = AzureResourceGroupId::try_new(
            "00000000-0000-0000-0000-000000000001",
            "synthetic-group",
        )?;
        let text = id.to_string();
        assert!(ReflectedValue::new(&text).navigation_target().is_none());
        let reflected = Some(ScopeImpl::ResourceGroup(id.clone()));
        assert_eq!(
            ReflectedValue::new(&reflected).navigation_target(),
            Some(PortalTabTarget::ResourceGroup(id))
        );
        Ok(())
    }

    #[test]
    fn application_lookup_uses_object_identity_even_when_client_id_has_the_same_text()
    -> eyre::Result<()> {
        let id = "00000000-0000-0000-0000-000000000001";
        let object: EntraApplicationObjectId = id.parse()?;
        let client: EntraApplicationClientId = id.parse()?;
        assert_eq!(
            ReflectedValue::new(&object).navigation_target(),
            Some(PortalTabTarget::EntraAppRegistration(object))
        );
        let client = ReflectedValue::new(&client);
        assert_eq!(client.table_text().as_deref(), Some(id));
        assert!(client.navigation_target().is_none());
        Ok(())
    }
}
