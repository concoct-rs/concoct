use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, parse_quote, Expr, FieldValue, FnArg, Item, ItemFn, Stmt};

#[proc_macro_attribute]
pub fn composable(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemFn);

    let ident = item.sig.ident;

    let mut composable_types = Vec::new();
    let mut composable_type_idents = Vec::new();
    let mut composable_states = Vec::new();

    let mut stmts = item.block.stmts;
    for stmt in &mut stmts {
        if let Stmt::Macro(stmt_macro) = stmt {
            if stmt_macro
                .mac
                .path
                .get_ident()
                .map(ToString::to_string)
                .as_deref()
                == Some("compose")
            {
                let type_ident = format_ident!("{}type{}", ident, composable_types.len());
                composable_types.push(quote! {
                    #[allow(non_camel_case_types)]
                    type #type_ident = impl Default;
                });

                let expr: Expr = stmt_macro.mac.parse_body().unwrap();
                *stmt = parse_quote! {
                    (#expr).compose(0, #type_ident);
                };

                composable_states.push(quote!(Default::default()));

                composable_type_idents.push(type_ident);
            }
        }
    }

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

    let state_type = quote! {
        (Option<(#(#input_types),*)>, #(#composable_type_idents),*)
    };

    let expanded = quote! {
        #(#composable_types)*

        #[must_use]
        fn #ident(#(#struct_fields),*) -> impl concoct::Composable<State = #state_type, Output = ()> {
            #struct_ident {
                #(#input_pats),*
            }
        }

        #[allow(non_camel_case_types)]
        struct #struct_ident {
            #(#struct_fields),*
        }

        impl concoct::Composable for #struct_ident {
            type State = #state_type;
            type Output = ();

            fn compose(self, changed: u32, state: &mut Self::State) -> Self::Output {
                compose!(());

                let Self { #(#input_pats),* } = self;
                let (inputs, #(#composable_type_idents),*) = state;

                if *inputs != Some((#(#input_pats),*)) {
                    *inputs = Some((#(#input_pats),*));

                    #(#stmts)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}
