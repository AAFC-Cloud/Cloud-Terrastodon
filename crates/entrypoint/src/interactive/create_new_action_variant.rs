use crate::menu_action;
use cloud_terrastodon_core_user_input::prelude::prompt_line;
use eyre::Context;
use eyre::bail;
use quote::quote;
use std::path::PathBuf;
use syn::Arm;
use syn::Expr;
use syn::ImplItem;
use syn::Item;
use syn::ItemEnum;
use syn::ItemUse;
use syn::Stmt;
use syn::Type;
use syn::Variant;
use syn::parse_file;
use syn::parse_str;
use tokio::process::Command;
use tracing::info;

pub async fn create_new_action_variant() -> eyre::Result<()> {
    let new_variant_decl =
        prompt_line("Enter the new enum variant name, e.g., \"BuildPolicyImports\":").await?;
    let new_variant_display =
        prompt_line("Enter the display name for the new variant, e.g., \"build policy imports\":")
            .await?;
    let function_name =
        prompt_line("Enter the function name, e.g., \"build_policy_imports\":").await?;

    update_menu_action_rs_file(&new_variant_decl, &new_variant_display, &function_name).await?;

    // Now, create the new file in src/interactive/<function_name>.rs with boilerplate code.
    create_new_function_file(&function_name).await?;
    // Updating the mod file should happen after to avoid issues about the function.rs file not existing.
    update_interactive_entrypoint_mod_rs_file(&function_name).await?;
    add_import_statement_to_menu_action_rs(&function_name).await?;
    Ok(())
}

async fn mutate_file<T>(path: &str, mut mutator: T) -> eyre::Result<()>
where
    T: FnMut(&mut syn::File) -> eyre::Result<()>,
{
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    #[cfg(not(test))]
    let manifest_dir = PathBuf::from(&manifest_dir);
    #[cfg(test)]
    let mut manifest_dir = PathBuf::from(&manifest_dir);
    #[cfg(test)]
    {
        manifest_dir.pop();
        manifest_dir.pop();
    }
    let full_path = manifest_dir.join(path);
    let content = tokio::fs::read_to_string(&full_path)
        .await
        .wrap_err(format!(
            "Failed to find {} in manifest dir {} given path {}",
            full_path.display(),
            manifest_dir.display(),
            &path
        ))?;
    let mut ast = parse_file(&content)?;

    info!("Modifying {}", full_path.display());
    mutator(&mut ast).wrap_err(format!("Failed mutating {}", full_path.display()))?;

    let modified_code = quote! {
        #ast
    };

    let pretty_code = prettyplease::unparse(&syn::parse2(modified_code)?);
    tokio::fs::write(&full_path, pretty_code).await?;

    Command::new("rustfmt")
        .arg(full_path.as_os_str())
        .args(["--edition", "2021"])
        .status()
        .await
        .wrap_err("Error applying rustfmt")?;
    Ok(())
}

async fn update_menu_action_rs_file(
    new_variant_decl: &str,
    new_variant_display: &str,
    function_name: &str,
) -> eyre::Result<()> {
    fn add_name_match_arm(
        method: &mut syn::ImplItemFn,
        variant_ident: &syn::Ident,
        display_name: &str,
    ) -> eyre::Result<()> {
        for stmt in &mut method.block.stmts {
            if let Stmt::Expr(Expr::Match(ref mut match_expr), _) = stmt {
                let new_arm: Arm = parse_str(&format!(
                    "MenuAction::{} => \"{}\",",
                    variant_ident, display_name
                ))?;
                match_expr.arms.push(new_arm);
                break;
            }
        }
        Ok(())
    }

    fn add_invoke_match_arm(
        method: &mut syn::ImplItemFn,
        variant_ident: &syn::Ident,
        function_name: &str,
    ) -> eyre::Result<()> {
        for stmt in &mut method.block.stmts {
            if let Stmt::Expr(Expr::Match(ref mut match_expr), _) = stmt {
                let new_arm: Arm = parse_str(&format!(
                    "MenuAction::{} => {}().await?,",
                    variant_ident, function_name
                ))?;
                match_expr.arms.push(new_arm);
                break;
            }
        }
        Ok(())
    }

    mutate_file(menu_action::THIS_FILE, |ast| {
        // Parse the new variant declaration into a syn::Variant
        let new_variant: Variant = syn::parse_str(&new_variant_decl)?;
        let new_variant_ident = new_variant.ident.clone(); // Clone ident before moving new_variant

        // Add the new variant to the MenuAction enum
        let mut variant_added = false;
        for item in &mut ast.items {
            if let Item::Enum(ItemEnum {
                ref ident,
                ref mut variants,
                ..
            }) = item
            {
                if ident == "MenuAction" {
                    variants.push(new_variant);
                    variant_added = true;
                    break; // Ensure we only push once
                }
            }
        }

        if !variant_added {
            bail!("MenuAction enum not found");
        }

        // Modify the impl block of MenuAction
        for item in &mut ast.items {
            if let Item::Impl(ref mut impl_item) = item {
                // Ensure we're modifying impl MenuAction, not any trait implementation
                if impl_item.trait_.is_none() {
                    if let Type::Path(ref type_path) = *impl_item.self_ty {
                        if type_path.path.is_ident("MenuAction") {
                            for impl_item in &mut impl_item.items {
                                if let ImplItem::Fn(ref mut method) = impl_item {
                                    // Modify the name() method
                                    if method.sig.ident == "name" {
                                        add_name_match_arm(
                                            method,
                                            &new_variant_ident,
                                            &new_variant_display,
                                        )?;
                                    }
                                    // Modify the invoke() method
                                    else if method.sig.ident == "invoke" {
                                        add_invoke_match_arm(
                                            method,
                                            &new_variant_ident,
                                            &function_name,
                                        )?;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    })
    .await?;

    Ok(())
}

async fn update_interactive_entrypoint_mod_rs_file(function_name: &str) -> eyre::Result<()> {
    mutate_file(crate::interactive::THIS_FILE, |ast| {
        // --- Add the new mod statement at the top ---
        // You may want to determine the proper location for the mod statement.
        let new_mod_code = format!("mod {};", function_name);
        let new_mod: syn::ItemMod =
            parse_str(&new_mod_code).wrap_err("Failed to parse new mod statement")?;
        // For example, insert at the beginning (or after other mod statements)
        ast.items.insert(0, Item::Mod(new_mod));

        // --- Add the new pub use statement in the prelude module ---
        for item in &mut ast.items {
            if let Item::Mod(ref mut item_mod) = item {
                if item_mod.ident == "prelude" {
                    // Ensure that the module is inline (has a body)
                    let (_, ref mut body) = item_mod
                        .content
                        .as_mut()
                        .ok_or_else(|| eyre::eyre!("prelude module has no inline content"))?;
                    let new_use_code = format!("pub use crate::interactive::{}::*;", function_name);
                    let new_use: ItemUse =
                        parse_str(&new_use_code).wrap_err("Failed to parse new use statement")?;
                    body.push(Item::Use(new_use));
                    break;
                }
            }
        }
        Ok(())
    })
    .await
}

async fn add_import_statement_to_menu_action_rs(function_name: &str) -> eyre::Result<()> {
    mutate_file(crate::menu_action::THIS_FILE, |ast| {
        let use_statement = format!("use crate::interactive::prelude::{};", function_name);
        let use_statement = parse_str(&use_statement)?;
        ast.items.insert(0, use_statement);
        Ok(())
    })
    .await
}

/// This function creates a new file at src/interactive/{function_name}.rs
/// with the following boilerplate:
///
/// ```rust
/// use eyre::Result;
///
/// pub async fn function_name() -> Result<()> {
///     Ok(())
/// }
/// ```
async fn create_new_function_file(function_name: &str) -> eyre::Result<()> {
    // Determine the manifest directory. This assumes that your source is in `src/interactive`
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let mut crate_dir = PathBuf::from(&manifest_dir).join(crate::interactive::THIS_FILE);
    crate_dir.pop();
    let file_path = crate_dir.join(format!("{}.rs", function_name));

    // Create the boilerplate code using a raw string literal.
    let boilerplate = format!(
        r#"use eyre::Result;

pub async fn {function_name}() -> Result<()> {{
    Ok(())
}}
"#,
        function_name = function_name
    );

    // Write the file asynchronously.
    tokio::fs::write(&file_path, boilerplate)
        .await
        .wrap_err_with(|| format!("Failed to write to {}", file_path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn it_works() -> eyre::Result<()> {
        update_interactive_entrypoint_mod_rs_file("hehe_testing").await?;
        Ok(())
    }
}
