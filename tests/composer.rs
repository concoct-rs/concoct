use accesskit::Role;
use concoct::{container, text, Composer};

#[test]
fn it_works() {
    fn app() {
        container(Role::Row, || {
            text(String::from("Hello World!"));
        });
    }

    app();

    Composer::with(|composer| {
        dbg!(composer.borrow());
    });
}
