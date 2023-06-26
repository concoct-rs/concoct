use std::path::PathBuf;

use clap::{Parser, command, arg};
use quote::ToTokens;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    src: PathBuf,
    dest: Option<String>,
}

fn main() {
    let args = Args::parse();

    let file = concoct::run(args.src);
    let formatted = prettyplease::unparse(&file);
    println!("{}", formatted);
}