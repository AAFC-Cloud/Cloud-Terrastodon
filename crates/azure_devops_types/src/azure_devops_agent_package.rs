use chrono::DateTime;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct AzureDevOpsAgentPackageVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct AzureDevOpsAgentPackage {
    pub created_on: DateTime<Utc>,
    pub download_url: String,
    pub filename: String,
    pub hash_value: Option<String>,
    pub info_url: String,
    pub platform: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub version: AzureDevOpsAgentPackageVersion,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn deserialize_sample() -> eyre::Result<()> {
        let s = r#"{
            "createdOn": "2025-12-17T14:25:22.87Z",
            "downloadUrl": "https://download.agent.dev.azure.com/agent/4.266.2/vsts-agent-win-x64-4.266.2.zip",
            "filename": "vsts-agent-win-x64-4.266.2.zip",
            "hashValue": "d0c0204972cb2298ad9a149d959b4c3d9085226f04e7b83a499980509b439146",
            "infoUrl": "https://go.microsoft.com/fwlink/?LinkId=798199",
            "platform": "win-x64",
            "type": "agent",
            "version": { "major": 4, "minor": 266, "patch": 2 }
        }"#;

        let pkg: AzureDevOpsAgentPackage = serde_json::from_str(s)?;
        assert_eq!(pkg.platform, "win-x64");
        assert_eq!(pkg.r#type, "agent");
        assert_eq!(pkg.version.major, 4);
        Ok(())
    }
}
