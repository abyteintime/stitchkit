use syn::ItemStruct;

mod parse;

#[proc_macro_derive(Parse)]
pub fn derive_parse(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(item as ItemStruct);

    parse::derive_parse_impl(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(PredictiveParse)]
pub fn derive_predictive_parse(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(item as ItemStruct);

    parse::derive_predictive_parse_impl(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
