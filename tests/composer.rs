use accesskit::Role;
use concoct::{container, text, Composer, Semantics};

#[test]
fn it_works() {
    fn app() {
        container(
            Role::Button,
            || {
                text(String::from("Hello World!"));
            },
            true,
        );
    }

    app();

    Composer::with(|composer| {
        let mut cx = composer.borrow_mut();

        let mut semantics = Semantics::default();
        cx.semantics(&mut semantics);

        dbg!(semantics);
    });
}
