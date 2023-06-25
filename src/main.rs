use quote::{format_ident, quote, ToTokens};
use std::{
    io::{Read, Write},
    mem,
};
use syn::{
    fold::Fold,
    parse_quote,
    visit::{self, Visit},
    Expr, File, Ident, Item,
};

fn main() {
    let mut source_file = std::fs::File::open("app.rs").unwrap();
    let mut content = String::new();
    source_file.read_to_string(&mut content).unwrap();
    let mut file = syn::parse_file(&content).unwrap();

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
                    sig.ident = format_ident!("{}Composable", sig.ident);

                    let arg = parse_quote!(composer: &mut Composer);
                    sig.inputs.insert(0, arg);

                    let block = parse_quote!({ panic!("Must be called from a concoct runtime.") });
                    let block = mem::replace(&mut item_fn.block, block);

                    let composable = parse_quote! {
                        #sig {
                            #block
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
                }
            }
        }

        i
    }
}
