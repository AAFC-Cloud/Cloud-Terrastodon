use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub struct VirtualMachineProperties {
    pub availability_set: Option<VirtualMachinePropertiesAvailabilitySet>,
    pub priority: Option<String>,
    pub provisioning_state: String,
    pub time_created: String,
    pub network_profile: NetworkProfile,
    pub storage_profile: StorageProfile,
    pub extensions_time_budget: Option<iso8601_duration::Duration>,
    pub hardware_profile: HardwareProfile,
    pub license_type: Option<String>,
    pub security_profile: Option<SecurityProfile>,
    pub os_profile: Option<OsProfile>,
    pub diagnostics_profile: Option<DiagnosticsProfile>,
    pub extended: ExtendedProperties,
    pub vm_id: Uuid,
    pub additional_capabilities: Option<AdditionalCapabilities>,
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub struct VirtualMachinePropertiesAvailabilitySet {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub struct NetworkProfile {
    pub network_interfaces: Vec<NetworkInterfaceReference>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub struct NetworkInterfaceReference {
    pub properties: Option<NetworkInterfacePropertiesReference>,
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub struct NetworkInterfacePropertiesReference {
    pub delete_option: Option<NetworkInterfacePropertiesDeleteOption>,
    pub primary: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum NetworkInterfacePropertiesDeleteOption {
    Delete,
    Detach,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub struct StorageProfile {
    pub image_reference: Option<StorageProfileImageReference>,
    pub disk_controller_type: Option<StorageProfileDiskControllerType>,
    #[serde(default)]
    pub data_disks: Vec<DataDisk>,
    pub os_disk: OsDisk,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(untagged)]
pub enum StorageProfileDiskControllerType {
    SCSI,
    Other(String),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(untagged)]
pub enum StorageProfileImageReference {
    ByPublisher(StorageProfileImageReferenceByPublisher),
    ById(StorageProfileImageReferenceById),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub struct StorageProfileImageReferenceByPublisher {
    exact_version: String,
    offer: String,
    publisher: String,
    sku: String,
    version: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub struct StorageProfileImageReferenceById {
    exact_version: Option<String>,
    id: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub struct OsDisk {
    pub name: String,
    pub create_option: String,
    pub delete_option: Option<OsDiskDeleteOption>,
    pub os_type: String,
    #[serde(rename = "diskSizeGB")]
    pub disk_size_gb: Option<usize>,
    pub managed_disk: ManagedDiskReference,
    pub caching: String,
    pub write_accelerator_enabled: Option<bool>,
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub struct DataDisk {
    pub caching: String,
    pub create_option: String,
    pub delete_option: Option<OsDiskDeleteOption>,
    #[serde(rename = "diskSizeGB")]
    pub disk_size_gb: Option<usize>,
    #[serde(rename = "lun")]
    pub logical_unit_number: usize,
    pub managed_disk: ManagedDiskReference,
    pub name: String,
    pub to_be_detached: bool,
    pub write_accelerator_enabled: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum OsDiskDeleteOption {
    Delete,
    Detach,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub struct ManagedDiskReference {
    pub id: String,
    pub storage_account_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub struct HardwareProfile {
    pub vm_size: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub struct SecurityProfile {
    pub security_type: String,
    pub uefi_settings: UefiSettings,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub struct UefiSettings {
    pub secure_boot_enabled: bool,
    pub v_tpm_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub struct OsProfile {
    pub computer_name: Option<String>,
    pub require_guest_provision_signal: Option<bool>,
    pub allow_extension_operations: Option<bool>,
    pub admin_username: Option<String>,
    pub secrets: Option<serde_json::Value>,
    pub linux_configuration: Option<serde_json::Value>,
    pub windows_configuration: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub struct DiagnosticsProfile {
    pub boot_diagnostics: BootDiagnostics,
    pub storage_uri: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub struct BootDiagnostics {
    pub enabled: bool,
    pub storage_uri: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub struct ExtendedProperties {
    pub instance_view: InstanceView,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub struct InstanceView {
    pub computer_name: Option<String>,
    pub hyper_v_generation: String,
    pub os_name: Option<String>,
    pub os_version: Option<String>,
    pub power_state: PowerState,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub struct PowerState {
    pub display_status: String,
    pub level: String,
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub struct AdditionalCapabilities {
    pub hibernation_enabled: bool,
}
