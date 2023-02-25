use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{spanned::Spanned, Expr, ItemStruct, LitStr};

pub fn derive_serialize_impl(st: ItemStruct) -> syn::Result<TokenStream> {
    let mut stmts = vec![];
    let mut constructor_fields = vec![];
    for (i, field) in st.fields.iter().enumerate() {
        let field_name = field
            .ident
            .clone()
            .unwrap_or_else(|| Ident::new(&i.to_string(), field.span()));
        let error = LitStr::new(
            &format!("cannot serialize field {}::{}", st.ident, field_name),
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

        let expr = serialized_when
            .map(|_cond| {
                quote! {
                    // TODO: Validate _cond somehow.
                    if let ::std::option::Option::Some(value) = &self.#field_name {
                        ::stitchkit_core::binary::Serialize::serialize(value, serializer)
                    } else {
                        ::std::result::Result::Ok(())
                    }
                }
            })
            .unwrap_or_else(|| {
                quote! {
                    ::stitchkit_core::binary::Serialize::serialize(&self.#field_name, serializer)
                }
            });
        let stmt = quote! {
            ::stitchkit_core::binary::ResultContextExt::context(#expr, #error)?;
        };
        stmts.push(stmt);
        constructor_fields.push(quote! { #field_name, })
    }

    let type_name = st.ident;
    let (impl_generics, type_generics, where_clause) = st.generics.split_for_impl();
    let stmts = TokenStream::from_iter(stmts);

    Ok(quote! {
        impl #impl_generics ::stitchkit_core::binary::Serialize for #type_name #type_generics #where_clause {
            fn serialize(&self, serializer: &mut ::stitchkit_core::binary::Serializer<impl ::std::io::Write>) -> ::std::result::Result<(), ::stitchkit_core::binary::Error> {
                #stmts
                Ok(())
            }
        }
    })
}
