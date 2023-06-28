use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, parse_quote, Expr, FieldValue, FnArg, Item, ItemFn, ReturnType, Stmt, Type,
};

#[proc_macro_attribute]
pub fn composable(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemFn);

    let ident = item.sig.ident;
    let vis = item.vis;

    let output = match item.sig.output {
        ReturnType::Type(_, ty) => Some(*ty),
        ReturnType::Default => None,
    };
    let output_ty = output.clone().unwrap_or(parse_quote!(()));

    let mut stmts = item.block.stmts;
    // TODO this is here for replaceable groups, etc
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
                let expr: Expr = stmt_macro.mac.parse_body().unwrap();
                *stmt = parse_quote! {
                    (#expr).compose(composer, 0);
                };
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

    let start_group = if output.is_none() {
        quote! {
            composer.start_restart_group(std::any::TypeId::of::<#struct_ident>());
        }
    } else {
        quote! {
            composer.start_replaceable_group(std::any::TypeId::of::<#struct_ident>());
        }
    };

    let end_group  = if output.is_none() {
        quote! {
            composer.end_restart_group(move || {
                Box::new(move |composer| #ident(#(#input_pats),*).compose(composer, changed | 1))
            });
        }
    } else {
        quote! {
            composer.end_replaceable_group();
        }
    };

    let expanded = quote! {
        #[must_use]
        #vis fn #ident(#(#struct_fields),*) -> impl concoct::Composable<Output = #output_ty> {
            #[allow(non_camel_case_types)]
            struct #struct_ident {
                #(#struct_fields),*
            }

            impl concoct::Composable for #struct_ident {
                type Output = #output_ty;

                fn compose(self, composer: &mut impl concoct::Compose, changed: u32) -> Self::Output {
                    compose!(());

                    #start_group

                    let Self { #(#input_pats),* } = self;

                    let output = {
                        #(#stmts)*
                    };

                    #end_group

                    output
                }
            }

            #struct_ident {
                #(#input_pats),*
            }
        }
    };

    TokenStream::from(expanded)
}
