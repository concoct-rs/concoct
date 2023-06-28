use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, parse_quote, Item, ItemFn};

#[proc_macro_attribute]
pub fn composable(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemFn);

    let ident = item.sig.ident;
    let block = item.block;

    let struct_ident = format_ident!("{}_composable", ident);

    let expanded = quote! {
        fn #ident() -> impl concoct::Composable<State = (), Output = ()> {
            #struct_ident {
                
            }
        }

        #[allow(non_camel_case_types)]
        struct #struct_ident {

        }

        impl concoct::Composable for #struct_ident {
            type State = ();
            type Output = ();

            fn compose(self, changed: u32, state: &mut Option<Self::State>) -> Self::Output {
                if state.is_none() {
                    *state = Some(());
                    #block
                }
            }
        }
    };

    TokenStream::from(expanded)
}
