use accesskit::Role;
use concoct::{container, text, Composer, Modifier, Semantics};

#[test]
fn it_works() {
    fn app() {
        container(
            Modifier::default().merge_descendants().role(Role::Button),
            || {
                text(String::from("Hello World!"));
            },
        )
    }

    app();

    Composer::with(|composer| {
        let mut cx = composer.borrow_mut();

        let mut semantics = Semantics::default();
        cx.semantics(&mut semantics);

        dbg!(semantics);
    });
}
