use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{spanned::Spanned, ItemStruct};

pub fn derive_parse_impl(st: ItemStruct) -> syn::Result<TokenStream> {
    let mut fields = vec![];
    for (i, field) in st.fields.iter().enumerate() {
        let field_name = field
            .ident
            .clone()
            .unwrap_or_else(|| Ident::new(&i.to_string(), field.span()));
        fields.push(quote! {
            #field_name: parser.parse()?,
        })
    }

    let type_name = st.ident;
    let (impl_generics, type_generics, where_clause) = st.generics.split_for_impl();
    let fields = TokenStream::from_iter(fields);

    Ok(quote! {
        impl #impl_generics ::muscript_parsing::parsing::Parse for #type_name #type_generics #where_clause {
            fn parse(
                parser: &mut ::muscript_parsing::parsing::Parser<'_, impl ::muscript_parsing::lexis::TokenStream>
            ) -> ::std::result::Result<Self, ::muscript_parsing::parsing::ParseError>
            {
                Ok(Self {
                    #fields
                })
            }
        }
    })
}

pub fn derive_predictive_parse_impl(st: ItemStruct) -> syn::Result<TokenStream> {
    let first_field = st.fields.iter().next().ok_or_else(|| {
        syn::Error::new_spanned(
            &st.ident,
            "PredictiveParse needs at least a single field to predict parsing from",
        )
    })?;

    let type_name = st.ident;
    let (impl_generics, type_generics, where_clause) = st.generics.split_for_impl();
    let ty = &first_field.ty;

    Ok(quote! {
        impl #impl_generics ::muscript_parsing::parsing::PredictiveParse for #type_name #type_generics #where_clause {
            fn starts_with(
                token: &::muscript_parsing::lexis::token::Token,
                input: &::std::primitive::str,
            ) -> bool
            {
                <#ty as ::muscript_parsing::parsing::PredictiveParse>::starts_with(token, input)
            }
        }
    })
}
