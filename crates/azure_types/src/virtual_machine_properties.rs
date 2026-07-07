use crate::{ArbitraryJson, iso8601_duration::IsoDuration};
use arbitrary::Arbitrary;
use uuid::Uuid;

#[derive(Debug, PartialEq, Clone, facet::Facet, Arbitrary)]
#[facet(rename_all = "camelCase")]
pub struct VirtualMachineProperties {
    pub availability_set: Option<VirtualMachinePropertiesAvailabilitySet>,
    pub priority: Option<String>,
    pub provisioning_state: String,
    pub time_created: String,
    pub network_profile: NetworkProfile,
    pub storage_profile: StorageProfile,
    pub extensions_time_budget: Option<IsoDuration>,
    pub hardware_profile: HardwareProfile,
    pub license_type: Option<String>,
    pub security_profile: Option<SecurityProfile>,
    pub os_profile: Option<OsProfile>,
    pub diagnostics_profile: Option<DiagnosticsProfile>,
    pub extended: ExtendedProperties,
    pub vm_id: Uuid,
    pub additional_capabilities: Option<AdditionalCapabilities>,
}
#[derive(Debug, PartialEq, Clone, facet::Facet, Arbitrary)]
#[facet(rename_all = "camelCase")]
pub struct VirtualMachinePropertiesAvailabilitySet {
    pub id: String,
}

#[derive(Debug, PartialEq, Clone, facet::Facet, Arbitrary)]
#[facet(rename_all = "camelCase")]
pub struct NetworkProfile {
    pub network_interfaces: Vec<NetworkInterfaceReference>,
}

#[derive(Debug, PartialEq, Clone, facet::Facet, Arbitrary)]
#[facet(rename_all = "camelCase")]
pub struct NetworkInterfaceReference {
    pub properties: Option<NetworkInterfacePropertiesReference>,
    pub id: String,
}

#[derive(Debug, PartialEq, Clone, facet::Facet, Arbitrary)]
#[facet(rename_all = "camelCase")]
pub struct NetworkInterfacePropertiesReference {
    pub delete_option: Option<NetworkInterfacePropertiesDeleteOption>,
    pub primary: Option<bool>,
}

#[derive(Debug, PartialEq, Clone, facet::Facet, Arbitrary)]
#[repr(C)]
pub enum NetworkInterfacePropertiesDeleteOption {
    Delete,
    Detach,
}

#[derive(Debug, PartialEq, Clone, facet::Facet, Arbitrary)]
#[facet(rename_all = "camelCase")]
pub struct StorageProfile {
    pub image_reference: Option<StorageProfileImageReference>,
    pub disk_controller_type: Option<StorageProfileDiskControllerType>,
    #[facet(default)]
    pub data_disks: Vec<DataDisk>,
    pub os_disk: OsDisk,
}

#[derive(Debug, PartialEq, Clone, facet::Facet, Arbitrary)]
#[facet(opaque, proxy = String)]
#[repr(C)]
pub enum StorageProfileDiskControllerType {
    SCSI,
    Other(String),
}

impl From<String> for StorageProfileDiskControllerType {
    fn from(value: String) -> Self {
        if value == "SCSI" {
            Self::SCSI
        } else {
            Self::Other(value)
        }
    }
}

impl From<StorageProfileDiskControllerType> for String {
    fn from(value: StorageProfileDiskControllerType) -> Self {
        match value {
            StorageProfileDiskControllerType::SCSI => "SCSI".to_string(),
            StorageProfileDiskControllerType::Other(value) => value,
        }
    }
}

impl From<&StorageProfileDiskControllerType> for String {
    fn from(value: &StorageProfileDiskControllerType) -> Self {
        String::from(value.clone())
    }
}

#[derive(Debug, PartialEq, Clone, facet::Facet, Arbitrary)]
#[facet(untagged)]
#[repr(C)]
pub enum StorageProfileImageReference {
    ByPublisher(StorageProfileImageReferenceByPublisher),
    ById(StorageProfileImageReferenceById),
}

#[derive(Debug, PartialEq, Clone, facet::Facet, Arbitrary)]
#[facet(rename_all = "camelCase")]
pub struct StorageProfileImageReferenceByPublisher {
    exact_version: String,
    offer: String,
    publisher: String,
    sku: String,
    version: String,
}

#[derive(Debug, PartialEq, Clone, facet::Facet, Arbitrary)]
#[facet(rename_all = "camelCase")]
pub struct StorageProfileImageReferenceById {
    exact_version: Option<String>,
    id: String,
}

#[derive(Debug, PartialEq, Clone, facet::Facet, Arbitrary)]
#[facet(rename_all = "camelCase")]
pub struct OsDisk {
    pub name: String,
    pub create_option: String,
    pub delete_option: Option<OsDiskDeleteOption>,
    pub os_type: String,
    #[facet(rename = "diskSizeGB")]
    pub disk_size_gb: Option<usize>,
    pub managed_disk: ManagedDiskReference,
    pub caching: String,
    pub write_accelerator_enabled: Option<bool>,
}

#[derive(Debug, PartialEq, Clone, facet::Facet, Arbitrary)]
#[facet(rename_all = "camelCase")]
pub struct DataDisk {
    pub caching: String,
    pub create_option: String,
    pub delete_option: Option<OsDiskDeleteOption>,
    #[facet(rename = "diskSizeGB")]
    pub disk_size_gb: Option<usize>,
    #[facet(rename = "lun")]
    pub logical_unit_number: usize,
    pub managed_disk: ManagedDiskReference,
    pub name: String,
    pub to_be_detached: bool,
    pub write_accelerator_enabled: Option<bool>,
}

#[derive(Debug, PartialEq, Clone, facet::Facet, Arbitrary)]
#[repr(C)]
pub enum OsDiskDeleteOption {
    Delete,
    Detach,
}

#[derive(Debug, PartialEq, Clone, facet::Facet, Arbitrary)]
#[facet(rename_all = "camelCase")]
pub struct ManagedDiskReference {
    pub id: String,
    pub storage_account_type: Option<String>,
}

#[derive(Debug, PartialEq, Clone, facet::Facet, Arbitrary)]
#[facet(rename_all = "camelCase")]
pub struct HardwareProfile {
    pub vm_size: String,
    pub vm_size_properties: Option<HardwareProfileVmSizeProperties>,
}

#[derive(Debug, PartialEq, Clone, facet::Facet, Arbitrary)]
pub struct HardwareProfileVmSizeProperties {
    #[facet(rename = "vCPUsAvailable")]
    pub v_cpus_available: usize,
    #[facet(rename = "vCPUsPerCore")]
    pub v_cpus_per_core: usize,
}

#[derive(Debug, PartialEq, Clone, facet::Facet, Arbitrary)]
#[facet(rename_all = "camelCase")]
pub struct SecurityProfile {
    pub security_type: String,
    pub uefi_settings: UefiSettings,
}

#[derive(Debug, PartialEq, Clone, facet::Facet, Arbitrary)]
#[facet(rename_all = "camelCase")]
pub struct UefiSettings {
    pub secure_boot_enabled: bool,
    pub v_tpm_enabled: bool,
}

#[derive(Debug, PartialEq, Clone, facet::Facet, Arbitrary)]
#[facet(rename_all = "camelCase")]
pub struct OsProfile {
    pub computer_name: Option<String>,
    pub require_guest_provision_signal: Option<bool>,
    pub allow_extension_operations: Option<bool>,
    pub admin_username: Option<String>,
    pub secrets: Option<ArbitraryJson>,
    pub linux_configuration: Option<ArbitraryJson>,
    pub windows_configuration: Option<ArbitraryJson>,
}

#[derive(Debug, PartialEq, Clone, facet::Facet, Arbitrary)]
#[facet(rename_all = "camelCase")]
pub struct DiagnosticsProfile {
    pub boot_diagnostics: BootDiagnostics,
    pub storage_uri: Option<String>,
}

#[derive(Debug, PartialEq, Clone, facet::Facet, Arbitrary)]
#[facet(rename_all = "camelCase")]
pub struct BootDiagnostics {
    pub enabled: bool,
    pub storage_uri: Option<String>,
}

#[derive(Debug, PartialEq, Clone, facet::Facet, Arbitrary)]
#[facet(rename_all = "camelCase")]
pub struct ExtendedProperties {
    pub instance_view: InstanceView,
}

#[derive(Debug, PartialEq, Clone, facet::Facet, Arbitrary)]
#[facet(rename_all = "camelCase")]
pub struct InstanceView {
    pub computer_name: Option<String>,
    pub hyper_v_generation: String,
    pub os_name: Option<String>,
    pub os_version: Option<String>,
    pub power_state: PowerState,
}

#[derive(Debug, PartialEq, Clone, facet::Facet, Arbitrary)]
#[facet(rename_all = "camelCase")]
pub struct PowerState {
    pub display_status: String,
    pub level: String,
    pub code: String,
}

#[derive(Debug, PartialEq, Clone, facet::Facet, Arbitrary)]
#[facet(rename_all = "camelCase")]
pub struct AdditionalCapabilities {
    pub hibernation_enabled: bool,
}
