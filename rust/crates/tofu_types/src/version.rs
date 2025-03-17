use crate::prelude::TofuProviderKind;
use eyre::OptionExt;
use eyre::bail;
use hcl::edit::Decorated;
use hcl::edit::expr::Expression;
use hcl::edit::expr::Object;
use hcl::edit::expr::ObjectKey;
use hcl::edit::expr::ObjectValue;
use hcl::edit::structure::Attribute;
use hcl::edit::structure::Block;
use hcl_primitives::Ident;
use itertools::Itertools;
use std::collections::HashMap;
use std::collections::HashSet;
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TFProviderHostname(pub String);
impl Default for TFProviderHostname {
    fn default() -> Self {
        Self("registry.terraform.io".to_string())
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TFProviderNamespace(pub String);
impl Default for TFProviderNamespace {
    fn default() -> Self {
        Self("hashicorp".to_string())
    }
}

/// https://developer.hashicorp.com/terraform/language/providers/requirements#source-addresses
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TFProviderSource {
    pub hostname: TFProviderHostname,
    pub namespace: TFProviderNamespace,
    pub kind: TofuProviderKind,
}
impl FromStr for TFProviderSource {
    type Err = eyre::ErrReport;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.split("/").collect_vec().as_slice() {
            [a] => Self {
                hostname: TFProviderHostname::default(),
                namespace: TFProviderNamespace::default(),
                kind: TofuProviderKind::from_str(a)?,
            },
            [a, b] => Self {
                hostname: TFProviderHostname::default(),
                namespace: TFProviderNamespace(a.to_string()),
                kind: TofuProviderKind::from_str(b)?,
            },
            [a, b, c] => Self {
                hostname: TFProviderHostname(a.to_string()),
                namespace: TFProviderNamespace(b.to_string()),
                kind: TofuProviderKind::from_str(c)?,
            },
            x => {
                bail!("Invalid TF provider source format: {x:?}, expected 1-3 slashes.")
            }
        })
    }
}
impl std::fmt::Display for TFProviderSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.hostname != TFProviderHostname::default() {
            f.write_str(&self.hostname.0)?;
            f.write_str("/")?;
        }
        f.write_str(&self.namespace.0)?;
        f.write_str("/")?;
        f.write_str(self.kind.provider_prefix())?;
        Ok(())
    }
}
impl From<TFProviderSource> for Expression {
    fn from(value: TFProviderSource) -> Self {
        Expression::String(Decorated::new(value.to_string()))
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TFProviderVersionConstraint {
    pub clauses: Vec<TFProviderVersionConstraintClause>,
}
impl FromStr for TFProviderVersionConstraint {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split(",");
        let mut clauses = Vec::new();
        for part in parts {
            clauses.push(part.parse()?)
        }
        eyre::Ok(Self { clauses })
    }
}
impl std::fmt::Display for TFProviderVersionConstraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.clauses.iter().map(|x| x.to_string()).join(", "))
    }
}
impl From<TFProviderVersionConstraint> for Expression {
    fn from(value: TFProviderVersionConstraint) -> Self {
        Expression::String(Decorated::new(value.to_string()))
    }
}
impl TFProviderVersionConstraint {
    pub fn is_satisfied_by(&self, other: &SemVer) -> bool {
        self.clauses
            .iter()
            .all(|clause| clause.is_satisfied_by(other))
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SemVer {
    pub major: u64,
    pub minor: Option<u64>,
    pub patch: Option<u64>,
    pub pre_release: Option<String>,
}
impl PartialOrd for SemVer {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for SemVer {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.major.cmp(&other.major) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        match self.minor.cmp(&other.minor) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        match self.patch.cmp(&other.patch) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.pre_release.cmp(&other.pre_release)
    }
}
impl std::fmt::Display for SemVer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.major))?;
        if let Some(minor) = self.minor {
            f.write_fmt(format_args!(".{minor}"))?;
        }
        if let Some(patch) = self.patch {
            f.write_fmt(format_args!(".{patch}"))?;
        }
        if let Some(prerelease) = &self.pre_release {
            f.write_str(prerelease)?;
        }
        Ok(())
    }
}
impl FromStr for SemVer {
    type Err = eyre::ErrReport;

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        let pre_release = match s.split_once('-') {
            Some((left, right)) => {
                s = left;
                Some(right.to_string())
            }
            None => None,
        };
        let (major, minor, patch) = match s.split(".").collect_vec().as_slice() {
            [major, minor, patch] => (major.parse()?, Some(minor.parse()?), Some(patch.parse()?)),
            [major, minor] => (major.parse()?, Some(minor.parse()?), None),
            [major] => (major.parse()?, None, None),
            x => {
                bail!("Expected major.minor.patch, got {x:?}")
            }
        };
        Ok(Self {
            major,
            minor,
            patch,
            pre_release,
        })
    }
}

/// https://developer.hashicorp.com/terraform/language/expressions/version-constraints#operators
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TFProviderVersionConstraintClause {
    Equals(SemVer),
    NotEquals(SemVer),
    Greater(SemVer),
    GreaterOrEqual(SemVer),
    Lesser(SemVer),
    LesserOrEqual(SemVer),
    PatchIncrement(SemVer),
}
impl TFProviderVersionConstraintClause {
    pub fn prefix(&self) -> &'static str {
        match self {
            TFProviderVersionConstraintClause::Equals(..) => "=",
            TFProviderVersionConstraintClause::NotEquals(..) => "!=",
            TFProviderVersionConstraintClause::Greater(..) => ">",
            TFProviderVersionConstraintClause::GreaterOrEqual(..) => ">=",
            TFProviderVersionConstraintClause::Lesser(..) => "<",
            TFProviderVersionConstraintClause::LesserOrEqual(..) => "<=",
            TFProviderVersionConstraintClause::PatchIncrement(..) => "~>",
        }
    }
    pub fn sem_ver(&self) -> &SemVer {
        match self {
            TFProviderVersionConstraintClause::Equals(sem_ver) => sem_ver,
            TFProviderVersionConstraintClause::NotEquals(sem_ver) => sem_ver,
            TFProviderVersionConstraintClause::Greater(sem_ver) => sem_ver,
            TFProviderVersionConstraintClause::GreaterOrEqual(sem_ver) => sem_ver,
            TFProviderVersionConstraintClause::Lesser(sem_ver) => sem_ver,
            TFProviderVersionConstraintClause::LesserOrEqual(sem_ver) => sem_ver,
            TFProviderVersionConstraintClause::PatchIncrement(sem_ver) => sem_ver,
        }
    }
}
impl std::fmt::Display for TFProviderVersionConstraintClause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}{}", self.prefix(), self.sem_ver()))
    }
}
impl FromStr for TFProviderVersionConstraintClause {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let prefix: String = s.chars().take_while(|x| !x.is_numeric()).collect();
        let remaining = &s[prefix.len()..];
        let sem_ver: SemVer = remaining.parse()?;
        Ok(match prefix.as_str() {
            "=" => TFProviderVersionConstraintClause::Equals(sem_ver),
            "!=" => TFProviderVersionConstraintClause::NotEquals(sem_ver),
            ">" => TFProviderVersionConstraintClause::Greater(sem_ver),
            ">=" => TFProviderVersionConstraintClause::GreaterOrEqual(sem_ver),
            "<" => TFProviderVersionConstraintClause::Lesser(sem_ver),
            "<=" => TFProviderVersionConstraintClause::LesserOrEqual(sem_ver),
            "~>" => TFProviderVersionConstraintClause::PatchIncrement(sem_ver),
            x => {
                bail!("Unable to interpret {x:?} as TFProviderVersionConstraintClause")
            }
        })
    }
}
impl TFProviderVersionConstraintClause {
    pub fn is_satisfied_by(&self, other: &SemVer) -> bool {
        match self {
            TFProviderVersionConstraintClause::Equals(sem_ver) => sem_ver == other,
            TFProviderVersionConstraintClause::NotEquals(sem_ver) => sem_ver != other,
            TFProviderVersionConstraintClause::Greater(sem_ver) => sem_ver < other,
            TFProviderVersionConstraintClause::GreaterOrEqual(sem_ver) => sem_ver <= other,
            TFProviderVersionConstraintClause::Lesser(sem_ver) => sem_ver > other,
            TFProviderVersionConstraintClause::LesserOrEqual(sem_ver) => sem_ver >= other,
            TFProviderVersionConstraintClause::PatchIncrement(sem_ver) => {
                sem_ver.patch > other.patch
            }
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TofuTerraformProviderVersionObject {
    pub source: TFProviderSource,
    pub version: TFProviderVersionConstraint,
}
impl TryFrom<&Object> for TofuTerraformProviderVersionObject {
    type Error = eyre::Error;

    fn try_from(obj: &Object) -> Result<Self, Self::Error> {
        let mut source = None;
        let mut version = None;
        for (key, value) in obj {
            match key.as_ident() {
                Some(s) => match s.to_string().as_str() {
                    "source" => {
                        if source.is_some() {
                            bail!("Duplicate key: source");
                        }
                        source = Some(
                            value
                                .expr()
                                .as_str()
                                .ok_or_eyre("Expected value to be a string literal")?
                                .to_string(),
                        );
                    }
                    "version" => {
                        if version.is_some() {
                            bail!("Duplicate key: source");
                        }
                        version = Some(
                            value
                                .expr()
                                .as_str()
                                .ok_or_eyre("Expected value to be a string literal")?
                                .to_string(),
                        );
                    }
                    x => {
                        bail!("Unexpected key: {x}");
                    }
                },
                None => {
                    bail!("Unexpected entry format, key is none\nkey={key:?}\nvalue={value:?}")
                }
            }
        }
        let source = source.ok_or_eyre("Missing source attribute")?;
        let source: TFProviderSource = source.parse()?;
        let version = version.ok_or_eyre("Missing version attribute")?;
        let version: TFProviderVersionConstraint = version.parse()?;
        Ok(TofuTerraformProviderVersionObject { source, version })
    }
}
impl From<TofuTerraformProviderVersionObject> for Object {
    fn from(value: TofuTerraformProviderVersionObject) -> Self {
        let mut obj = Object::new();
        obj.insert(
            ObjectKey::Ident(Decorated::new(Ident::new("source"))),
            ObjectValue::new(value.source),
        );
        obj.insert(
            ObjectKey::Ident(Decorated::new(Ident::new("version"))),
            ObjectValue::new(value.version),
        );
        obj
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TofuTerraformRequiredProvidersBlock(
    pub HashMap<String, TofuTerraformProviderVersionObject>,
);
impl Default for TofuTerraformRequiredProvidersBlock {
    fn default() -> Self {
        Self(
            [
                (
                    "azurerm".to_string(),
                    TofuTerraformProviderVersionObject {
                        source: TFProviderSource {
                            hostname: TFProviderHostname::default(),
                            namespace: TFProviderNamespace::default(),
                            kind: TofuProviderKind::AzureRM,
                        },
                        version: TFProviderVersionConstraint {
                            clauses: vec![TFProviderVersionConstraintClause::GreaterOrEqual(
                                SemVer {
                                    major: 4,
                                    minor: Some(18),
                                    patch: Some(0),
                                    pre_release: None,
                                },
                            )],
                        },
                    },
                ),
                (
                    "azuread".to_string(),
                    TofuTerraformProviderVersionObject {
                        source: TFProviderSource {
                            hostname: TFProviderHostname::default(),
                            namespace: TFProviderNamespace::default(),
                            kind: TofuProviderKind::AzureAD,
                        },
                        version: TFProviderVersionConstraint {
                            clauses: vec![TFProviderVersionConstraintClause::GreaterOrEqual(
                                SemVer {
                                    major: 3,
                                    minor: Some(1),
                                    patch: Some(0),
                                    pre_release: None,
                                },
                            )],
                        },
                    },
                ),
                (
                    "azuredevops".to_string(),
                    TofuTerraformProviderVersionObject {
                        source: TFProviderSource {
                            hostname: TFProviderHostname::default(),
                            namespace: TFProviderNamespace("microsoft".to_string()),
                            kind: TofuProviderKind::AzureDevOps,
                        },
                        version: TFProviderVersionConstraint {
                            clauses: vec![TFProviderVersionConstraintClause::GreaterOrEqual(
                                SemVer {
                                    major: 1,
                                    minor: Some(6),
                                    patch: Some(0),
                                    pre_release: None,
                                },
                            )],
                        },
                    },
                ),
            ]
            .into(),
        )
    }
}

impl TryFrom<Block> for TofuTerraformRequiredProvidersBlock {
    type Error = eyre::Error;

    fn try_from(block: Block) -> Result<Self, Self::Error> {
        if block.ident.to_string() != "required_providers" {
            bail!("Block must use 'required_providers' ident");
        }
        let mut entries = HashMap::new();
        for attr in block.body.attributes() {
            let provider_label = &attr.key;
            let version_block = attr
                .value
                .as_object()
                .ok_or_eyre("Expected required provider value to be an object")?
                .try_into()?;
            entries.insert(provider_label.to_string(), version_block);
        }
        Ok(TofuTerraformRequiredProvidersBlock(entries))
    }
}
impl From<TofuTerraformRequiredProvidersBlock> for Block {
    fn from(value: TofuTerraformRequiredProvidersBlock) -> Self {
        let mut builder = Block::builder(Ident::new("required_providers"));
        for (provider, body) in value.0 {
            let body: Object = body.into();
            builder = builder.attribute(Attribute::new(Decorated::new(Ident::new(provider)), body));
        }
        builder.build()
    }
}

#[derive(Debug, PartialEq, Hash, Eq)]
pub struct ProviderAvailability {
    pub hostname: TFProviderHostname,
    pub namespace: TFProviderNamespace,
    pub kind: TofuProviderKind,
    pub version: SemVer,
}

impl TofuTerraformRequiredProvidersBlock {
    pub fn identify_missing(&self, available_providers: &HashSet<ProviderAvailability>) -> Self {
        let mut unsatisfied_constraints = Self(Default::default());
        for constraint in self.0.clone() {
            let available_versions: HashSet<SemVer> = available_providers
                .iter()
                .filter(|p| {
                    p.hostname == constraint.1.source.hostname
                        && p.namespace == constraint.1.source.namespace
                        && p.kind == constraint.1.source.kind
                })
                .map(|p| p.version.clone())
                .collect();
            if !available_versions
                .iter()
                .any(|available_version| constraint.1.version.is_satisfied_by(available_version))
            {
                unsatisfied_constraints.0.insert(constraint.0, constraint.1);
            }
        }

        unsatisfied_constraints
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use super::ProviderAvailability;
    use super::SemVer;
    use super::TFProviderHostname;
    use super::TFProviderNamespace;
    use super::TFProviderSource;
    use super::TofuTerraformProviderVersionObject;
    use crate::prelude::TofuProviderKind;
    use crate::prelude::TofuTerraformRequiredProvidersBlock;
    use crate::version::TFProviderVersionConstraint;
    use crate::version::TFProviderVersionConstraintClause;
    use hcl::edit::structure::Body;
    use indoc::indoc;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let tf = indoc! {r#"
            required_providers {
                azurerm = {
                    source = "hashicorp/azurerm"
                    version = ">=4.18.0"
                }
            }
        "#};
        let tf = tf.parse::<Body>()?.into_blocks().next().unwrap();
        let rp = TofuTerraformRequiredProvidersBlock::try_from(tf)?;
        println!("{rp:?}");
        Ok(())
    }

    #[test]
    pub fn semver_works() -> eyre::Result<()> {
        let x = "1.2.3";
        let y: SemVer = x.parse()?;
        let z = SemVer {
            major: 1,
            minor: Some(2),
            patch: Some(3),
            pre_release: None,
        };
        assert_eq!(y, z);

        let x = "1";
        let y: SemVer = x.parse()?;
        let z = SemVer {
            major: 1,
            minor: None,
            patch: None,
            pre_release: None,
        };

        assert_eq!(y, z);
        let x = "1.2";
        let y: SemVer = x.parse()?;
        let z = SemVer {
            major: 1,
            minor: Some(2),
            patch: None,
            pre_release: None,
        };
        assert_eq!(y, z);
        Ok(())
    }

    #[test]
    pub fn clause_works() -> eyre::Result<()> {
        let x = ">=1.2.3";
        let y: TFProviderVersionConstraintClause = x.parse()?;
        let z = TFProviderVersionConstraintClause::GreaterOrEqual(SemVer {
            major: 1,
            minor: Some(2),
            patch: Some(3),
            pre_release: None,
        });
        assert_eq!(y, z);
        Ok(())
    }

    #[test]
    pub fn clause_works2() -> eyre::Result<()> {
        let clause: TFProviderVersionConstraintClause = ">=1.2.3".parse()?;
        let a: SemVer = "1.2.4".parse()?;
        let b: SemVer = "1.2.2".parse()?;
        let c: SemVer = "1.2".parse()?;
        let d: SemVer = "1.3".parse()?;
        let e: SemVer = "1".parse()?;
        let f: SemVer = "2".parse()?;
        assert!(clause.is_satisfied_by(&a));
        assert!(!clause.is_satisfied_by(&b));
        assert!(!clause.is_satisfied_by(&c));
        assert!(clause.is_satisfied_by(&d));
        assert!(!clause.is_satisfied_by(&e));
        assert!(clause.is_satisfied_by(&f));
        Ok(())
    }

    #[test]
    pub fn identify_missing_test() -> eyre::Result<()> {
        let available_providers_fail: HashSet<ProviderAvailability> = [ProviderAvailability {
            hostname: TFProviderHostname::default(),
            namespace: TFProviderNamespace::default(),
            kind: TofuProviderKind::AzureRM,
            version: "1.2.3".parse()?,
        }]
        .into();
        let available_providers_pass: HashSet<ProviderAvailability> = [ProviderAvailability {
            hostname: TFProviderHostname::default(),
            namespace: TFProviderNamespace::default(),
            kind: TofuProviderKind::AzureRM,
            version: "4.19.0".parse()?,
        }]
        .into();
        let required_providers = TofuTerraformRequiredProvidersBlock::try_from(
            indoc! {r#"
                required_providers {
                    azurerm = {
                        source = "hashicorp/azurerm"
                        version = ">=4.18.0"
                    }
                }
            "#}
            .parse::<Body>()?
            .into_blocks()
            .next()
            .unwrap(),
        )?;
        assert_eq!(
            required_providers.identify_missing(&available_providers_pass),
            TofuTerraformRequiredProvidersBlock(Default::default()),
        );
        assert_eq!(
            required_providers.identify_missing(&available_providers_fail),
            TofuTerraformRequiredProvidersBlock(
                [(
                    "azurerm".to_string(),
                    TofuTerraformProviderVersionObject {
                        source: TFProviderSource {
                            hostname: TFProviderHostname::default(),
                            namespace: TFProviderNamespace::default(),
                            kind: TofuProviderKind::AzureRM
                        },
                        version: TFProviderVersionConstraint {
                            clauses: vec![TFProviderVersionConstraintClause::GreaterOrEqual(
                                SemVer {
                                    major: 4,
                                    minor: Some(18),
                                    patch: Some(0),
                                    pre_release: None
                                }
                            )]
                        }
                    }
                )]
                .into()
            )
        );
        Ok(())
    }
}
