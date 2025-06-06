use eyre::Result;
use eyre::bail;

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
    let forbidden_chars = r#"#<>*%&\?+/"#;
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

/// https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/resource-name-rules#microsoftmanagement
pub fn validate_management_group_name(name: &str) -> Result<()> {
    // names can be empty if no management groups exist
    // if name.is_empty() || name.len() > 90 {
    //     bail!(
    //         "Name {name:?} must be  1-90 characters (was {})",
    //         name.len()
    //     );
    // }

    if let Some(first_char) = name.chars().next()
        && !first_char.is_alphanumeric()
    {
        bail!("Name must start with a letter or number");
    }

    if name.ends_with('.') {
        bail!("Name must not end with a period");
    }

    for char in name.chars() {
        if !(char.is_alphanumeric() || "-_().".contains(char)) {
            bail!("Name must not contain the character {char:?}");
        }
    }
    Ok(())
}
