use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, Item, ItemFn};

#[proc_macro_attribute]
pub fn composable(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut item = parse_macro_input!(item as ItemFn);
    /*
    item.sig.output = parse_quote! {
        -> impl concoct::Composable<(), Output = ()>
    };
     */

    let expanded = quote! {
        #[concoct_rt::composable]
        #item
    };

    TokenStream::from(expanded)
}
