use darling::FromAttributes;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Item, ItemEnum, ItemStruct, LitStr};

use crate::parse::ParseFieldAttrs;

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
        impl #impl_generics ::muscript_syntax::PredictiveParse for #type_name #type_generics #where_clause {
            const LISTEN_TO_CHANNELS: ::muscript_lexer::token_stream::Channel =
                <#ty as ::muscript_syntax::PredictiveParse>::LISTEN_TO_CHANNELS;

            #[allow(deprecated)]
            fn started_by(
                token: &::muscript_lexer::token::AnyToken,
                sources: &::muscript_lexer::sources::LexedSources<'_>,
            ) -> bool
            {
                <#ty as ::muscript_syntax::PredictiveParse>::started_by(token, sources)
            }
        }
    })
}

fn for_enum(item: ItemEnum) -> syn::Result<TokenStream> {
    let mut listen_to_channels = TokenStream::new();
    let mut started_by = TokenStream::new();
    for (i, variant) in item.variants.iter().enumerate() {
        let attrs = ParseFieldAttrs::from_attributes(&variant.attrs)?;
        let first_field = variant.fields.iter().next().ok_or_else(|| {
            syn::Error::new_spanned(
                &item.ident,
                "PredictiveParse needs at least a single field to predict parsing from",
            )
        })?;
        let ty = &first_field.ty;

        if i != 0 {
            started_by.extend(quote!(||))
        }
        let test = if let Some(keyword) = &attrs.keyword {
            let keyword = LitStr::new(keyword, variant.ident.span());
            quote! {
                (token.kind == ::muscript_lexer::token::TokenKind::Ident &&
                    sources.source(token).eq_ignore_ascii_case(#keyword))
            }
        } else {
            quote! { <#ty as ::muscript_syntax::PredictiveParse>::started_by(token, sources) }
        };
        started_by.extend(test);

        listen_to_channels.extend(if i != 0 {
            quote! {
                .union(<#ty as ::muscript_syntax::PredictiveParse>::LISTEN_TO_CHANNELS)
            }
        } else {
            quote! {
                <#ty as ::muscript_syntax::PredictiveParse>::LISTEN_TO_CHANNELS
            }
        });
    }

    let type_name = item.ident;
    let (impl_generics, type_generics, where_clause) = item.generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics ::muscript_syntax::PredictiveParse for #type_name #type_generics #where_clause {
            fn started_by(
                token: &::muscript_lexer::token::AnyToken,
                sources: &::muscript_lexer::sources::LexedSources<'_>,
            ) -> bool
            {
                #started_by
            }
        }
    })
}
