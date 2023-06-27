#![feature(rustc_private)]

extern crate rustc_driver;
extern crate rustc_driver_impl;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

use quote::format_ident;
use rustc_driver_impl::Compilation;
use rustc_hir::intravisit::{self, Visitor};
use rustc_interface::interface;
use rustc_middle::ty::TyCtxt;
use rustc_session::parse::ParseSess;
use rustc_span::symbol::Symbol;
use syn::parse_quote;

use std::env;
use std::ops::Deref;
use std::path::Path;
use std::process::exit;

/// If a command-line option matches `find_arg`, then apply the predicate `pred` on its value. If
/// true, then return it. The parameter is assumed to be either `--arg=value` or `--arg value`.
fn arg_value<'a, T: Deref<Target = str>>(
    args: &'a [T],
    find_arg: &str,
    pred: impl Fn(&str) -> bool,
) -> Option<&'a str> {
    let mut args = args.iter().map(Deref::deref);
    while let Some(arg) = args.next() {
        let mut arg = arg.splitn(2, '=');
        if arg.next() != Some(find_arg) {
            continue;
        }

        match arg.next().or_else(|| args.next()) {
            Some(v) if pred(v) => return Some(v),
            _ => {}
        }
    }
    None
}

#[test]
fn test_arg_value() {
    let args = &["--bar=bar", "--foobar", "123", "--foo"];

    assert_eq!(arg_value(&[] as &[&str], "--foobar", |_| true), None);
    assert_eq!(arg_value(args, "--bar", |_| false), None);
    assert_eq!(arg_value(args, "--bar", |_| true), Some("bar"));
    assert_eq!(arg_value(args, "--bar", |p| p == "bar"), Some("bar"));
    assert_eq!(arg_value(args, "--bar", |p| p == "foo"), None);
    assert_eq!(arg_value(args, "--foobar", |p| p == "foo"), None);
    assert_eq!(arg_value(args, "--foobar", |p| p == "123"), Some("123"));
    assert_eq!(
        arg_value(args, "--foobar", |p| p.contains("12")),
        Some("123")
    );
    assert_eq!(arg_value(args, "--foo", |_| true), None);
}

struct DefaultCallbacks;
impl rustc_driver::Callbacks for DefaultCallbacks {}

/// This is different from `DefaultCallbacks` that it will inform Cargo to track the value of
/// `CLIPPY_ARGS` environment variable.
struct RustcCallbacks {
    clippy_args_var: Option<String>,
}

impl rustc_driver::Callbacks for RustcCallbacks {
    fn config(&mut self, config: &mut interface::Config) {
        let clippy_args_var = self.clippy_args_var.take();
    }
}

struct ClippyCallbacks {
    clippy_args_var: Option<String>,
}

impl rustc_driver::Callbacks for ClippyCallbacks {
    fn after_parsing<'tcx>(
        &mut self,
        compiler: &interface::Compiler,
        queries: &'tcx rustc_interface::Queries<'tcx>,
    ) -> rustc_driver_impl::Compilation {
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
                    let name = format_ident!("{}", tcx.hir().name(id.hir_id()).to_string());

                    let attrs = hir_krate.attrs(id.hir_id());
                    let item = if attrs
                        .get(0)
                        .and_then(|attr| attr.ident())
                        .map(|ident| ident.to_string())
                        .as_deref()
                        == Some("inline")
                    {
                        let expr = tcx.hir().body(body_id);

                        let mut visitor = Visit {
                            tcx,
                            items: Vec::new(),
                        };
                        visitor.visit_expr(expr.value);
                        let items = visitor.items;

                        let name = format_ident!("{}", tcx.hir().name(id.hir_id()).to_string());

                        parse_quote! {
                            fn #name(composer: &mut impl concoct::Compose, changed: u32) {
                                #(#items)*
                            }
                        }
                    } else {
                        parse_quote!(
                            fn #name() {

                            }
                        )
                    };

                    cooked.items.push(item);
                }
            }
        });

        std::fs::write("out.rs", prettyplease::unparse(&cooked)).unwrap();

        Compilation::Stop
    }
}

#[allow(clippy::too_many_lines)]
pub fn main() {
    rustc_driver::init_rustc_env_logger();

    // rustc_driver::install_ice_hook(BUG_REPORT_URL, |handler| {});

    exit(rustc_driver::catch_with_exit_code(move || {
        let mut orig_args: Vec<String> = env::args().collect();
        let has_sysroot_arg = arg_value(&orig_args, "--sysroot", |_| true).is_some();

        let sys_root_env = std::env::var("SYSROOT").ok();
        let pass_sysroot_env_if_given = |args: &mut Vec<String>, sys_root_env| {
            if let Some(sys_root) = sys_root_env {
                if !has_sysroot_arg {
                    args.extend(vec!["--sysroot".into(), sys_root]);
                }
            };
        };

        // make "clippy-driver --rustc" work like a subcommand that passes further args to "rustc"
        // for example `clippy-driver --rustc --version` will print the rustc version that clippy-driver
        // uses
        if let Some(pos) = orig_args.iter().position(|arg| arg == "--rustc") {
            orig_args.remove(pos);
            orig_args[0] = "rustc".to_string();

            let mut args: Vec<String> = orig_args.clone();
            pass_sysroot_env_if_given(&mut args, sys_root_env);

            return rustc_driver::RunCompiler::new(&args, &mut DefaultCallbacks).run();
        }

        if orig_args.iter().any(|a| a == "--version" || a == "-V") {
            exit(0);
        }

        // Setting RUSTC_WRAPPER causes Cargo to pass 'rustc' as the first argument.
        // We're invoking the compiler programmatically, so we ignore this/
        let wrapper_mode =
            orig_args.get(1).map(Path::new).and_then(Path::file_stem) == Some("rustc".as_ref());

        if wrapper_mode {
            // we still want to be able to invoke it normally though
            orig_args.remove(1);
        }

        if !wrapper_mode
            && (orig_args.iter().any(|a| a == "--help" || a == "-h") || orig_args.len() == 1)
        {
            // display_help();
            exit(0);
        }

        let mut args: Vec<String> = orig_args.clone();
        pass_sysroot_env_if_given(&mut args, sys_root_env);

        let mut no_deps = false;
        let clippy_args_var = env::var("CLIPPY_ARGS").ok();
        let clippy_args = clippy_args_var
            .as_deref()
            .unwrap_or_default()
            .split("__CLIPPY_HACKERY__")
            .filter_map(|s| match s {
                "" => None,
                "--no-deps" => {
                    no_deps = true;
                    None
                }
                _ => Some(s.to_string()),
            })
            .chain(vec!["--cfg".into(), r#"feature="cargo-clippy""#.into()])
            .collect::<Vec<String>>();

        // We enable Clippy if one of the following conditions is met
        // - IF Clippy is run on its test suite OR
        // - IF Clippy is run on the main crate, not on deps (`!cap_lints_allow`) THEN
        //    - IF `--no-deps` is not set (`!no_deps`) OR
        //    - IF `--no-deps` is set and Clippy is run on the specified primary package
        let cap_lints_allow = arg_value(&orig_args, "--cap-lints", |val| val == "allow").is_some()
            && arg_value(&orig_args, "--force-warn", |val| val.contains("clippy::")).is_none();
        let in_primary_package = env::var("CARGO_PRIMARY_PACKAGE").is_ok();

        let clippy_enabled = !cap_lints_allow && (!no_deps || in_primary_package);
        if clippy_enabled {
            args.extend(clippy_args);
            rustc_driver::RunCompiler::new(&args, &mut ClippyCallbacks { clippy_args_var }).run()
        } else {
            rustc_driver::RunCompiler::new(&args, &mut RustcCallbacks { clippy_args_var }).run()
        }
    }))
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
