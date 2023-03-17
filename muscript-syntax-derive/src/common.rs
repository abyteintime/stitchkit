use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Field, Index};

pub fn field_name(i: usize, field: &Field) -> TokenStream {
    field
        .ident
        .as_ref()
        .map(|ident| ident.to_token_stream())
        .unwrap_or_else(|| {
            let i = Index::from(i);
            quote! { #i }
        })
}
