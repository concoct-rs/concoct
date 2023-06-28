use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, parse_quote, FieldValue, FnArg, Item, ItemFn};

#[proc_macro_attribute]
pub fn composable(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemFn);

    let ident = item.sig.ident;
    let block = item.block;

    let mut input_pats = Vec::new();
    let mut input_types = Vec::new();
    for input in item.sig.inputs {
        match input {
            FnArg::Typed(typed) => {
                input_pats.push(typed.pat);
                input_types.push(typed.ty);
            }
            _ => todo!(),
        }
    }

    let struct_ident = format_ident!("{}_composable", ident);
    let struct_fields: Vec<FieldValue> = input_pats
        .iter()
        .zip(&input_types)
        .map(|(pat, ty)| parse_quote!(#pat: #ty))
        .collect();

    let expanded = quote! {
        #[must_use]
        fn #ident(#(#struct_fields),*) -> impl concoct::Composable<State = Option<(#(#input_types),*)>, Output = ()> {
            #struct_ident {
                #(#input_pats),*
            }
        }

        #[allow(non_camel_case_types)]
        struct #struct_ident {
            #(#struct_fields),*
        }

        impl concoct::Composable for #struct_ident {
            type State = Option<(#(#input_types),*)>;
            type Output = ();

            fn compose(self, changed: u32, state: &mut Self::State) -> Self::Output {
                let Self { #(#input_pats),* } = self;
                if *state != Some((#(#input_pats),*)) {
                    *state = Some((#(#input_pats),*));
                    #block
                }
            }
        }
    };

    TokenStream::from(expanded)
}
