use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, DeriveInput, Data, Fields};

/// Attribute macro to derive GraphNodeIconData and add necessary attributes.
#[proc_macro_attribute]
pub fn derive_graph_node_icon_data(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(item as DeriveInput);
    let struct_ident = input.ident.clone();

    // Ensure the input is a struct
    let fields = match &input.data {
        Data::Struct(data) => &data.fields,
        _ => {
            return syn::Error::new_spanned(
                input.ident,
                "derive_graph_node_icon_data can only be applied to structs",
            )
            .to_compile_error()
            .into();
        }
    };

    // Extract field names and types
    let getters = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        if let Some(ident) = name {
            // Create a getter method name, e.g., `icon_width` -> `icon_width()`
            let method_name = ident.clone();
            quote! {
                fn #method_name(&self) -> #ty {
                    self.#ident.clone()
                }
            }
        } else {
            // Handle unnamed fields (tuples) if necessary
            // For this use case, it's assumed all fields are named
            quote! {}
        }
    });

    // Prepare the new derives and attributes
    let expanded = quote! {
        #[derive(Debug, Resource, Default, Reflect)]
        #[reflect(Resource)]
        #input

        impl GraphNodeIconData for #struct_ident {
            #(#getters)*
        }
    };

    TokenStream::from(expanded)
}
