extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{self, punctuated::Punctuated, token::Comma, Field, Ident};

#[proc_macro_derive(WithOptionDefaults)]
pub fn with_option_defaults_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input)
        .expect("Valid Rust code must be provided.");

    let struct_name: &Ident = &ast.ident;

    let named_fields: &Punctuated<Field, Comma> = match &ast.data {
        syn::Data::Struct(x) => match &x.fields {
            syn::Fields::Named(y) => &y.named,
            _ => panic!("Only named fields supported!"),
        },
        _ => panic!("Only structs supported!"),
    };

    let field_name: Vec<&Ident> = named_fields.iter().map(|f: &Field| {
        f.ident.as_ref().expect("Identifier expected")
    }).collect();

    let generated = quote! {
        impl #struct_name {
            pub fn with_option_defaults(&self, defaults: Self) -> Self {
                Self {
                    #(#field_name: self.#field_name.clone().or(defaults.#field_name)),*
                }
            }
        }
    };

    generated.into()
}

