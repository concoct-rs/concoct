#![feature(type_alias_impl_trait)]

use concoct::view::{Adapt, Text, View};

fn text() -> impl View<i32, ()> {
    Text::from(String::from("WAT"))
}

fn app() -> impl View<(), ()> {
    Adapt::new(|_, _| None, text())
}

fn main() {}
