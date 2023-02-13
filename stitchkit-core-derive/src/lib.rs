use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, ItemStruct, LitStr};

fn derive_deserialize_impl(st: ItemStruct) -> syn::Result<TokenStream> {
    let mut constructor_fields = vec![];
    for (i, field) in st.fields.iter().enumerate() {
        let field_name = field
            .ident
            .clone()
            .unwrap_or_else(|| Ident::new(&i.to_string(), field.span()));
        let error = LitStr::new(
            &format!("cannot deserialize field {}::{}", st.ident, field_name),
            field.span(),
        );
        constructor_fields.push(quote! {
            #field_name: ::anyhow::Context::context(
                ::stitchkit_core::binary::Deserialize::deserialize(&mut reader),
                #error
            )?,
        })
    }

    let type_name = st.ident;
    let (impl_generics, type_generics, where_clause) = st.generics.split_for_impl();
    let constructor_fields = TokenStream::from_iter(constructor_fields);

    Ok(quote! {
        impl #impl_generics ::stitchkit_core::binary::Deserialize for #type_name #type_generics #where_clause {
            fn deserialize(mut reader: impl ::std::io::Read) -> ::anyhow::Result<Self> {
                Ok(Self {
                    #constructor_fields
                })
            }
        }
    })
}

#[proc_macro_derive(Deserialize)]
pub fn derive_deserialize(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as ItemStruct);

    derive_deserialize_impl(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
