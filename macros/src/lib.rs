use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Item};

#[proc_macro_attribute]
pub fn composable(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as Item);

    let expanded = quote! {
        #[concoct_rt::composable]
        #item
    };

    TokenStream::from(expanded)
}
