use arbitrary::Arbitrary;

/// The numeric discriminator of a build definition's process.
///
/// Microsoft documentation: [BuildProcess.type](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/get?view=azure-devops-rest-7.1#buildprocess).
/// Microsoft's [pipeline creation implementation](https://github.com/Azure/azure-devops-cli-extension/blob/master/azure-devops/azext_devops/dev/pipelines/pipeline_create.py)
/// uses 2 for YAML. Other integers panic in debug builds so additional process
/// kinds can be modeled; release builds preserve their numeric discriminators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Arbitrary, facet::Facet)]
#[facet(proxy = i32)]
#[repr(C)]
pub enum AzureDevOpsBuildProcessType {
    Yaml,
    #[arbitrary(skip)]
    Unknown(i32),
}

impl AzureDevOpsBuildProcessType {
    pub const fn get(self) -> i32 {
        match self {
            Self::Yaml => 2,
            Self::Unknown(value) => value,
        }
    }
}

impl From<i32> for AzureDevOpsBuildProcessType {
    fn from(value: i32) -> Self {
        match value {
            2 => Self::Yaml,
            value => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&AzureDevOpsBuildProcessType> for i32 {
    fn from(value: &AzureDevOpsBuildProcessType) -> Self {
        value.get()
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildProcessType);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildProcessType);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yaml_discriminator_is_recognized() {
        assert_eq!(
            AzureDevOpsBuildProcessType::from(2),
            AzureDevOpsBuildProcessType::Yaml
        );
        assert_eq!(AzureDevOpsBuildProcessType::Yaml.get(), 2);
    }
}
