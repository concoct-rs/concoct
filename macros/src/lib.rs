use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{
    fold::{self, Fold},
    parse::Parse,
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
    token::Comma,
    Expr, FnArg, GenericParam, ItemFn, Macro, ReturnType, Stmt, Type,
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

    let generics_clause = item.sig.generics.params;
    let where_clause = item.sig.generics.where_clause;

    let output = match item.sig.output {
        ReturnType::Type(_, ty) => Some(*ty),
        ReturnType::Default => None,
    };
    let output_ty = output.clone().unwrap_or(parse_quote!(()));

    let block = Folder.fold_block(*item.block);

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
            composer.replaceable_group(#group_id, move |composer| #block)
        }
    } else {
        if inputs.is_empty() {
            quote! {
                composer.restart_group(#group_id, move |composer| {
                    /*
                    if changed == 0 && composer.is_skipping() {
                        composer.skip_to_group_end();
                    } else {
                        #block
                    }
                     */

                     #block
                });
            }
        } else {
            let checks = input_pats.iter().enumerate().map(|(idx, input)| {
                let i: u32 = 0b111 << (idx * 3 + 1);
                quote! {
                    if changed & #i == 0 {
                        dirty = changed | if composer.changed(&x) { 4 } else { 2 };
                    }
                }
            });

            let mut mask = 1u32;
            let mut value = 0u32;
            for idx in 0..input_pats.len() {
                mask |= 0b101 << (idx * 3 + 1);
                value |= 0b10 << (idx * 3);
            }

            quote! {
                composer.restart_group(#group_id, move |composer| {
                    /*
                        let mut dirty = changed;

                        #(#checks)*

                        if dirty & #mask == #value  && composer.is_skipping() {
                            composer.skip_to_group_end();
                        } else {
                            #block
                        }
                     */

                    #block
                });
            }
        }
    };

    let mut constructor_fields = Punctuated::<_, Comma>::new();
    constructor_fields.extend(input_pats.iter().map(|pat| pat.to_token_stream()));
    constructor_fields.extend(struct_markers.clone());

    let mut struct_pattern = Punctuated::<_, Comma>::new();
    struct_pattern.extend(input_pats.iter().map(|pat| pat.to_token_stream()));
    struct_pattern.push(quote!(..));

    let expanded = quote! {
        #[must_use]
        #vis fn #ident <#generics_clause> (#(#inputs),*) -> impl concoct::Composable<Output = #output_ty>  #where_clause {
            #[allow(non_camel_case_types)]
            struct #struct_ident <#(#generics),*> {
                #(#struct_fields),*
            }

            impl<#generics_clause> concoct::Composable<> for #struct_ident <#(#generics),*> #where_clause {
                type Output = #output_ty;

                fn compose(self, composer: &mut concoct::Composer, changed: u32) -> Self::Output {
                    compose!(());

                    let Self { #struct_pattern } = self;

                    #group
                }
            }

            #struct_ident {
                #constructor_fields
            }
        }
    };

    TokenStream::from(expanded)
}

struct Folder;

impl Fold for Folder {
    fn fold_stmt(&mut self, mut i: syn::Stmt) -> syn::Stmt {
        if let Stmt::Macro(stmt_macro) = &i {
            if let Some(expr) = get_compose_macro(&stmt_macro.mac) {
                i = parse_quote! {
                    (#expr).compose(composer, 0);
                };
            }
        }

        fold::fold_stmt(self, i)
    }

    fn fold_expr(&mut self, mut i: Expr) -> Expr {
        if let Expr::Macro(expr_macro) = &i {
            if let Some(expr) = get_compose_macro(&expr_macro.mac) {
                i = parse_quote! {
                    (#expr).compose(composer, 0)
                };
            }
        }

        fold::fold_expr(self, i)
    }
}

fn get_compose_macro(mac: &Macro) -> Option<Expr> {
    if mac.path.get_ident().map(ToString::to_string).as_deref() == Some("compose") {
        let body = mac.parse_body().unwrap();
        Some(body)
    } else {
        None
    }
}
