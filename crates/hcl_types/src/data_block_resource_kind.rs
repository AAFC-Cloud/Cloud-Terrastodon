use crate::prelude::AzureAdDataBlockKind;
use crate::prelude::AzureDevOpsDataBlockKind;
use crate::prelude::AzureRmDataBlockKind;
use crate::prelude::OtherDataBlockKind;
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum DataBlockResourceKind {
    AzureAD(AzureAdDataBlockKind),
    AzureRM(AzureRmDataBlockKind),
    AzureDevOps(AzureDevOpsDataBlockKind),
    Other(OtherDataBlockKind),
}
impl std::fmt::Display for DataBlockResourceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataBlockResourceKind::AzureAD(kind) => kind.fmt(f),
            DataBlockResourceKind::AzureRM(kind) => kind.fmt(f),
            DataBlockResourceKind::AzureDevOps(kind) => kind.fmt(f),
            DataBlockResourceKind::Other(kind) => kind.fmt(f),
        }
    }
}
impl FromStr for DataBlockResourceKind {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(kind) = s.parse::<AzureRmDataBlockKind>() {
            return Ok(DataBlockResourceKind::AzureRM(kind));
        }
        if let Ok(kind) = s.parse::<AzureAdDataBlockKind>() {
            return Ok(DataBlockResourceKind::AzureAD(kind));
        }
        if let Ok(kind) = s.parse::<AzureDevOpsDataBlockKind>() {
            return Ok(DataBlockResourceKind::AzureDevOps(kind));
        }
        Ok(DataBlockResourceKind::Other(OtherDataBlockKind::from_str(
            s,
        )?))
    }
}
