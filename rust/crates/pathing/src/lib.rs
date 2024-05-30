// https://stackoverflow.com/a/74942075/11141271
#[cfg(test)]
fn cd_to_workspace_dir() -> anyhow::Result<std::path::PathBuf> {
    use std::env::current_dir;
    use std::env::set_current_dir;
    use std::path::Path;
    use anyhow::anyhow;

    let output = std::process::Command::new(env!("CARGO"))
        .arg("locate-project")
        .arg("--workspace")
        .arg("--message-format=plain")
        .output()
        .unwrap()
        .stdout;
    let cargo_path = Path::new(std::str::from_utf8(&output).unwrap().trim());
    let workspace_dir = cargo_path
        .parent()
        .ok_or(anyhow!("Cargo path parent is not valid"))?;
    let rtn = current_dir()?;
    set_current_dir(workspace_dir)?;
    Ok(rtn)
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use super::*;

    #[test]
    fn it_works() -> anyhow::Result<()> {
        cd_to_workspace_dir()?;
        let mut manifest = std::fs::OpenOptions::new().read(true).open("Cargo.toml")?;
        let mut contents = String::new();
        manifest.read_to_string(&mut contents)?;
        contents
            .lines()
            .find(|l| *l == r#"name = "cloud_terrastodon""#)
            .unwrap();
        Ok(())
    }
}
