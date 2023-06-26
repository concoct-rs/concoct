#![feature(rustc_private)]

extern crate rustc_ast_pretty;
extern crate rustc_driver;
extern crate rustc_error_codes;
extern crate rustc_errors;
extern crate rustc_hash;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_session;
extern crate rustc_span;

use clap::Parser;
use rustc_ast_pretty::pprust::item_to_string;
use rustc_errors::registry;
use rustc_session::config::{self, CheckCfg};
use rustc_span::source_map;
use std::{
    path::{self, PathBuf},
    process, str,
};

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
            // TODO: add this to -Z unpretty
            let ast_krate = queries.parse().unwrap().get_mut().clone();
            for item in ast_krate.items {
                println!("{}", item_to_string(&item));
            }
            // Analyze the crate and inspect the types under the cursor.
            queries.global_ctxt().unwrap().enter(|tcx| {
                // Every compilation contains a single crate.
                let hir_krate = tcx.hir();
           
                // Iterate over the top-level items in the crate, looking for the main function.
                for id in hir_krate.items() {
                    let item = hir_krate.item(id);
                    // Use pattern-matching to find a specific node inside the main function.
                    if let rustc_hir::ItemKind::Fn(_, _, body_id) = item.kind {
                        let expr = &tcx.hir().body(body_id).value;
                        dbg!(&expr);
                    }
                }
            })
        });
    });
}
