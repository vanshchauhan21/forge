use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Trait for types that have documentation.
pub trait Documented {
    fn doc() -> &'static str;
}

#[proc_macro_derive(Documented)]
pub fn derive_documented(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let doc = input.attrs.iter()
        .find(|attr| attr.path().is_ident("doc"))
        .and_then(|attr| attr.parse_args::<syn::LitStr>().ok())
        .map(|lit| lit.value())
        .unwrap_or_else(|| String::from(""));

    let expanded = quote! {
        impl Documented for #name {
            fn doc() -> &'static str {
                #doc
            }
        }
    };

    TokenStream::from(expanded)
}
