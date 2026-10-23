use crate::AzureDevOpsProject;
use crate::AzureDevOpsProjectId;
use crate::AzureDevOpsProjectName;
use arbitrary::Arbitrary;
use eyre::bail;
use std::borrow::Cow;
use std::str::FromStr;

/// An explicit project ID or project name.
///
/// Microsoft documentation: [project parameter](https://learn.microsoft.com/en-us/rest/api/azure/devops/core/projects/get?view=azure-devops-rest-7.1).
#[derive(Debug, Clone, PartialEq, Eq, Hash, facet::Facet)]
#[facet(proxy = String)]
#[facet(traits(Clone))]
#[repr(C)]
pub enum AzureDevOpsProjectArgument<'a> {
    Id(Cow<'a, AzureDevOpsProjectId>),
    Name(Cow<'a, AzureDevOpsProjectName>),
}
impl std::fmt::Display for AzureDevOpsProjectArgument<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AzureDevOpsProjectArgument::Id(id) => id.fmt(f),
            AzureDevOpsProjectArgument::Name(name) => name.fmt(f),
        }
    }
}
impl From<AzureDevOpsProjectId> for AzureDevOpsProjectArgument<'_> {
    fn from(value: AzureDevOpsProjectId) -> Self {
        AzureDevOpsProjectArgument::Id(Cow::Owned(value))
    }
}
impl<'a> From<&'a AzureDevOpsProjectId> for AzureDevOpsProjectArgument<'a> {
    fn from(value: &'a AzureDevOpsProjectId) -> Self {
        AzureDevOpsProjectArgument::Id(Cow::Borrowed(value))
    }
}
impl From<AzureDevOpsProject> for AzureDevOpsProjectArgument<'_> {
    fn from(value: AzureDevOpsProject) -> Self {
        AzureDevOpsProjectArgument::Id(Cow::Owned(value.id))
    }
}
impl<'a> From<&'a AzureDevOpsProject> for AzureDevOpsProjectArgument<'a> {
    fn from(value: &'a AzureDevOpsProject) -> Self {
        AzureDevOpsProjectArgument::Id(Cow::Borrowed(&value.id))
    }
}
impl From<AzureDevOpsProjectName> for AzureDevOpsProjectArgument<'_> {
    fn from(value: AzureDevOpsProjectName) -> Self {
        AzureDevOpsProjectArgument::Name(Cow::Owned(value))
    }
}
impl<'a> From<&'a AzureDevOpsProjectName> for AzureDevOpsProjectArgument<'a> {
    fn from(value: &'a AzureDevOpsProjectName) -> Self {
        AzureDevOpsProjectArgument::Name(Cow::Borrowed(value))
    }
}

impl AzureDevOpsProjectArgument<'_> {
    pub fn into_owned(self) -> AzureDevOpsProjectArgument<'static> {
        match self {
            AzureDevOpsProjectArgument::Id(id) => {
                AzureDevOpsProjectArgument::Id(Cow::Owned(id.into_owned()))
            }
            AzureDevOpsProjectArgument::Name(name) => {
                AzureDevOpsProjectArgument::Name(Cow::Owned(name.into_owned()))
            }
        }
    }

    /// Returns true if this argument matches the supplied project.
    pub fn matches(&self, project: &AzureDevOpsProject) -> bool {
        match self {
            AzureDevOpsProjectArgument::Id(id) => &project.id == id.as_ref(),
            AzureDevOpsProjectArgument::Name(name) => project
                .name
                .as_ref()
                .eq_ignore_ascii_case(name.as_ref().as_ref()),
        }
    }
}
impl<'a> FromStr for AzureDevOpsProjectArgument<'a> {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(id) = s.parse::<AzureDevOpsProjectId>() {
            Ok(Self::Id(Cow::Owned(id)))
        } else if let Ok(name) = AzureDevOpsProjectName::try_new(s) {
            Ok(Self::Name(Cow::Owned(name)))
        } else {
            bail!("'{s}' is not a valid Azure DevOps project id or name")
        }
    }
}

impl<'a> TryFrom<String> for AzureDevOpsProjectArgument<'a> {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl From<&AzureDevOpsProjectArgument<'_>> for String {
    fn from(value: &AzureDevOpsProjectArgument<'_>) -> Self {
        value.to_string()
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsProjectArgument<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        // Synthetic requests and URLs must not depend on local configuration.
        Ok(match u.int_in_range(0..=1)? {
            0 => Self::Id(Cow::Owned(AzureDevOpsProjectId::arbitrary(u)?)),
            _ => Self::Name(Cow::Owned(AzureDevOpsProjectName::arbitrary(u)?)),
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsProjectArgument<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsProjectArgument<'static>);
cloud_terrastodon_registry::register_from!(AzureDevOpsProjectId => AzureDevOpsProjectArgument<'static>);
cloud_terrastodon_registry::register_from!(AzureDevOpsProjectName => AzureDevOpsProjectArgument<'static>);

#[cfg(test)]
mod tests {
    use super::AzureDevOpsProjectArgument;
    use crate::AzureDevOpsProject;
    use crate::AzureDevOpsProjectId;
    use crate::AzureDevOpsProjectName;
    use crate::AzureDevOpsProjectState;
    use crate::AzureDevOpsProjectVisibility;
    use chrono::DateTime;
    use cloud_terrastodon_command::CancellationToken;
    use cloud_terrastodon_registry::FunctionInvocation;
    use cloud_terrastodon_registry::RuntimeValue;
    use cloud_terrastodon_registry::functions_from_to;
    use facet::Facet;
    use std::any::Any;

    fn registered_project_argument<T>(input: T) -> AzureDevOpsProjectArgument<'static>
    where
        T: Facet<'static> + Any + Send,
    {
        let functions = functions_from_to(T::SHAPE, AzureDevOpsProjectArgument::SHAPE);
        assert_eq!(
            functions.len(),
            1,
            "expected one project selector conversion"
        );
        CancellationToken::new()
            .with_scope(|cancellation| {
                let invocation = functions[0]
                    .invoke_value_boxed(Box::new(input), &cancellation)
                    .unwrap();
                let FunctionInvocation::Ready(value) = invocation else {
                    panic!("project selector conversions must complete synchronously");
                };
                Ok::<_, eyre::Report>(*value.downcast::<AzureDevOpsProjectArgument>().unwrap())
            })
            .unwrap()
    }

    #[test]
    fn registered_project_conversion_preserves_the_project_id() {
        let id: AzureDevOpsProjectId = "11111111-1111-1111-1111-111111111111".parse().unwrap();
        let project = AzureDevOpsProject {
            abbreviation: None,
            default_team_image_url: None,
            description: None,
            id: id.clone(),
            last_update_time: DateTime::UNIX_EPOCH,
            name: AzureDevOpsProjectName::try_new("Synthetic project").unwrap(),
            revision: 1,
            state: AzureDevOpsProjectState::WellFormed,
            url: "https://dev.azure.com/synthetic/_apis/projects/11111111-1111-1111-1111-111111111111".to_owned(),
            visibility: AzureDevOpsProjectVisibility::Private,
        };

        assert_eq!(
            registered_project_argument(project),
            AzureDevOpsProjectArgument::from(id)
        );
    }

    #[test]
    fn registered_exact_selectors_preserve_id_versus_name_identity() {
        let text = "11111111-1111-1111-1111-111111111111";
        let id: AzureDevOpsProjectId = text.parse().unwrap();
        let name = AzureDevOpsProjectName::try_new(text).unwrap();

        assert!(matches!(
            registered_project_argument(id.clone()),
            AzureDevOpsProjectArgument::Id(actual) if actual.as_ref() == &id
        ));
        assert!(matches!(
            registered_project_argument(name.clone()),
            AzureDevOpsProjectArgument::Name(actual) if actual.as_ref() == &name
        ));
    }

    #[test]
    fn exposes_a_reflected_clone_operation() {
        let project_name = AzureDevOpsProjectName::try_new("test-project").unwrap();
        let value =
            RuntimeValue::from_box(Box::new(AzureDevOpsProjectArgument::from(project_name)))
                .unwrap();

        assert!(value.try_clone().is_ok());
    }
}
