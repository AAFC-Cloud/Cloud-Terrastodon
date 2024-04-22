use std::path::Path;
use std::path::PathBuf;

use anyhow::anyhow;
use anyhow::Result;
use itertools::Itertools;

pub fn split_to_files(code: &str, out_dir: &Path) -> Result<Vec<(PathBuf, String)>> {
    let body = hcl::parse(code)?;
    let mut rtn = Vec::new();
    for block in body
        .blocks()
        .filter(|b| b.identifier.as_str() == "resource")
    {
        let [kind, name, ..] = block.labels.as_slice() else {
            return Err(anyhow!("failed to destructure").context(format!("{:?}", block.labels)));
        };
        let out_file = out_dir.join(PathBuf::from_iter([
            kind.as_str(),
            format!("{}.tf", name.as_str()).as_str(),
        ]));
        rtn.push((out_file, hcl::to_string(block)?));
    }

    Ok(rtn)
}

pub fn get_blocks(code: &str) -> Result<String> {
    let body = hcl::parse(code)?;
    Ok(body
        .blocks()
        .map(|b| b.labels.iter().map(|l| l.as_str()).join(","))
        .join("\n"))
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let input = indoc::indoc! {r#"
            resource "azurerm_policy_definition" "my_definition" {
                display_name = "beans"
            }
        "#};
        let blocks = super::get_blocks(input);
        println!("got {:?}", blocks);
    }
}
