use syn::Item;

mod parse;
mod predictive_parse;

#[proc_macro_derive(Parse, attributes(parse))]
pub fn derive_parse(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(item as Item);

    parse::derive_parse_impl(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(PredictiveParse)]
pub fn derive_predictive_parse(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(item as Item);

    predictive_parse::derive_predictive_parse_impl(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
