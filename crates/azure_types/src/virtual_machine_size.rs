use crate::prelude::ComputeSkuName;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VirtualMachineSize {
    pub max_data_disk_count: usize,
    #[serde(rename = "memoryInMB")]
    pub memory_in_mb: usize,
    pub name: ComputeSkuName,
    pub number_of_cores: usize,
    #[serde(rename = "osDiskSizeInMB")]
    pub os_disk_size_in_mb: usize,
    #[serde(rename = "resourceDiskSizeInMB")]
    pub resource_disk_size_in_mb: usize,
}
