use proc_macro2::TokenStream;
use quote::quote;
use syn::{Item, ItemEnum, ItemStruct};

pub fn derive_predictive_parse_impl(item: Item) -> syn::Result<TokenStream> {
    match item {
        Item::Struct(item) => for_struct(item),
        Item::Enum(item) => for_enum(item),
        _ => Err(syn::Error::new_spanned(
            &item,
            "PredictiveParse can only be derived for structs and enums",
        )),
    }
}

fn for_struct(item: ItemStruct) -> syn::Result<TokenStream> {
    let first_field = item.fields.iter().next().ok_or_else(|| {
        syn::Error::new_spanned(
            &item.ident,
            "PredictiveParse needs at least a single field to predict parsing from",
        )
    })?;

    let type_name = item.ident;
    let (impl_generics, type_generics, where_clause) = item.generics.split_for_impl();
    let ty = &first_field.ty;

    Ok(quote! {
        impl #impl_generics ::muscript_parsing::PredictiveParse for #type_name #type_generics #where_clause {
            fn started_by(
                token: &::muscript_parsing::lexis::token::Token,
                input: &::std::primitive::str,
            ) -> bool
            {
                <#ty as ::muscript_parsing::PredictiveParse>::started_by(token, input)
            }
        }
    })
}

fn for_enum(item: ItemEnum) -> syn::Result<TokenStream> {
    let mut expr = TokenStream::new();
    for (i, variant) in item.variants.iter().enumerate() {
        let first_field = variant.fields.iter().next().ok_or_else(|| {
            syn::Error::new_spanned(
                &item.ident,
                "PredictiveParse needs at least a single field to predict parsing from",
            )
        })?;
        let ty = &first_field.ty;
        if i != 0 {
            expr.extend(quote!(||))
        }
        expr.extend(quote! {
            <#ty as ::muscript_parsing::PredictiveParse>::started_by(token, input)
        });
    }

    let type_name = item.ident;
    let (impl_generics, type_generics, where_clause) = item.generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics ::muscript_parsing::PredictiveParse for #type_name #type_generics #where_clause {
            fn started_by(
                token: &::muscript_parsing::lexis::token::Token,
                input: &::std::primitive::str,
            ) -> bool
            {
                #expr
            }
        }
    })
}
