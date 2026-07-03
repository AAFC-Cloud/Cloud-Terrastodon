use arbitrary::Arbitrary;
use crate::ComputeSkuName;

#[derive(Debug, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct VirtualMachineSize {
    pub max_data_disk_count: usize,
    #[facet(rename = "memoryInMB")]
    pub memory_in_mb: usize,
    pub name: ComputeSkuName,
    pub number_of_cores: usize,
    #[facet(rename = "osDiskSizeInMB")]
    pub os_disk_size_in_mb: usize,
    #[facet(rename = "resourceDiskSizeInMB")]
    pub resource_disk_size_in_mb: usize,
}

cloud_terrastodon_registry::register_thing!(VirtualMachineSize);
cloud_terrastodon_registry::register_arbitrary!(VirtualMachineSize);

