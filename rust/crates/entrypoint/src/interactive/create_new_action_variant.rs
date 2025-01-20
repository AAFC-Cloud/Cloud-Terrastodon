use crate::menu_action;
use cloud_terrastodon_core_user_input::prelude::prompt_line;
use quote::quote;
use std::path::PathBuf;
use syn::parse_file;
use syn::parse_str;
use syn::Arm;
use syn::Expr;
use syn::ImplItem;
use syn::Item;
use syn::ItemEnum;
use syn::Stmt;
use syn::Type;
use syn::Variant;
use tokio::process::Command;
use tracing::info;

pub async fn create_new_action_variant() -> anyhow::Result<()> {
    let menu_action_file =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join(menu_action::THIS_FILE);
    info!("Going to modify {}", menu_action_file.display());
    let content = tokio::fs::read_to_string(&menu_action_file).await?;
    let mut ast = parse_file(&content)?;

    let new_variant_decl =
        prompt_line("Enter the new enum variant name, e.g., \"BuildPolicyImports\":").await?;
    let new_variant_display =
        prompt_line("Enter the display name for the new variant, e.g., \"build policy imports\":")
            .await?;
    let function_name =
        prompt_line("Enter the function name, e.g., \"build_policy_imports\":").await?;

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
        return Err(anyhow::anyhow!(
            "MenuAction enum not found in {}",
            menu_action_file.display()
        ));
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

    let modified_code = quote! {
        #ast
    };

    let pretty_code = prettyplease::unparse(&syn::parse2(modified_code)?);
    tokio::fs::write(&menu_action_file, pretty_code).await?;

    Command::new("rustfmt")
        .arg(menu_action_file.as_os_str())
        .args(["--edition", "2021"])
        .status()
        .await?;

    Ok(())
}

fn add_name_match_arm(
    method: &mut syn::ImplItemFn,
    variant_ident: &syn::Ident,
    display_name: &str,
) -> anyhow::Result<()> {
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
) -> anyhow::Result<()> {
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
