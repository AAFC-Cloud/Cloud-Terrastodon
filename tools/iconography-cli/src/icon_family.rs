use facet::Facet;

/// Asset families maintained by the standalone iconography tool.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Facet)]
#[repr(u8)]
pub enum IconFamily {
    #[facet(rename = "azure")]
    Azure,
    #[facet(rename = "azure-devops")]
    AzureDevOps,
}

impl IconFamily {
    pub const ALL: [Self; 2] = [Self::Azure, Self::AzureDevOps];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Azure => "azure",
            Self::AzureDevOps => "azure-devops",
        }
    }

    pub const fn crate_directory(self) -> &'static str {
        match self {
            Self::Azure => "iconography_azure",
            Self::AzureDevOps => "iconography_azure_devops",
        }
    }

    pub const fn type_prefix(self) -> &'static str {
        match self {
            Self::Azure => "Azure",
            Self::AzureDevOps => "AzureDevOps",
        }
    }

    pub const fn module_prefix(self) -> &'static str {
        match self {
            Self::Azure => "azure",
            Self::AzureDevOps => "azure_devops",
        }
    }

    pub fn icon_type_name(self, identity: &str) -> String {
        format!("{}{identity}Icon", self.type_prefix())
    }

    pub fn icon_module_name(self, identity: &str) -> String {
        let characters = identity.chars().collect::<Vec<_>>();
        let mut name = String::new();
        for (index, character) in characters.iter().copied().enumerate() {
            if index > 0
                && character.is_uppercase()
                && (characters[index - 1].is_lowercase()
                    || characters[index - 1].is_numeric()
                    || (characters[index - 1].is_uppercase()
                        && characters
                            .get(index + 1)
                            .is_some_and(|next| next.is_lowercase())))
            {
                name.push('_');
            }
            name.extend(character.to_lowercase());
        }
        format!("{}_{name}_icon", self.module_prefix())
    }
}
