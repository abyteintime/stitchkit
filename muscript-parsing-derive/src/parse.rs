use darling::FromAttributes;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{spanned::Spanned, Index, Item, ItemEnum, ItemStruct, Path};

pub fn derive_parse_impl(item: Item) -> syn::Result<TokenStream> {
    match item {
        Item::Struct(item) => for_struct(item),
        Item::Enum(item) => for_enum(item),
        _ => Err(syn::Error::new_spanned(
            &item,
            "Parse can only be derived for structs and enums",
        )),
    }
}

fn for_struct(item: ItemStruct) -> syn::Result<TokenStream> {
    let mut fields = vec![];
    for (i, field) in item.fields.iter().enumerate() {
        let field_name = field
            .ident
            .clone()
            .unwrap_or_else(|| Ident::new(&i.to_string(), field.span()));
        let field_ty = &field.ty;
        fields.push(quote! {
            #field_name: match parser.parse::<#field_ty>() {
                Ok(n) => n,
                Err(e) => return Err(e),
            },
        })
    }

    let type_name = item.ident;
    let (impl_generics, type_generics, where_clause) = item.generics.split_for_impl();
    let fields = TokenStream::from_iter(fields);

    Ok(quote! {
        impl #impl_generics ::muscript_parsing::Parse for #type_name #type_generics #where_clause {
            fn parse(
                parser: &mut ::muscript_parsing::Parser<'_, impl ::muscript_parsing::ParseStream>
            ) -> ::std::result::Result<Self, ::muscript_parsing::ParseError>
            {
                Ok(Self {
                    #fields
                })
            }
        }
    })
}

fn for_enum(item: ItemEnum) -> syn::Result<TokenStream> {
    let ParseAttrs { error } = ParseAttrs::from_attributes(&item.attrs)?;
    let type_name = item.ident;

    let mut match_arms = vec![];
    let mut fallback = None;
    for variant in &item.variants {
        let attrs = ParseFieldAttrs::from_attributes(&variant.attrs)?;
        let variant_name = &variant.ident;
        let first_field = variant.fields.iter().next().ok_or_else(|| {
            syn::Error::new_spanned(
                variant,
                "enum variant must have at least one field to parse",
            )
        })?;
        let first_field_type = &first_field.ty;
        let constructor_fields: TokenStream = variant
            .fields
            .iter()
            .enumerate()
            .map(|(i, field)| {
                let ty = &field.ty;
                let do_parse = quote! {
                    match parser.parse::<#ty>() {
                        Ok(n) => n,
                        Err(e) => return Err(e),
                    }
                };
                if let Some(field_name) = &field.ident {
                    quote! { #field_name: #do_parse, }
                } else {
                    let index = Index::from(i);
                    quote! { #index: #do_parse, }
                }
            })
            .collect();
        let construct = quote! { #type_name::#variant_name { #constructor_fields } };
        if !attrs.fallback {
            match_arms.push(quote! {
                _ if <#first_field_type as ::muscript_parsing::PredictiveParse>::started_by(&token, parser.input) => {
                    #construct
                }
            });
        } else {
            if fallback.is_some() {
                return Err(syn::Error::new_spanned(
                    variant,
                    "only one fallback variant is allowed",
                ));
            }
            fallback = Some(quote! {
                _ => #construct
            });
        }
    }

    let (impl_generics, type_generics, where_clause) = item.generics.split_for_impl();
    let match_arms = TokenStream::from_iter(match_arms);
    let catchall_arm = fallback.unwrap_or_else(|| {
        quote! {
            _ => {
                let ref_parser: &::muscript_parsing::Parser<'_, _> = parser;
                let the_error = #error(ref_parser, &token);
                parser.emit_diagnostic(the_error);
                return Err(parser.make_error(token.span));
                // parser.bail::<Self>(token.span, the_error)?
            },
        }
    });

    Ok(quote! {
        impl #impl_generics ::muscript_parsing::Parse for #type_name #type_generics #where_clause {
            fn parse(
                parser: &mut ::muscript_parsing::Parser<'_, impl ::muscript_parsing::ParseStream>
            ) -> ::std::result::Result<Self, ::muscript_parsing::ParseError>
            {
                let token = match parser.peek_token() {
                    Ok(n) => n,
                    Err(e) => return Err(e),
                };
                Ok(match token {
                    #match_arms
                    #catchall_arm
                })
            }
        }
    })
}

#[derive(Debug, FromAttributes)]
#[darling(attributes(parse))]
struct ParseAttrs {
    error: Path,
}

#[derive(Debug, FromAttributes)]
#[darling(attributes(parse))]
struct ParseFieldAttrs {
    #[darling(default)]
    fallback: bool,
}
