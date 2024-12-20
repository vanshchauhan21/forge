use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Description)]
pub fn derive_description(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let doc = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("doc"))
        .and_then(|attr| attr.parse_args::<syn::LitStr>().ok())
        .map(|lit| lit.value())
        .unwrap_or_else(|| String::from(""));

    let expanded = quote! {
        impl Description for #name {
            fn description() -> &'static str {
                #doc
            }
        }
    };

    TokenStream::from(expanded)
}
