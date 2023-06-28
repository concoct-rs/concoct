use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, parse_quote, Expr, FieldValue, FnArg, GenericParam, Item, ItemFn,
    ReturnType, Stmt, Type, TypePath,
};

#[proc_macro_attribute]
pub fn composable(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemFn);

    let ident = item.sig.ident;
    let vis = item.vis;
    let mut generics = Vec::new();
    for param in &item.sig.generics.params {
        match param {
            GenericParam::Type(type_param) => {
                generics.push(type_param.ident.clone());
            }
            _ => todo!(),
        }
    }

    let generics_clause = item.sig.generics;

    let output = match item.sig.output {
        ReturnType::Type(_, ty) => Some(*ty),
        ReturnType::Default => None,
    };
    let output_ty = output.clone().unwrap_or(parse_quote!(()));

    let mut stmts = item.block.stmts;
    // TODO this is here for replaceable groups, etc
    for stmt in &mut stmts {
        if let Stmt::Macro(stmt_macro) = stmt {
            if let Some(macro_ident) = stmt_macro.mac.path.get_ident().map(ToString::to_string) {
                if macro_ident == "compose" {
                    let expr: Expr = stmt_macro.mac.parse_body().unwrap();
                    *stmt = parse_quote! {
                        (#expr).compose(composer, 0);
                    };
                }
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
    let inputs: Vec<_> = input_pats
        .iter()
        .zip(&input_types)
        .map(|(pat, ty)| quote!(#pat: #ty))
        .collect();

    let mut struct_fields = inputs.clone();

    let input_generics: Vec<_> = input_types
        .iter()
        .filter_map(|ty| {
            match &**ty {
                Type::Path(type_path) => {
                    if let Some(ident) = type_path.path.get_ident() {
                        return Some(ident);
                    }
                }
                _ => {}
            }

            None
        })
        .collect();

    let mut struct_markers = Vec::new();
    for (idx, generic) in generics.iter().enumerate() {
        if !input_generics.contains(&generic) {
            let ident = format_ident!("_marker{}", idx);
            struct_fields.push(parse_quote!(#ident: std::marker::PhantomData<#generic>));
            struct_markers.push(quote!(#ident: std::marker::PhantomData));
        }
    }

    let group_id = quote!(std::any::TypeId::of::<#struct_ident::<#(#generics,)*>>());
    let group = if output.is_some() {
        quote! {
            composer.start_replaceable_group(#group_id);

            let output = {
                #(#stmts)*
            };

            composer.end_replaceable_group();

            output
        }
    } else {
        quote! {
            composer.start_restart_group(#group_id);

            if changed == 0 && composer.is_skipping() {
                composer.skip_to_group_end();
            } else {
                #(#stmts)*;
            }

            composer.end_restart_group(move || {
                Box::new(move |composer| #ident(#(#input_pats),*).compose(composer, changed | 1))
            });
        }
    };

    let expanded = quote! {
        #[must_use]
        #vis fn #ident #generics_clause (#(#inputs),*) -> impl concoct::Composable<Output = #output_ty> {
            #[allow(non_camel_case_types)]
            struct #struct_ident <#(#generics),*> {
                #(#struct_fields),*,
            }

            impl #generics_clause concoct::Composable for #struct_ident <#(#generics),*> {
                type Output = #output_ty;

                fn compose(self, composer: &mut impl concoct::Compose, changed: u32) -> Self::Output {
                    compose!(());

                    let Self { #(#input_pats),*, .. } = self;

                    #group
                }
            }

            #struct_ident {
                #(#input_pats),*,
                #(#struct_markers),*
            }
        }
    };

    TokenStream::from(expanded)
}
