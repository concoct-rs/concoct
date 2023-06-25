mod applier;
pub use applier::Applier;

pub trait Compose {
    fn changed<T: Clone>(&mut self, value: &T) -> bool;

    fn is_skipping(&self) -> bool;

    fn skip_to_group_end(&mut self);

    fn start_restart_group(&mut self, id: u64);

    fn end_restart_group(&mut self, update: impl FnMut(&mut Self));
}

#[derive(Default)]
pub struct Composer {
    is_changed: bool,
}

impl Compose for Composer {
    fn changed<T: Clone>(&mut self, _value: &T) -> bool {
        self.is_changed = !self.is_changed;
        self.is_changed
    }

    fn is_skipping(&self) -> bool {
        true
    }

    fn skip_to_group_end(&mut self) {}

    fn start_restart_group(&mut self, _id: u64) {}

    fn end_restart_group(&mut self, _update: impl FnMut(&mut Self)) {}
}

use quote::{format_ident, quote, ToTokens};
use std::{io::Read, mem, path::Path};
use syn::{fold::Fold, parse_quote, Expr, FnArg, Ident, Item, Pat};

pub fn run(src: impl AsRef<Path>, dst: impl AsRef<Path>) {
    let mut source_file = std::fs::File::open(src).unwrap();
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
                if let Some(idx) = item_fn
                    .attrs
                    .iter_mut()
                    .position(|attr| attr.path().get_ident().unwrap().to_string() == "composable")
                {
                    item_fn.attrs.remove(idx);

                    fold.composables.push(item_fn.sig.ident.clone());

                    let mut sig = item_fn.sig.clone();
                    let ident = format_ident!("{}Composable", sig.ident);
                    sig.ident = ident.clone();

                    let args: Vec<_> = sig
                        .inputs
                        .clone()
                        .into_iter()
                        .map(|input| match input {
                            FnArg::Typed(pat_type) => match &*pat_type.pat {
                                Pat::Ident(pat_ident) => pat_ident.ident.clone(),
                                _ => todo!(),
                            },
                            FnArg::Receiver(_) => todo!(),
                        })
                        .collect();

                    sig.inputs
                        .insert(0, parse_quote!(composer: &mut impl concoct::Compose));
                    sig.inputs.insert(1, parse_quote!(changed: u64));

                    let old_block =
                        parse_quote!({ panic!("Must be called from a concoct runtime.") });
                    let block = mem::replace(&mut item_fn.block, old_block);

                    let id = current_id;
                    current_id += 1;

                    let dirty = if args.is_empty() {
                        quote!()
                    } else {
                        let checks = args.iter().enumerate().map(|(idx, arg)| {
                            let bits: u64 = 0b111 << (idx * 3 + 1);
                            quote! {
                                if changed & #bits == 0 {
                                    dirty = changed | if composer.changed(&#arg) { 4 } else { 2 };
                                }
                            }
                        });

                        quote! {
                            let mut dirty = changed;
                            #(#checks)*
                        }
                    };

                    let check = if args.is_empty() {
                        quote!(changed == 0 &&)
                    } else {
                        let checks = args.iter().enumerate().map(|(idx, _arg)| {
                            let bits: u64 = (0b101 << (idx * 3 + 1)) + 1;
                            quote! {
                               dirty & #bits == 2
                            }
                        });

                        quote! {
                            #(#checks && )*
                        }
                    };

                    let composable = parse_quote! {
                        #sig {
                            composer.start_restart_group(#id);

                            #dirty

                            if #check composer.is_skipping() {
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

    std::fs::write(dst, file.into_token_stream().to_string()).unwrap();
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
