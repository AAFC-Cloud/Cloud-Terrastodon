use crate::menu_action;
use cloud_terrastodon_core_user_input::prelude::prompt_line;
use quote::quote;
use std::path::PathBuf;
use syn::parse_file;
use syn::Item;
use syn::ItemEnum;
use syn::Variant;
use tracing::info;

pub async fn create_new_action_variant() -> anyhow::Result<()> {
    let menu_action_file =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join(menu_action::THIS_FILE);
    info!("Going to modify {}", menu_action_file.display());
    let content = tokio::fs::read_to_string(&menu_action_file).await?;
    let mut ast = parse_file(&content)?;

    let new_variant_decl = prompt_line("Enter the new variant declaration:").await?;
    let new_variant_display = prompt_line("Enter the new variant display str:").await?;
    for item in &mut ast.items {
        if let Item::Enum(ItemEnum {
            ref mut variants, ..
        }) = item
        {
            let new_variant: Variant = syn::parse_quote! {
                {new_variant_decl}
            };
            variants.push(new_variant);
        }
    }
    let modified_code = quote! {
        #ast
    };

    tokio::fs::write(menu_action_file, modified_code.to_string()).await?;
    Ok(())
}
