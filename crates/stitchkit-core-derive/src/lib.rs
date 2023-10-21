mod deserialize;
mod serialize;

use syn::{parse_macro_input, ItemStruct};

#[proc_macro_derive(Deserialize, attributes(serialized_when))]
pub fn derive_deserialize(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as ItemStruct);

    deserialize::derive_deserialize_impl(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(Serialize, attributes(serialized_when))]
pub fn derive_serialize(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as ItemStruct);

    serialize::derive_serialize_impl(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
