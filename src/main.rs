#![feature(rustc_private)]

extern crate rustc_ast_pretty;
extern crate rustc_driver;
extern crate rustc_error_codes;
extern crate rustc_errors;
extern crate rustc_hash;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

use clap::Parser;
use quote::format_ident;
use rustc_errors::registry;
use rustc_hir::intravisit::{self, Visitor};
use rustc_middle::ty::TyCtxt;
use rustc_session::config::{self, CheckCfg};
use rustc_span::source_map;
use std::{
    path::{self, PathBuf},
    process, str,
};
use syn::parse_quote;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    input: PathBuf,
}

fn main() {
    let args = Args::parse();
    let content = std::fs::read_to_string(args.input).unwrap();
    compile(content)
}

pub fn compile(input: String) {
    let out = process::Command::new("rustc")
        .arg("--print=sysroot")
        .current_dir(".")
        .output()
        .unwrap();
    let sysroot = str::from_utf8(&out.stdout).unwrap().trim();
    let config = rustc_interface::Config {
        opts: config::Options {
            maybe_sysroot: Some(path::PathBuf::from(sysroot)),
            ..config::Options::default()
        },
        input: config::Input::Str {
            name: source_map::FileName::Custom("main.rs".to_string()),
            input,
        },
        crate_cfg: rustc_hash::FxHashSet::default(),
        crate_check_cfg: CheckCfg::default(),
        output_dir: None,
        output_file: None,
        file_loader: None,
        locale_resources: rustc_driver::DEFAULT_LOCALE_RESOURCES,
        lint_caps: rustc_hash::FxHashMap::default(),
        parse_sess_created: None,
        register_lints: None,
        override_queries: None,
        make_codegen_backend: None,
        registry: registry::Registry::new(&rustc_error_codes::DIAGNOSTICS),
    };
    rustc_interface::run_compiler(config, |compiler| {
        compiler.enter(|queries| {
            let mut cooked = syn::File {
                shebang: None,
                attrs: Vec::new(),
                items: Vec::new(),
            };

            queries.global_ctxt().unwrap().enter(|tcx| {
                let hir_krate = tcx.hir();

                for id in hir_krate.items() {
                    let item = hir_krate.item(id);

                    if let rustc_hir::ItemKind::Fn(_sig, _, body_id) = item.kind {
                        let expr = tcx.hir().body(body_id);

                        let mut visitor = Visit {
                            tcx,
                            items: Vec::new(),
                        };
                        visitor.visit_expr(expr.value);
                        let items = visitor.items;

                        let name = format_ident!("{}", tcx.hir().name(id.hir_id()).to_string());

                        let item = parse_quote! {
                            fn #name(composer: &mut impl concoct::Compose, changed: u32) {
                                #(#items)*
                            }
                        };
                        cooked.items.push(item);
                    }
                }
            });

            println!("{}", prettyplease::unparse(&cooked));
        });
    });
}

struct Visit<'a> {
    tcx: TyCtxt<'a>,
    items: Vec<syn::Stmt>,
}

impl<'a, 'v> Visitor<'v> for Visit<'a> {
    fn visit_expr(&mut self, ex: &'v rustc_hir::Expr<'v>) {
        if let rustc_hir::ExprKind::Call(func, _args) = ex.kind {
            if let rustc_hir::ExprKind::Path(path) = func.kind {
                if let rustc_hir::QPath::Resolved(_, path) = path {
                    let id = path.res.def_id();

                    let ident = format_ident!("{}", self.tcx.item_name(id).to_string());

                    let attrs = self.tcx.get_attrs_unchecked(id);
                    if attrs
                        .get(0)
                        .and_then(|attr| attr.ident())
                        .map(|ident| ident.to_string())
                        .as_deref()
                        == Some("inline")
                    {
                        self.items.push(parse_quote! {
                            #ident(composer, changed);
                        });
                    } else {
                        self.items.push(parse_quote! {
                            #ident();
                        });
                    }
                }
            }
        }

        intravisit::walk_expr(self, ex)
    }
}
