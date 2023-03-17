use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, Ident, Item, ItemEnum, ItemStruct};

use crate::common::field_name;

pub fn derive_spanned_impl(item: Item) -> syn::Result<TokenStream> {
    match item {
        Item::Struct(item) => for_struct(item),
        Item::Enum(item) => for_enum(item),
        _ => Err(syn::Error::new_spanned(
            &item,
            "Spanned can only be derived for structs and enums",
        )),
    }
}

fn for_struct(item: ItemStruct) -> syn::Result<TokenStream> {
    if item.fields.is_empty() {
        return Err(syn::Error::new_spanned(
            item.ident,
            "Spanned requires at least one field to get the span from",
        ));
    }

    // NOTE: It would be incorrect to assume we can just take the first and last field and call it
    // a day. This is because either of them can be `Option<T>` or `Vec<T>`, which can produce an
    // empty span, in which case only the first field's span would be used. Or heck, even both
    // the first and the last field could produce an empty span, and then we're in for a bad time.

    let mut get_span = TokenStream::new();
    for (i, field) in item.fields.iter().enumerate() {
        let field_name = field_name(i, field);
        let ty = &field.ty;
        let get_field_span = quote! {
            <#ty as ::muscript_foundation::source::Spanned>::span(&self.#field_name)
        };
        if i == 0 {
            get_span.extend(get_field_span);
        } else {
            get_span.extend(quote! { .join(&#get_field_span) });
        }
    }

    let type_name = item.ident;
    let (impl_generics, type_generics, where_clause) = item.generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics ::muscript_foundation::source::Spanned for #type_name #type_generics #where_clause {
            fn span(&self) -> ::muscript_foundation::source::Span {
                #get_span
            }
        }
    })
}

fn for_enum(item: ItemEnum) -> syn::Result<TokenStream> {
    let mut arms = TokenStream::new();
    for (_, variant) in item.variants.iter().enumerate() {
        let mut get_span = TokenStream::new();
        let mut destructuring = TokenStream::new();
        for (i, field) in variant.fields.iter().enumerate() {
            let field_name = field_name(i, field);
            let destructured_var_name = Ident::new(&format!("__span_{i}"), field.ident.span());
            let ty = &field.ty;
            let get_field_span = quote! {
                <#ty as ::muscript_foundation::source::Spanned>::span(&#destructured_var_name)
            };
            if i == 0 {
                get_span.extend(get_field_span);
            } else {
                get_span.extend(quote! { .join(&#get_field_span) });
            }
            destructuring.extend(quote! { #field_name: #destructured_var_name, });
        }

        let variant_name = &variant.ident;
        let arm = quote! {
            Self::#variant_name { #destructuring } => #get_span,
        };
        arms.extend(arm);
    }

    let type_name = item.ident;
    let (impl_generics, type_generics, where_clause) = item.generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics ::muscript_foundation::source::Spanned for #type_name #type_generics #where_clause {
            fn span(&self) -> ::muscript_foundation::source::Span {
                match self {
                    #arms
                }
            }
        }
    })
}
