use cloud_terrastodon_azure::AzureStorageAccountBlobContainerId;
use cloud_terrastodon_azure_cli::AzureStorageAccountAuditFinding;
use std::borrow::Cow;

/// A finding or an evidence line shown directly below its expanded finding.
pub(crate) enum PortalStorageAuditRow<'a> {
    Finding {
        finding: &'a AzureStorageAccountAuditFinding,
        subscription_name: Cow<'a, str>,
        expanded: bool,
    },
    Evidence {
        label: Cow<'a, str>,
        value: Cow<'a, str>,
        container_id: Option<&'a AzureStorageAccountBlobContainerId>,
    },
}

impl<'a> PortalStorageAuditRow<'a> {
    pub fn append_evidence(rows: &mut Vec<Self>, finding: &'a AzureStorageAccountAuditFinding) {
        let account = finding.account();
        let properties = &account.properties;
        let anonymous_access = match properties.allow_blob_public_access {
            Some(true) => "Allowed",
            Some(false) => "Disabled",
            None => "Not reported",
        };
        let public_network_access = properties
            .public_network_access
            .as_ref()
            .map_or(Cow::Borrowed("Not reported"), |access| {
                Cow::Owned(String::from(access))
            });
        let firewall_default = properties
            .network_acls
            .as_ref()
            .map_or(Cow::Borrowed("Not reported"), |rules| {
                Cow::Owned(String::from(&rules.default_action))
            });
        rows.extend(
            [
                ("Anonymous blob access", Cow::Borrowed(anonymous_access)),
                ("Public network access", public_network_access),
                ("Firewall default action", firewall_default),
            ]
            .map(|(label, value)| Self::Evidence {
                label: Cow::Borrowed(label),
                value,
                container_id: None,
            }),
        );

        let mut tags: Vec<_> = account.tags.iter().collect();
        tags.sort_unstable_by_key(|(key, _)| *key);
        rows.extend(tags.into_iter().map(|(key, value)| Self::Evidence {
            label: Cow::Owned(format!("Tag · {key}")),
            value: Cow::Borrowed(value),
            container_id: None,
        }));

        match finding {
            AzureStorageAccountAuditFinding::UnusedAnonymousAccess { containers, .. } => {
                rows.extend(containers.iter().map(|container| Self::Evidence {
                    label: Cow::Borrowed("Private container"),
                    value: Cow::Borrowed(container.name.as_str()),
                    container_id: Some(&container.id),
                }));
            }
            AzureStorageAccountAuditFinding::Inconclusive { reason, .. } => {
                rows.push(Self::Evidence {
                    label: Cow::Borrowed("Inconclusive reason"),
                    value: Cow::Owned(reason.to_string()),
                    container_id: None,
                });
            }
        }
    }
}
