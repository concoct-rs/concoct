use quote::{format_ident, ToTokens};
use std::{io::Read, mem};
use syn::{fold::Fold, parse_quote, Expr, FnArg, Ident, Item, Type, PatType, Pat};

fn main() {
    let mut source_file = std::fs::File::open("app.rs").unwrap();
    let mut content = String::new();
    source_file.read_to_string(&mut content).unwrap();
    let mut file = syn::parse_file(&content).unwrap();

    let mut current_id = 0u64;

    let mut composables = Vec::new();
    let mut fold = ParenthesizeEveryExpr {
        composables: Vec::new(),
    };

    for item in &mut file.items {
        match item {
            Item::Fn(item_fn) => {
                if item_fn
                    .attrs
                    .iter()
                    .any(|attr| attr.path().get_ident().unwrap().to_string() == "composable")
                {
                    fold.composables.push(item_fn.sig.ident.clone());

                    let mut sig = item_fn.sig.clone();
                    let ident = format_ident!("{}Composable", sig.ident);
                    sig.ident = ident.clone();

                    let args = sig.inputs.clone().into_iter().map(|input| match input {
                        FnArg::Typed(pat_type) => match &*pat_type.pat {
                            Pat::Ident(pat_ident) => pat_ident.ident.clone(),
                            _ => todo!(),
                        },
                        FnArg::Receiver(_) => todo!(),
                    });

                    sig.inputs
                        .insert(0, parse_quote!(composer: &mut impl Composer));
                    sig.inputs.insert(1, parse_quote!(changed: u64));

                    let old_block =
                        parse_quote!({ panic!("Must be called from a concoct runtime.") });
                    let block = mem::replace(&mut item_fn.block, old_block);

                    let id = current_id;
                    current_id += 1;

                    let composable = parse_quote! {
                        #sig {
                            composer.start_restart_group(#id);

                            if changed == 0 && composer.is_skipping() {
                                composer.skip_to_group_end();
                            } else {
                                #block
                            }

                            composer.end_restart_group(|composer| {
                                #ident(composer, changed | 1, #(#args),*)
                            });

                        }
                    };
                    composables.push(composable);
                }
            }
            _ => todo!(),
        }
    }

    file.items.extend(
        composables
            .into_iter()
            .map(|composable| Item::Fn(fold.fold_item_fn(composable))),
    );

    println!("{}", file.into_token_stream());
}

struct ParenthesizeEveryExpr {
    composables: Vec<Ident>,
}

impl Fold for ParenthesizeEveryExpr {
    fn fold_expr_call(&mut self, mut i: syn::ExprCall) -> syn::ExprCall {
        if let Expr::Path(path) = &mut *i.func {
            if let Some(segment) = path.path.segments.last_mut() {
                if self.composables.contains(&segment.ident) {
                    segment.ident = format_ident!("{}Composable", segment.ident);
                    i.args.insert(0, parse_quote!(composer));
                    i.args.insert(1, parse_quote!(changed));
                }
            }
        }

        i
    }
}
