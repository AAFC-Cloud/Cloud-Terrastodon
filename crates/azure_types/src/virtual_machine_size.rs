use crate::prelude::ComputeSkuName;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VirtualMachineSize {
    max_data_disk_count: usize,
    #[serde(rename = "memoryInMB")]
    memory_in_mb: usize,
    name: ComputeSkuName,
    number_of_cores: usize,
    #[serde(rename = "osDiskSizeInMB")]
    os_disk_size_in_mb: usize,
    #[serde(rename = "resourceDiskSizeInMB")]
    resource_disk_size_in_mb: usize,
}
