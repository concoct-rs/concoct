use concoct::{text, Composer, Semantics};

#[test]
fn it_works() {
    fn app() {
        text(String::from("Hello World!"));
    }

    app();

    let mut semantics = Semantics::default();
    Composer::with(|composer| {
        let mut cx = composer.borrow_mut();
        for widget in cx.widgets.values_mut() {
            widget.semantics(&mut semantics);
        }
    });

    app();
    Composer::with(|composer| {
        let mut cx = composer.borrow_mut();
        for widget in cx.widgets.values_mut() {
            widget.semantics(&mut semantics);
        }
    });

    Composer::with(|composer| {
        assert_eq!(composer.borrow().widgets.len(), 1);
    });

    assert_eq!(semantics.nodes.len(), 1);
}
