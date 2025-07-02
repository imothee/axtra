use inflector::Inflector;
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Lit, Meta, parse_macro_input};

pub fn response_key_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    // Generate the default snake_case name once
    let default_key = struct_name.to_string().to_snake_case();

    // Look for the response_key attribute
    let response_key = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("response_key"))
        .map(|attr| {
            match &attr.meta {
                Meta::List(meta_list) => {
                    // #[response_key("custom_name")]
                    if let Ok(lit) = syn::parse2::<Lit>(meta_list.tokens.clone()) {
                        if let Lit::Str(lit_str) = lit {
                            return lit_str.value();
                        }
                    }
                    // Fallback to auto-derived name if parsing fails
                    default_key.clone()
                }
                Meta::Path(_) => {
                    // #[response_key] - auto-derive from struct name
                    default_key.clone()
                }
                Meta::NameValue(name_value) => {
                    // #[response_key = "custom_name"]
                    if let syn::Expr::Lit(expr_lit) = &name_value.value {
                        if let Lit::Str(lit_str) = &expr_lit.lit {
                            lit_str.value()
                        } else {
                            default_key.clone()
                        }
                    } else {
                        default_key.clone()
                    }
                }
            }
        })
        .unwrap_or(default_key);

    let expanded = quote! {
        impl ::axtra::response::ResponseKey for #struct_name {
            fn response_key() -> &'static str {
                #response_key
            }
        }
    };

    TokenStream::from(expanded)
}
