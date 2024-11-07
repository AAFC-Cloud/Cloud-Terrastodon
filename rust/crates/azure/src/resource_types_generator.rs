#[cfg(test)]
mod tests {
    use cloud_terrastodon_core_command::prelude::CommandBuilder;
    use cloud_terrastodon_core_command::prelude::CommandKind;
    use proc_macro2::Ident;
    use quote::quote;
    use serde::Deserialize;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use std::process::Command;

    // Helper function to sanitize a kind string into a valid Rust enum variant
    fn to_enum_variant(kind: &str) -> String {
        kind.replace('.', "_DOT_")
            .replace('/', "_SLASH_")
            .to_uppercase()
    }

    #[tokio::test]
    #[ignore]
    async fn gen_definitions() -> anyhow::Result<()> {
        let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
        cmd.args([
            "rest",
            "--method",
            "get",
            "--url",
            "https://management.azure.com/providers?api-version=2020-01-01",
        ]);
        #[derive(Deserialize)]
        struct RT {
            #[serde(rename = "resourceType")]
            resource_type: String,
        }
        #[derive(Deserialize)]
        struct Provider {
            namespace: String,
            #[serde(rename = "resourceTypes")]
            resource_types: Vec<RT>,
        }
        #[derive(Deserialize)]
        struct Resp {
            value: Vec<Provider>,
        }
        let mut resource_types = Vec::new();
        let resp = cmd.run::<Resp>().await?;
        for prov in resp.value {
            for rt in prov.resource_types {
                resource_types.push(format!("{}/{}", &prov.namespace, rt.resource_type));
            }
        }

        // println!("{resource_types:#?}");

        // Generate enum variants and attributes
        let enum_variants: Vec<proc_macro2::TokenStream> = resource_types
            .iter()
            .map(|kind| {
                let variant_name =
                    Ident::new(&to_enum_variant(kind), proc_macro2::Span::call_site());
                quote! {
                    #variant_name,
                }
            })
            .collect();

        // Add the `Other(String)` variant manually
        let other_variant = quote! {
            Other(String),
        };

        // Combine all the generated variants into a full enum definition
        // use serde::{Deserialize, Serialize};
        // #[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
        let generated_enum = quote! {
            #[derive(Debug, Clone, Eq, PartialEq, Hash)]
            #[allow(non_camel_case_types)]
            pub enum ResourceType {
                #(#enum_variants)*
                #other_variant
            }
        };

        let match_arms: Vec<proc_macro2::TokenStream> = resource_types
            .iter()
            .map(|kind| {
                let kind = kind.to_lowercase();
                let variant_name =
                    Ident::new(&to_enum_variant(&kind), proc_macro2::Span::call_site());
                quote! {
                    #kind => return Ok(ResourceType::#variant_name),
                }
            })
            .collect();
        let impl_fromstr = quote! {
            impl std::str::FromStr for ResourceType {
                type Err = anyhow::Error;
                fn from_str(value: &str) -> Result<Self, Self::Err> {
                    match value.to_lowercase().as_str() {
                        #(#match_arms)*
                        _ => {}
                    }
                    Ok(ResourceType::Other(value.to_string()))
                }
            }
        };

        let match_arms: Vec<proc_macro2::TokenStream> = resource_types
            .iter()
            .map(|kind| {
                let variant_name =
                    Ident::new(&to_enum_variant(kind), proc_macro2::Span::call_site());
                quote! {
                    ResourceType::#variant_name => #kind ,
                }
            })
            .collect();
        let impl_asrefstr = quote! {
            impl AsRef<str> for ResourceType {
                fn as_ref(&self) -> &str {
                    match self {
                        #(#match_arms)*
                        ResourceType::Other(value) => value.as_str()
                    }
                }
            }
        };

        let all_together = quote! {
            #generated_enum
            #impl_fromstr
            #impl_asrefstr
        };

        // Format the generated code using prettyplease
        let formatted_code = prettyplease::unparse(&syn::parse2(all_together).unwrap());

        // Write the generated code to a file
        let target = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
            .join("../azure_types/src/resource_types.rs");
        println!("Writing to {}", target.display());
        let mut file = File::create(&target)?;
        write!(
            file,
            "// DO NOT MODIFY BY HAND!!!\n// THIS FILE IS AUTO-GENERATED BY {}\n{}",
            file!(),
            formatted_code
        )?;

        // Call rustfmt to format the generated code
        Command::new("rustfmt")
            .arg(target.as_os_str())
            .status()
            .expect("Failed to execute rustfmt");

        Ok(())
    }
}
