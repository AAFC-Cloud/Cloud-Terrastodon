use arbitrary::Arbitrary;
use cloud_terrastodon_azure_types::ArbitraryJson;
use eyre::Result;
use eyre::ensure;
use std::collections::BTreeMap;
use std::ops::Deref;
use std::ops::DerefMut;

/// Raw object representation used only while encoding or decoding one JSON
/// Patch operation. This is deliberately distinct from work item fields even
/// though both happen to use a string-keyed JSON object.
#[derive(Debug, Clone, Arbitrary, facet::Facet)]
#[facet(transparent)]
struct AzureDevOpsJsonPatchOperationObject(BTreeMap<String, ArbitraryJson>);

impl Deref for AzureDevOpsJsonPatchOperationObject {
    type Target = BTreeMap<String, ArbitraryJson>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AzureDevOpsJsonPatchOperationObject {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// One JSON Patch operation accepted by the Azure DevOps work item API.
///
/// The operation is an enum so each variant carries exactly the members that
/// apply to that JSON Patch operation. In particular, `remove` cannot carry a
/// value and `move`/`copy` cannot accidentally be sent without `from`.
#[derive(Debug, Clone, Arbitrary, facet::Facet)]
#[facet(proxy = AzureDevOpsJsonPatchOperationObject)]
#[repr(u8)]
pub enum AzureDevOpsJsonPatchOperation {
    Add { path: String, value: ArbitraryJson },
    Remove { path: String },
    Replace { path: String, value: ArbitraryJson },
    Move { from: String, path: String },
    Copy { from: String, path: String },
    Test { path: String, value: ArbitraryJson },
}

impl AzureDevOpsJsonPatchOperation {
    pub fn path(&self) -> &str {
        match self {
            Self::Add { path, .. }
            | Self::Remove { path }
            | Self::Replace { path, .. }
            | Self::Move { path, .. }
            | Self::Copy { path, .. }
            | Self::Test { path, .. } => path,
        }
    }

    pub fn from(&self) -> Option<&str> {
        match self {
            Self::Move { from, .. } | Self::Copy { from, .. } => Some(from),
            Self::Add { .. } | Self::Remove { .. } | Self::Replace { .. } | Self::Test { .. } => {
                None
            }
        }
    }

    pub fn value(&self) -> Option<&ArbitraryJson> {
        match self {
            Self::Add { value, .. } | Self::Replace { value, .. } | Self::Test { value, .. } => {
                Some(value)
            }
            Self::Remove { .. } | Self::Move { .. } | Self::Copy { .. } => None,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Add { .. } => "add",
            Self::Remove { .. } => "remove",
            Self::Replace { .. } => "replace",
            Self::Move { .. } => "move",
            Self::Copy { .. } => "copy",
            Self::Test { .. } => "test",
        }
    }
}

// A normal Option<T> decoder maps JSON null to None. Decode through a map so
// an explicit null value remains present as ArbitraryJson("null").
impl TryFrom<AzureDevOpsJsonPatchOperationObject> for AzureDevOpsJsonPatchOperation {
    type Error = eyre::Report;

    fn try_from(mut wire: AzureDevOpsJsonPatchOperationObject) -> Result<Self> {
        fn required<T: facet::Facet<'static>>(
            wire: &mut AzureDevOpsJsonPatchOperationObject,
            name: &str,
        ) -> Result<T> {
            let value = wire
                .remove(name)
                .ok_or_else(|| eyre::eyre!("Missing patch member {name}"))?;
            facet_json::from_str(value.as_ref())
                .map_err(|_| eyre::eyre!("Invalid patch member {name}"))
        }

        let name = required::<String>(&mut wire, "op")?;
        let path = required::<String>(&mut wire, "path")?;
        let operation = match name.as_str() {
            "add" => Self::Add {
                path,
                value: required(&mut wire, "value")?,
            },
            "remove" => Self::Remove { path },
            "replace" => Self::Replace {
                path,
                value: required(&mut wire, "value")?,
            },
            "move" => Self::Move {
                from: required(&mut wire, "from")?,
                path,
            },
            "copy" => Self::Copy {
                from: required(&mut wire, "from")?,
                path,
            },
            "test" => Self::Test {
                path,
                value: required(&mut wire, "value")?,
            },
            _ => return Err(eyre::eyre!("Invalid JSON Patch operation")),
        };
        ensure!(wire.is_empty(), "Unexpected JSON Patch member");
        Ok(operation)
    }
}

impl From<&AzureDevOpsJsonPatchOperation> for AzureDevOpsJsonPatchOperationObject {
    fn from(operation: &AzureDevOpsJsonPatchOperation) -> Self {
        fn json<T: facet::Facet<'static>>(value: &T) -> ArbitraryJson {
            // Serializing these fixed enum/string shapes is infallible.
            facet_json::RawJson::from_owned(
                facet_json::to_string(value).expect("serializing a patch scalar"),
            )
            .into()
        }

        let mut wire = BTreeMap::from([
            ("op".to_owned(), json(&operation.name().to_owned())),
            ("path".to_owned(), json(&operation.path().to_owned())),
        ]);
        match operation {
            AzureDevOpsJsonPatchOperation::Add { value, .. }
            | AzureDevOpsJsonPatchOperation::Replace { value, .. }
            | AzureDevOpsJsonPatchOperation::Test { value, .. } => {
                wire.insert("value".to_owned(), value.clone());
            }
            AzureDevOpsJsonPatchOperation::Move { from, .. }
            | AzureDevOpsJsonPatchOperation::Copy { from, .. } => {
                wire.insert("from".to_owned(), json(from));
            }
            AzureDevOpsJsonPatchOperation::Remove { .. } => {}
        }
        AzureDevOpsJsonPatchOperationObject(wire)
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsJsonPatchOperation);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsJsonPatchOperation);
