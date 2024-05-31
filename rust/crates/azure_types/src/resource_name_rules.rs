use anyhow::bail;
use anyhow::Result;

// https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/resource-name-rules

/// https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/resource-name-rules#microsoftauthorization
pub fn validate_policy_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("Name {name:?} must not be empty");
    }
    if name.len() > 64 {
        bail!("Name {name:?} must not be longer than 64 characters");
    }

    // https://github.com/MicrosoftDocs/azure-docs/issues/122963
    let forbidden_chars = r#"#<>*%&\?.+/"#;
    for char in name.chars() {
        if forbidden_chars.contains(char) {
            bail!("Name {name:?} must not contain the character {char:?}");
        }
        if char.is_control() {
            bail!("Name {name:?} must not contain control characters");
        }
    }

    if name.ends_with('.') {
        bail!("Name {name:?} must not end with a period");
    }
    if name.ends_with(' ') {
        bail!("Name {name:?} must not end with a space");
    }
    Ok(())
}
