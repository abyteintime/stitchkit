use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{spanned::Spanned, Expr, ItemStruct, LitStr};

pub fn derive_deserialize_impl(st: ItemStruct) -> syn::Result<TokenStream> {
    let mut variables = vec![];
    let mut constructor_fields = vec![];
    for (i, field) in st.fields.iter().enumerate() {
        let field_name = field
            .ident
            .clone()
            .unwrap_or_else(|| Ident::new(&i.to_string(), field.span()));
        let field_type = field.ty.clone();
        let error = LitStr::new(
            &format!("cannot deserialize field {}::{}", st.ident, field_name),
            field.span(),
        );

        let serialized_when = if let Some(serialized_when_attr) = field
            .attrs
            .iter()
            .find(|attr| attr.path.is_ident("serialized_when"))
        {
            Some(serialized_when_attr.parse_args::<Expr>()?)
        } else {
            None
        };

        let variable_value = serialized_when
            .map(|cond| {
                quote! {
                    if #cond {
                        ::stitchkit_core::binary::Deserialize::deserialize(deserializer)
                            .map(|val| ::std::option::Option::Some(val))
                    } else {
                        ::std::result::Result::Ok(::std::option::Option::None)
                    }
                }
            })
            .unwrap_or_else(|| {
                quote! {
                    ::stitchkit_core::binary::Deserialize::deserialize(deserializer)
                }
            });
        let variable_value = quote! {
            ::anyhow::Context::context(#variable_value, #error)?
        };
        variables.push(quote! {
            let #field_name: #field_type = #variable_value;
        });
        constructor_fields.push(quote! { #field_name, })
    }

    let type_name = st.ident;
    let (impl_generics, type_generics, where_clause) = st.generics.split_for_impl();
    let variables = TokenStream::from_iter(variables);
    let constructor_fields = TokenStream::from_iter(constructor_fields);

    Ok(quote! {
        impl #impl_generics ::stitchkit_core::binary::Deserialize for #type_name #type_generics #where_clause {
            fn deserialize(deserializer: &mut ::stitchkit_core::binary::Deserializer<impl ::std::io::Read>) -> ::anyhow::Result<Self> {
                #variables
                Ok(Self {
                    #constructor_fields
                })
            }
        }
    })
}
