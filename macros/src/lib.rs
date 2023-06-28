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
        fn #ident() -> impl concoct::Composable<Input = (), Output = ()> {
            #struct_ident {
                is_done: false
            }
        }

        #[allow(non_camel_case_types)]
        struct #struct_ident {
            is_done: bool
        }

        impl concoct::Composable for #struct_ident {
            type Input = ();
            type Output = ();

            fn compose(&mut self, changed: u32, input: Self::Input) -> Self::Output {
                if !self.is_done {
                    self.is_done = true;
                    #block
                }
            }
        }
    };

    TokenStream::from(expanded)
}
